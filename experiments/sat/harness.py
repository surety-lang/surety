#!/usr/bin/env python3
"""The SAT loop experiment's harness: what the checker cannot supply.

The checker's generator derives inputs from the types, so it produces small random
formulas and nothing hard; and it has no step counter, so it measures no cost. This
harness supplies both, outside the checker, and judges no correctness of its own:
correctness is `surety check`'s, and on hard instances the plain performance's where
it reaches, a direct model check (linear, in the obligation's own terms) where it does
not. An unsat answer beyond the plain performance's reach is recorded as unverified.

    python3 experiments/sat/harness.py                     # selftest, records, every family, the report
    python3 experiments/sat/harness.py --stage records     # the checker's verdicts alone
    python3 experiments/sat/harness.py --stage cost --families php,tseitin --perfs dpll,cdcl
    python3 experiments/sat/harness.py --stage report --run experiments/sat/runs/<stamp>

Stages:
  selftest  the generators against the plain performance at sizes it reaches: PHP unsat,
            Tseitin odd charge unsat and even charge satisfiable, factoring decodes x*y = N
  records   `surety check` on each performance against sat.sur and sat_canonical.sur,
            `surety mutants` on both obligations, `surety diff` between performances, and
            `surety parse` on each line in attempts/ that the fragment was expected to refuse
  cost      each performance on each instance of each family, in its own process, under a
            sys.settrace line counter the harness installs; lines and wall time recorded
  report    log(lines) fitted against size per family and performance, both as an
            exponential (base per unit of size) and as a polynomial (exponent), with the
            range each fit covers; the canaries; the per-instance table

Every run writes to experiments/sat/runs/<stamp>/: records/, instances/<family>/*.cnf and
*.json, results.jsonl, fits.json, cost.md, summary.md. The checker is target/release/surety
unless --surety names another; its commit is recorded, as closed_loop.py does.

Cost is measured in lines of the performance's own source executed, counted by a trace
function in the child process that ignores every other file, so the count is external to
the performance and deterministic. Wall time comes from a second, untraced run with the
checker's own 10-second cap, since tracing slows Python several times over.
"""

from __future__ import annotations

import argparse
import datetime as dt
import json
import math
import pathlib
import random
import subprocess
import sys
import time

ROOT = pathlib.Path(__file__).resolve().parent.parent.parent
HERE = ROOT / "experiments" / "sat"
RUNS = HERE / "runs"
PERFS = {"plain": "sat_plain.py", "dpll": "sat_dpll.py", "cdcl": "sat_cdcl.py"}
OBLIGATIONS = ["sat.sur", "sat_canonical.sur"]
TIMEOUT = 10.0  # the checker's per-call cap, mirrored
TRACE_FACTOR = 12  # the traced run may take this many times the cap before it is abandoned
RATIO = 4.26  # random 3-SAT clause/variable ratio at the threshold


# ---------------------------------------------------------------------------
# Instances
# ---------------------------------------------------------------------------


class Cnf:
    """A formula in DIMACS terms: variables 1..nvars, a literal a signed int."""

    def __init__(self, nvars: int = 0):
        self.nvars = nvars
        self.clauses: list[list[int]] = []

    def new(self) -> int:
        self.nvars += 1
        return self.nvars

    def add(self, *lits: int) -> None:
        self.clauses.append(list(lits))

    def dimacs(self, comment: str) -> str:
        lines = [f"c {comment}", f"p cnf {self.nvars} {len(self.clauses)}"]
        lines += [" ".join(str(l) for l in c) + " 0" for c in self.clauses]
        return "\n".join(lines) + "\n"

    def obligation(self) -> dict:
        """The formula in the obligation's literal shape."""
        return {
            "n": self.nvars,
            "clauses": [{"lits": [{"var": abs(l), "neg": l < 0} for l in c]} for c in self.clauses],
        }


def random3sat(n: int, seed: int) -> tuple[Cnf, dict]:
    r = random.Random(seed)
    m = round(RATIO * n)
    cnf = Cnf(n)
    for _ in range(m):
        vs = r.sample(range(1, n + 1), 3)
        cnf.add(*[v if r.random() < 0.5 else -v for v in vs])
    return cnf, {"ratio": RATIO, "seed": seed}


def php(n: int, seed: int = 0) -> tuple[Cnf, dict]:
    """PHP_n: n + 1 pigeons into n holes. Variable (i, j): pigeon i sits in hole j."""
    cnf = Cnf(n * (n + 1))
    var = lambda i, j: (i - 1) * n + j  # noqa: E731
    for i in range(1, n + 2):
        cnf.add(*[var(i, j) for j in range(1, n + 1)])
    for j in range(1, n + 1):
        for i in range(1, n + 2):
            for k in range(i + 1, n + 2):
                cnf.add(-var(i, j), -var(k, j))
    return cnf, {"pigeons": n + 1, "holes": n}


