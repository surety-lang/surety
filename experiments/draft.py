#!/usr/bin/env python3
"""The draft: an obligation's properties asked for in two stages, with the
file in between and the examples never.

    python3 experiments/draft.py sentences <problem.md> [--types <file.sur>] --out <name>.sur
    python3 experiments/draft.py lines <name>.sur

Both stages take `--withhold <name>` (repeatable): a corpus obligation kept out
of the prompt by its file stem, so a draft against a solved problem does not
see the author's answer.

`sentences` hands the model the problem's prose, the author's types if any,
and the public docs and corpus, and takes back the types it had to add (each
under a `// drafted` line, so the author reads them first), the signature,
and `#` sentences, one claim each, with no property line beneath any of them
and no examples. The file starts with `// draft`. The author then edits the
sentences in the editor.

`lines` asks, for every sentence with no property line beneath it, for one
line in the notation; inserts it; runs `surety parse`; feeds errors back for
up to three rounds; then runs `surety mutants` and prints its report. A
sentence the model cannot express gets `// reaching for: <what it wanted>`
beneath it, and the run ends with those listed, for the sheet.

What the model never writes: an `e:` block or an `accept` line. Both are the
human's; the tool strips either and says so. `surety pin` refuses a file that
still carries the `// draft` mark.

Credentials come from the Anthropic SDK's own resolution; nothing here reads a
key. `--reply <file>` uses a saved reply instead of calling the model, which is
how the tool's own mechanics are tested without one. Each run writes under
experiments/draft/runs/<stamp>/: the prompt, the raw reply, the file before
and after, and the checker's reports.
"""

from __future__ import annotations

import argparse
import datetime as dt
import os
import pathlib
import re
import subprocess
import sys

ROOT = pathlib.Path(__file__).resolve().parent.parent
CORPUS = ROOT / "crates" / "surety" / "tests" / "corpus"
DOCS = ROOT / "docs"
RUNS = ROOT / "experiments" / "draft" / "runs"
MODEL = "claude-opus-5-5"
MARK = "// draft"

NOTATION = """The notation, by the forms the corpus uses:

- Types: `type Name = { field: Type, other: Type }`. Fields: Int, Float, Text, Bool, Id, Time, UUID, another type's name, `T[]` (a list), `T[unique f]` (no two elements share f), `T[0..3]` (length bound), `Option<T>`, `Result<T, E>`. `{ ...Base, extra: Type }` spreads Base's fields.
- Named values: `name: Type = { field: value, ... }`. Abbreviations: `name(p: Type) = <boolean>` at file level, or `name = <boolean>` inside the obligation, named by the property lines.
- The signature: `name(input: Type, other: Type) -> Type`. Under it, one tab in: `note "<one sentence of intent>"`, then the properties.
- A `#` line is a sentence: one claim in prose, pinned to the property line beneath it. A property line is one claim in the notation, about the relation between the inputs and `out` and never about execution.
- Quantifiers: `all x in L: <body>`, `any x in L: <body>`, `none x in L: <body>`, `exactly_one x in L: <body>`; `where <condition>` before the colon filters; `all x leftover in L: <body>` ranges over the input list minus what appears in out; `Ok(x)` or `Err(x)` as a binder matches one side of a Result.
- Membership and comparison: `x in L`, `x not in L`, `==`, `!=`, `<`, `<=`, `>`, `>=`, `and`, `or`, `not`, `->` (implies), `<->` (iff). Field access `x.f`; `L.f` is the list of every element's f.
- Whole-list claims: `unique out.f`, `out unordered` (order is not claimed), `out same_order_as L`, `out same_elements_as L`, `out drawn_from L`, `out sorted_by (f desc, g asc)`, `a before b in L`.
- Built-ins: `sum(L)`, `len(L)`, `min(a, b)`, `max(a, b)`, `count(L)`, `flatten(L)`. `None`, `Ok(x)`, `Err(x)` as values.
- Examples (`e:` / `in:` / `out:`) and `accept` lines are the human author's. Never write them."""

