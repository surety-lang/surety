# Surety

Surety is an obligation language plus a checker plus a graph. Humans write obligations (types, properties, examples). A model writes performances (implementations) in a target language; Python is the first target. The checker verifies a performance against its obligation and pins evidence. Nothing in this repo generates code yet; slice 1 is the parser, the example and property checker, and the Python runner.

## Sources of truth, in order

1. `surety-docs/Implement 1-9 in surety.md` — the corpus: eleven obligations written by hand. The grammar is whatever parses these. When the brief and the corpus disagree, the corpus wins and the disagreement is a question (see below).
2. `surety-docs/Build brief — slice 1.md` — scope, interfaces, acceptance tests for the current slice.
3. `surety-docs/Catch-up — 2026-10-20, ...md` — every rule decided so far, one line each; the mutant list.
4. `surety-docs/Decision - *.md` — individual decisions with their reasoning.
5. `surety-docs/Surety - Agent-Native Language Spec.md` — the v1 spec; design principles and architecture. Read for intent, not for syntax; the syntax there predates the corpus.

## Rules for working here

- Do not invent syntax. The language surface is the list in the build brief; the corpus is its test. A form that is in the brief but in no fixture (`any`, `none`, `count`, `max`, `->`, `<`, `[0..3, unique f]`) needs a unit test of its own, since no fixture will catch it. A form in neither place does not exist.
- When the corpus is ambiguous, pick the reading that makes every corpus file parse, implement that, and write the question with the reading you chose in `surety-docs/Questions from the build.md`. Do not block on it.
- The eleven obligations are fixtures. Copy them into the test tree verbatim; never edit a fixture to make a test pass.
- A property is a claim about the relation between input and output, never about execution. The checker evaluates properties by brute force over example-sized inputs; cost on the checker side is not a concern in slice 1.
- Keep crates small and few. Rust, edition 2024, workspace already set up.
- Commit messages: plain, present tense, one line of what changed. No attribution trailers.