def _random_3_regular(v: int, r: random.Random) -> list[tuple[int, int]]:
    """A random 3-regular simple graph on v vertices by the configuration model, redrawn
    until it has no loop and no repeated edge."""
    while True:
        points = [i for i in range(v) for _ in range(3)]
        r.shuffle(points)
        edges = set()
        ok = True
        for k in range(0, len(points), 2):
            a, b = points[k], points[k + 1]
            if a == b or (min(a, b), max(a, b)) in edges:
                ok = False
                break
            edges.add((min(a, b), max(a, b)))
        if ok:
            return sorted(edges)


def _parity(cnf: Cnf, vars_: list[int], charge: int) -> None:
    """XOR of the variables equals the charge: one clause per assignment of the wrong parity."""
    k = len(vars_)
    for bits in range(1 << k):
        values = [(bits >> i) & 1 for i in range(k)]
        if sum(values) % 2 != charge:
            cnf.add(*[-x if val else x for x, val in zip(vars_, values)])


def tseitin(v: int, seed: int, odd: bool = True) -> tuple[Cnf, dict]:
    """The Tseitin formula on a random 3-regular graph: an edge is a variable, a vertex a
    parity constraint on its three edges. Odd total charge (vertex 0 charged) is unsat;
    even (no vertex charged) is satisfiable, by the all-false assignment among others."""
    if v % 2:
        raise ValueError("a 3-regular graph needs an even vertex count")
    r = random.Random(seed)
    edges = _random_3_regular(v, r)
    cnf = Cnf(len(edges))
    index = {e: i + 1 for i, e in enumerate(edges)}
    incident: dict[int, list[int]] = {i: [] for i in range(v)}
    for e in edges:
        incident[e[0]].append(index[e])
        incident[e[1]].append(index[e])
    for vertex in range(v):
        _parity(cnf, incident[vertex], 1 if odd and vertex == 0 else 0)
    return cnf, {"vertices": v, "edges": len(edges), "charge": "odd" if odd else "even", "seed": seed}


def _is_prime(x: int) -> bool:
    if x < 2:
        return False
    i = 2
    while i * i <= x:
        if x % i == 0:
            return False
        i += 1
    return True


def _full_adder(cnf: Cnf, a: int, b: int, c: int) -> tuple[int, int]:
    s, co = cnf.new(), cnf.new()
    for bits in range(8):
        va, vb, vc = bits & 1, (bits >> 1) & 1, (bits >> 2) & 1
        par = va ^ vb ^ vc
        # if a=va and b=vb and c=vc then s=par
        cnf.add(-a if va else a, -b if vb else b, -c if vc else c, s if par else -s)
    cnf.add(-a, -b, co)
    cnf.add(-a, -c, co)
    cnf.add(-b, -c, co)
    cnf.add(a, b, -co)
    cnf.add(a, c, -co)
    cnf.add(b, c, -co)
    return s, co


def _half_adder(cnf: Cnf, a: int, b: int) -> tuple[int, int]:
    s, co = cnf.new(), cnf.new()
    cnf.add(-a, -b, -s)
    cnf.add(a, b, -s)
    cnf.add(a, -b, s)
    cnf.add(-a, b, s)
    cnf.add(-a, -b, co)
    cnf.add(a, -co)
    cnf.add(b, -co)
    return s, co


def factoring(bits: int, seed: int) -> tuple[Cnf, dict]:
    """x * y = N for a semiprime N = p * q with p and q both `bits` bits wide, as an array
    multiplier (AND partial products, columns reduced by full and half adders) whose output
    bits are fixed to N. The factors are the two solutions; 1 * N does not fit the width."""
    r = random.Random(seed)
    primes = [x for x in range(1 << (bits - 1), 1 << bits) if _is_prime(x)]
    p, q = sorted(r.sample(primes, 2))
    n_value = p * q
    cnf = Cnf()
    xs = [cnf.new() for _ in range(bits)]
    ys = [cnf.new() for _ in range(bits)]
    columns: list[list[int]] = [[] for _ in range(2 * bits + 2)]
    for i in range(bits):
        for j in range(bits):
            pp = cnf.new()
            cnf.add(-pp, xs[i])
            cnf.add(-pp, ys[j])
            cnf.add(pp, -xs[i], -ys[j])
            columns[i + j].append(pp)
    for k in range(len(columns)):
        col = columns[k]
        while len(col) >= 3:
            a, b, c = col.pop(), col.pop(), col.pop()
            s, co = _full_adder(cnf, a, b, c)
            col.append(s)
            columns[k + 1].append(co) if k + 1 < len(columns) else cnf.add(-co)
        if len(col) == 2:
            a, b = col.pop(), col.pop()
            s, co = _half_adder(cnf, a, b)
            col.append(s)
            columns[k + 1].append(co) if k + 1 < len(columns) else cnf.add(-co)
    for k, col in enumerate(columns):
        bit = (n_value >> k) & 1
        if col:
            cnf.add(col[0] if bit else -col[0])
        elif bit:
            cnf.add(1)
            cnf.add(-1)  # cannot happen for a product of this width; makes the formula unsat if it did
    return cnf, {"N": n_value, "p": p, "q": q, "bits": bits, "x_vars": xs, "y_vars": ys}


