# Surety Spec v2

Oct 19, 2026 · @Bennett Dixon

Supersedes v1 (2026-10-18). v1 is preserved unchanged. The difference is the target reframe: Surety is the obligation language, the checker, and the graph; performances are written by the model in a target language, Python first, with Surety's own core deferred to named triggers.

## BLUF

**Surety is an obligation language, a checker, and a graph. Humans write obligations; a model writes the performance in a target language; the checker verifies the performance against the obligation behind a sandbox boundary and pins the evidence to both.** Python is the first target. Surety's own core language is a deferred target, built when a named trigger fires. Start with pure data transformations.

- **The thesis was never the syntax.** The economics of software flipped: writing is nearly free, verification is the bottleneck. What makes a model reliable is a truthful oracle in the loop, the spec present at generation time, small units, and no ambient state. v2 supplies those over an existing language and defers the new one.
- **The obligation is target-agnostic and is the durable asset.** Nine obligations written by hand in the first exercise mention no implementation language. The corpus, the claim shapes, and the mutant library are the whole product.
- **Trust by boundary, then by proof.** The sandbox enforces purity for every target today; the core target proves it later. The evidence table says which, per definition, so nothing is promised before it exists.
- **The novelty is assembly, not invention.** Content-addressed obligations (Unison), a restricted property fragment (QuickCheck, Hypothesis), mutation against the spec, a container-style boundary (OCI), arranged with the model as author and tokens as the cost function.
- **First gate.** Nine obligations, model-written Python, the checker fills the evidence table, and the mutant that passed the filter obligation before "nothing lost" was added is on screen. If the evidence table does not replace reading for a reviewer, the spec layer has failed and the project stops.

**Name: Surety.** A surety is the party that stands behind another's performance; it was never the party that performs. GitHub org `surety-lang`, npm org `surety-lang`, crates.io `surety` (claimed and published as a 0.0.1 placeholder). CLI `surety`, obligation files `.sur`.

## Why now

Every existing language was designed for a world where writing was expensive and the writer was also the reader. Writing is now near-free and the bottleneck is verification. The language ladder (assembly → C → Python) was never about ease; it was about how much intent survives into the artifact. That is the quantity to optimize.

| Driver | What changed | What it demands |
| --- | --- | --- |
| Token cost | Reading code now costs money; generating costs more | Callers see contracts, never source; retrieval by node |
| Context pollution | Agents re-read source because no shorter artifact is trustworthy; stale docs are a token tax | Verified obligations pinned to hashes; a contract that is checked is a summary that can be trusted |
| Trust | Humans read code because they do not trust the system | A sandbox boundary plus an evidence table as the reviewed surface; residuals are what humans read |
| Good practice | Edge cases, boundaries, and intent are discovered in bug reports | The obligation forces them to be decided at spec time; "keep first occurrence" became two functions before any code existed |
| Model competence | Models outperform their training share where a fast, truthful oracle pushes back | Put the oracle in the loop over a language models already speak |

**What changed from v1.** v1 planned a new implementation language first and the trust layer after. The first day of writing obligations by hand showed that the obligations are target-agnostic, that the spec layer is where all the difficulty lives, and that everything a new implementation language buys (provable purity, proof-strength evidence, canonical hashing) has a trigger that v0 does not hit. So v2 builds the spec language, the checker, and the graph over Python, and names the triggers for the core.

The field confirms the gap. SPELL, AILang, Pel, NERD, and MoonBit attack the generation half (syntax a model can emit without hallucinating). None is centered on the trust half: spec as the durable artifact, evidence attached to definitions, a boundary that bounds blast radius, and a projection that replaces reading source. Under v2, Surety does not compete with those projects at all; it could sit over any of them as a target.

## Design principles

Every layer is judged against these. A feature that violates one does not ship, however convenient.

