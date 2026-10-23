#!/usr/bin/env python3
"""Run the draft, both stages, over the census problems; checkpoint to
results.jsonl so a stopped run resumes.

    python3 experiments/census/run_census.py [--jobs N] [--limit N] [--only ID ...]
        [--retry-failed] [--model M] [--effort E]

For each function in functions.jsonl with no complete record in results.jsonl:
stage 1 (`draft.py sentences <problem> --out out/<id>.sur`), then, when the
file parses, stage 2 (`draft.py lines out/<id>.sur --census`). `--census` is
draft.py's census-only flag, passed by this runner alone: it skips the example
gate the author's flow keeps (no census draft has an example), so the second
half runs on the terms of the first, drafted before the gate. draft.py is run
as a subprocess with the prebuilt checker passed by `--surety`. Each
stage's stdout and stderr go to logs/<id>.<stage>.txt. One JSON record per
function is appended to results.jsonl:

    id, qualname, source, complete, error, prompt_sha256, census_flag,
    stage1: exit, rounds, tokens_in, tokens_out, parse_ok, sentences,
            examples_wanted, notes, round_errors, last_error, run_dir, seconds
    stage2: exit, rounds, tokens_in, tokens_out, parse_ok, lines, reach,
            sentences [{text, line, reach}], refused, mutants (verbatim),
            mutants_summary {applicable, of, slipped, caught, not_judged,
            no_passing, underdetermined, gaps, accepted}, example_gate,
            round_errors, round_hints, mended, round_reach, last_error,
            run_dir, seconds

prompt_sha256 is the hash of the prompt material every problem shares: both
stages' system prompts (the notation block in each), the docs and the corpus
draft.py hands over. Two records with one hash were drafted from one prompt.
census_flag says stage 2 was asked with `--census`; stage2.example_gate is what
the run directory's census.json says of the gate, "skipped" for a draft with
no example. Records from the first half, run before the gate, have neither.

round_errors has one entry per round, the checker's report, or null for the
round that parsed. For stage 2 it is read from the run's rounds.json, one
error per sentence still failing, as draft.py records it since slice 6; a run
directory from before has parse<k>.txt files instead, read as before.
round_hints is each round's hints by sentence, and mended the lines the
checker's verified edit mended before the model was asked again. round_reach
is how many sentences stand as reach-fors after each round, from rounds.json:
a retry asks again only for the failing sentences, so a line kept in one round
stays. (Before slice 6 a retry regenerated every pair, and round_reach counted
each reply's own reach-fors.) stage2.parse_ok is read from the last round, and
stage2.lines is null when the file never parsed.

Ctrl-C starts nothing new: queued functions are cancelled, a function in
flight starts no further stage, and only complete records are appended, so a
resume runs the interrupted ones again.

`--reply1`/`--reply2` pass saved replies to draft.py (its own `--reply`), for
testing the mechanics on one problem without a model.
"""
from __future__ import annotations

import argparse
import concurrent.futures as cf
import datetime as dt
import hashlib
import importlib.util
import json
import pathlib
import re
import signal
import subprocess
import sys
import threading
import time

ROOT = pathlib.Path(__file__).resolve().parent.parent.parent
CENSUS = ROOT / "experiments" / "census"
DRAFT = ROOT / "experiments" / "draft.py"
FUNCTIONS = CENSUS / "functions.jsonl"
RESULTS = CENSUS / "results.jsonl"
PROBLEMS = CENSUS / "problems"
OUT = CENSUS / "out"
LOGS = CENSUS / "logs"
SURETY = ROOT / "target" / "release" / "surety"

LOCK = threading.Lock()
STAGE_TIMEOUT = 45 * 60

ROUND = re.compile(r"^round (\d+): tokens (\d+)/(\d+)", re.M)
ROUND_ANY = re.compile(r"^round (\d+):", re.M)
RUN_DIR = re.compile(r"^run written to (.+)$", re.M)
PARSE_OK = re.compile(r": ok \S+ props=")