FAMILIES = {
    "random3sat": {
        "sizes": [10, 12, 14, 16, 18, 20, 22, 24, 30, 40, 50, 75, 100, 150, 200],
        "instances": 3,
        "size": "n, the variable count (clauses = round(4.26 n))",
        "gen": lambda n, k: random3sat(n, 1000 * n + k),
    },
    "php": {
        "sizes": list(range(3, 13)),
        "instances": 1,
        "size": "n, the hole count (n + 1 pigeons, n(n+1) variables)",
        "gen": lambda n, k: php(n),
    },
    "tseitin": {
        "sizes": [6, 8, 10, 12, 14, 16, 20, 24, 30, 40],
        "instances": 1,
        "size": "the vertex count of the 3-regular graph (3/2 as many variables); odd charge, unsat",
        "gen": lambda v, k: tseitin(v, 1000 * v + k, odd=True),
    },
    "factoring": {
        "sizes": [5, 6, 7, 8, 9, 10, 11, 12],
        "instances": 1,
        "size": "the width of each factor in bits (the semiprime has twice as many, less one)",
        "gen": lambda b, k: factoring(b, 1000 * b + k),
    },
}


# ---------------------------------------------------------------------------
# The worker: one performance on one instance, in its own process
# ---------------------------------------------------------------------------


class _Budget(Exception):
    pass


class _Counter:
    """Counts the line events of frames whose code lives in the performance's file and
    nothing else; the global hook declines every other frame."""

    def __init__(self, filename: str, deadline: float):
        self.filename = filename
        self.deadline = deadline
        self.count = 0

    def trace(self, frame, event, arg):
        if frame.f_code.co_filename != self.filename:
            return None
        return self.local

    def local(self, frame, event, arg):
        if event == "line":
            self.count += 1
            if (self.count & 0x3FFFF) == 0 and time.monotonic() > self.deadline:
                raise _Budget()
        return self.local


def worker(perf: pathlib.Path, instance: pathlib.Path, out: pathlib.Path, trace: bool, deadline_s: float) -> None:
    source = perf.read_text(encoding="utf-8")
    code = compile(source, str(perf), "exec")
    namespace: dict = {"__name__": "performance"}
    exec(code, namespace)  # noqa: S102  (the performance is this repository's own file)
    fn = namespace["sat"]
    formula = json.loads(instance.read_text(encoding="utf-8"))
    result: dict = {"status": "ok", "trace": trace}
    counter = _Counter(str(perf), time.monotonic() + deadline_s) if trace else None
    answer = None
    t0 = time.perf_counter()
    if counter:
        sys.settrace(counter.trace)
    try:
        answer = fn(formula)
    except _Budget:
        result["status"] = "trace_budget"
    except RecursionError as e:
        result["status"] = "error"
        result["error"] = f"RecursionError: {e}"
    finally:
        sys.settrace(None)
    result["wall"] = time.perf_counter() - t0
    if counter:
        result["lines"] = counter.count
    result["out"] = answer
    out.write_text(json.dumps(result), encoding="utf-8")


def run_worker(perf: pathlib.Path, instance: pathlib.Path, scratch: pathlib.Path, trace: bool) -> dict:
    out = scratch / f"{perf.stem}-{instance.stem}-{'traced' if trace else 'plain'}.json"
    if out.exists():
        out.unlink()
    deadline = TIMEOUT * TRACE_FACTOR if trace else TIMEOUT
    cmd = [sys.executable, "-I", str(pathlib.Path(__file__).resolve()), "worker", str(perf), str(instance), str(out),
           "--deadline", str(deadline)]
    if trace:
        cmd.append("--trace")
    try:
        proc = subprocess.run(cmd, capture_output=True, text=True, timeout=deadline + 30 if trace else TIMEOUT + 2)
    except subprocess.TimeoutExpired:
        return {"status": "timeout", "trace": trace}
    if proc.returncode != 0 or not out.exists():
        return {"status": "error", "trace": trace, "error": (proc.stderr or proc.stdout)[-800:]}
    result = json.loads(out.read_text(encoding="utf-8"))
    out.unlink()
    if not trace and result.get("status") == "ok" and result["wall"] > TIMEOUT:
        result["status"] = "timeout"  # answered, but past the checker's cap: the checker would have killed it
    return result


