#!/usr/bin/env python3
"""The closed-loop run (Spec v2's first gate, in its mechanical half).

Hand each obligation in a corpus directory to a model, take back a Python
performance, run `surety check` on it, and write a report. The human half of
the gate (does the evidence table replace reading for a reviewer) is read
off the report by a person.

    python3 experiments/closed_loop.py                 # all eleven, one round
    python3 experiments/closed_loop.py --only p2,p8    # a subset
    python3 experiments/closed_loop.py --rounds 2      # feed failures back once
    python3 experiments/closed_loop.py --dry-run       # write the prompts, call nothing

Credentials come from the Anthropic SDK's own resolution (ANTHROPIC_API_KEY,
ANTHROPIC_AUTH_TOKEN, or an `ant auth login` profile). Nothing here reads a
key. Each run writes to experiments/closed-loop/runs/<stamp>-<model>/:
the copied .sur files, one .py per obligation, the raw model replies, the
check reports, and summary.md / summary.json. A dry run writes under target/.
"""

from __future__ import annotations

import argparse
import datetime as dt
import json
import pathlib
import re
import subprocess
import sys
import time

ROOT = pathlib.Path(__file__).resolve().parent.parent
CORPUS = ROOT / "crates" / "surety" / "tests" / "corpus"
RUNS = ROOT / "experiments" / "closed-loop" / "runs"
ORDER = ["p1", "p2", "p3", "p4", "p5a", "p5b", "p6", "p7", "p8", "apply_diff", "p14"]
BUILTINS = {"sum", "len", "min", "max", "count", "flatten"}
KEYWORDS = {"all", "any", "none", "exactly_one", "where", "else", "and", "or", "not", "in", "leftover",
            "before", "sorted_by", "unordered", "unique", "same_order_as", "same_elements_as", "drawn_from"}
MODEL = "claude-opus-5-5"

SYSTEM = """You write performances for Surety obligations.

A Surety obligation is a specification: type declarations, a function signature, properties (claims about the relation between the inputs and the output), and examples. You write the implementation, called a performance, in Python. A checker runs it against every example and every property and never reads your source, only the values crossing the boundary.

The boundary contract, which the checker enforces on every call:
- The module defines a function named exactly as the signature, taking the inputs as positional parameters in signature order.
- A record is a dict with exactly the declared fields, no more and no fewer. `type EnrichedOrder = { ...Order, user_name: Option<Text> }` means every field of Order plus user_name.
- A list `T[]` is a Python list. `T[0..3]` bounds its length; `T[unique f]` means no two elements share field f.
- `Option<T>` is the bare value or None. There is no Some.
- `Result<T, E>` is {"ok": v} or {"err": e}. `Err([...])` in an example is {"err": [...]}.
- `Int` is an int, `Float` a float, `Text` and `Id` are str, `Time` is a str in RFC 3339 form exactly as given.
- Named values (`email_error: ValidationError = { ... }`) are literals the obligation reuses; where your output must contain one, return a dict with exactly those fields and values.
- `out` in a property is your function's return value. `unordered` on a list means the checker does not care about its order; otherwise order matters and the examples show it.
- `all x leftover in L` ranges over the input list L minus the elements that appear in out.

Names called in a property that no obligation defines (for example well_formed_email) are graph references the checker resolves to a function of that name in your module: define them. Names that another obligation defines (for example apply_diff, called from snapshot_diff's properties) are supplied by that obligation's own performance; do not define them.

Rules: standard library only, no I/O, no randomness, no global state between calls, deterministic. Do not print. Reply with one ```python code block containing the whole module and nothing else outside it."""


def read(p: pathlib.Path) -> str:
    return p.read_text(encoding="utf-8")


