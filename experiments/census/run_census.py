#!/usr/bin/env python3
"""Run the draft, both stages, over the census problems; checkpoint to
results.jsonl so a stopped run resumes.

    python3 experiments/census/run_census.py [--jobs N] [--limit N] [--only ID ...]
        [--retry-failed] [--model M] [--effort E]

For each function in functions.jsonl with no complete record in results.jsonl:
stage 1 (`draft.py sentences <problem> --out out/<id>.sur`), then, when the
file parses, stage 2 (`draft.py lines out/<id>.sur`). draft.py is not changed;
it is run as a subprocess with the prebuilt checker passed by `--surety`. Each
stage's stdout and stderr go to logs/<id>.<stage>.txt. One JSON record per
function is appended to results.jsonl:

    id, qualname, source, complete, error,
    stage1: exit, rounds, tokens_in, tokens_out, parse_ok, sentences,
            examples_wanted, notes, last_error, run_dir, seconds
    stage2: exit, rounds, tokens_in, tokens_out, parse_ok, lines, reach,
            sentences [{text, line, reach}], refused, mutants (verbatim),
            mutants_summary {applicable, of, slipped, caught,
            underdetermined, gaps, accepted}, last_error, run_dir, seconds

`--reply1`/`--reply2` pass saved replies to draft.py (its own `--reply`), for
testing the mechanics on one problem without a model.
"""
from __future__ import annotations

import argparse
import concurrent.futures as cf
import datetime as dt
import json
import pathlib
import re
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
    race on it (draft.py is not changed here)."""
    t0 = time.time()
    with LAUNCH_LOCK:
        wait = LAST_LAUNCH[0] + 1.2 - time.time()
        if wait > 0:
            time.sleep(wait)
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
    errs = [m for m in re.finditer(r"^round (\d+): (?!tokens)(.+)$", text, re.M)]
    return errs[-1].group(2).strip() if errs else None


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


MUT = re.compile(r"inputs (\d+)\+(\d+)\s+mutants (\d+) of (\d+) apply\s+slipped (\d+)\s+caught (\d+)\s+underdetermined (\d+)(?: in (\d+) gaps?)?(?:\s+accepted (\d+))?")


def mutants_summary(text: str) -> dict | None:
    m = MUT.search(text)
    if not m:
        return None
    g = m.groups()
    return {"example_inputs": int(g[0]), "generated_inputs": int(g[1]), "applicable": int(g[2]), "of": int(g[3]),
            "slipped": int(g[4]), "caught": int(g[5]), "underdetermined": int(g[6]),
            "gaps": int(g[7]) if g[7] else 0, "accepted": int(g[8]) if g[8] else 0,
            "input_only": text.count("  input only  "), "unneeded": text.count("  unneeded  ")}


def census_one(fn: dict, args: argparse.Namespace) -> dict:
    fid = fn["id"]
    problem = PROBLEMS / f"{fid}.md"
    out = OUT / f"{fid}.sur"
    rec = {"id": fid, "qualname": fn["qualname"], "source": fn["source"], "complete": False, "error": None,
           "started": dt.datetime.now().isoformat(timespec="seconds"), "stage1": None, "stage2": None}
    common = ["--surety", str(SURETY), "--model", args.model, "--effort", args.effort]
    cmd1 = [sys.executable, str(DRAFT), "sentences", str(problem), "--out", str(out), *common]
    if args.reply1:
        cmd1 += ["--reply", args.reply1]
    code, text, secs = run(cmd1, LOGS / f"{fid}.sentences.txt")
    rounds, tin, tout = rounds_and_tokens(text)
    m_s = re.search(r"sentences: (\d+)", text)
    m_w = re.search(r"examples wanted: (\d+)", text)
    m_d = RUN_DIR.search(text)
    s1 = {"exit": code, "rounds": rounds, "tokens_in": tin, "tokens_out": tout,
          "parse_ok": bool(re.search(r": parse ok", text)),
          "sentences": int(m_s.group(1)) if m_s else None,
          "examples_wanted": int(m_w.group(1)) if m_w else None,
          "notes": [l.strip() for l in text.splitlines() if l.startswith(("stripped ", "dropped a property line"))],
          "last_error": last_error(text), "run_dir": m_d.group(1).strip() if m_d else None, "seconds": round(secs)}
    rec["stage1"] = s1
    if code == -9 or "Traceback" in text or (code != 0 and not s1["parse_ok"] and rounds == 0):
        rec["error"] = f"stage 1 failed: exit {code}; " + (text.strip().splitlines()[-1] if text.strip() else "no output")
        rec["complete"] = s1["parse_ok"] is False and code != -9 and "Traceback" not in text
        return rec
    if not s1["parse_ok"]:
        rec["complete"] = True  # stage 1 never parsed after its rounds: that is a result
        return rec
    cmd2 = [sys.executable, str(DRAFT), "lines", str(out), *common]
    if args.reply2:
        cmd2 += ["--reply", args.reply2]
    code, text, secs = run(cmd2, LOGS / f"{fid}.lines.txt")
    rounds, tin, tout = rounds_and_tokens(text)
    m_d = RUN_DIR.search(text)
    run_dir = pathlib.Path(m_d.group(1).strip()) if m_d else None
    s2 = {"exit": code, "rounds": rounds, "tokens_in": tin, "tokens_out": tout, "parse_ok": code == 0,
          "lines": None, "reach": [], "sentences": [], "refused": [l.strip() for l in text.splitlines() if l.startswith("refused a line")],
          "mutants": None, "mutants_summary": None, "last_error": last_error(text),
          "run_dir": str(run_dir) if run_dir else None, "seconds": round(secs)}
    rec["stage2"] = s2
    if code == -9 or "Traceback" in text or run_dir is None:
        rec["error"] = f"stage 2 failed: exit {code}; " + (text.strip().splitlines()[-1] if text.strip() else "no output")
        return rec
    after = run_dir / "after.sur"
    if after.exists():
        sents = read_after(after)
        s2["sentences"] = sents
        s2["lines"] = sum(1 for s in sents if s["line"])
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
    with cf.ThreadPoolExecutor(max_workers=args.jobs) as ex:
        futs = {ex.submit(census_one, fn, args): fn for fn in todo}
        for fut in cf.as_completed(futs):
            fn = futs[fut]
            try:
                rec = fut.result()
            except Exception as e:  # never lose a slot to a bug
                rec = {"id": fn["id"], "qualname": fn["qualname"], "source": fn["source"], "complete": False,
                       "error": f"runner exception: {type(e).__name__}: {e}", "stage1": None, "stage2": None}
            append_result(rec)
            n += 1
            s1, s2 = rec.get("stage1") or {}, rec.get("stage2") or {}
            print(f"[{n}/{len(todo)} {time.time()-t0:5.0f}s] {rec['id']}: "
                  f"s1 parse={s1.get('parse_ok')} rounds={s1.get('rounds')} sents={s1.get('sentences')}; "
                  f"s2 parse={s2.get('parse_ok')} rounds={s2.get('rounds')} lines={s2.get('lines')} reach={len(s2.get('reach') or [])}"
                  + (f"; ERROR {rec['error']}" if rec.get("error") else ""), flush=True)
    return 0


if __name__ == "__main__":
    sys.exit(main())