# ---------------------------------------------------------------------------
# Verification on hard instances: never the harness's judgement of correctness, only the
# plain performance's answer where it reaches, and the obligation's own model check.
# ---------------------------------------------------------------------------


def model_holds(formula: dict, out: dict) -> str | None:
    """None when the Ok output passes the obligation's properties; else what fails."""
    vals = out["ok"]["vals"]
    assignment: dict = {}
    for v in vals:
        if v["var"] in assignment:
            return f"var {v['var']} assigned twice"
        assignment[v["var"]] = v["value"]
    for c in formula["clauses"]:
        for l in c["lits"]:
            if l["var"] not in assignment:
                return f"var {l['var']} unassigned"
        if not any(assignment[l["var"]] == (not l["neg"]) for l in c["lits"]):
            return "a clause has no true literal"
    return None


def classify(formula: dict, out: dict | None, plain_answer: str | None, is_plain: bool) -> str:
    if out is None:
        return "no answer"
    if "ok" in out:
        failure = model_holds(formula, out)
        if failure:
            return f"MODEL WRONG: {failure}"
        if plain_answer == "unsat":
            return "model verified; plain said unsat (plain is wrong, or the instance differs)"
        return "model verified"
    if is_plain:
        return "unsat by exhaustion (the oracle)"
    if plain_answer == "unsat":
        return "unsat, agrees with plain"
    if plain_answer == "sat":
        return "UNSAT DISAGREES: plain found a model"
    return "unsat, unverified (no certificate; a DRAT check is the next step)"


# ---------------------------------------------------------------------------
# Stages
# ---------------------------------------------------------------------------


def checker_identity(surety: pathlib.Path) -> dict:
    git = lambda *a: subprocess.run(["git", *a], cwd=ROOT, capture_output=True, text=True).stdout.strip()  # noqa: E731
    commit = git("rev-parse", "--short", "HEAD") or "unknown"
    if git("status", "--porcelain", "--", "crates"):
        commit += "+dirty"
    built = dt.datetime.fromtimestamp(surety.stat().st_mtime).strftime("%Y-%m-%d %H:%M:%S") if surety.exists() else ""
    return {"commit": commit, "binary": str(surety.relative_to(ROOT)) if surety.is_relative_to(ROOT) else str(surety),
            "built": built, "python": sys.version.split()[0]}


def surety_run(surety: pathlib.Path, args: list[str], into: pathlib.Path) -> tuple[int, str]:
    proc = subprocess.run([str(surety), *args], capture_output=True, text=True, timeout=1800, cwd=ROOT)
    text = proc.stdout + proc.stderr
    into.write_text(text, encoding="utf-8")
    return proc.returncode, text


def rel(p: pathlib.Path) -> str:
    return str(p.relative_to(ROOT))