SYSTEM_SENTENCES = """You draft Surety obligations from prose, in the first of two stages: the sentences.

A Surety obligation is a specification a checker enforces: type declarations, a function signature, properties (claims about the relation between the inputs and the output), and examples. In this stage you write the types the problem needs that the author did not give, the signature, and the claims as prose sentences only: one `#` line per claim, each a single claim a reader can judge true or false, nothing beneath any of them. No property lines, no examples, no accept lines: the author reviews the sentences first, then the notation is asked for separately.

Reply with one ```surety code block and nothing else outside it, laid out as:

// draft
// drafted
type <Name> = { ... }          (each type you had to add, each under its own `// drafted` line; the author's own types verbatim, unmarked)

<name>(<inputs>) -> <output>
	note "<one sentence of intent>"
	# <sentence>
	# <sentence>

Use the vocabulary of the problem. Prefer several short claims to one long one. Say what must be in the output, what must not be, what is preserved, and what the order means or that it does not matter. If the problem has an error case, say what each error means and when exactly one of them is reported.

""" + NOTATION

SYSTEM_LINES = """You draft Surety obligations from prose, in the second of two stages: the property lines.

You are given an obligation whose author has reviewed its `#` sentences, and a list of the sentences that have no property line beneath them. For each listed sentence, give exactly one line in the notation that says what the sentence says, or, when the notation cannot say it, `reaching for: <what you wanted to write>` in plain words. Never widen or narrow a sentence to make it expressible; reach instead.

Reply with one ```surety code block and nothing else outside it, holding one pair per listed sentence:

# <the sentence, verbatim>
	<one property line, or: reaching for: <what you wanted>>

Never write an `e:` block, an `in:` or `out:` line, or an `accept` line; those are the author's. Do not repeat the file, the types or the signature.

""" + NOTATION


def read(p: pathlib.Path) -> str:
    return p.read_text(encoding="utf-8")


def corpus_text(withhold: list[str]) -> str:
    """The corpus, minus the obligations withheld by stem: the draft against a
    solved problem must not be shown the author's answer."""
    parts = []
    for f in sorted(CORPUS.glob("*.sur")):
        if f.stem in withhold:
            continue
        parts.append(f"`{f.name}`:\n\n```\n{read(f)}```")
    return f"The corpus, {len(parts)} obligations written by hand:\n\n" + "\n\n".join(parts)


def docs_text() -> str:
    """The public docs' decision notes: short, and the rules the notation rests on."""
    parts = []
    for f in sorted(DOCS.glob("Decision - *.md")):
        parts.append(f"`{f.name}`:\n\n{read(f).strip()}")
    return "The decisions the notation rests on:\n\n" + "\n\n".join(parts) if parts else ""


def call_model(system: str, prompt: str, model: str, effort: str, workspace: str | None) -> tuple[str, dict]:
    import anthropic

    client = anthropic.Anthropic()
    headers: dict[str, str] = {"anthropic-beta": "server-side-fallback-2026-07-01"}
    if workspace:
        headers["anthropic-workspace-id"] = workspace
    with client.messages.stream(
        model=model,
        max_tokens=32000,
        system=[{"type": "text", "text": system, "cache_control": {"type": "ephemeral"}}],
        messages=[{"role": "user", "content": prompt}],
        thinking={"type": "adaptive"},
        output_config={"effort": effort},
        extra_body={"fallbacks": "default"},
        extra_headers=headers,
    ) as stream:
        msg = stream.get_final_message()
    text = "".join(b.text for b in msg.content if getattr(b, "type", "") == "text")
    usage = {"input": msg.usage.input_tokens, "output": msg.usage.output_tokens}
    return text, usage


def code_block(reply: str) -> str | None:
    m = re.search(r"```surety\s*\n(.*?)```", reply, re.S)
    if m:
        return m.group(1)
    m = re.search(r"```\s*\n(.*?)```", reply, re.S)
    return m.group(1) if m else None


def strip_human_parts(text: str) -> tuple[str, list[str]]:
    """Remove `e:` blocks and `accept` lines the model should not have written."""
    out: list[str] = []
    removed: list[str] = []
    skip = False
    for line in text.split("\n"):
        t = line.strip()
        if t == "e:" or t.startswith("e: "):
            skip = True
            removed.append("an `e:` block")
            continue
        if skip:
            if t == "" or line.startswith("\t\t") or (line.startswith("\t") and (t.startswith(("note", "in:", "out:")) or t[0] in "{[}],")):
                continue
            skip = False
        if t.startswith("accept ") or t == "accept":
            removed.append(f"an accept line: {t}")
            continue
        out.append(line)
    return "\n".join(out), removed


def run_surety(surety: str, *args: str) -> tuple[int, str]:
    proc = subprocess.run([surety, *args], capture_output=True, text=True, timeout=900)
    return proc.returncode, proc.stdout + proc.stderr