STOP = threading.Event()


class Stopped(Exception):
    """An interrupt reached this function before its record was complete."""


def load_draft():
    """draft.py as a module, for its own reading of a stage-2 reply."""
    spec = importlib.util.spec_from_file_location("draft", DRAFT)
    mod = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(mod)
    return mod


draft = load_draft()


def prompt_sha256() -> str:
    """The prompt material every problem shares, hashed (see the module's
    docstring)."""
    parts = [draft.SYSTEM_SENTENCES, draft.SYSTEM_LINES, draft.docs_text(), draft.corpus_text([])]
    return hashlib.sha256("\0".join(parts).encode("utf-8")).hexdigest()


PROMPT = prompt_sha256()


def load_results() -> dict[str, dict]:
    done: dict[str, dict] = {}
    if RESULTS.exists():
        for line in RESULTS.read_text(encoding="utf-8").splitlines():
            if line.strip():
                r = json.loads(line)
                done[r["id"]] = r
    return done


def append_result(rec: dict) -> None:
    with LOCK:
        with RESULTS.open("a", encoding="utf-8") as f:
            f.write(json.dumps(rec) + "\n")


LAUNCH_LOCK = threading.Lock()
LAST_LAUNCH = [0.0]


def run(cmd: list[str], log: pathlib.Path) -> tuple[int, str, float]:
    """Start draft.py, no two starts within the same second: its run
    directory is named by a one-second stamp, and two starts in one second
    race on it (draft.py is not changed here). After an interrupt nothing starts."""
    t0 = time.time()
    with LAUNCH_LOCK:
        wait = LAST_LAUNCH[0] + 1.2 - time.time()
        if wait > 0:
            time.sleep(wait)
        if STOP.is_set():
            raise Stopped
        LAST_LAUNCH[0] = time.time()
        p = subprocess.Popen(cmd, cwd=ROOT, stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True)
    try:
        out, err = p.communicate(timeout=STAGE_TIMEOUT)
        code, text = p.returncode, out + ("\n--- stderr ---\n" + err if err.strip() else "")
    except subprocess.TimeoutExpired:
        p.kill()
        out, err = p.communicate()
        code, text = -9, f"TIMEOUT after {STAGE_TIMEOUT}s\n" + (out or "") + "\n" + (err or "")
    secs = time.time() - t0
    log.write_text(f"$ {' '.join(cmd)}\nexit {code} in {secs:.0f}s\n\n{text}", encoding="utf-8")
    return code, text, secs


def rounds_and_tokens(text: str) -> tuple[int, int, int]:
    rounds = max((int(m.group(1)) for m in ROUND_ANY.finditer(text)), default=0)
    tin = sum(int(m.group(2)) for m in ROUND.finditer(text))
    tout = sum(int(m.group(3)) for m in ROUND.finditer(text))
    return rounds, tin, tout


def last_error(text: str) -> str | None:
    errs = [m for m in re.finditer(r"^round (\d+): (?!tokens|mended by|hint: )(.+)$", text, re.M)]
    return errs[-1].group(2).strip() if errs else None


def round_errors(d: pathlib.Path) -> list[str | None]:
    """Each round's parse report from the run directory, None for one that parsed."""
    out: list[str | None] = []
    k = 1
    while (d / f"parse{k}.txt").exists():
        report = (d / f"parse{k}.txt").read_text(encoding="utf-8").strip()
        out.append(None if PARSE_OK.search(report) else report)
        k += 1
    return out


def stage2_rounds(d: pathlib.Path) -> tuple[list[str | None], list[dict], list[dict]]:
    """Stage 2's rounds from rounds.json: each round's errors (None for a
    round with none left), its hints, and every mend. A run directory from
    before slice 6 has parse<k>.txt files instead."""
    f = d / "rounds.json"
    if not f.exists():
        return round_errors(d), [], []
    rounds = json.loads(f.read_text(encoding="utf-8"))
    errors = ["\n".join(r["errors"].values()) if r.get("errors") else None for r in rounds]
    hints = [r.get("hints", {}) for r in rounds]
    mended = [m for r in rounds for m in r.get("mended", [])]
    return errors, hints, mended