def records(run: pathlib.Path, surety: pathlib.Path) -> None:
    rec = run / "records"
    rec.mkdir(parents=True, exist_ok=True)
    (rec / "checker.json").write_text(json.dumps(checker_identity(surety), indent=2), encoding="utf-8")
    verdicts: list[str] = []
    perfs = dict(PERFS)
    perfs["always_unsat"] = "sat_always_unsat.py"
    for obl in OBLIGATIONS:
        for name, py in perfs.items():
            stem = f"check_{obl[:-4]}_{name}"
            code, text = surety_run(surety, ["check", rel(HERE / obl), rel(HERE / py)], rec / f"{stem}.txt")
            surety_run(surety, ["check", "--json", rel(HERE / obl), rel(HERE / py)], rec / f"{stem}.json")
            first = text.splitlines()[0] if text else "(no output)"
            verdicts.append(f"check {obl} {py} (exit {code}): {first}")
            print(verdicts[-1], flush=True)
        stem = f"mutants_{obl[:-4]}"
        code, text = surety_run(surety, ["mutants", rel(HERE / obl)], rec / f"{stem}.txt")
        surety_run(surety, ["mutants", "--full", rel(HERE / obl)], rec / f"{stem}.full.txt")
        surety_run(surety, ["mutants", "--json", rel(HERE / obl)], rec / f"{stem}.json")
        verdicts.append(f"mutants {obl} (exit {code}): {text.splitlines()[0] if text else '(no output)'}")
        print(verdicts[-1], flush=True)
    pairs = [("sat.sur", "plain", "dpll"), ("sat.sur", "plain", "cdcl"), ("sat.sur", "dpll", "cdcl"),
             ("sat_canonical.sur", "plain", "dpll"), ("sat_canonical.sur", "plain", "cdcl")]
    for obl, a, b in pairs:
        stem = f"diff_{obl[:-4]}_{a}_{b}"
        code, text = surety_run(surety, ["diff", rel(HERE / obl), rel(HERE / PERFS[a]), rel(HERE / PERFS[b])], rec / f"{stem}.txt")
        surety_run(surety, ["diff", "--full", rel(HERE / obl), rel(HERE / PERFS[a]), rel(HERE / PERFS[b])], rec / f"{stem}.full.txt")
        surety_run(surety, ["diff", "--json", rel(HERE / obl), rel(HERE / PERFS[a]), rel(HERE / PERFS[b])], rec / f"{stem}.json")
        verdicts.append(f"diff {obl} {a} {b} (exit {code}): {text.splitlines()[0] if text else '(no output)'}")
        print(verdicts[-1], flush=True)
    attempts = rec / "attempts"
    attempts.mkdir(exist_ok=True)
    for sur in sorted((HERE / "attempts").glob("*.sur")):
        code, text = surety_run(surety, ["parse", rel(sur)], attempts / f"{sur.stem}.parse.txt")
        surety_run(surety, ["parse", "--json", rel(sur)], attempts / f"{sur.stem}.parse.json")
        verdicts.append(f"parse attempts/{sur.name} (exit {code}): {text.strip().splitlines()[-1] if text.strip() else '(no output)'}")
        print(verdicts[-1], flush=True)
        py = sur.with_suffix(".py")
        if code == 0 and py.exists():
            code2, text2 = surety_run(surety, ["check", rel(sur), rel(py)], attempts / f"{sur.stem}.check.txt")
            surety_run(surety, ["mutants", "--full", rel(sur)], attempts / f"{sur.stem}.mutants.full.txt")
            verdicts.append(f"check attempts/{sur.name} {py.name} (exit {code2}): {text2.splitlines()[0] if text2 else '(no output)'}")
            print(verdicts[-1], flush=True)
    (rec / "verdicts.md").write_text("# Verdict lines\n\n" + "\n".join(f"- `{v}`" for v in verdicts) + "\n", encoding="utf-8")


def write_instance(run: pathlib.Path, family: str, size: int, k: int) -> tuple[pathlib.Path, dict]:
    spec = FAMILIES[family]
    cnf, meta = spec["gen"](size, k)
    d = run / "instances" / family
    d.mkdir(parents=True, exist_ok=True)
    name = f"{family}_{size:04d}_{k}"
    comment = f"{family} size {size} instance {k}: " + ", ".join(f"{a}={b}" for a, b in meta.items() if a not in ("x_vars", "y_vars"))
    (d / f"{name}.cnf").write_text(cnf.dimacs(comment), encoding="utf-8")
    (d / f"{name}.json").write_text(json.dumps(cnf.obligation()), encoding="utf-8")
    meta = dict(meta, name=name, family=family, size=size, instance=k, vars=cnf.nvars, clauses=len(cnf.clauses))
    return d / f"{name}.json", meta


def decode_factors(meta: dict, out: dict) -> str:
    """For a factoring instance with a model: the x and y the model encodes, and whether
    they multiply to N. A shape check outside the obligation; printed, never judged."""
    a = {v["var"]: v["value"] for v in out["ok"]["vals"]}
    x = sum(1 << i for i, var in enumerate(meta["x_vars"]) if a.get(var))
    y = sum(1 << i for i, var in enumerate(meta["y_vars"]) if a.get(var))
    return f"x={x} y={y} x*y{'==' if x * y == meta['N'] else '!='}N"