1. **Optimize for the reader and the verifier, not the writer.** The writer is a model and its hands do not hurt. Terseness for its own sake is the old paradigm.
2. **Bits of intent per token is the cost function.** Not fewer tokens; more intent per token. Boilerplate is cheap and worthless.
3. **Intent is attached to the thing it describes, never beside it.** A property, example, or prose note is pinned to the hash of its definition. Drift is detected or impossible; stale is never served.
4. **Trust is granted by scope, not earned by reading.** A performance behind a boundary that admits nothing but computation needs review for correctness within its obligation, not for exfiltration.
5. **Push verification out of the token budget.** Every property a deterministic checker can settle is tokens the model never spends re-reading its own output. The design goal of the toolchain is to shrink the residual.
6. **The obligation is the durable asset; the performance is regenerable.** A full rework is cheap only if the obligation is what you keep. A spec that is prose in a markdown file is the wrong thing to keep.
7. **The model never reads a file.** It queries the graph for a node, its obligation, and one hop of neighbors. Everything else is one more query away.
8. **Tool outputs are terse and structured by default.** Cheap CPU is only an advantage if its output is not paid for in tokens on the way back into context.
9. **Every performance is an untrusted component.** The checker sees only values crossing the sandbox boundary. The boundary is the trust boundary, for Surety-written code and foreign code alike.
10. **Properties describe input-to-output relations, never execution traces.** A property that needs an intermediate state belongs in the performance. If the model could produce the implementation by copying the property, it is a trace, not a property.
11. **Evidence is never inherited.** Not from a near-identical definition, not from a previous version, not from the model's confidence. Nothing counts until the checker runs on this exact obligation hash and performance hash and records the result. The rule costs CPU, which is the cheap resource.

## Architecture

Four layers and a graph beside them. The obligation never changes shape when the target does; the checker is target-agnostic because it only sees values crossing the sandbox boundary; targets are plugins.

*[diagram, in the live doc: architecture · obligation, checker, three targets, one graph]*

**Interfaces between layers**

- **Surfaces → graph:** a query API, never file reads. Requests name a node; responses are the obligation, its evidence status, and one hop of neighbors.
- **Obligation → graph:** hash-addressed writes. Normalization contract: the hash is computed over a canonical form in which record fields, properties, and examples are sets and bound names are erased, so two obligations that differ only in clause order or local names hash identically. Formatting, comments, and the prose note are pinned to the hash, not part of it. Example order is kept for diagnostics and ignored for identity.
- **Checker → target:** a performance is invoked with a value of the obligation's input type and returns a value of its output type, or fails. The checker never inspects the performance's source. The target's boundary validator enforces the types on both sides.
- **Target → graph:** a performance is attached to its obligation keyed by target, with a structural hash from the target's normalizer. One obligation may hold several performances.
- **Checker → kernel → graph:** the checker produces an evidence bundle; the kernel (small, hand-written, human-audited) validates that each claim in the bundle is backed by a recorded run before it is pinned.

**What a target supplies.** Four things, and nothing else: a sandbox runner, a boundary validator, an AST normalizer for the performance hash, and an evidence-kind profile (the table in the Evidence kinds section).

**Checker queries, v0.** Type and totality checks apply to the obligation. Against a performance: examples run, properties are tested with generated inputs, determinism is tested by running twice. Against the obligation alone, with no performance present: the **underdetermination check**. A library of mutant performances (return empty; return the input unchanged; drop one element; duplicate one; append a fabricated one; swap two adjacent elements; satisfy each clause independently) is run against the obligation. Any mutant that passes is reported with the clauses it slipped through. This is adversarial review pointed at the spec, and it is what makes the spec layer workable for an author who is not a senior engineer.

## Layers and vocabulary

The terms come from how a surety bond works: a principal owes an obligation, and a surety stands behind the performance.

| Layer | Artifact | Author | Contents |
| --- | --- | --- | --- |
| Spec | **Obligation** | Human (a model may draft; a human approves) | Type declarations, signatures, examples, properties, named predicates, prose notes. No bodies. Written in Surety |
| Implementation | **Performance** | Model | A function body in the declared target language (Python in v0). Not read by humans by default |
| Checker | **Evidence** | Tooling | Per-obligation status by evidence kind: types validated, examples N/N, properties tested N runs, determinism N/N, purity enforced or proven, mutants passing: none |

A definition is one obligation, one or more performances keyed by target, and one evidence record per performance, all pinned by hash. An obligation with no performance is open; that is what the model is handed. A performance whose obligation hash has changed is stale and is never served as verified.

**Why the obligation is its own language and the performance is not.** The obligation has to be checked, analyzed by the mutant library, and read by a product owner; that needs a small, restricted grammar. The performance only has to run behind a boundary; any language with a sandbox will do. The v1 argument for one language (no drift between spec and body) survives: the body is checked against the spec on every change regardless of what it is written in.

