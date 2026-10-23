# Reach-for census (2026-10-23)

The census measures which forms a model reaches for that the property fragment lacks, over real pure Python functions. `pick_functions.py` picks the functions and writes one problem file each; `run_census.py` runs the draft, both stages, per problem and checkpoints to `results.jsonl`. It passes stage 2 `--census`, `draft.py`'s census-only flag: the slice 6 lines stage refuses a file with no example, no census draft has one, and the flag skips that gate so the second half runs on the terms of the first, drafted before it. Each record says so (`census_flag`, and `stage2.example_gate` from the run directory's `census.json`), and carries `prompt_sha256`, the hash of the prompt material every problem shares. `NOTICE` names the libraries the problems quote, with their licenses.

## Running it locally

1. `pip install anthropic` (and, only to regenerate the problems, `more-itertools toolz boltons django requests humanize`).
2. `cargo build --release -p surety`.
3. Credentials come from the SDK: `ANTHROPIC_API_KEY` or `ANTHROPIC_AUTH_TOKEN` in the environment.
4. `python3 experiments/census/run_census.py --jobs 3` from the repository root. It writes `out/<id>.sur`, `logs/<id>.<stage>.txt`, one record per function to `results.jsonl`, and the usual draft run directories under `experiments/draft/runs/`. A stopped run resumes: Ctrl-C starts nothing new and records only what finished, functions with a complete record are skipped; `--retry-failed` re-runs incomplete ones; `--limit N` and `--only <id> ...` narrow a run. `--jobs` is the number of problems in flight; 3 is gentle on rate limits.
5. Each stage uses the draft's defaults (`claude-opus-5-5`, effort high); `--model` and `--effort` pass through.

`pick_functions.py` regenerates `functions.jsonl`, `skipped.jsonl` and `problems/` from the installed libraries (333 selected, 177 skipped with a reason). The problems committed here came from more-itertools 11.2.1, toolz 1.2.0, boltons 26.2.0, Django 6.1.2, requests 2.34.2 and humanize 4.16.0.

## What to push back for the review

`experiments/census/results.jsonl`, `experiments/census/out/`, `experiments/census/logs/`, and the run directories under `experiments/draft/runs/` from the census run. The classification and the report are built from those.

## What the checker accepts today

Probed 2026-10-23 against `target/release/surety parse`: `+ - *` on Int, `abs(x)`, the slice `xs[a .. b]`, the closed range `Int[0..3]`. Not yet: the open range `Int[0..]`, `xs[i]`, `/`, `%`, unary minus on a name, `+` on Float or Text, tuples, maps, generics, `Some(x)` as a binder.
