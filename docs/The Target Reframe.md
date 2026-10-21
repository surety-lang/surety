# The Target Reframe

Oct 19, 2026 · @Bennett Dixon

> Live copy (with diagram): https://claude.ai/artifact/PvPuCCDBA66K4VirkRL3x2 . This markdown was exported 2026-10-21; the diagram did not export and is noted inline.

A decision record. Surety is the spec; the model writes the implementation in any target language, Python to start, Surety's own core eventually and only when a named trigger fires.

## Decision

**Surety is the obligation language, the checker, and the graph. It is not, for now, an implementation language.** The human writes obligations in Surety. The model writes performances in a target language: Python first, others later, Surety's own core when and only when one of the triggers below fires. The checker treats every performance as a black box behind a sandbox boundary and verifies it by running it against the obligation.

This reverses the spec's original build order (core language first, trust layer after) and it does so because of what the first day of obligation-writing showed: nothing in the nine obligations mentions an implementation language. They are already target-agnostic. The corpus, the claim shapes, the spellings, and the mutant library are the whole product under this framing, not the first half of it.

The name holds. A surety is the party that stands behind another's performance; it was never the party that performs.

## What a new implementation language buys

The case for building Surety-core first rested on four things a harness over an existing language cannot give. Each is real. None is needed for v0, and each has a trigger that names the moment it becomes needed.

| What the core buys | Why Python cannot | v0 substitute | Trigger |
| --- | --- | --- | --- |
| Provable purity and capability scopes | Static analysis loses to dynamic imports, `getattr`, monkeypatching | Enforced purity: a sandbox with an empty import set, no filesystem, network, or clock, seeded randomness, time and memory caps. Plus a determinism check | Fine-grained scopes inside one component (db-read but not db-write), or purity of a subterm. This is v1 effects |
| Evidence stronger than testing | Totality and SMT discharge need a core calculus | Property testing with recorded run counts | Someone needs "always" rather than "500 inputs, 0 failures" |
| A canonical form that hashes semantically | Python has many spellings for one meaning | Hash the obligation canonically (it is in Surety); hash the performance on a normalized AST with locals erased | Performance identity at graph scale, or hash churn making regenerations unreadable as diffs |
| Tokenizer-dense output | Python is verbose | None needed | Never on its own; output is the smaller half of spend |

No language gives semantic hashing of implementations; program equivalence is undecidable and Unison hashes a canonical AST with names erased, nothing more. The core would reduce dialect variation, not eliminate equivalence classes. The gap is bounded.

The thing v0 does need is a spec language. Python has no checked, analyzable, non-engineer-readable form for a contract. Hypothesis tests are arbitrary code, which is why a mutant library cannot reason about them and why "is this a trace or a property" is undecidable there. The restricted property fragment is what makes mutants enumerable and the spec small enough for a product owner. That is a language requirement, and it is entirely on the spec side.

## What Surety is under this framing

*[diagram, in the live doc: architecture under the reframe · one obligation, one checker, three targets]*

The obligation sits on top and never changes shape when the target does. The checker is target-agnostic because it only ever sees values crossing the sandbox boundary. Targets are plugins beneath it, and each one supplies four things:

1. **A sandbox runner** that executes a performance with nothing available but computation.
2. **A boundary validator** that enforces the obligation's types on the way in and out: strict records with every field present and none extra, `Option` as value-or-`None`, `Result` as a tagged pair.
3. **An AST normalizer** that produces the performance hash: formatting stripped, locals renamed in order of appearance.
4. **An evidence-kind profile** stating how strong each kind of evidence can get on this target (the table two sections down).

The graph holds obligations as nodes and performances as attachments keyed by target. One obligation can carry a Python performance today and a Surety-core performance later, each with its own evidence. Nothing has to migrate; the stronger performance is added beside the weaker one.

## The Docker mapping

The reframe makes the container analogy exact rather than loose.

| Docker | Surety |
| --- | --- |
| OCI image spec, separate from any runtime | The obligation language, separate from any target |
| `FROM python:3.12` | The performance target declared on the obligation |
| The container boundary: the host neither knows nor cares what is inside | The sandbox boundary: the checker sees only values crossing it |
| runc, containerd, and the other runtimes | Target runners: Python sandbox, TypeScript sandbox, Surety-core |
| Image digest, content-addressed | Performance hash, pinned to the obligation's hash |
| Registry | The graph |
| Vulnerability scan attached to the image | Evidence bundle attached to the definition |

Two things the analogy earns. Docker is the existence proof for the trust pillar: the industry already accepted "nobody reads what is inside the container; you trust the boundary, the digest, and the scan report." That is design principle 4 (trust granted by scope, not earned by reading) adopted at scale years ago, for isolation alone. Surety extends a habit people already have. And the content-addressed registry is the part of Docker people forget it got right; it is the same move as the graph, identity by hash and "has this changed" answered by comparing digests.