def find_surety(given: str | None) -> str:
    if given:
        return given
    release = ROOT / "target" / "release" / "surety"
    if subprocess.run(["cargo", "build", "--release", "-q", "-p", "surety"], cwd=ROOT).returncode:
        raise SystemExit("cargo build --release failed")
    return str(release)


def run_dir(stage: str) -> pathlib.Path:
    stamp = dt.datetime.now().strftime("%Y%m%d-%H%M%S")
    d = RUNS / f"{stamp}-{stage}"
    d.mkdir(parents=True)
    return d


def sentences_without_lines(text: str) -> list[str]:
    """Each `#` sentence whose next non-blank line is not a property line."""
    lines = text.split("\n")
    out = []
    for i, line in enumerate(lines):
        t = line.strip()
        if not t.startswith("#"):
            continue
        sentence = t[1:].strip()
        nxt = next((l for l in lines[i + 1:] if l.strip()), "")
        n = nxt.strip()
        if n == "" or n.startswith("#") or n.startswith("//") or n == "e:" or not nxt.startswith("\t"):
            out.append(sentence)
    return out


def insert_lines(text: str, pairs: dict[str, str]) -> tuple[str, list[str]]:
    """Put each sentence's line beneath it; a `reaching for` becomes a comment."""
    lines = text.split("\n")
    out: list[str] = []
    reach: list[str] = []
    for line in lines:
        out.append(line)
        t = line.strip()
        if t.startswith("#"):
            sentence = t[1:].strip()
            if sentence in pairs:
                proposed = pairs.pop(sentence).strip()
                if proposed.lower().startswith("reaching for:"):
                    out.append(f"\t// {proposed}")
                    reach.append(proposed[len("reaching for:"):].strip())
                else:
                    out.append("\t" + proposed)
    return "\n".join(out), reach


def parse_pairs(block: str) -> tuple[dict[str, str], list[str]]:
    """`# sentence` followed by one line → {sentence: line}; refused lines listed."""
    pairs: dict[str, str] = {}
    refused: list[str] = []
    current: str | None = None
    for line in block.split("\n"):
        t = line.strip()
        if not t:
            continue
        if t.startswith("#"):
            current = t[1:].strip()
            continue
        if current is None:
            continue
        if t.startswith("accept") or t in ("e:", "in:", "out:") or t.startswith(("in:", "out:")):
            refused.append(t)
            pairs[current] = "reaching for: a line the model wrote as an example or an accept, which are the author's"
        else:
            pairs[current] = t
        current = None
    return pairs, refused


def cmd_sentences(args: argparse.Namespace) -> int:
    prose = read(pathlib.Path(args.problem))
    types = read(pathlib.Path(args.types)) if args.types else ""
    parts = [f"The problem, in prose:\n\n{prose.strip()}"]
    if types:
        parts.append(f"The author's types, to use verbatim and unmarked:\n\n```\n{types}```")
    parts.append(docs_text())
    parts.append(corpus_text(args.withhold))
    parts.append("Write the first stage for this problem: the types you had to add, the signature, and the sentences.")
    prompt = "\n\n".join(p for p in parts if p)
    d = run_dir("sentences")
    (d / "prompt.md").write_text(prompt, encoding="utf-8")
    if args.reply:
        reply, usage = read(pathlib.Path(args.reply)), {}
    else:
        reply, usage = call_model(SYSTEM_SENTENCES, prompt, args.model, args.effort, args.workspace)
    (d / "reply.md").write_text(reply, encoding="utf-8")
    block = code_block(reply)
    if block is None:
        print("no ```surety block in the reply; see", d / "reply.md", file=sys.stderr)
        return 1
    text, removed = strip_human_parts(block)
    for r in removed:
        print(f"stripped {r} (the author's, never the model's)")
    # Sentences only: a property line the model wrote beneath a sentence is dropped.
    kept = []
    for line in text.split("\n"):
        t = line.strip()
        if line.startswith("\t") and not line.startswith("\t\t") and t and not t.startswith(("#", "//", "note")) and not re.match(r"^[a-z_][A-Za-z0-9_]* *=[^=]", t):
            print(f"dropped a property line from stage 1: {t}")
            continue
        kept.append(line)
    text = "\n".join(kept).strip("\n") + "\n"
    if not text.startswith(MARK + "\n"):
        text = MARK + "\n" + text
    out = pathlib.Path(args.out)
    out.write_text(text, encoding="utf-8")
    (d / "after.sur").write_text(text, encoding="utf-8")
    surety = find_surety(args.surety)
    code, report = run_surety(surety, "parse", str(out))
    (d / "parse.txt").write_text(report, encoding="utf-8")
    sentences = sentences_without_lines(text)
    print(f"wrote {out}: {len(sentences)} sentences, no property lines, no examples; parse: {'ok' if code == 0 else report.strip()}")
    if usage:
        print(f"tokens {usage['input']}/{usage['output']}")
    print(f"run written to {d}")
    return 0 if code == 0 else 1