def defined_names(corpus: pathlib.Path) -> tuple[set[str], set[str]]:
    """(function signatures, abbreviations) across the corpus. Abbreviations are
    evaluated by the checker and never need a Python definition."""
    signatures: set[str] = set()
    abbrevs: set[str] = set()
    for f in corpus.glob("*.sur"):
        for line in read(f).splitlines():
            m = re.match(r"^([A-Za-z_][A-Za-z_0-9]*)\s*\([^)]*\)\s*->", line)
            if m:
                signatures.add(m.group(1))
                continue
            m = re.match(r"^\t?([A-Za-z_][A-Za-z_0-9]*)\s*(\([^)]*\))?\s*=\s", line)
            if m:
                abbrevs.add(m.group(1))
    return signatures, abbrevs


def graph_refs(text: str, signatures: set[str], abbrevs: set[str], own: str) -> tuple[list[str], list[str]]:
    """Names called in properties: (undefined anywhere, defined by another obligation)."""
    undefined, elsewhere = [], []
    for line in text.splitlines():
        if not line.startswith("\t") or line.lstrip().startswith(("#", "//", "e:", "in:", "out:", "note")):
            continue
        for name in re.findall(r"\b([A-Za-z_][A-Za-z_0-9]*)\s*\(", line):
            if name in BUILTINS or name in KEYWORDS or name in abbrevs or name in ("Ok", "Err") or name == own:
                continue
            target = elsewhere if name in signatures else undefined
            if name not in target:
                target.append(name)
    return undefined, elsewhere


def signature_name(text: str) -> str:
    for line in text.splitlines():
        m = re.match(r"^([A-Za-z_][A-Za-z_0-9]*)\s*\(.*\)\s*->", line)
        if m:
            return m.group(1)
    raise SystemExit("no signature found")


def referenced_types(text: str, corpus: pathlib.Path, own: pathlib.Path) -> list[pathlib.Path]:
    """Sibling obligations that define a type or function this one names: one hop."""
    own_defs = {m.group(1) for m in re.finditer(r"^type\s+([A-Za-z_][A-Za-z_0-9]*)", text, re.M)}
    wanted = set(re.findall(r"\b([A-Z][A-Za-z_0-9]*)\b", text)) - own_defs
    wanted -= {"Int", "Float", "Text", "Bool", "Id", "Time", "UUID", "Option", "Result", "Ok", "Err", "None"}
    hops = []
    for f in sorted(corpus.glob("*.sur")):
        if f == own:
            continue
        defs = set(re.findall(r"^type\s+([A-Za-z_][A-Za-z_0-9]*)", read(f), re.M))
        fns = set(re.findall(r"^([A-Za-z_][A-Za-z_0-9]*)\s*\(.*\)\s*->", read(f), re.M))
        if defs & wanted or fns & set(re.findall(r"\b([a-z_][a-z_0-9]*)\s*\(", text)):
            hops.append(f)
    return hops


def build_prompt(name: str, corpus: pathlib.Path) -> tuple[str, dict]:
    own = corpus / f"{name}.sur"
    text = read(own)
    fn = signature_name(text)
    signatures, abbrevs = defined_names(corpus)
    undefined, elsewhere = graph_refs(text, signatures, abbrevs, fn)
    hops = referenced_types(text, corpus, own)
    parts = [f"Obligation `{name}.sur`:\n\n```\n{text}```"]
    for h in hops:
        parts.append(f"Referenced obligation `{h.name}` (for its types; its performance is supplied separately):\n\n```\n{read(h)}```")
    musts = [f"`{fn}`"] + [f"`{u}`" for u in undefined]
    parts.append("Your module must define: " + ", ".join(musts) + ".")
    if elsewhere:
        parts.append("Do not define: " + ", ".join(f"`{e}`" for e in elsewhere) + " (another obligation's performance supplies it).")
    meta = {"function": fn, "must_define": [fn] + undefined, "supplied_elsewhere": elsewhere, "hops": [h.name for h in hops]}
    return "\n\n".join(parts), meta


def extract_module(reply: str) -> str | None:
    m = re.search(r"```python\s*\n(.*?)```", reply, re.S)
    if m:
        return m.group(1)
    m = re.search(r"```\s*\n(.*?)```", reply, re.S)
    return m.group(1) if m else None