**What a human writes.** Obligations. In v0 that is types, examples, and properties; pre/post contracts arrive with the trust layer. Sketch, exact syntax to be fixed by the consolidation pass:

```
type Order   = { customer: Text, amount: Int }
type Summary = { customer: Text, total: Int }

target python

def top_customers : List Order -> List Summary
  note "customers whose order total exceeds 100"
  all x in out: x.total > 100
  example
    in:  [{customer: "a", amount: 60}, {customer: "a", amount: 50}, {customer: "b", amount: 10}]
    out: [{customer: "a", total: 110}]
```

**What the model writes.** A performance in the declared target for each open obligation. The checker runs it against every clause and emits evidence. The human reads the evidence, and opens the performance only when the evidence reports a residual.

**Usage rule.** "Spec" and "obligation" refer to the human layer; "performance" and "implementation" to the model layer; "evidence" to the checker's output; "target" to the language a performance is written in. "Canonical form" now refers only to the obligation's hashed form and, later, to the Surety-core target's serialization.

## Evidence kinds by strength

This table is the honest account of what "verified" means per target. It is what the projection shows, and it is how the Surety-core target is justified: row by row, when someone needs the stronger column.

| Evidence kind | Python target (v0) | Surety-core target | What moves it |
| --- | --- | --- | --- |
| Types at the boundary | Validated at runtime on every call | Checked statically | Nothing; both are complete for the boundary |
| Examples | Run; N/N pass | Run; N/N pass | Nothing |
| Properties | Tested; run count recorded | Tested, or discharged by SMT where the fragment allows | Someone needs "always" |
| Underdetermination | Mutant library run; passing mutants reported | Same | Nothing |
| Purity | Enforced by sandbox (empty import set) | Proven by type | Fine-grained scopes or subterm purity |
| Determinism | Tested: run twice, compare, N/N | Proven by purity | Same as purity |
| Totality | Tested: time cap, no timeouts in N | Checked structurally | Someone needs "terminates" rather than "did not time out" |
| Performance identity | Structural hash of normalized AST | Canonical hash | Hash churn, or identity needed at graph scale |
| Capability scope | Boundary-wide: all or nothing | Per definition, in the type | v1 effects |

An evidence record that reads `purity: enforced, determinism: 500/500, properties: 4/4 tested` tells a reader exactly what was and was not established. That is better than a language that promises `proven` before it exists, and it is the mechanism by which the core gets built for a reason rather than a hope.

## Human-facing surfaces

Humans get two surfaces. The spec language is the real UX bet and the most likely point of failure; the projection is what replaces reading code. Everything else a human might want is a query against the graph.

**The spec language.** Humans write intent, not implementation. A spec is partial by design and grows in layers of strength:

| Layer | Form | Checked by | Cost to write |
| --- | --- | --- | --- |
| Types | Input and output shapes, including refinements | Boundary validator | Low |
| Examples | Concrete input → expected output pairs; `in` always a full literal, `out` may end in `...` for fields a property covers | Execution | Low; the floor, and the one layer the model never drafts |
| Properties | Whole-shape claims about the result: element identity, order, correspondence, uniqueness, complement, iff | Generated inputs, then SMT where the fragment allows (core target only) | Medium; the model drafts, the human reviews |
| Named predicates | A declared boolean function properties reference (`well_formed_email`) | Treated as a property's operand | Low |
| Contracts | Pre- and post-conditions | Checker (v1) | Medium to high |
| Prose notes | Free text, hash-pinned to a definition | Staleness detection only | Low |

The exercise that produced the first nine obligations found that examples alone make the spec layer unworkable past trivial functions, that properties cost more to compose than code and less to review, and that the hardest skill (imagining the wrong implementation) is exactly what the underdetermination check automates. So the workflow is: the model drafts the obligation, the human writes the examples by hand and reviews the rest, the checker reports what the obligation fails to constrain.

**The projection.** What a reviewer sees instead of source, in this order:

1. **Evidence**, by kind, from the table above: what was established and how strongly.
2. **Residuals.** Claims no checker could settle and no cheap model could judge. This is what the human reads.
3. **Underdetermination.** Mutants that passed, with the clauses they slipped through.
4. **Behavioral diff.** For a change: the inputs on which output differs, with three concrete examples. Never a text diff.
5. **Source, on demand.** The performance in its target language. Always one query away, never the default.