def example_gate(d: pathlib.Path) -> str | None:
    """What a `--census` run directory's census.json says of the example gate."""
    f = d / "census.json"
    return json.loads(f.read_text(encoding="utf-8")).get("example_gate") if f.exists() else None


def round_reach(d: pathlib.Path) -> list[int | None]:
    """How many sentences stand as reach-fors after each round, from
    rounds.json; for a run directory from before slice 6, how many each
    round's reply left, read as draft.py reads it, None for a reply with no
    block."""
    if (d / "rounds.json").exists():
        return [r.get("reach") for r in json.loads((d / "rounds.json").read_text(encoding="utf-8"))]
    before = d / "before.sur"
    if not before.exists():
        return []
    todo = draft.sentences_without_lines(before.read_text(encoding="utf-8"))
    out: list[int | None] = []
    k = 1
    while (d / f"reply{k}.md").exists():
        block = draft.code_block((d / f"reply{k}.md").read_text(encoding="utf-8"))
        if block is None:
            out.append(None)
        else:
            pairs = draft.parse_pairs(block)[0]
            out.append(sum(1 for s in todo if pairs.get(s, "reaching for:").strip().lower().startswith("reaching for:")))
        k += 1
    return out


def sentence_groups(lines: list[str]) -> list[tuple[int, int, str]]:
    out = []
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


def read_after(path: pathlib.Path) -> list[dict]:
    """Each sentence with what sits beneath it: a property line, a reach-for
    comment (joined across the renderer's wrapped continuation lines), or nothing."""
    lines = path.read_text(encoding="utf-8").split("\n")
    out = []
    for _, last, sentence in sentence_groups(lines):
        rec = {"text": sentence, "line": None, "reach": None}
        j = last + 1
        while j < len(lines) and not lines[j].strip():
            j += 1
        if j < len(lines):
            nxt = lines[j]
            t = nxt.strip()
            if t.startswith("// reaching for:"):
                reach = t[len("// reaching for:"):].strip()
                k = j + 1
                while k < len(lines) and lines[k].strip().startswith("//") and not lines[k].strip().startswith(("// reaching for", "// example wanted", "// drafted")):
                    reach += " " + lines[k].strip()[2:].strip()
                    k += 1
                rec["reach"] = reach
            elif nxt.startswith("\t") and t and not t.startswith(("#", "//", "e:", "note")):
                rec["line"] = t
        out.append(rec)
    return out


MUT = re.compile(r"inputs (\d+)\+(\d+)\s+mutants (\d+) of (\d+) apply\s+slipped (\d+)\s+caught (\d+)(?:\s+not judged (\d+))?\s+underdetermined (\d+)(?: in (\d+) gaps?)?(?:\s+accepted (\d+))?")
NO_PASSING = re.compile(r"no passing output on (\d+) of (\d+) inputs with a catch")


def mutants_summary(text: str) -> dict | None:
    m = MUT.search(text)
    if not m:
        return None
    g = m.groups()
    np = NO_PASSING.search(text.split("\n", 1)[0])
    return {"example_inputs": int(g[0]), "generated_inputs": int(g[1]), "applicable": int(g[2]), "of": int(g[3]),
            "slipped": int(g[4]), "caught": int(g[5]), "not_judged": int(g[6]) if g[6] else 0,
            "no_passing": int(np.group(1)) if np else 0, "underdetermined": int(g[7]),
            "gaps": int(g[8]) if g[8] else 0, "accepted": int(g[9]) if g[9] else 0,
            "input_only": text.count("  input only  "), "unneeded": text.count("  unneeded  ")}


