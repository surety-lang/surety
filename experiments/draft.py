#!/usr/bin/env python3
"""The draft: an obligation's properties asked for in two stages, with the
file in between and the examples never.

    python3 experiments/draft.py sentences <problem.md> [--types <file.sur>] --out <name>.sur
    python3 experiments/draft.py lines <name>.sur

Both stages take `--withhold <name>` (repeatable): a corpus obligation kept out
of the prompt by its file stem, so a draft against a solved problem does not
see the author's answer.

`sentences` hands the model the problem's prose, the author's types if any,
the spec's two summary paragraphs (generated inputs; the property fragment as
it stands), the decision notes and the corpus, and takes back the types and named values it had to add (each under a
`// drafted` line, so the author reads them first), the signature,
and `#` sentences, one claim each, with no property line beneath any of them
and no examples. The file starts with `// draft`. The types are this stage's
output, so the file is parsed and an error goes back to the model for up to
three rounds, as in stage 2. The author then edits the sentences in the editor.

`lines` refuses a file that does not parse (exit 2, with the error: it only
adds lines under sentences, and the types are stage 1's). Otherwise it asks,
for every sentence with no property line beneath it, for one line in the
notation; inserts it; runs `surety parse`; feeds errors back for
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
SIGNATURE = re.compile(r"^[a-z_][A-Za-z0-9_]*\s*\(.*\)\s*->")

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

A Surety obligation is a specification a checker enforces: type declarations, a function signature, properties (claims about the relation between the inputs and the output), and examples. In this stage you write the types the problem needs that the author did not give, the named values a claim will need to refer to, the signature, and the claims as prose sentences only: one `#` line per claim, each a single claim a reader can judge true or false, nothing beneath any of them. No property lines, no examples, no accept lines: the author reviews the sentences first, then the notation is asked for separately.

Reply with one ```surety code block and nothing else outside it, laid out as:

// draft
// drafted
type <Name> = { ... }          (each type you had to add, each under its own `// drafted` line; the author's own types verbatim, unmarked)
// drafted
<name>: <Type> = { ... }       (each named value a sentence needs to refer to, each under its own `// drafted` line)

<name>(<inputs>) -> <output>
	note "<one sentence of intent>"
	# <sentence>
	# <sentence>

Every type the file names must be declared in the file. The built-in types are Int, Float, Text, Bool, Id, Time and UUID, with `T[]`, `Option<T>` and `Result<T, E>` over them; there is no library of types beyond these, and no type from another obligation or from the corpus is in scope (a corpus file may name a type declared in a file beside it; the draft stands alone). A type you need that the author did not give, you declare under its own `// drafted` line, with its fields. A file that names an undeclared type does not parse and comes back to you with the error.

A named value is a declaration, not an example. Draft one beside the types, under its own `// drafted` line, only where a sentence needs a value to refer to: an error case (`age_error: SignupError = { code: "e-age-range", display: "..." }`), a reused record. Give every field, since a performance must return the value byte for byte; take the fields from the problem and invent none beyond them. Examples and accept lines stay the author's, always.

Shape every sentence as the fragment shapes a claim, since a line will be asked for each one and a sentence shaped outside the fragment gets no line. The shapes: "every <element> in <list> [with <condition>] <has/is> ..." (a quantifier, also "no", "some", "exactly one"); "<x> is in <list>" or "is not in"; "the output is drawn from the input" (every output element is an occurrence of an input element, no occurrence used twice; with "every input element left out ..." this pins how many times each appears); "the output keeps the input's order", "has the same elements as", "is not claimed in any order", "is sorted by <field> [then <field>]", "has no two elements sharing <field>"; "<a> comes before <b> in <list>"; "if ... then ...", "... exactly when ..."; "the length of <list or text> is ..."; "<text> contains <text>". Do not write a sentence that counts occurrences, sums or compares a count other than a length, does arithmetic over elements, or operates on text beyond its length and whether it contains something: those have no line. For a text's shape (an email, a phone number), say that a sibling obligation decides it ("the email is well-formed"), since that is a graph reference, never a claim of the fragment. Say what must be in the output, what must not be, what is preserved, and what the order means or that it does not matter.

Use the vocabulary of the problem. Prefer several short claims to one long one. If the problem has an error case, say what each error means and when exactly one of them is reported.

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


SPEC = DOCS / "Surety Spec v2.md"
SPEC_PARAGRAPHS = ("**Generated inputs", "**The property fragment as of")


def spec_text() -> str:
    """The two paragraphs of the spec that define the notation as it stands:
    the generated-inputs paragraph and the fragment summary under Layers.
    The whole spec is too long to hand over; these are its summary of it."""
    if not SPEC.exists():
        print(f"no {SPEC}; the draft runs without the spec's fragment summary", file=sys.stderr)
        return ""
    paragraphs = [p.strip() for p in read(SPEC).split("\n\n")]
    found = []
    for marker in SPEC_PARAGRAPHS:
        hits = [p for p in paragraphs if p.startswith(marker)]
        if not hits:
            print(f"{SPEC.name} has no paragraph starting {marker!r}; the draft runs without it", file=sys.stderr)
            continue
        found.append(hits[-1])
    if not found:
        return ""
    return "The spec's summary of the notation (`docs/Surety Spec v2.md`):\n\n" + "\n\n".join(found)


def docs_text() -> str:
    """What the public docs say about the notation: the spec's two summary
    paragraphs, then the decision notes."""
    parts = []
    for f in sorted(DOCS.glob("Decision - *.md")):
        parts.append(f"`{f.name}`:\n\n{read(f).strip()}")
    decisions = "The decisions the notation rests on:\n\n" + "\n\n".join(parts) if parts else ""
    return "\n\n".join(t for t in (spec_text(), decisions) if t)


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
    n = 2
    while d.exists():  # two runs in one second
        d = RUNS / f"{stamp}-{n}-{stage}"
        n += 1
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


def stage1_file(reply: str) -> tuple[str | None, list[str]]:
    """The file stage 1 writes from a reply: the block, the human parts and
    any property line stripped, the mark first; None when there is no block."""
    block = code_block(reply)
    if block is None:
        return None, []
    text, removed = strip_human_parts(block)
    notes = [f"stripped {r} (the author's, never the model's)" for r in removed]
    # Sentences only: a property line the model wrote beneath a sentence is
    # dropped. Only under the signature: before it, an indented line is the
    # body of a type or a named value.
    kept = []
    in_obligation = False
    for line in text.split("\n"):
        t = line.strip()
        if SIGNATURE.match(line):
            in_obligation = True
        if in_obligation and line.startswith("\t") and not line.startswith("\t\t") and t and not t.startswith(("#", "//", "note")) and not re.match(r"^[a-z_][A-Za-z0-9_]* *=[^=]", t):
            notes.append(f"dropped a property line from stage 1: {t}")
            continue
        kept.append(line)
    text = "\n".join(kept).strip("\n") + "\n"
    if not text.startswith(MARK + "\n"):
        text = MARK + "\n" + text
    return text, notes


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
    out = pathlib.Path(args.out)
    surety = find_surety(args.surety)
    replies = args.reply if args.reply else []
    text, errors, code = "", "", 1
    for round_no in range(1, args.rounds + 1):
        ask = prompt if round_no == 1 else prompt + f"\n\nYour file did not parse. The checker reports:\n\n```\n{errors.strip()}\n```\n\nGive the whole block again, corrected: every type the file names declared in it, the signature, the sentences."
        (d / f"prompt{round_no}.md").write_text(ask, encoding="utf-8")
        if replies:
            reply, usage = read(pathlib.Path(replies[min(round_no, len(replies)) - 1])), {}
        else:
            reply, usage = call_model(SYSTEM_SENTENCES, ask, args.model, args.effort, args.workspace)
        (d / f"reply{round_no}.md").write_text(reply, encoding="utf-8")
        text, notes = stage1_file(reply)
        if text is None:
            print(f"round {round_no}: no ```surety block in the reply; see", d / f"reply{round_no}.md", file=sys.stderr)
            return 1
        for n in notes:
            print(n)
        out.write_text(text, encoding="utf-8")
        (d / f"after{round_no}.sur").write_text(text, encoding="utf-8")
        code, errors = run_surety(surety, "parse", str(out))
        (d / f"parse{round_no}.txt").write_text(errors, encoding="utf-8")
        if usage:
            print(f"round {round_no}: tokens {usage['input']}/{usage['output']}")
        if code == 0:
            break
        print(f"round {round_no}: {errors.strip()}")
    (d / "after.sur").write_text(text, encoding="utf-8")
    sentences = sentences_without_lines(text)
    parse = "ok" if code == 0 else f"still failing after {args.rounds} rounds: {errors.strip()}"
    print(f"wrote {out}: {len(sentences)} sentences, no property lines, no examples; parse: {parse}")
    print(f"run written to {d}")
    return 0 if code == 0 else 1


def cmd_lines(args: argparse.Namespace) -> int:
    path = pathlib.Path(args.file)
    original = read(path)
    surety = find_surety(args.surety)
    code, errors = run_surety(surety, "parse", str(path))
    if code != 0:
        print(f"{path} does not parse, so stage 2 does not start: it only adds lines under the sentences, and the types are stage 1's. The checker reports:\n{errors.strip()}", file=sys.stderr)
        return 2
    d = run_dir("lines")
    (d / "before.sur").write_text(original, encoding="utf-8")
    todo = sentences_without_lines(original)
    if not todo:
        print("every sentence already has a line beneath it")
        return 0
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
        p.add_argument("--rounds", type=int, default=3, help="parse-feedback rounds (default 3)")
        p.add_argument("--reply", action="append", help="a saved reply per round, instead of calling the model")
        if name == "sentences":
            p.add_argument("problem", help="the problem's prose, a markdown or text file")
            p.add_argument("--types", help="the author's types, a .sur file, used verbatim")
            p.add_argument("--out", required=True, help="the .sur file to write")
        else:
            p.add_argument("file", help="the .sur file whose sentences want lines")
    args = ap.parse_args()
    return cmd_sentences(args) if args.stage == "sentences" else cmd_lines(args)


if __name__ == "__main__":
    sys.exit(main())