The honest version of "I want to read the code" is "I want to check it myself when I don't trust the system." Evidence, residuals, and the mutant report satisfy the check.

**Hypothesis: one definition, one projection per audience.** Status: hypothesis. The obligation's layers have different natural readers. A product owner reads and writes the `note` and the examples. An engineer writes the types and properties. The model writes the performance. All three are parts of one hash-pinned artifact, not translations of each other, which is why this might work where UML, model-driven design, and Gherkin did not. The product-owner layer is the least-checked layer, so the evidence summary has to be designed as a product surface. Test: hand a non-engineer a one-line ask, have them write the note and examples, write the properties, and count what the properties catch. See also the separate notes on the apprenticeship hypothesis and the hardware parallel.

## Model-facing surface

The model is the author of performances. Under v2 it writes the declared target language, which in v0 is Python, so the surface is mostly about what it is handed and what it gets back.

- **It is handed a finished obligation and one hop.** The obligation, the types it references, and the obligations of definitions it may call. Never a file. Context occupancy by category (obligation, neighbors, tool output, history) is tracked per session as a first-class metric.
- **It asks about gaps before the first stroke.** Coverage and finality are separate: an obligation may be partial in what it covers; what it states is fixed before generation starts. Proposed amendments come back as a spec delta, never as edits to the obligation mid-generation.
- **The checker is the training signal.** Every performance runs through the checker. Diagnostics are machine-readable, name the failing clause, and carry a concrete counterexample. A failed property comes back as an input and two outputs.
- **Mechanical content is not its job.** The boundary validator, the sandbox harness, and serialization are supplied by the target. The model writes the function.
- **Stable identity feeds the cache.** The obligation's canonical hash is the prompt-cache key. Unchanged obligations are served from cache at near-zero marginal cost.

**Fluency risk shifts; it does not vanish.** The model has native fluency in Python and none in reading `.sur` files or respecting the boundary contract (strict records, `None` for an empty `Option`, a tagged pair for `Result`). That skill has no pretraining data either. It is a smaller gap than writing a new language, and the same medicine applies: few-shot from the obligation corpus, and a checker-verified set of obligation/performance pairs as manufactured training data. Measure it the same way experiment 3 would have: pass rate on the held-out suite over hours of exposure.

**Deferred to the Surety-core target.** The tokenizer-aware canonical form, constrained decoding against a grammar, and the synthetic-corpus bootstrap (transpile verified code, keep what passes, fine-tune). These were v1's model surface. They are real and they are the right design for a new language; they are not needed while the target is one the model already speaks.

## Data flow: lifecycle of a change

A change is an obligation delta that ends as two hashes in the graph (obligation, performance) with evidence pinned to both. The human touches it twice: at the obligation and at the residuals.

*[diagram, in the live doc: lifecycle of a change · 7 steps, 2 decisions, 2 loops]*

The inner loop (regenerate on a counterexample) is cheap and runs without a human; the outer loop (amend the obligation) is where human tokens go. The mutant library runs at the first step, before any performance exists, so a weak obligation is caught before the model spends a token on it.

**Coverage and finality are separate.** An obligation may be partial in what it covers; it is never partial in what it states. What the obligation says is fixed before generation starts. The model's job of surfacing gaps ("you did not say what happens on an empty list") happens before the first stroke, never during; amendments come back as a new spec delta.

**What moves between steps**

| Artifact | Produced by | Consumed by | Shape |
| --- | --- | --- | --- |
| Obligation delta | Human (or model, on a surfaced gap; human approves) | Mutant library, then model | Types, examples, properties, named predicates; hash-pinned |
| Mutant report | Checker | Human | Passing mutants with the clauses they slipped through; empty if the obligation is tight |
| Performance | Model | Checker | A function in the target language, opaque to the checker |
| Diagnostics | Checker | Model | Failing clause plus a concrete counterexample: an input and two outputs |
| Evidence bundle | Checker, validated by the kernel | Projection, graph | Per-kind status from the evidence table |
| Residual set | Tooling | Human | Claims no checker or cheap model could settle; the only thing the human reads |
| Merge record | Graph | Everyone | Obligation hash, performance hash keyed by target, evidence, provenance |