def cost(run: pathlib.Path, families: list[str], perfs: list[str]) -> None:
    results_path = run / "results.jsonl"
    scratch = run / "scratch"
    scratch.mkdir(parents=True, exist_ok=True)
    with results_path.open("a", encoding="utf-8") as results:
        for family in families:
            spec = FAMILIES[family]
            stopped: set[str] = set()  # performances that timed out on every instance of a size
            for size in spec["sizes"]:
                timed_out: dict[str, int] = {p: 0 for p in perfs}
                for k in range(spec["instances"]):
                    instance, meta = write_instance(run, family, size, k)
                    formula = json.loads(instance.read_text(encoding="utf-8"))
                    plain_answer = None
                    order = sorted(perfs, key=lambda p: p != "plain")  # plain first: it is the oracle
                    for perf in order:
                        row = {k2: v for k2, v in meta.items() if k2 not in ("x_vars", "y_vars")}
                        row.update({"perf": perf})
                        if perf in stopped:
                            row.update({"status": "skipped", "note": "timed out on every instance of a smaller size"})
                            results.write(json.dumps(row) + "\n")
                            results.flush()
                            continue
                        py = HERE / PERFS[perf]
                        untraced = run_worker(py, instance, scratch, trace=False)
                        row["status"] = untraced["status"]
                        row["wall_s"] = round(untraced.get("wall", 0.0), 4) if "wall" in untraced else None
                        out = untraced.get("out")
                        answer = None if out is None else ("sat" if "ok" in out else "unsat")
                        row["answer"] = answer
                        if untraced["status"] == "ok":
                            row["verification"] = classify(formula, out, plain_answer, perf == "plain")
                            if perf == "plain":
                                plain_answer = answer
                            if family == "factoring" and answer == "sat":
                                row["factors"] = decode_factors(meta, out)
                            traced = run_worker(py, instance, scratch, trace=True)
                            row["trace_status"] = traced["status"]
                            row["lines"] = traced.get("lines")
                            row["traced_wall_s"] = round(traced["wall"], 3) if "wall" in traced else None
                        else:
                            timed_out[perf] += 1
                            if "error" in untraced:
                                row["error"] = untraced["error"]
                        results.write(json.dumps(row) + "\n")
                        results.flush()
                        print(f"{family:10s} size {size:4d} #{k} {perf:5s} {row['status']:12s} "
                              f"{(answer or '-'):5s} wall {row['wall_s'] if row['wall_s'] is not None else '-':>8} "
                              f"lines {row.get('lines') if row.get('lines') is not None else '-':>12} "
                              f"{row.get('verification', '')} {row.get('factors', '')}", flush=True)
                for perf in perfs:
                    if timed_out[perf] == spec["instances"] and perf not in stopped:
                        stopped.add(perf)
                        print(f"{family}: {perf} timed out on every instance at size {size}; larger sizes skipped", flush=True)
    try:
        scratch.rmdir()
    except OSError:
        pass


# ---------------------------------------------------------------------------
# The report
# ---------------------------------------------------------------------------


def least_squares(xs: list[float], ys: list[float]) -> tuple[float, float, float]:
    """slope, intercept, R^2 of y on x."""
    n = len(xs)
    mx, my = sum(xs) / n, sum(ys) / n
    sxx = sum((x - mx) ** 2 for x in xs)
    sxy = sum((x - mx) * (y - my) for x, y in zip(xs, ys))
    if sxx == 0:
        return float("nan"), my, float("nan")
    slope = sxy / sxx
    intercept = my - slope * mx
    ss_res = sum((y - (intercept + slope * x)) ** 2 for x, y in zip(xs, ys))
    ss_tot = sum((y - my) ** 2 for y in ys)
    r2 = 1 - ss_res / ss_tot if ss_tot > 0 else float("nan")
    return slope, intercept, r2


def fit_family(rows: list[dict]) -> dict:
    """Both fits over the completed, traced points: log(lines) against size, and against
    log(size). Returns the base per unit of size, the exponent, each R^2, and the range."""
    points = [(r["size"], r["lines"]) for r in rows
              if r.get("status") == "ok" and r.get("trace_status") == "ok" and r.get("lines")]
    sizes = sorted({s for s, _ in points})
    out = {"points": len(points), "sizes": sizes, "range": [sizes[0], sizes[-1]] if sizes else None,
           "timed_out_at": sorted({r["size"] for r in rows if r.get("status") == "timeout"}),
           "skipped_at": sorted({r["size"] for r in rows if r.get("status") == "skipped"})}
    if len(sizes) < 3:
        out["note"] = "fewer than three sizes completed; no fit"
        return out
    xs = [float(s) for s, _ in points]
    ys = [math.log(max(l, 1)) for _, l in points]
    slope, _, r2 = least_squares(xs, ys)
    out["exponential"] = {"base_per_unit": math.exp(slope), "r2": r2}
    lxs = [math.log(s) for s, _ in points]
    k, _, r2p = least_squares(lxs, ys)
    out["polynomial"] = {"exponent": k, "r2": r2p}
    out["better"] = "exponential" if r2 >= r2p else "polynomial"
    return out


def fmt(x, digits=2) -> str:
    if x is None:
        return "-"
    if isinstance(x, float):
        if math.isnan(x):
            return "nan"
        return f"{x:.{digits}f}"
    return str(x)


