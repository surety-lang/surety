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
and `#` sentences, one claim each about the whole relation between the
inputs and the output, with no property line beneath any of them and no
examples; an instance the model wanted to pin goes under the sentences as
`// example wanted: <the case, and what decides it>` with the input as a
literal on a `// in: (...)` line beneath, never with an output, for the
author to turn into an `e:` block (the editor's code action writes the block
with `out:` left empty) or delete; the tool never turns one into an example
itself, and strips an output written after a literal. The file
starts with `// draft`. The types are this stage's
output, so the file is parsed and an error goes back to the model for up to
three rounds, as in stage 2. The author then edits the sentences in the editor.

`lines` refuses a file that does not parse (exit 2, with the error: it only
adds lines under sentences, and the types are stage 1's), and a file with no
`e:` block (exit 2: a line is asked for only against an example that can run
it). `--census`, passed by the census runner alone, skips the second refusal
and says so in the run directory's census.json. Otherwise it asks,
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
import json
import os
import pathlib
import re
import subprocess
import sys

ROOT = pathlib.Path(__file__).resolve().parent.parent
CORPUS = ROOT / "crates" / "surety" / "tests" / "corpus"
DOCS = ROOT / "docs"
# A test points the run directories elsewhere; the author's runs land here.
RUNS = pathlib.Path(os.environ.get("SURETY_DRAFT_RUNS", ROOT / "experiments" / "draft" / "runs"))
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
- Built-ins: `sum(L)`, `len(L)`, `min(a, b)`, `max(a, b)`, `count(L)`, `flatten(L)`, `abs(n)`. `None`, `Ok(x)`, `Err(x)`, `true`, `false` as values.
- Arithmetic on Int: `a + b`, `a - b`, `a * b`, `a % b`, with `*` and `%` binding tighter and parentheses as usual, over Int fields, `len`, `sum`, `count` and literals, so `page * size > len(items)` and `out * 10000 <= sum(items.amount_cents) * (10000 + rate_bps) + 5000` are lines. `%` is the exact remainder with the divisor's sign, as Python's, so `n % 2 == 0` says n is even and a half-even tie is `x % 10000 == 5000`. No `/`: a quotient carries a rounding decision that is a sibling obligation's claim, and a ratio is compared as products. No arithmetic on Float, Time or Text. A difference within a bound is one line: `abs(a - b) <= k`.
- The slice: `xs[a .. b]` is the elements of `xs` from position `a` up to but not including `b`, positions counted from 0, both clamped to the list: a negative `a` clamps to 0 and never counts from the end (Python's `xs[-1:]` means something else), a `b` past the end stops at the end, a `b` below `a` is empty. So `items[page * size .. (page + 1) * size]` is the page, the partial last page and the empty page past the end alike. In a type, `Item[0..3]` is a length bound, a different thing.
- "The output is the error exactly when the condition holds" is `C <-> (exactly_one Err(e) in out: e == x)`, the quantifier last and in parentheses, or, for a list of errors, the named list and `C <-> (exactly_one Err(errs) in out: x in errs)`; never `out == Err(x)`, since `Err(x)` is a pattern, not a value.
- A bodied `any` or `exactly_one` over `out` standing alone claims that side on every input: `exactly_one Ok(p) in out: p.total >= 0` says every output is an `Ok`. A claim about one side alone is `all Ok(p) in out: …`, true where the output is the other side.
- Examples (`e:` / `in:` / `out:`) and `accept` lines are the human author's. Never write them."""

SYSTEM_SENTENCES = """You draft Surety obligations from prose, in the first of two stages: the sentences.

A Surety obligation is a specification a checker enforces: type declarations, a function signature, properties (claims about the relation between the inputs and the output), and examples. In this stage you write the types the problem needs that the author did not give, the named values a claim will need to refer to, the signature, and the claims as prose sentences only: one `#` line per claim about the whole relation between the inputs and the output, each a single claim a reader can judge true or false, nothing beneath any of them, a blank line between sentences (consecutive `#` lines read as one sentence). No property lines, no examples, no accept lines: the author reviews the sentences first, then the notation is asked for separately.

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

	// example wanted: <the case in one line, and what decides it; never the output>
	// in: <the case's input as a literal in the signature's types; never the output>

Every type the file names must be declared in the file. The built-in types are Int, Float, Text, Bool, Id, Time and UUID, with `T[]`, `Option<T>` and `Result<T, E>` over them; there is no library of types beyond these, and no type from another obligation or from the corpus is in scope (a corpus file may name a type declared in a file beside it; the draft stands alone). A type you need that the author did not give, you declare under its own `// drafted` line, with its fields. Prefer an exact integer representation wherever the problem allows: cents over a decimal amount, basis points over a float rate, whole units over fractions. The reason: with a float rate, 125 * 0.1 is 12.500000000000002, so a half-even tie does not exist and every example's expected value depends on the performance's arithmetic rather than on the problem; with cents and basis points the expected value is a fact. A file that names an undeclared type does not parse and comes back to you with the error.

A named value is a declaration, not an example. Draft one beside the types, under its own `// drafted` line, only where a sentence needs a value to refer to: an error case (`age_error: SignupError = { code: "e-age-range", display: "..." }`), a reused record. Give every field, since a performance must return the value byte for byte; take the fields from the problem and invent none beyond them. Examples and accept lines stay the author's, always.

A sentence is a claim about the whole relation between the inputs and the output: what holds on every input, in the fragment's vocabulary. Membership ("every output element is one of the input elements"), order ("the output keeps the input's order", "is sorted by <field>", "is not claimed in any order"), drawn from with left out ("the output is drawn from the input" means every output element is an occurrence of an input element and no occurrence is used twice; "every input element left out of the output has ..." then pins how many times each appears), uniqueness ("no two output elements share <field>"), "<a> comes before <b>", equalities and bounds over `sum` and `len` ("the total is the sum of the amounts", "the output has at most three elements"), "if ... then ..." and "... exactly when ...", with "every", "no", "some" and "exactly one" over a list with a condition. A sentence that fixes a specific input is not a claim and is left out: a length ("if the list has one item ..."), a single value ("if the rate is 0 ..."), a particular combination ("a rate of 0 with one item"). Those are instances, and an instance is an example, which the author writes.

An instance you want pinned goes under the sentences as one line, `// example wanted: <the case in one line, and what decides it>`, for the author to turn into an `e:` block or delete. Write one for each boundary the claims leave to the reader: the empty list, a zero rate, the exact threshold. Give the case and the method, never the expected output: "amounts [125] at 1000 basis points: the tax is exactly 12.5, a half-even tie", not "-> 137". The author decides every output, and a number written here gets copied as if it were decided; on one draft the suggested outputs rounded the tax where the note rounds the total, and the performance was the one that was right. Never write an `e:` block yourself. Beneath each example-wanted line, on a line of its own, give the case's input as a literal in the signature's types, `// in: <literal>`: the bare literal for one input, the tuple `(a, b, c)` in the signature's order for several, written the way the corpus's `e:` blocks write each type (records with every field the input type has, lists in brackets, `None` for an empty option). The literal is checked against the signature as an example's input is, and a literal that does not fit comes back to you with the error. Write the input only: nothing after it, no `->`, no output, in words or in notation.

A sentence implied by another is left out, so that every sentence carries a claim of its own. Two cases. A boundary case of a claim is an example, not a second sentence: "if the list is empty, the total is 0" follows from "the total is the sum of the amounts", so it goes out as an example wanted. A bound implied by a stronger bound is dropped: "the total is at least 0" follows from "the total is at least the sum of the amounts" when every amount is at least 0, so only the stronger one stays. Three to six sentences is the usual size; a problem with error cases has one more per error.

Shapes that have no line, so no sentence takes them: counting occurrences, division (a quotient is a sibling obligation's claim; state a ratio as a comparison of products), text operations beyond its length and whether it contains something. For a text's shape (an email, a phone number) say that a sibling obligation decides it ("the email is well-formed"): a graph reference, never a claim of the fragment. Shapes that have a line and take a sentence each: a threshold or a total as arithmetic over integer fields and lengths ("the page is past the end when page times size is more than the item count"), the sub-list from one position to another ("the page's items are the items from position page times size up to but not including the next page's start"), a difference within a bound ("the total is within half a cent of the subtotal with the tax applied"), and a rounding's tie rule as a remainder ("when the exact total is a half cent, the total is even"). Say what must be in the output, what must not be, what is preserved, and what the order means or that it does not matter.

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


def format_file(surety: str, path: pathlib.Path) -> bool:
    """Write the file back through `surety render`, so no prose line runs past
    the column and every comment stays where it was; the hash is unchanged by
    construction. False when render refuses (the file does not parse)."""
    code, text = run_surety(surety, "render", str(path))
    if code != 0 or not text.strip():
        return False
    path.write_text(text if text.endswith("\n") else text + "\n", encoding="utf-8")
    return True


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


def sentence_groups(lines: list[str]) -> list[tuple[int, int, str]]:
    """Each sentence as (first line index, last line index, text): consecutive
    `#` lines are one sentence, joined by single spaces, as the grammar reads
    them; a blank line, a comment or a line of code ends one."""
    out: list[tuple[int, int, str]] = []
    i = 0
    while i < len(lines):
        if lines[i].strip().startswith("#"):
            start = i
            parts = []
            while i < len(lines) and lines[i].strip().startswith("#"):
                parts.append(lines[i].strip()[1:].strip())
                i += 1
            out.append((start, i - 1, " ".join(x for x in parts if x)))
        else:
            i += 1
    return out


def sentences_without_lines(text: str) -> list[str]:
    """Each sentence whose next non-blank line is not a property line."""
    lines = text.split("\n")
    out = []
    for _, last, sentence in sentence_groups(lines):
        nxt = next((l for l in lines[last + 1:] if l.strip()), "")
        n = nxt.strip()
        if n == "" or n.startswith("#") or n.startswith("//") or n == "e:" or not nxt.startswith("\t"):
            out.append(sentence)
    return out


def insert_lines(text: str, pairs: dict[str, str], comments: dict[str, list[str]] | None = None) -> tuple[str, list[str]]:
    """Put each sentence's line beneath its last `#` line; a `reaching for`
    becomes a comment; a comment the model wrote beneath the pair follows
    as `// drafted: ...`, so the author reads it as the model's."""
    lines = text.split("\n")
    ends = {last: sentence for _, last, sentence in sentence_groups(lines)}
    pairs = dict(pairs)
    out: list[str] = []
    reach: list[str] = []
    for i, line in enumerate(lines):
        out.append(line)
        sentence = ends.get(i)
        if sentence is not None and sentence in pairs:
            proposed = pairs.pop(sentence).strip()
            if proposed.lower().startswith("reaching for:"):
                out.append(f"\t// {proposed}")
                reach.append(proposed[len("reaching for:"):].strip())
            else:
                out.append("\t" + proposed)
            for c in (comments or {}).get(sentence, []):
                out.append(f"\t// drafted: {c}")
    return "\n".join(out), reach


def parse_pairs(block: str) -> tuple[dict[str, str], list[str], dict[str, list[str]]]:
    """`# sentence` followed by one line → {sentence: line}; refused lines
    listed; a `//` comment the model wrote beneath a pair kept for it (an
    echoed `example wanted`, `in:` or reach-for is not a comment of its)."""
    pairs: dict[str, str] = {}
    refused: list[str] = []
    comments: dict[str, list[str]] = {}
    current: str | None = None
    current_done = True
    last: str | None = None
    for line in block.split("\n"):
        t = line.strip()
        if not t:
            continue
        if t.startswith("#"):
            # A sentence wrapped over several `#` lines is one sentence.
            part = t[1:].strip()
            current = f"{current} {part}" if current is not None and not current_done else part
            current_done = False
            last = None
            continue
        current_done = True
        if t.startswith("//"):
            c = t[2:].strip()
            if current is None and last is not None and c and not c.startswith(("example wanted", "in:", "reaching for", "drafted")):
                comments.setdefault(last, []).append(c)
            continue
        if current is None:
            continue
        if t.startswith("accept") or t in ("e:", "in:", "out:") or t.startswith(("in:", "out:")):
            refused.append(t)
            pairs[current] = "reaching for: a line the model wrote as an example or an accept, which are the author's"
        else:
            pairs[current] = t
        last = current
        current = None
    return pairs, refused, comments


def stage1_file(reply: str) -> tuple[str | None, list[str]]:
    """The file stage 1 writes from a reply: the block, the human parts and
    any property line stripped, the mark first; None when there is no block."""
    block = code_block(reply)
    if block is None:
        return None, []
    text, removed = strip_human_parts(block)
    notes = [f"stripped {r} (the author's, never the model's)" for r in removed]
    # The input literal under an example-wanted line is the most a model
    # gives: an output written after it, `->` or `out:`, goes, and is said.
    after_literal = re.compile(r"\s*(->|\bout\s*:).*$")
    cleaned = []
    for line in text.split("\n"):
        if line.strip().startswith("// in:"):
            head, body = line.split("// in:", 1)
            cut = after_literal.search(body)
            if cut:
                notes.append(f"stripped an output after a // in: literal (the author's, never the model's): {body[cut.start():].strip()}")
                line = f"{head}// in:{body[:cut.start()]}"
        cleaned.append(line)
    text = "\n".join(cleaned)
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
    # Each `#` line the model wrote is one sentence: a blank line between
    # consecutive ones keeps them apart, since the grammar reads consecutive
    # `#` lines as one sentence (render wraps a long one into those).
    spaced: list[str] = []
    for line in kept:
        if line.strip().startswith("#") and spaced and spaced[-1].strip().startswith("#"):
            spaced.append("")
        spaced.append(line)
    text = "\n".join(spaced).strip("\n") + "\n"
    if not text.startswith(MARK + "\n"):
        text = MARK + "\n" + text
    return text, notes


def plural(n: int, one: str, many: str | None = None) -> str:
    return f"{n} {one}" if n == 1 else f"{n} {many or one + 's'}"


def stage1_summary(out: pathlib.Path, text: str, parse: str) -> str:
    """What stage 1 wrote, part by part: the declarations drafted, the
    sentences, the examples wanted (listed, since they are the author's to
    act on), and the examples, none by rule."""
    lines = text.split("\n")
    types = values = 0
    for i, line in enumerate(lines):
        if line.strip() == "// drafted":
            nxt = next((l for l in lines[i + 1:] if l.strip()), "")
            if nxt.startswith("type "):
                types += 1
            elif re.match(r"^[a-z_][A-Za-z0-9_]*\s*:", nxt):
                values += 1
    sentences = sentences_without_lines(text)
    # An example-wanted line, the continuation lines render wrapped it into,
    # and the `// in:` literal on the line beneath (the checker's own reading
    # is `wanted.rs`; this one only counts).
    wanted: list[list[str]] = []
    prev = ""
    for line in lines:
        t = line.strip()
        if t.startswith("// example wanted:"):
            wanted.append([t[len("// example wanted:"):].strip(), ""])
        elif wanted and not wanted[-1][1] and prev.startswith("//") and t.startswith("// in:"):
            wanted[-1][1] = t[len("// in:"):].strip()
        elif wanted and not wanted[-1][1] and prev.startswith("//") and t.startswith("//") and not t.startswith(("// drafted", "// draft", "// reaching for", "// example wanted:")):
            wanted[-1][0] += " " + t[2:].strip()
        prev = t
    literals = sum(1 for _, lit in wanted if lit)
    s = [f"wrote {out}: parse {parse}"]
    s.append(f"  drafted: {plural(types, 'type')}, {plural(values, 'named value')}; each under // drafted, read first")
    s.append(f"  sentences: {len(sentences)}, no property lines yet; stage 2 asks for them")
    s.append(f"  examples wanted: {len(wanted)}, as // example wanted lines, {literals} with its input on a // in: line; yours to turn into e: blocks (the editor's code action writes one, the output left to you) or delete")
    for case, lit in wanted:
        s.append(f"    {case}" + (f"\n      in: {lit}" if lit else ""))
    with_output = [case for case, _ in wanted if "->" in case]
    if with_output:
        s.append(f"  {plural(len(with_output), 'example-wanted line carries', 'example-wanted lines carry')} an output (->): the output is yours to decide, not the model's")
    # Prose is not mechanically strippable: a case that states its output in
    # words is flagged, to be read before an output is copied from it.
    in_words = [case for case, _ in wanted if "the output is" in case.lower() or "is empty" in case.lower()]
    if in_words:
        s.append(f"  {plural(len(in_words), 'example-wanted line states', 'example-wanted lines state')} an output in words; read before accepting, the output is yours to decide:")
        for case in in_words:
            s.append(f"    {case}")
    s.append("  examples: none, by rule")
    return "\n".join(s)


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
    out.parent.mkdir(parents=True, exist_ok=True)
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
            format_file(surety, out)
            text = read(out)
            break
        print(f"round {round_no}: {errors.strip()}")
    (d / "after.sur").write_text(text, encoding="utf-8")
    parse = "ok" if code == 0 else f"still failing after {args.rounds} rounds: {errors.strip()}"
    print(stage1_summary(out, text, parse))
    print(f"run written to {d}")
    return 0 if code == 0 else 1


def wanted_left(text: str) -> list[tuple[str, str]]:
    """The example-wanted lines still in a file, each with its `// in:`
    literal when it has one: cases not yet turned into `e:` blocks."""
    out: list[tuple[str, str]] = []
    prev = ""
    for line in text.split("\n"):
        t = line.strip()
        if t.startswith("// example wanted:"):
            out.append((t[len("// example wanted:"):].strip(), ""))
        elif out and not out[-1][1] and prev.startswith("//") and t.startswith("// in:"):
            out[-1] = (out[-1][0], t[len("// in:"):].strip())
        elif out and not out[-1][1] and prev.startswith("//") and t.startswith("//") and not t.startswith(("// drafted", "// draft", "// reaching for", "// example wanted:")):
            out[-1] = (out[-1][0] + " " + t[2:].strip(), "")
        prev = t
    return out


def diagnose(surety: str, path: pathlib.Path, text: str) -> list[dict]:
    """`surety parse --json` on `text` written at `path`: every error and
    hint, each with the edit it carries when the checker verified one."""
    path.write_text(text, encoding="utf-8")
    code, out = run_surety(surety, "parse", "--json", str(path))
    found = []
    for line in out.splitlines():
        try:
            j = json.loads(line)
        except ValueError:
            continue
        if isinstance(j, dict) and j.get("kind") in ("error", "hint"):
            found.append(j)
    # A failure that printed no error object is still a failure.
    if code != 0 and not any(d["kind"] == "error" for d in found):
        found.append({"kind": "error", "line": 0, "code": "parse", "message": out.strip()})
    return found


def reported(d: dict) -> str:
    return f"line {d['line']}: {d['message']} ({d['code']})"


def on_line(text: str, d: dict, proposed: str) -> bool:
    """Whether the error or hint is on the proposed line, as inserted."""
    lines = text.split("\n")
    n = d["line"] - 1
    return 0 <= n < len(lines) and lines[n].strip() == proposed.strip().split("\n")[0].strip()


def mended(text: str, d: dict, proposed: str) -> str | None:
    """The proposed line as the checker's verified edit has it, when the
    error carries one and it replaces exactly that line."""
    e = d.get("edit")
    if not e or e["from"] != e["to"] or not on_line(text, d, proposed):
        return None
    return e["text"].strip()


def cmd_lines(args: argparse.Namespace) -> int:
    path = pathlib.Path(args.file)
    original = read(path)
    surety = find_surety(args.surety)
    code, errors = run_surety(surety, "parse", str(path))
    if code != 0:
        print(f"{path} does not parse, so stage 2 does not start: it only adds lines under the sentences, and the types are stage 1's. The checker reports:\n{errors.strip()}", file=sys.stderr)
        return 2
    # A line is asked for only against an example that can run it. The
    # census runner alone passes --census, which skips this so the census's
    # second half runs on the terms of its first, drafted before the gate.
    has_example = any(line.strip() == "e:" for line in original.split("\n"))
    if not has_example and not args.census:
        print(f"{path} has no example, so stage 2 does not start: a line is asked for only against an example that can run it, and a line with none to run it says nothing yet. Each `// example wanted:` line is a case to turn into one: the editor's code action writes the block with the input filled in, and the output is yours to type.", file=sys.stderr)
        for case, lit in wanted_left(original):
            print(f"  example wanted: {case}" + (f"\n    in: {lit}" if lit else ""), file=sys.stderr)
        return 2
    d = run_dir("lines")
    (d / "before.sur").write_text(original, encoding="utf-8")
    if args.census:
        gate = "passed" if has_example else "skipped"
        (d / "census.json").write_text(json.dumps({"census": True, "example_gate": gate}) + "\n", encoding="utf-8")
        print(f"census run (--census): the example gate is {gate}; the author's flow never passes this flag")
    todo = sentences_without_lines(original)
    if not todo:
        print("every sentence already has a line beneath it")
        return 0
    header = [f"The obligation, with its author's sentences:\n\n```\n{original}```",
              "The sentences with no property line beneath them, one line each wanted:\n\n" + "\n".join(f"# {s}" for s in todo),
              docs_text(), corpus_text(args.withhold)]
    prompt = "\n\n".join(p for p in header if p)
    # A line that parsed is kept across rounds; a retry asks again only for
    # the sentences whose line failed, each with what the checker said. An
    # error whose edit the checker verified is mended by the tool before
    # the model is asked at all; a hint keeps its line and goes back once,
    # as an error does, for the model to keep the line or give a better one.
    kept: dict[str, str] = {}
    comments: dict[str, list[str]] = {}
    failing: dict[str, tuple[str, str]] = {}
    hinted: dict[str, tuple[str, str]] = {}
    pending = list(todo)
    rounds: list[dict] = []
    replies = args.reply if args.reply else []
    for round_no in range(1, args.rounds + 1):
        if round_no == 1:
            ask = prompt
        else:
            ask = prompt + "\n\nEvery other line parsed and is kept."
            again = "\n\n".join(f"# {s}\n\t{failing[s][0]}\nThe checker reports: {failing[s][1]}" for s in pending if s in failing)
            if again:
                ask += f" These did not; give the pairs again for these sentences only, corrected:\n\n{again}"
            again = "\n\n".join(f"# {s}\n\t{hinted[s][0]}\nThe checker hints: {hinted[s][1]}" for s in pending if s not in failing)
            if again:
                ask += f"\n\nThese parse and are kept as they are unless you give a better pair; the checker has a hint on each:\n\n{again}"
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
        pairs, refused, said = parse_pairs(block)
        for r in refused:
            print(f"refused a line the model wrote (the author's): {r}")
        asked, pending = pending, []
        mends: list[dict] = []
        hints: dict[str, str] = {}
        for sentence in asked:
            if sentence in hinted and sentence not in pairs:
                continue  # the hinted line stands
            proposed = pairs.get(sentence, "reaching for: a line the model gave no answer for")
            if sentence in said:
                comments[sentence] = said[sentence]
            if proposed.lower().startswith("reaching for:"):
                kept[sentence] = proposed
                failing.pop(sentence, None)
                continue
            # Each line on its own under its sentence, the kept ones beside it.
            text, _ = insert_lines(original, {**kept, sentence: proposed})
            found = diagnose(surety, path, text)
            errors = [d for d in found if d["kind"] == "error"]
            fixed = mended(text, errors[0], proposed) if errors else None
            if fixed is not None:
                text, _ = insert_lines(original, {**kept, sentence: fixed})
                again = diagnose(surety, path, text)
                if not any(d["kind"] == "error" for d in again):
                    mends.append({"sentence": sentence, "code": errors[0]["code"], "was": proposed, "now": fixed})
                    print(f"round {round_no}: mended by the checker's verified edit ({errors[0]['code']}): {proposed}  ->  {fixed}")
                    proposed, found, errors = fixed, again, []
            if errors:
                failing[sentence] = (proposed, reported(errors[0]))
                pending.append(sentence)
                continue
            kept[sentence] = proposed
            failing.pop(sentence, None)
            hint = next((d for d in found if d["kind"] == "hint" and on_line(text, d, proposed)), None)
            if hint is not None and sentence not in hinted:
                hinted[sentence] = (proposed, reported(hint))
                hints[sentence] = reported(hint)
                pending.append(sentence)
        text, reach = insert_lines(original, kept, comments)
        rounds.append({
            "round": round_no,
            "asked": asked,
            "errors": {s: failing[s][1] for s in pending if s in failing},
            "hints": hints,
            "mended": mends,
            "reach": len(reach),
        })
        if usage:
            print(f"round {round_no}: tokens {usage['input']}/{usage['output']}")
        for s in pending:
            print(f"round {round_no}: {failing[s][1] if s in failing else 'hint: ' + hinted[s][1]}")
        if not pending:
            break
    # Lines still failing after the last round go in as written, so the
    # author sees them where they stand.
    text, reach = insert_lines(original, {**kept, **{s: failing[s][0] for s in pending if s in failing}}, comments)
    path.write_text(text, encoding="utf-8")
    code, errors = run_surety(surety, "parse", str(path))
    (d / "rounds.json").write_text(json.dumps(rounds, indent=1) + "\n", encoding="utf-8")
    if code == 0 and format_file(surety, path):
        text = read(path)
    (d / "after.sur").write_text(text, encoding="utf-8")
    code, report = run_surety(surety, "mutants", str(path))
    (d / "mutants.txt").write_text(report, encoding="utf-8")
    print(report.rstrip())
    if reach:
        print("\nreaching for (for the sheet):")
        for r in reach:
            print(f"  - {r}")
    drafted = [(sentence, c) for sentence in todo for c in comments.get(sentence, [])]
    if drafted:
        print("\nthe model's comments, kept as // drafted: beneath their lines, read them first:")
        for sentence, c in drafted:
            print(f"  - {c}  (under: {sentence})")
    left = wanted_left(text)
    if left:
        print(f"\nexamples wanted still in the file: {len(left)}, {sum(1 for _, lit in left if lit)} with a // in: literal not yet turned into an e: block:")
        for case, lit in left:
            print(f"  - {case}" + (f"\n      in: {lit}" if lit else ""))
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
            p.add_argument("--census", action="store_true",
                           help="for the census runner alone: ask for lines on a file with no example, which the author's flow refuses, "
                                "so the census's second half runs on the terms of its first; recorded as census.json in the run directory")
    args = ap.parse_args()
    return cmd_sentences(args) if args.stage == "sentences" else cmd_lines(args)


if __name__ == "__main__":
    sys.exit(main())