## Git integration

Unchanged from v1 in substance. Git stays as a durable store and transport, never as the thing that understands the code. The graph serializes to a canonical on-disk form git can hold; the toolchain does every semantic operation before git sees a conflict.

| Git concept | Maps to | Notes |
| --- | --- | --- |
| File | One obligation or one performance, named by its hash | Deterministic serialization; zero formatting churn. Adding definitions on two branches never conflicts |
| Rename / move | Metadata change on a label | Names are labels; moving code is a no-op in the graph |
| Commit | Merge record | Carries provenance: which obligation, which target, which performance hash, which evidence |
| Blame | Provenance query | "Which obligation delta and which generation produced this hash" |
| Conflict | Two versions of one obligation | Resolved semantically by the toolchain, or both kept as distinct hashes and the human picks from a behavioral diff |
| Pull request | Change bundle | Obligation delta + mutant report + performance + evidence summary + residual set. GitHub still shows a text diff; the review surface is the projection, surfaced into the PR as a check |
| CI | The checker | Green means the evidence bundle is complete and the residual set is what the human must read |
| Build artifacts | Evidence detail and sandbox images | Summaries go in git; full runs and runner images live in a cache keyed by hash |

**What is source and what is derived.** Obligation deltas are the human-authored commits. Performances are generated and are committed in v0 anyway, because generation is not reproducible enough to be a build step, and because the evidence is pinned to the performance hash. One change from v1: a performance file carries its target in its name, so one obligation's Python and core performances sit side by side.

**Hard rules.** Serialization is deterministic and one-definition-per-file, with a registered merge driver; raw text merge is never the fallback. A behavioral diff is posted on every PR. The residual set is the merge gate.

## Build

The toolchain is Rust, the projection is TypeScript, the experiments are Python. The deciding reason stands from v1: the toolchain will be largely agent-written, and Rust is where the checker loop is most proven. The project is much smaller than v1's, because there is no compiler to write.

| Part | Contains | Language | Notes |
| --- | --- | --- | --- |
| Obligation parser | The `.sur` grammar: types, properties, examples, named predicates, notes, target declaration; normalization and hashing | Rust | The only language Surety defines in v0 |
| Checker | Generators from types, property evaluation over boundary values, example runner, determinism and timeout checks | Rust | Target-agnostic by construction |
| Mutant library | The underdetermination check: a fixed set of wrong performances run against the obligation alone | Rust | Grows as the corpus finds new kangaroos |
| Target: Python | Sandbox runner, boundary validator, AST normalizer, evidence profile | Rust host, Python guest | Runner choice is an open question (WASM-hosted Python vs locked subprocess) |
| Kernel | Validates that every claim in an evidence bundle is backed by a recorded run, before it is pinned | Rust, hand-written, small, human-audited | Never rewritten; the only trusted part. More important than in v1 because it now validates evidence from several targets with different strength profiles |
| Graph store and query engine | Files-by-hash on disk, git-backed; salsa for incremental queries; node + one hop; behavioral diff | Rust | No database until a measurement demands one |
| Projection UI | Evidence table, residuals, mutant report, behavioral diff, source on demand | TypeScript |  |
| Experiments | Context audit, summary substitution, residual review | Python | Throwaway instrumentation |

**The Surety-core target, deferred.** v1's pure core (parser, type and totality checker, contract elaboration, canonical form, WASM emission) and its model surface (constrained decoding, synthetic corpus) become a target like any other, built when one of these fires:

1. Someone needs `purity: proven` or per-definition capability scopes. This is v1 effects and is the expected trigger.
2. Someone needs `properties: discharged` or `totality: checked` rather than tested.
3. Performance hash churn makes regenerations unreadable as diffs, or identity is needed at graph scale.
4. A confirmed sandbox leak in the v0 target, which makes `enforced` false and promotes `proven` from nice-to-have to necessary.

When it is built, its profile is the right-hand column of the evidence table, and self-hosting (the pure core rewritten in Surety-core with its own obligations) remains the long-term milestone it was in v1.

**Community fit.** Unchanged. The audience most likely to try this is in the Rust and WASM orbit, and a spec language with a checker over Python reaches the much larger audience that already uses Hypothesis and wishes it were more than a test file.