def report(run: pathlib.Path) -> None:
    rows = [json.loads(l) for l in (run / "results.jsonl").read_text(encoding="utf-8").splitlines() if l.strip()]
    fits: dict = {}
    for family in FAMILIES:
        for perf in PERFS:
            sub = [r for r in rows if r["family"] == family and r["perf"] == perf]
            if sub:
                fits[f"{family}/{perf}"] = fit_family(sub)
    (run / "fits.json").write_text(json.dumps(fits, indent=2), encoding="utf-8")

    lines = ["# Cost by family", "",
             "Cost is lines of the performance's own source executed (a `sys.settrace` count in the child process),",
             "fitted as log(lines) against size (exponential: the base per unit of size) and against log(size)",
             "(polynomial: the exponent). Both fits are given with R^2; the range is the sizes the fit covers.",
             "A fit over fewer than three sizes is not reported. Timed out means past the checker's 10 s cap.", "",
             "| Family | Size means | Performance | Fitted range (points) | Exponential: base per unit (R^2) | Polynomial: exponent (R^2) | Better fit | Timed out at | Skipped at |",
             "| --- | --- | --- | --- | --- | --- | --- | --- | --- |"]
    for key, f in fits.items():
        family, perf = key.split("/")
        rng = f"{f['range'][0]}..{f['range'][1]} ({f['points']})" if f.get("range") else "none"
        exp = f"{fmt(f['exponential']['base_per_unit'], 3)} ({fmt(f['exponential']['r2'])})" if "exponential" in f else "-"
        poly = f"{fmt(f['polynomial']['exponent'])} ({fmt(f['polynomial']['r2'])})" if "polynomial" in f else "-"
        lines.append(f"| {family} | {FAMILIES[family]['size']} | {perf} | {rng} | {exp} | {poly} | {f.get('better', f.get('note', '-'))} | "
                     f"{', '.join(map(str, f['timed_out_at'])) or '-'} | {', '.join(map(str, f['skipped_at'])) or '-'} |")
    lines += ["", "## The canaries", ""]
    for perf in PERFS:
        f = fits.get(f"php/{perf}")
        if f and "exponential" in f:
            lines.append(f"- PHP_n, {perf}: base {fmt(f['exponential']['base_per_unit'], 3)} per hole (R^2 {fmt(f['exponential']['r2'])}), "
                         f"fitted on n = {f['range'][0]}..{f['range'][1]}; polynomial exponent {fmt(f['polynomial']['exponent'])} "
                         f"(R^2 {fmt(f['polynomial']['r2'])}); timed out at {', '.join(map(str, f['timed_out_at'])) or 'none'}.")
        elif f:
            lines.append(f"- PHP_n, {perf}: {f.get('note')}; sizes completed {f['sizes']}; timed out at {f['timed_out_at']}.")
    for perf in PERFS:
        solved = [r for r in rows if r["family"] == "factoring" and r["perf"] == perf and r.get("status") == "ok" and r.get("answer") == "sat"]
        if solved:
            best = max(solved, key=lambda r: r["N"])
            lines.append(f"- Factoring, {perf}: largest semiprime solved within the cap N = {best['N']} = {best['p']} x {best['q']} "
                         f"({best['N'].bit_length()} bits, factors {best['bits']} bits; {best['vars']} variables, {best['clauses']} clauses; "
                         f"{best.get('lines')} lines, {best['wall_s']} s). "
                         f"Timed out at bits {', '.join(str(r['size']) for r in rows if r['family'] == 'factoring' and r['perf'] == perf and r.get('status') == 'timeout') or 'none'}.")
        else:
            lines.append(f"- Factoring, {perf}: no instance solved within the cap.")
    lines += ["", "## Every instance", "",
              "| Family | Size | # | Vars | Clauses | Performance | Status | Answer | Wall s | Lines | Verification | Note |",
              "| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |"]
    for r in rows:
        note = r.get("factors", "") or r.get("note", "") or (r.get("error", "")[:80].replace("\n", " ") if r.get("error") else "")
        if r.get("trace_status") and r["trace_status"] != "ok":
            note = (note + " " if note else "") + f"trace {r['trace_status']}"
        lines.append(f"| {r['family']} | {r['size']} | {r['instance']} | {r['vars']} | {r['clauses']} | {r['perf']} | {r['status']} | "
                     f"{r.get('answer') or '-'} | {fmt(r.get('wall_s'), 3)} | {r.get('lines') if r.get('lines') is not None else '-'} | "
                     f"{r.get('verification', '')} | {note} |")
    (run / "cost.md").write_text("\n".join(lines) + "\n", encoding="utf-8")
    print(f"report written to {run / 'cost.md'}")


# ---------------------------------------------------------------------------
# Self-test of the generators against the plain performance
# ---------------------------------------------------------------------------