Where it breaks is the point. Docker isolates and does not verify. A container can be wrong in any way that stays inside its boundary, and the scan says "no known CVEs," not "does what the author meant." Surety is Docker's boundary plus a checker that runs the thing against its contract. Isolation gives "cannot do harm"; the obligation gives "does what was asked"; the evidence says which of those has been established and how strongly. That is the product in one sentence, and a Docker user already half-understands it.

## Evidence kinds by strength

This table becomes a first-class part of the spec. It is the honest account of what "verified" means per target, it is what the projection shows, and it is how the core language gets justified: row by row, when someone needs the stronger column.

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

Every row's left column is honest and visible. An evidence table that reads `purity: enforced, determinism: 500/500` tells a reader exactly what was and was not established, which is better than a language that promises `proven` before it exists.

## What changes in the plan

**Simpler.** Experiment 3 (model competence in a new language) stops being the gate; Python has native speakers. It returns later as "competence in Surety-core" when that target exists. The Rust project shrinks to the obligation parser, the checker, the generators, the mutant library, and a Python sandbox runner: weeks, not a quarter. The first demo becomes the trust layer rather than a parser: nine obligations, the model writes Python for each, the checker runs them, the evidence table fills in, and the mutant that passed the filter obligation before "nothing lost" was added is on screen.

**Unchanged.** The obligation corpus and every finding from the exercise. The design principles, including the two added today. The three hypotheses (audience projections, apprenticeship, hardware parallel). The graph as content-addressed. The name, the orgs, the crate. Surety-core is not abandoned; it is deferred with named triggers, which is a stronger position than "we think a language would be good."

**Reordered.** The spec's Build section currently describes shell, pure core, and kernel in Rust. Under the reframe the pure core and the WASM backend move to a "Surety-core target" section with its triggers attached. The kernel (the small trusted thing that validates evidence) stays, and is more important than before because it now validates evidence from several targets with different strength profiles.

## Risks this reframe introduces

| Risk | Why it matters | What to do |
| --- | --- | --- |
| Sandbox leaks | Enforced purity is only as good as the sandbox, and Python sandboxes have histories. A leak makes the trust pillar false for the v0 target | Pick the runner for isolation first (a WASM-hosted Python, or a locked subprocess with seccomp), test it adversarially in the first month, and treat a confirmed leak as the purity trigger arriving early |
| Hash churn | If AST normalization still produces a new hash on most regenerations, the graph fills with near-duplicates and "what changed" becomes unreadable | Measure the regeneration-to-new-hash rate on the nine obligations; if it is high, that is the canonical-hash trigger, and the measurement is the justification |
| The model drafts both halves | A model-drafted obligation will be shaped by what the model finds easy to satisfy; a plausible drafted spec can lull a reviewer worse than code does | The underdetermination check is the first defense; the second is that the human writes the examples by hand, always, since examples are cheap and are the one layer the model cannot bend toward itself |
| Goodhart on the checker | Every unstated property is a degree of freedom the model will use in whatever way is easiest, and "easiest" can be strange | Make this visible rather than pretend it is gone: the evidence table shows which clauses constrain and which do not, and the mutant report shows the slack |
| The reframe reads as a retreat | "We decided not to build the language" can sound like the thesis failed | It is the opposite: the thesis was never the syntax. Say so in the README, and keep the trigger list public so the core is visibly deferred, not dropped |

## Spec changes required

- [ ] BLUF: rewrite the first line. Surety is an obligation language, checker, and graph; performances are written in a target language, Python first; Surety-core is a deferred target with named triggers
- [ ] Architecture: replace the "core calculus" layer with "performance target," and add the four things a target supplies
- [ ] Layers and vocabulary: the performance is written in the declared target; the canonical form is a property of the Surety-core target, not of the language
- [ ] Model-facing surface: the model writes the target language; constrained decoding and the tokenizer-aware form move under the Surety-core target
- [ ] Evidence kinds: add the table from this document as its own section, and make the projection show it
- [ ] v0 scope: add "performance target: Python" and the boundary contract (strict records, `Option` as value-or-`None`, `Result` as a tagged pair); keep the property fragment, named predicates, named example values, and the underdetermination check as they are
- [ ] Build: shell, checker, mutant library, generators, Python sandbox runner, kernel. Move the pure core and WASM backend to a "Surety-core target" section with triggers
- [ ] Validation plan: retire experiment 3 as a gate; add sandbox-leak and hash-churn measurements as first-month checks; keep experiments 1, 2, 4 and the underdetermination rate
- [ ] Risks: add the five rows above
- [ ] Open questions: add "which Python sandbox" and "what the boundary does with values the obligation's types cannot express"