## v0 scope: data transformations, Python target

v0 is the obligation language, the checker, and the graph, over pure data transformations with Python as the performance target. This is the smallest scope in which every thesis can be tested.

**Why this category first.** It is the largest share of agent-written code nobody enjoys reviewing. It is pure by nature, so the sandbox boundary with an empty import set is the whole capability story. Properties are natural to state (the nine obligations needed element identity, order, correspondence, uniqueness, complement, iff, and nothing harder), and the function is total by construction with a time cap.

**In scope**

| Capability | v0 commitment |
| --- | --- |
| Performance target | Python, declared on the obligation. The model writes a single function per obligation |
| Boundary contract | Strict records: every declared field present, none extra. `Option<T>` as a value or `None`. `Result<T, E>` as a tagged pair. Lists as lists. Validated on input and output of every call |
| Types | Records, sums (`Option`, `Result`), lists, maps, primitives including `Time` with an ISO-8601 literal. Refinement types on primitives and on list length |
| Properties | The property fragment, tested with generated inputs: a quantifier family (`all`, `any`, `none`, `exactly_one`) over lists; a pairwise order quantifier; whole-list relations (unique, subsequence-of, same-elements-as, sorted-by with a lexicographic key); comparison, arithmetic, field access; implication and iff; a broadcast form over a list's field. Spellings provisional until the consolidation pass closes |
| Named predicates | A declared boolean function that properties reference. Required: the validation obligation cannot be stated without it |
| Named example values | A literal bound once and reused across examples |
| Underdetermination check | The mutant library runs against every obligation; passing mutants are reported with the clauses they slipped through |
| Graph | Content-addressed obligations under the normalization contract; performances attached by target with a structural hash |
| Query engine | Node + obligation + one hop; behavioral diff; counterexample search |
| Projection | Evidence table by kind, residuals, mutant report, behavioral diff. Source view on demand |
| Sandbox | Empty import set, no filesystem, network, or clock; seeded randomness; fixed hash seed; time and memory caps; determinism check |

**Deliberately out of v0**

- Effects and capabilities beyond the all-or-nothing boundary. The evidence-kind slot exists; nothing populates it until v1.
- Pre/post contracts and SMT discharge.
- A second target. TypeScript is the obvious one and it waits until the Python target is measured.
- The Surety-core target, per its triggers.
- A standard library of any kind. Performances use the target's own.
- Importing foreign components. v0 performances are called; they do not call.

**v1 candidates, in order:** read-only capabilities (data sources) as the first effect class, which is also the first trigger for the core target; then write capabilities; then service handlers composed from v0 transformations. Each step adds one effect class and tests whether the boundary model holds under it.

## Validation plan

Three experiments gate the project, two first-month checks gate the Python target, and one metric measures the spec layer directly. v1's experiment 3 (model competence in a new language) is retired as a gate and returns when the Surety-core target is built.

| # | Experiment | Measures | Kill if |
| --- | --- | --- | --- |
| 1 | **Context audit.** Instrument real agent sessions on an existing codebase. Break context down: first reads, re-reads, tool output, history, docs | Tokens per accepted change; re-read ratio; share of context that is verifiable by tooling | Re-reads and tool output are not the dominant cost |
| 2 | **Obligation substitution.** For a subset of functions, write obligations and serve those to callers instead of source. Same tasks, same model | Delta in tokens per accepted change; delta in defect rate | No token reduction, or defects rise |
| 4 | **Residual review.** Give reviewers only the projection (evidence, residuals, mutant report, behavioral diff) for v0 changes. Compare to source review | Defects caught; time per review; how often reviewers open source anyway | Reviewers open source on most changes, or miss defects source review catches |

**First-month checks on the Python target**

| Check | Measures | If it fails |
| --- | --- | --- |
| Sandbox adversarial test | Attempt escape from the runner: imports, reflection, resource exhaustion, clock and randomness leakage | A confirmed leak is trigger 4 for the core target; the v0 evidence profile changes `purity: enforced` to `purity: unverified` until fixed |
| Hash churn | Regenerate each of the nine performances ten times; count distinct structural hashes | A high rate is trigger 3; the measurement is the justification |

**Metrics that are first-class from day one** (tracked per session by the toolchain):