def cmd_lines(args: argparse.Namespace) -> int:
    path = pathlib.Path(args.file)
    original = read(path)
    d = run_dir("lines")
    (d / "before.sur").write_text(original, encoding="utf-8")
    todo = sentences_without_lines(original)
    if not todo:
        print("every sentence already has a line beneath it")
        return 0
    surety = find_surety(args.surety)
    header = [f"The obligation, with its author's sentences:\n\n```\n{original}```",
              "The sentences with no property line beneath them, one line each wanted:\n\n" + "\n".join(f"# {s}" for s in todo),
              docs_text(), corpus_text(args.withhold)]
    prompt = "\n\n".join(p for p in header if p)
    text = original
    reach: list[str] = []
    errors = ""
    replies = args.reply if args.reply else []
    for round_no in range(1, args.rounds + 1):
        ask = prompt if round_no == 1 else prompt + f"\n\nYour last lines did not parse. The checker reports:\n\n```\n{errors.strip()}\n```\n\nGive the pairs again, every listed sentence, corrected."
        (d / f"prompt{round_no}.md").write_text(ask, encoding="utf-8")
        if replies:
            reply = read(pathlib.Path(replies[min(round_no, len(replies)) - 1]))
            usage = {}
        else:
            reply, usage = call_model(SYSTEM_LINES, ask, args.model, args.effort, args.workspace)
        (d / f"reply{round_no}.md").write_text(reply, encoding="utf-8")
        block = code_block(reply)
        if block is None:
            print(f"round {round_no}: no ```surety block in the reply", file=sys.stderr)
            return 1
        pairs, refused = parse_pairs(block)
        for r in refused:
            print(f"refused a line the model wrote (the author's): {r}")
        for s in todo:
            if s not in pairs:
                pairs[s] = "reaching for: a line the model gave no answer for"
        text, reach = insert_lines(original, {s: pairs[s] for s in todo})
        path.write_text(text, encoding="utf-8")
        code, errors = run_surety(surety, "parse", str(path))
        (d / f"parse{round_no}.txt").write_text(errors, encoding="utf-8")
        if usage:
            print(f"round {round_no}: tokens {usage['input']}/{usage['output']}")
        if code == 0:
            break
        print(f"round {round_no}: {errors.strip()}")
    (d / "after.sur").write_text(text, encoding="utf-8")
    code, report = run_surety(surety, "mutants", str(path))
    (d / "mutants.txt").write_text(report, encoding="utf-8")
    print(report.rstrip())
    if reach:
        print("\nreaching for (for the sheet):")
        for r in reach:
            print(f"  - {r}")
    print(f"\n{path} still carries `{MARK}`; review the lines, remove the mark, then `surety pin`.")
    print(f"run written to {d}")
    return 0 if code == 0 else 1


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    sub = ap.add_subparsers(dest="stage", required=True)
    for name in ("sentences", "lines"):
        p = sub.add_parser(name)
        p.add_argument("--model", default=MODEL)
        p.add_argument("--effort", default="high", choices=["low", "medium", "high", "xhigh", "max"])
        p.add_argument("--surety", help="checker binary (default: build target/release/surety first)")
        p.add_argument("--workspace", default=os.environ.get("ANTHROPIC_WORKSPACE_ID"))
        p.add_argument("--withhold", action="append", default=[], metavar="NAME",
                       help="keep this corpus obligation out of the prompt, by file stem (p2); repeatable")
        if name == "sentences":
            p.add_argument("problem", help="the problem's prose, a markdown or text file")
            p.add_argument("--types", help="the author's types, a .sur file, used verbatim")
            p.add_argument("--out", required=True, help="the .sur file to write")
            p.add_argument("--reply", help="a saved model reply to use instead of calling the model")
        else:
            p.add_argument("file", help="the .sur file whose sentences want lines")
            p.add_argument("--rounds", type=int, default=3)
            p.add_argument("--reply", action="append", help="a saved reply per round, instead of calling the model")
    args = ap.parse_args()
    return cmd_sentences(args) if args.stage == "sentences" else cmd_lines(args)


if __name__ == "__main__":
    sys.exit(main())