def run_check(surety: pathlib.Path, sur: pathlib.Path, perf: pathlib.Path) -> tuple[int, str]:
    proc = subprocess.run([str(surety), "check", str(sur), str(perf)], capture_output=True, text=True, timeout=600)
    return proc.returncode, proc.stdout + proc.stderr


def tally(report: str) -> dict:
    counts = {"pass": 0, "fail": 0, "error": 0, "boundary": 0, "skip": 0}
    killers = []
    for line in report.splitlines():
        s = line.strip()
        if s.startswith("pass "):
            counts["pass"] += 1
        elif s.startswith("FAIL "):
            counts["fail"] += 1
            killers.append(s[5:].split("\n")[0][:160])
        elif s.startswith("ERROR "):
            counts["error"] += 1
            killers.append(s[6:][:160])
        elif s.startswith("BOUNDARY "):
            counts["boundary"] += 1
            killers.append(s[9:][:160])
        elif s.startswith("skip "):
            counts["skip"] += 1
    counts["killers"] = killers
    return counts


def call_model(client, model: str, effort: str, messages: list, fallbacks: bool):
    import anthropic  # noqa: F401  (the client is already constructed)

    kwargs = dict(
        model=model,
        max_tokens=32000,
        system=[{"type": "text", "text": SYSTEM, "cache_control": {"type": "ephemeral"}}],
        messages=messages,
        thinking={"type": "adaptive"},
        output_config={"effort": effort},
    )
    if fallbacks:
        # Server-side fallback on a safety refusal; opt out with --no-fallbacks.
        kwargs["extra_headers"] = {"anthropic-beta": "server-side-fallback-2026-07-01"}
        kwargs["extra_body"] = {"fallbacks": "default"}
    with client.messages.stream(**kwargs) as stream:
        return stream.get_final_message()


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("--only", help="comma-separated fixture names")
    ap.add_argument("--rounds", type=int, default=1, help="attempts per obligation; failures are fed back")
    ap.add_argument("--model", default=MODEL)
    ap.add_argument("--effort", default="high", choices=["low", "medium", "high", "xhigh", "max"])
    ap.add_argument("--surety", default=str(ROOT / "target" / "release" / "surety"))
    ap.add_argument("--corpus", default=str(CORPUS))
    ap.add_argument("--dry-run", action="store_true", help="write prompts, call no model")
    ap.add_argument("--no-fallbacks", action="store_true", help="do not request server-side refusal fallbacks")
    args = ap.parse_args()

    corpus = pathlib.Path(args.corpus)
    surety = pathlib.Path(args.surety)
    names = args.only.split(",") if args.only else ORDER
    stamp = dt.datetime.now().strftime("%Y%m%d-%H%M%S")
    base = ROOT / "target" / "closed-loop" if args.dry_run else RUNS
    run_dir = base / f"{stamp}-{args.model}{'-dry' if args.dry_run else ''}"
    run_dir.mkdir(parents=True)
    # The run directory is the graph and the module directory: every .sur is
    # copied in, so graph references resolve to the model's own performances.
    for f in sorted(corpus.glob("*.sur")):
        (run_dir / f.name).write_text(read(f), encoding="utf-8")

    client = None
    if not args.dry_run:
        import anthropic

        client = anthropic.Anthropic()
    if not args.dry_run and not surety.exists():
        print(f"no surety binary at {surety}; run cargo build --release", file=sys.stderr)
        return 2

    summary = {"model": args.model, "effort": args.effort, "rounds": args.rounds, "started": stamp, "obligations": []}
    for name in names:
        prompt, meta = build_prompt(name, corpus)
        (run_dir / f"{name}.prompt.md").write_text(prompt, encoding="utf-8")
        entry = {"name": name, **meta, "attempts": []}
        summary["obligations"].append(entry)
        if args.dry_run:
            print(f"{name}: prompt written ({len(prompt)} chars), must define {meta['must_define']}")
            continue
        messages = [{"role": "user", "content": prompt}]
        for attempt in range(1, args.rounds + 1):
            t0 = time.time()
            try:
                msg = call_model(client, args.model, args.effort, messages, not args.no_fallbacks)
            except Exception as e:  # noqa: BLE001  (recorded, the run goes on)
                entry["attempts"].append({"round": attempt, "error": f"{type(e).__name__}: {e}"})
                print(f"{name} round {attempt}: API error {type(e).__name__}: {e}")
                break
            seconds = round(time.time() - t0, 1)
            text = "".join(b.text for b in msg.content if b.type == "text")
            (run_dir / f"{name}.round{attempt}.reply.md").write_text(text, encoding="utf-8")
            usage = {"input": msg.usage.input_tokens, "output": msg.usage.output_tokens,
                     "cache_read": getattr(msg.usage, "cache_read_input_tokens", 0) or 0}
            record = {"round": attempt, "seconds": seconds, "stop_reason": msg.stop_reason, "usage": usage}
            if msg.stop_reason == "refusal":
                record["refusal"] = getattr(msg, "stop_details", None) and msg.stop_details.__dict__
            module = extract_module(text)
            if module is None:
                record["error"] = "no code block in the reply"
                entry["attempts"].append(record)
                print(f"{name} round {attempt}: no code block")
                break
            perf = run_dir / f"{name}.py"
            perf.write_text(module, encoding="utf-8")
            code, report = run_check(surety, run_dir / f"{name}.sur", perf)
            (run_dir / f"{name}.round{attempt}.report.txt").write_text(report, encoding="utf-8")
            counts = tally(report)
            record.update({"exit": code, **counts})
            entry["attempts"].append(record)
            verdict = "PASS" if code == 0 else "fail"
            print(f"{name} round {attempt}: {verdict}  pass={counts['pass']} fail={counts['fail']} error={counts['error']} boundary={counts['boundary']}  {seconds}s  in={usage['input']} out={usage['output']}")
            if code == 0 or attempt == args.rounds:
                break
            # v2's inner loop: the counterexample goes back, the obligation does not change.
            messages.append({"role": "assistant", "content": msg.content})
            messages.append({"role": "user", "content": "The checker reports:\n\n```\n" + report + "```\n\nFix the performance. Reply with the whole module in one ```python block."})
        entry["passed"] = bool(entry["attempts"]) and entry["attempts"][-1].get("exit") == 0

    (run_dir / "summary.json").write_text(json.dumps(summary, indent=2), encoding="utf-8")
    lines = [f"# Closed-loop run {stamp}", "", f"model `{args.model}`, effort `{args.effort}`, rounds {args.rounds}", "",
             "| Obligation | Result | Round | Pass | Fail | Error | Boundary | Seconds | Tokens in/out | First killer |",
             "| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |"]
    passed = 0
    for e in summary["obligations"]:
        if not e["attempts"]:
            lines.append(f"| {e['name']} | not run | | | | | | | | |")
            continue
        a = e["attempts"][-1]
        if "exit" not in a:
            lines.append(f"| {e['name']} | error | {a['round']} | | | | | {a.get('seconds', '')} | | {a.get('error', '')} |")
            continue
        passed += e["passed"]
        killer = (a["killers"][0] if a["killers"] else "").replace("|", "\\|")
        lines.append(f"| {e['name']} | {'pass' if e['passed'] else 'fail'} | {a['round']} | {a['pass']} | {a['fail']} | {a['error']} | {a['boundary']} | {a['seconds']} | {a['usage']['input']}/{a['usage']['output']} | {killer} |")
    lines += ["", f"{passed} of {len(summary['obligations'])} obligations passed."]
    (run_dir / "summary.md").write_text("\n".join(lines) + "\n", encoding="utf-8")
    print(f"\nrun written to {run_dir}")
    if not args.dry_run:
        print(f"{passed} of {len(names)} passed")
    return 0


if __name__ == "__main__":
    sys.exit(main())