def selftest(run: pathlib.Path) -> bool:
    scratch = run / "scratch"
    scratch.mkdir(parents=True, exist_ok=True)
    ok = True

    def answer(perf: str, cnf: Cnf, name: str) -> dict | None:
        inst = scratch / f"selftest_{name}.json"
        inst.write_text(json.dumps(cnf.obligation()), encoding="utf-8")
        r = run_worker(HERE / PERFS[perf], inst, scratch, trace=False)
        inst.unlink()
        return r.get("out") if r.get("status") == "ok" else None

    checks: list[tuple[str, bool]] = []
    for n in (3, 4):
        cnf, _ = php(n)
        out = answer("plain", cnf, f"php{n}")
        checks.append((f"PHP_{n} is unsat by exhaustion", out is not None and "err" in out))
    for v in (6, 8):
        cnf, _ = tseitin(v, v, odd=True)
        out = answer("plain", cnf, f"tseitin_odd{v}")
        checks.append((f"Tseitin odd charge on {v} vertices is unsat by exhaustion", out is not None and "err" in out))
        cnf, _ = tseitin(v, v, odd=False)
        out = answer("plain", cnf, f"tseitin_even{v}")
        checks.append((f"Tseitin even charge on {v} vertices has a model that holds",
                       out is not None and "ok" in out and model_holds(cnf.obligation(), out) is None))
    for bits in (3, 4):
        cnf, meta = factoring(bits, bits)
        out = answer("dpll", cnf, f"factoring{bits}")
        holds = out is not None and "ok" in out and model_holds(cnf.obligation(), out) is None
        decoded = decode_factors(meta, out) if holds else "no model"
        checks.append((f"factoring N={meta['N']} ({bits}-bit factors): dpll's model holds and decodes to the factors ({decoded})",
                       holds and decoded.endswith("x*y==N")))
    for name, passed in checks:
        print(f"selftest {'pass' if passed else 'FAIL'}: {name}", flush=True)
        ok &= passed
    (run / "selftest.md").write_text("# Generator self-test\n\n" + "\n".join(f"- {'pass' if p else 'FAIL'}: {n}" for n, p in checks) + "\n",
                                    encoding="utf-8")
    try:
        scratch.rmdir()
    except OSError:
        pass
    return ok


# ---------------------------------------------------------------------------


def main() -> int:
    if len(sys.argv) > 1 and sys.argv[1] == "worker":
        ap = argparse.ArgumentParser()
        ap.add_argument("worker")
        ap.add_argument("perf")
        ap.add_argument("instance")
        ap.add_argument("out")
        ap.add_argument("--trace", action="store_true")
        ap.add_argument("--deadline", type=float, default=TIMEOUT)
        a = ap.parse_args()
        worker(pathlib.Path(a.perf), pathlib.Path(a.instance), pathlib.Path(a.out), a.trace, a.deadline)
        return 0

    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("--stage", default="all", choices=["all", "selftest", "records", "cost", "report"])
    ap.add_argument("--families", default=",".join(FAMILIES), help="comma-separated: " + ", ".join(FAMILIES))
    ap.add_argument("--perfs", default=",".join(PERFS), help="comma-separated: " + ", ".join(PERFS))
    ap.add_argument("--sizes", help="comma-separated sizes, overriding every chosen family's list")
    ap.add_argument("--surety", default=str(ROOT / "target" / "release" / "surety"))
    ap.add_argument("--run", help="an existing run directory to add to (default: a new experiments/sat/runs/<stamp>)")
    args = ap.parse_args()

    surety = pathlib.Path(args.surety)
    run = pathlib.Path(args.run) if args.run else RUNS / dt.datetime.now().strftime("%Y%m%d-%H%M%S")
    run.mkdir(parents=True, exist_ok=True)
    families = [f for f in args.families.split(",") if f]
    perfs = [p for p in args.perfs.split(",") if p]
    if args.sizes:
        sizes = [int(s) for s in args.sizes.split(",")]
        for f in families:
            FAMILIES[f] = dict(FAMILIES[f], sizes=sizes)
    for f in families:
        if f not in FAMILIES:
            raise SystemExit(f"unknown family {f}")
    for p in perfs:
        if p not in PERFS:
            raise SystemExit(f"unknown performance {p}")

    stages = ["selftest", "records", "cost", "report"] if args.stage == "all" else [args.stage]
    if "selftest" in stages and not selftest(run):
        print("selftest failed; the generators are not trusted, stopping", file=sys.stderr)
        return 2
    if "records" in stages:
        if not surety.exists():
            print(f"no surety binary at {surety}; build with cargo build --release", file=sys.stderr)
            return 2
        records(run, surety)
    if "cost" in stages:
        cost(run, families, perfs)
    if "report" in stages:
        report(run)
    print(f"run directory: {run}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