- Tokens per accepted change, split input and output
- Context occupancy by category
- Residual size per change, and the trend over time as the checker grows
- Evidence by kind: share tested vs. enforced vs. proven, per target
- Source-open rate: how often a human asks for the source view
- Underdetermination rate: how often the mutant check fires on human-written obligations, and the trend as authors learn. This is the direct measure of the spec layer's difficulty

**Gate to the core target:** one of the four triggers in the Build section, with the measurement that fired it. **Gate to continuing at all:** experiments 1 and 2 show the token thesis holds, and experiment 4 shows the projection replaces reading for a reviewer. If 4 fails, the spec layer has failed and no language fixes it.

## Risks and open questions

The spec layer is the risk that kills the project; the rest delay it.

| Risk | Why it matters | Mitigation |
| --- | --- | --- |
| Humans won't write specs | 4GLs, model-driven development, and Dafny all stalled here. The first day of writing obligations by hand confirmed the cost is real | The model drafts the obligation and the human reviews; examples are hand-written and cheap; the underdetermination check tells the author when a spec is too weak, so the adversarial skill is not required of them; v0 picks a domain where properties are natural to state |
| The model drafts both halves | A model-drafted obligation is shaped by what the model finds easy to satisfy, and a plausible drafted spec can lull a reviewer worse than code does | The mutant library is the first defense; the second is that examples are always human-written, since they are the one layer the model cannot bend toward itself |
| Goodhart on the checker | Every unstated property is a degree of freedom the model will use in whatever way is easiest | Make it visible rather than pretend it is gone: the evidence table shows which clauses constrain, the mutant report shows the slack |
| Sandbox leaks | Enforced purity is only as good as the sandbox, and Python sandboxes have histories | Choose the runner for isolation first; adversarial test in the first month; treat a confirmed leak as the core target's trigger arriving early |
| Hash churn | AST normalization may still produce a new hash on most regenerations, filling the graph with near-duplicates | Measure on the nine obligations in the first month; a high rate is the canonical-hash trigger |
| Underdetermination is the floor, not a bug | For functions that are large case analyses, the properties are the implementation and there is no compression | Accept it: v0 is data transformations where invariants are elegant; the generalization is tested one domain at a time, and the validation shape (one iff per check) is the honest form for rule tables |
| The trust layer has no demo | "You didn't read the code and nothing broke" is a statistic that takes months | The first demo is the nine obligations with the evidence table and the mutant that passed the filter obligation; residual size and source-open rate visible from the first session |
| Chatty tooling | Verbose tool output recreates markdown soup one call at a time | Terse structured output is a hard rule; detail is one more query away |
| Checker cost | Property testing has real latency; a slow oracle breaks the loop | Tiered: boundary types instantly, examples in seconds, properties and mutants asynchronously; the model proceeds on the fast tier |
| Unpatchable density | Whole-definition generation means a small obligation change forces a full regeneration | Keep definitions small and connect them through the graph |
| Reads as a retreat | "We decided not to build the language" can sound like the thesis failed | The thesis was never the syntax; say so, and keep the trigger list public |
| Reading code stays good enough | The stable end state may be humans who read code fluently without writing it, because code is the best bridge between intent and deterministic behavior we have. If that holds, the projection never earns their trust and the evidence table is a curiosity beside the source view | This is what experiment 4 tests, and it is a different failure from "humans won't write specs." Keep source one query away so the design does not depend on winning the argument; measure source-open rate from the first session; and make the projection show what reading cannot (the mutant report, the behavioral diff), so it is additive to reading rather than a replacement someone has to accept |

**Open questions**

- [ ] Which Python sandbox: a WASM-hosted Python, or a locked subprocess with seccomp? Isolation first, then speed.
- [ ] What does the boundary do with values the obligation's types cannot express (NaN, oversized ints, non-UTF-8 text)? Reject at the boundary, or widen the type language?
- [ ] Does the spec language need its own surface syntax beyond the `.sur` grammar, or is one grammar enough for human and model readers?
- [ ] Who merges: can the checker auto-merge when the residual set and the mutant report are both empty, or does a human always approve?
- [ ] Objectives. Every obligation so far is a constraint. Some real specifications are objectives (the valid output that minimizes X). An `optimize` clause is outside v0, and property testing cannot confirm optimality, so admitting it later means a new evidence kind or an honest "not checkable" status.
- [ ] Trailing commas, casing of primitives, the two meanings of `in`: the small grammar decisions the corpus made by accident, to be made on purpose in the consolidation pass.

