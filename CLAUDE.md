# Surety

Surety is an obligation language plus a checker plus a graph. Humans write obligations (types, properties, examples). A model writes performances (implementations) in a target language; Python is the first target. The checker verifies a performance against its obligation and pins the evidence to both hashes in the graph under `graph/`. A model writes performances through `experiments/closed_loop.py` and drafts obligations through `experiments/draft.py`; the binary itself stays offline. Slices 1 to 3 are built (parser and checker; generated inputs and the mutant library; the report, Time at the boundary, the behavioural diff, the store); slice 4, author tooling, is current.

## Sources of truth, in order

`surety-docs/` is local-only: it is gitignored as internal, so a clone does not have it. `docs/` is public and holds the spec, the reframe, the problem table, and the decision notes. Beyond those, the repository carries the extracted fixtures and the acceptance tests under `crates/surety/tests/`; the rules below stand on their own.

1. `surety-docs/Implement 1-9 in surety.md` — the corpus: eleven obligations written by hand. The grammar is whatever parses these. When the brief and the corpus disagree, the corpus wins and the disagreement is a question (see below).
2. `surety-docs/Build brief — slice 4, author tooling.md` — scope, interfaces, acceptance tests for the current slice. The earlier briefs (slice 1; slice 2; slice 3, the store) say what was built and why, each with a header naming its deviations.
3. `surety-docs/Catch-up — 2026-10-20, ...md` — every rule decided so far, one line each; the mutant list.
4. `docs/Decision - *.md` — individual decisions with their reasoning.
5. `docs/Surety Spec v2.md` and `docs/The Target Reframe.md` — the current spec and the decision record behind it: Surety is the obligation language, the checker, and the graph; the model writes performances in a target language (Python first); Surety's own core is deferred behind four named triggers. Read for intent and architecture, not for syntax; the syntax lives in the corpus and the brief. `surety-docs/Surety - Agent-Native Language Spec.md` is v1, superseded on 2026-10-19 and kept unchanged for history; its validation plan (experiment numbering, the model-competence gate, constrained decoding) does not apply under v2.

## Rules for working here

- Do not invent syntax. The language surface is the list in the build brief; the corpus is its test. A form that is in the brief but in no fixture (`any`, `none`, `count`, `max`, `->`, `<`, `[0..3, unique f]`) needs a unit test of its own, since no fixture will catch it. A form in neither place does not exist.
- When the corpus is ambiguous, pick the reading that makes every corpus file parse, implement that, and write the question with the reading you chose in `surety-docs/Questions from the build.md`. Do not block on it.
- The eleven obligations are fixtures. Copy them into the test tree verbatim; never edit a fixture to make a test pass.
- A property is a claim about the relation between input and output, never about execution. The checker evaluates properties by brute force over example-sized inputs; cost on the checker side is not a concern.
- `graph/` is committed and must reproduce: after changing a corpus obligation or performance, re-pin it with `surety pin` (callees and type providers first), or the store test fails. Nothing under `graph/nodes/` is edited by hand.
- Keep crates small and few. Rust, edition 2024, workspace already set up.
- Commit messages: plain, present tense, one line of what changed. No attribution trailers.