def census_one(fn: dict, args: argparse.Namespace) -> dict:
    fid = fn["id"]
    problem = PROBLEMS / f"{fid}.md"
    out = OUT / f"{fid}.sur"
    rec = {"id": fid, "qualname": fn["qualname"], "source": fn["source"], "complete": False, "error": None,
           "started": dt.datetime.now().isoformat(timespec="seconds"), "prompt_sha256": PROMPT,
           "census_flag": True, "stage1": None, "stage2": None}
    common = ["--surety", str(SURETY), "--model", args.model, "--effort", args.effort]
    cmd1 = [sys.executable, str(DRAFT), "sentences", str(problem), "--out", str(out), *common]
    if args.reply1:
        cmd1 += ["--reply", args.reply1]
    code, text, secs = run(cmd1, LOGS / f"{fid}.sentences.txt")
    if code == -signal.SIGINT:
        raise Stopped
    rounds, tin, tout = rounds_and_tokens(text)
    m_s = re.search(r"sentences: (\d+)", text)
    m_w = re.search(r"examples wanted: (\d+)", text)
    m_d = RUN_DIR.search(text)
    s1 = {"exit": code, "rounds": rounds, "tokens_in": tin, "tokens_out": tout,
          "parse_ok": bool(re.search(r": parse ok", text)),
          "sentences": int(m_s.group(1)) if m_s else None,
          "examples_wanted": int(m_w.group(1)) if m_w else None,
          "notes": [l.strip() for l in text.splitlines() if l.startswith(("stripped ", "dropped a property line"))],
          "round_errors": round_errors(pathlib.Path(m_d.group(1).strip())) if m_d else [],
          "last_error": last_error(text), "run_dir": m_d.group(1).strip() if m_d else None, "seconds": round(secs)}
    rec["stage1"] = s1
    if code == -9 or "Traceback" in text or (code != 0 and not s1["parse_ok"] and rounds == 0):
        rec["error"] = f"stage 1 failed: exit {code}; " + (text.strip().splitlines()[-1] if text.strip() else "no output")
        rec["complete"] = s1["parse_ok"] is False and code != -9 and "Traceback" not in text
        return rec
    if not s1["parse_ok"]:
        rec["complete"] = True  # stage 1 never parsed after its rounds: that is a result
        return rec
    cmd2 = [sys.executable, str(DRAFT), "lines", str(out), "--census", *common]
    if args.reply2:
        cmd2 += ["--reply", args.reply2]
    code, text, secs = run(cmd2, LOGS / f"{fid}.lines.txt")
    if code == -signal.SIGINT:
        raise Stopped
    rounds, tin, tout = rounds_and_tokens(text)
    m_d = RUN_DIR.search(text)
    run_dir = pathlib.Path(m_d.group(1).strip()) if m_d else None
    errors, hints, mended = stage2_rounds(run_dir) if run_dir else ([], [], [])
    # draft.py's exit is `surety mutants`' exit, so whether the file parsed is read from its last round.
    s2 = {"exit": code, "rounds": rounds, "tokens_in": tin, "tokens_out": tout,
          "parse_ok": bool(errors) and errors[-1] is None,
          "lines": None, "reach": [], "sentences": [], "refused": [l.strip() for l in text.splitlines() if l.startswith("refused a line")],
          "mutants": None, "mutants_summary": None, "example_gate": example_gate(run_dir) if run_dir else None,
          "round_errors": errors, "round_hints": hints, "mended": mended,
          "round_reach": round_reach(run_dir) if run_dir else [], "last_error": last_error(text),
          "run_dir": str(run_dir) if run_dir else None, "seconds": round(secs)}
    rec["stage2"] = s2
    if code == -9 or "Traceback" in text or run_dir is None:
        rec["error"] = f"stage 2 failed: exit {code}; " + (text.strip().splitlines()[-1] if text.strip() else "no output")
        return rec
    after = run_dir / "after.sur"
    if after.exists():
        sents = read_after(after)
        s2["sentences"] = sents
        # A file that never parsed has lines the checker refused; they are not counted as lines.
        s2["lines"] = sum(1 for s in sents if s["line"]) if s2["parse_ok"] else None
        s2["reach"] = [s["reach"] for s in sents if s["reach"]]
    mut = run_dir / "mutants.txt"
    if mut.exists():
        s2["mutants"] = mut.read_text(encoding="utf-8")
        s2["mutants_summary"] = mutants_summary(s2["mutants"])
    rec["complete"] = True
    return rec


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("--jobs", type=int, default=3)
    ap.add_argument("--limit", type=int, default=0)
    ap.add_argument("--only", nargs="*", default=[])
    ap.add_argument("--retry-failed", action="store_true", help="re-run functions whose record is incomplete")
    ap.add_argument("--model", default="claude-opus-5-5")
    ap.add_argument("--effort", default="high")
    ap.add_argument("--reply1")
    ap.add_argument("--reply2")
    args = ap.parse_args()
    if not SURETY.exists():
        print(f"no {SURETY}; run cargo build --release -p surety first", file=sys.stderr)
        return 1
    OUT.mkdir(exist_ok=True)
    LOGS.mkdir(exist_ok=True)
    fns = [json.loads(l) for l in FUNCTIONS.read_text(encoding="utf-8").splitlines() if l.strip()]
    done = load_results()
    todo = []
    for fn in fns:
        if args.only and fn["id"] not in args.only:
            continue
        prev = done.get(fn["id"])
        if prev and (prev["complete"] or not args.retry_failed):
            continue
        todo.append(fn)
    if args.limit:
        todo = todo[: args.limit]
    print(f"{len(fns)} functions, {len(done)} with a record, {len(todo)} to run, {args.jobs} jobs", flush=True)
    t0 = time.time()
    n = 0

    def settle(fut: cf.Future) -> None:
        nonlocal n
        fn = futs[fut]
        try:
            rec = fut.result()
        except Stopped:
            print(f"[stopped] {fn['id']}: interrupted, not recorded; a resume runs it again", flush=True)
            return
        except Exception as e:  # never lose a slot to a bug
            rec = {"id": fn["id"], "qualname": fn["qualname"], "source": fn["source"], "complete": False,
                   "error": f"runner exception: {type(e).__name__}: {e}", "prompt_sha256": PROMPT,
                   "census_flag": True, "stage1": None, "stage2": None}
        append_result(rec)
        n += 1
        s1, s2 = rec.get("stage1") or {}, rec.get("stage2") or {}
        print(f"[{n}/{len(todo)} {time.time()-t0:5.0f}s] {rec['id']}: "
              f"s1 parse={s1.get('parse_ok')} rounds={s1.get('rounds')} sents={s1.get('sentences')}; "
              f"s2 parse={s2.get('parse_ok')} rounds={s2.get('rounds')} lines={s2.get('lines')} reach={len(s2.get('reach') or [])}"
              + (f"; ERROR {rec['error']}" if rec.get("error") else ""), flush=True)

    ex = cf.ThreadPoolExecutor(max_workers=args.jobs)
    futs = {ex.submit(census_one, fn, args): fn for fn in todo}
    left = set(futs)
    try:
        for fut in cf.as_completed(futs):
            left.discard(fut)
            settle(fut)
    except KeyboardInterrupt:
        # Leaving the pool's own shutdown to run would start every queued function.
        STOP.set()
        ex.shutdown(wait=False, cancel_futures=True)
        running = [f for f in left if not f.cancelled()]
        print(f"\ninterrupted: {len(left) - len(running)} queued cancelled; waiting for {len(running)} in flight, "
              "which start no further stage (Ctrl-C again to stop waiting)", flush=True)
        for fut in cf.as_completed(running):
            settle(fut)
        return 130
    ex.shutdown()
    return 0


if __name__ == "__main__":
    sys.exit(main())