## Prior art

Nearly every piece has a mature reference. None are assembled with the model as the author and tokens as the cost function; the assembly is the bet. v2 adds the spec-side and boundary-side precedents that v1's core-first framing left out.

| Project | What we take | What we leave |
| --- | --- | --- |
| Unison | Content-addressed definitions; names as labels; alpha-equivalent hashing; semantic diffs | Its single-language model; here the obligation is hashed canonically and performances structurally |
| QuickCheck, Hypothesis | Property-based testing with generated inputs; shrinking counterexamples | Properties as arbitrary code in a test file; here they are a restricted fragment the mutant library can analyze |
| Mutation testing (PIT, mutmut) | Mutants as a measure of test strength | Mutating the implementation; here mutants are run against the obligation to measure spec strength |
| Dafny, Lean, Verus | The obligation model: spec + implementation + evidence; counterexamples as diagnostics | Requiring humans to write proofs; evidence here is tiered, and proof is a stronger column reached later |
| TLA+, Alloy | Specification as a first-class artifact separate from code | Specs unconnected to implementations; here the checker runs the implementation against the spec |
| OCI / Docker | A boundary the host does not look inside; content-addressed images; registry; scan reports attached to digests | Isolation without verification; Surety is the boundary plus a checker |
| Koka, Austral | Effects and capabilities in the type system | Deferred to the Surety-core target |
| WASM component model | Sandboxed runtime; capability-scoped imports | Adopted as the runner model; a WASM-hosted Python is one candidate for the v0 sandbox |
| [MoonBit](https://dsa.hkust-gz.edu.cn/?p=8170) | Closest ground-up language attempt: flat syntax, manifest typing, machine-readable diagnostics | A general-purpose language humans read; under v2 it could be a target rather than a competitor |
| [SPELL](https://zenodo.org/records/17826541), AILang | Tokenizer-aware canonical form choices | Deferred with the core target |
| [Pel](https://arxiv.org/pdf/2505.13453) | Capability control at the syntax level | Orchestration focus |

Sources: [Pel (arXiv)](https://arxiv.org/pdf/2505.13453) · [SPELL (Zenodo)](https://zenodo.org/records/17826541) · [MoonBit case study (HKUST-GZ)](https://dsa.hkust-gz.edu.cn/?p=8170) · [What Will Be the First Agent-Native Programming Language? (dev.to)](https://dev.to/mattsenter/what-will-be-the-first-agent-native-programming-language-92f)

## Changelog from v1

| Section | v1 | v2 |
| --- | --- | --- |
| BLUF | Build a small language; humans spec, model writes the core form | Build an obligation language, checker, and graph; model writes a target language; core deferred to triggers |
| Architecture | Six layers: surfaces, query engine, graph, core calculus, runtime, checker | Obligation, checker, targets behind a sandbox, graph, kernel. "Core calculus" becomes a target |
| Layers and vocabulary | Performance in the canonical form | Performance in the declared target; canonical form belongs to the core target |
| Evidence kinds | Implicit | New section; per-target profile; the mechanism by which the core is justified |
| Human surfaces | Examples in v0, properties later | Properties, named predicates, named example values in v0; model drafts, human writes examples; audience-projection hypothesis |
| Model surface | Canonical form, constrained decoding, synthetic corpus | Handed a finished obligation and one hop; writes the target; v1's surface deferred to the core target |
| Data flow | Spec → generate → check → evidence → review → merge | Same, plus mutants at the first step and the coverage/finality rule |
| Build | Shell, pure core, kernel; WASM backend | Parser, checker, mutants, Python target, kernel, graph; core as a deferred target with four triggers |
| v0 scope | Data transformations in the core language | Data transformations with a Python target and a boundary contract; underdetermination check |
| Validation | Experiments 1-4; model competence (3) as the gate | Experiments 1, 2, 4; two first-month checks (sandbox, hash churn); underdetermination rate; 3 retired until the core exists |
| Design principles | 9 | 11 (properties never traces; evidence never inherited) |
| Risks | 6 | 12 |

v1 is preserved as a separate document and is not edited further.
