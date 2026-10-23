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

**The ceiling (2026-10-22).** Generation capacity is on its own curve; what is not is how fast a person can say what they meant and confirm they got it. The shape is Amdahl's: verification is the serial fraction of AI-built software, and a thousand agents hit the same ceiling as one. A hundred agents can produce something the shape of a large product, and nobody can say which parts are right, so it is slop however much of it there is. The ceiling moves only if the form of saying and confirming changes. The bet is that an obligation plus a checker is that form: intent stated once in a checkable shape, confirmed by running where the evidence is strong, and read where the report says it is thin, so a reviewer's hour goes to the pieces that need it instead of to the whole file. The measure is verified intent per human-hour, with the reading share beside it; both are in the validation plan.

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

&#91;embedded content: architecture · obligation, checker, three targets, one graph\]

**Interfaces between layers**

- **Surfaces → graph:** a query API, never file reads. Requests name a node; responses are the obligation, its evidence status, and one hop of neighbors.
- **Obligation → graph:** hash-addressed writes. Normalization contract: the hash is computed over a canonical form in which record fields, properties, and examples are sets and bound names are erased, so two obligations that differ only in clause order or local names hash identically. Formatting, comments, and the prose note are pinned to the hash, not part of it. Example order is kept for diagnostics and ignored for identity.
- **Checker → target:** a performance is invoked with a value of the obligation's input type and returns a value of its output type, or fails. The checker never inspects the performance's source. The target's boundary validator enforces the types on both sides.
- **Target → graph:** a performance is attached to its obligation keyed by target, with a structural hash from the target's normalizer. One obligation may hold several performances.
- **Checker → kernel → graph:** the checker produces an evidence bundle; the kernel (small, hand-written, human-audited) validates that each claim in the bundle is backed by a recorded run before it is pinned.

**What a target supplies.** Four things, and nothing else: a sandbox runner, a boundary validator, an AST normalizer for the performance hash, and an evidence-kind profile (the table in the Evidence kinds section).

**Checker queries, v0.** Type and totality checks apply to the obligation. Against a performance: examples run, properties are tested with generated inputs, determinism is tested by running twice. Against the obligation alone, with no performance present: the **underdetermination check**. A library of mutant performances (return empty; return the input unchanged; drop one element; duplicate one; append a fabricated one; swap two adjacent elements; satisfy each clause independently) is run against the obligation. Any mutant that passes is reported with the clauses it slipped through. This is adversarial review pointed at the spec, and it is what makes the spec layer workable for an author who is not a senior engineer.

**What the checker is (2026-10-22).** A deterministic program, not a model, and no model is in the loop of any verdict: a parser and normalizer (the canonical form and its hash); a boundary validator; an evaluator that runs the examples, tests every property by brute force over inputs from a seeded generator, runs twice for determinism and counts totality; the mutant library; a kernel that refuses any record whose counts are not backed by its recorded runs; and `verify`, which re-runs all of it and requires the evidence byte for byte. Models appear only outside it, writing performances and drafting obligations, and it treats both as untrusted input. It never reads source, never interprets prose (a pinned sentence is carried, not understood) and never judges meaning: the examples and properties are the meaning, and it runs them. Even which party a finding belongs to is a search for a passing output, not an opinion.

**Where the mutants come from.** Not from a model: a mutant has to be enumerable, deterministic and reproducible, or "none slipped" means nothing and `verify` cannot re-run it. Three mechanical sources. Generic strategies over the types: from the signature and the run's inputs alone the library builds the empty value, the input unchanged, one element dropped, duplicated or fabricated from empty fields, two adjacent swapped, always `Err`, stop at the first; the strategies a lazy implementation might use, and the question is never whether one is realistic but whether the obligation notices. Skeletons from the obligation's own shape: the frame rule pairs output records with input records, and from the pairs the library builds a performance that copies the preserved and matched fields and leaves every computed field empty, the bookkeeping with none of the work, then applies the tweaks. The search for two passing outputs: for each generated input, candidate outputs are built from the input's own elements and checked for two different ones that satisfy every clause, so no wrong implementation is needed at all. The name is mutation testing's, inverted: that mutates source and asks whether the tests notice; Surety never reads source, so it mutates behaviour at the boundary and asks whether the obligation notices. The limit is the reason the families grow by hand: the library finds generic underdetermination, not domain-specific wrongness, and a new named strategy is added when a shape of obligation shows the old ones blind, which the report's hollows make visible.

**The scalar family (2026-10-23).** Added to the generic strategies over the types, for the reason the last sentence gives: on an `Int` output two of the structural thirteen applied, so on the invoice total the underdetermination check was nearly silent and a clause about a total could only land in the look-here section. For each `Int` or `Bool` of the output outside a list (at the top level, or in a record it holds through `Ok`): the constants 1 and -1 and, for each `Int` the inputs afford, that value, one more, one less, negated and times 100 (the unit swap, cents for dollars); a `Bool` flipped. Each is named by its formula in the obligation's spelling, `out = sum(items.amount_cents) + 1`, `has_next = true`, so a mutant is a name plus parameters, stored as data in the record's `caught_by` and `slipped`, and the sensitivity knob the Float slice will want (scale by 1 + ε and report the smallest ε caught) is a parameter of a mutant that exists, not a new library. Round half-up is not type-derivable and stays with the examples and the metamorphic line.

**Examples run against the lines (2026-10-23).** An `e:` block is a run the author wrote: an input and the output he decided. On every check, before any performance is looked for, the checker evaluates every clause over each example's own pair, under `check` and under `mutants` alike, since a save with no performance runs the second. A clause false on an example is the obligation's finding, named with the example and the clause, because both are the author's and one of them is wrong; a performance that reproduces the example and fails the clause on it is never blamed for that. The search for two passing outputs never counts an example output a clause rejects. An open pin, an obligation entering the graph before any performance, runs this evaluation and refuses on a contradiction; nothing in the graph moves. These evaluations are not runs of a performance and do not enter the evidence record's counts. The verdict line keeps four counts apart (examples evaluated, performance runs, mutant evaluations, inputs searched), leads with what is missing, never reads as a pass with no example, and says `nothing ran` only when all four are zero. A catch on an input where the search found no passing output is a catch, marked unconfirmed; the clause keeps its credit, since a bounded search is not a proof, and the report says `no passing output among N candidates`, the bound in the words. Only the proved case withdraws credit: a clause about the inputs alone false on an input, where "unsatisfiable" keeps its word. The search is seeded with every output already in hand on the input, the example's, the performance's, the mutants' own and the skeletons, before it builds a candidate; measured on the eleven with no performance and an unseeded pool, seven obligations had inputs the pool could not satisfy, and with a performance none. A mutant whose every catch is unconfirmed is not judged, a third state beside caught and slipped, listed by name like them; the record carries the search's outcome per input, which is a shape change and so the next format move. An example reaches a line when, evaluated on the example's own input and output, flipping some of the line's output-reading parts could make it false; a part never evaluated on that pair cannot be flipped, which is what vacuous means, and the editor names a line no example reaches. Decided after the author's problem 13 of the same day: four lines false on every error input, no example in the file, a save that read as a pass.

**Underdetermination, defined (2026-10-21).** An input is underdetermined when two outputs that differ under the obligation's order claims both satisfy every clause. The search draws candidate outputs from the inputs first (every sub-multiset of a matching input list), then from the output type, because a filter's or a selection's answers come from its input and sampling never lands on them. A mutant has slipped when no clause is false on any input it applies to and every output it produced passed the boundary; the type can catch a mutant too. A slipping mutant or an underdetermined input is a finding. On generated inputs, a false claim, an output that fails the boundary, a raise, a timeout, and a second run that differs are all findings rather than failures, because the obligation itself can force a boundary failure (a diff whose lists repeat a key); examples and the claims over them stay binding, and `--strict` makes every finding binding. The report says which party a finding belongs to: a performance that is wrong, or an obligation that no output could satisfy on that input, with the refinement that closes it when one exists.

**Generated inputs (2026-10-21).** The domain carries every integer literal in the obligation and its two neighbours, so a threshold is crossed from both sides whether or not an example crosses it. An example seeds the domain, never the run: `check` and `mutants` run on an obligation with no examples, and a claim no example reached is evidenced by the generated tier rather than skipped.

## Layers and vocabulary

The terms come from how a surety bond works: a principal owes an obligation, and a surety stands behind the performance.

| Layer | Artifact | Author | Contents |
| --- | --- | --- | --- |
| Spec | **Obligation** | Human (a model may draft; a human approves) | Type declarations, named values, signatures, examples, properties, named predicates, prose notes. No bodies. Written in Surety. A model may draft the types, the named values, the signature and the properties; the examples and `accept` lines are the human's, always |
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

**The property fragment as of 2026-10-21.** Consolidated from eleven obligations and fourteen claim shapes; the full rule list is in the catch-up note and the decision notes, and this is the shape of it. A quantifier head has five slots, each at most once: `all|any|none|exactly_one <binder> [leftover] in <list> [where <cond>]: <body> [else <body>]`; a body runs to the end of the line or is the indented block under it, so a quantifier is always the last operand on a line; binders never leave their quantifier; a pattern in the binder slot (`Ok(p)`, `Err(errs)`, `_`) selects a side of a sum type. Binders range over *occurrences*: `==`, `in`, and the list relations compare values, `before` compares positions, and `leftover` assigns each output element the earliest matching occurrence, so a dedupe obligation is true on duplicates, and `out drawn_from orders` together with `all o leftover in orders: <o was excluded>` pins multiplicity exactly, with no count-where form needed. One list relation underneath the family: `drawn_from` (sub-multiset); `same_order_as` is it plus order (the subsequence relation), `same_elements_as` is it both ways, `unordered` is the explicit no-claim, `sorted_by` takes a lexicographic key, `unique` is whole-list. Connectives `and or not -> <->` with conventional precedence; built-ins `sum len min max count flatten`, with `len` and `in` applying to `Text` as well as to lists; broadcast (`out.id` is a list of ids); a named predicate with a body is an abbreviation inlined before hashing, and one with no body (`well_formed_email`) is a graph reference, so a `Text`'s shape is a sibling obligation's claim and never the fragment's. A precondition that fits on one list or field is a refinement in the brackets (`Item[unique sku]`, `Score[0..3]`), checked at the boundary; one that relates two things is an error case in a `Result`. The frame rule: when a property pairs an output record with an input record, same-named fields are equal unless stated. `accept <boolean over inputs>` marks a deliberate non-decision, in the hash, reported ACCEPTED, a finding if it never fires. Prose above a property is optional and rendered; `#` is a sentence a human chose to pin, `//` is a comment. One sentence per property (2026-10-22): consecutive `#` lines above a property or an accept line are that one sentence, joined by spaces, so a sentence may be wrapped without touching the hash; any further prose is a `//` comment. No line limit in the grammar; the renderer wraps at a column, and everything that writes a file goes through it. A property may call its own obligation (2026-10-22): in a check the call runs the performance under test, so the clause relates two of its outputs, the metamorphic form (the total of the items is the total of one item carrying their sum); under the underdetermination check the call means the mutant, so the clause never catches a mutant on its own and always needs companions that pin values, and the report says so. In the search for a passing output or a second one the call is set aside whether or not a performance is present (2026-10-23): a performance judging candidates for its own obligation would be the performance as referee, and on a one-item input problem 11's metamorphic line would pin every tie to whatever the performance answered.

**The fragment after slice 5 (2026-10-23).** Integer arithmetic: `+`, `-` and `*` on `Int`, left-associative, `*` above `+` and `-`, all above every comparison and relation, parentheses as before; `abs(x)` as a built-in; `len`, `sum`, `count`, `abs` and `Int` fields compose, so `page * size > len(items)` and `abs(out * 10000 - sum(items.amount_cents) * (10000 + tax_rate_bp)) <= 5000` are lines. The obligation's arithmetic is exact: the checker never rounds an `Int` and never wraps one, and an overflow of the obligation's own line is the obligation's error, named under `obligation:` with the clause and the input, never the performance's. `/` is refused on `Int`, since every division carries a rounding decision that is a sibling obligation's claim; `%` on `Int` is allowed (decided 2026-10-23 evening): it is exact, the remainder with the sign of the divisor as Python has it, so for a positive divisor it lies in 0 to b − 1 on negatives too, at `*`'s precedence, with `% 0` the obligation's error like an overflow. It exists to name a tie: problem 11's half-even rounding is then two lines, the total within half a cent of the exact value, and when the exact value's remainder on 10000 is 5000 the total is even, after which the ties are no longer gaps; a `Float`, `Time` or `Text` operand is refused with a message naming what it waits on. The canonical form prints the operation as written, `(+ a b)`, `(abs (- a b))`, with no reordering, so `a + b` and `b + a` are two obligations; binders are what the canonical erases, not algebra. A tolerance is a line, `abs(a - b) <= k`, signed where the properties are; `==` is an equivalence and is never made approximate, on `Float` too, because `unique`, `drawn_from`, `same_elements_as`, `same_order_as` and `leftover` are built on it. The positional slice `xs[a .. b]` is an expression: the elements of `xs` from position `a` up to but not including `b`, positions from 0, clamped to the list so a negative `a` starts at 0 and never counts from the end, the refinements dropped; it never raises, so `items[page * size .. (page + 1) * size]` is the partial last page and the empty page past the end with no sentence about either, and `xs[i]` is refused with the one-element slice named. In a type position the same brackets are the length bound, `Item[0..3]`, inclusive: a slice has an expression before the bracket, a refinement a type name. `true` and `false` are values, so an example can state a `Bool` output; `accept true` stays refused. Unary minus is part of a literal and nothing else; `0 - x` is the spelling until a draft reaches for more. When `Float` arithmetic lands it is evaluated exactly, in rationals, with `/` on `Float` and never on `Int`, and a division by zero in the obligation's line is the obligation's error like an overflow; that slice opens on problem 10 plus three float problems from ordinary software.

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

The exercise that produced the first nine obligations found that examples alone make the spec layer unworkable past trivial functions, that properties cost more to compose than code and less to review, and that the hardest skill (imagining the wrong implementation) is exactly what the underdetermination check automates. So the workflow is: the model drafts the obligation, the human writes the examples by hand and reviews the rest, the checker reports what the obligation fails to constrain. A named value (the literal an error or a reused record is) is a declaration, drafted and marked like a type, never an example: the performance must return it byte for byte, so it bends toward the model no more than a type does, and the author's examples cite it, so a drafted value no example reaches is visible as unused (2026-10-21). The model may name the examples it wants (`// example wanted: <the case, and the method if it matters>`) and never their expected output: the author converts or deletes each, and a number in that line was copied once as an expected value when the performance was the one that was right (2026-10-22).

**The draft flow (2026-10-23).** The draft runs sentences, then the author's examples, then lines, in that order and never with the examples first: the sentences are what produce the boundary cases the examples pin. An example-wanted line carries the case, what decides it, and the input as a literal in the signature's types on its own line beneath; never the output. The editor turns one into an `e:` block with the input filled and the output left to the author, who types it or deletes the example; no model writes an output into an example. The lines stage refuses a file with no example, as it refuses one that does not parse: a line is asked for only against an example that can run it. After the lines, the editor names any clause no example reaches. Whether the draft may propose an output stays open.

**The projection.** What a reviewer sees instead of source, in this order:

1. **Evidence**, by kind, from the table above: what was established and how strongly.
2. **Residuals.** Claims no checker could settle and no cheap model could judge. This is what the human reads.
3. **Underdetermination.** Mutants that passed, with the clauses they slipped through.
4. **Behavioral diff.** For a change: the inputs on which output differs, with three concrete examples. Never a text diff.
5. **Source, on demand.** The performance in its target language. Always one query away, never the default.

**Review by difference (2026-10-22).** The review page is this projection applied to the three deltas of a change: the obligation delta to sign, the evidence delta as the verdict line the record already prints, the behaviour delta as witness inputs with old and new outputs. Above the rows, coverage: every function in the area with an empty slot for each that has no obligation, so the limit of the promise is the first thing a reviewer sees. Each delta implies its next action: a difference the obligation does not decide offers tightening the obligation on that claim; a slipped mutant offers its description; a claim no mutant tests says so. A picture may sit beside the words for noticing; the sentence is for knowing, and the picture is regenerated from the record, never read from pixels. Built on the store, the prose projection and the behavioural diff, in that order, and measured before it is hosted: reviewers given seeded changes, minutes to a decision and the catch rate, the page against a code diff plus the report.

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

**A change is three deltas (2026-10-22).** In the graph a change is three derived differences, each with a hash at both ends, so each is reproducible: the obligation delta (claims, types, named values, examples added, removed or edited), which a person authored and a person signs; the evidence delta (same obligation, new performance: the verdict, which claims a mutant exercised, which output shapes an example reaches), which the checker judges and a person reads; and the behaviour delta (the inputs on which old and new performances disagree, classed by the claims each output fails, from `surety diff`), which the checker classes and a person decides where the obligation is silent. A pull request is a set of these. Approval is signing the obligation delta and nothing else; the implementation's text is attached, never central.

&#91;embedded content: lifecycle of a change · 7 steps, 2 decisions, 2 loops\]

The inner loop (regenerate on a counterexample) is cheap and runs without a human; the outer loop (amend the obligation) is where human tokens go. The mutant library runs at the first step, before any performance exists, so a weak obligation is caught before the model spends a token on it.

**Acceptance gate (2026-10-21).** An obligation enters the graph only after `check`'s generated tier and `mutants` have run on it against at least one passing performance and the author has answered every finding: a fix, a refinement, or an `accept` line. The mutant library is the first step of a change in the flow above; this makes it a gate on acceptance rather than a step. Reading is not a check; running is.

**Coverage and finality are separate.** An obligation may be partial in what it covers; it is never partial in what it states. What the obligation says is fixed before generation starts. The model's job of surfacing gaps ("you did not say what happens on an empty list") happens before the first stroke, never during; amendments come back as a new spec delta.

**The checked boundary moves one definition at a time (2026-10-22).** A property may call a name no obligation in the directory declares. The call fixes the shape; the performance supplies the definition, unchecked, until a sibling obligation declares it, and the editor and the record say so. So a program can start with the obligations that matter and let the rest be plain performance code that the graph names as unchecked, and an existing codebase can be entered the other way round, its functions called as unchecked references and given obligations one at a time; both are one mechanism. The hole is bounded and named: it is exactly the size of the reference, and until the sibling exists the author's examples are what pin its meaning, since a performance that supplies `well_formed_email` also supplies what "well formed" means. Acceptance states what it leans on: the verdict line carries each unchecked reference and how many examples exercise it, the way it carries `purity`.

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

**Boundary contract, amended 2026-10-21.** `Time` crosses the Python boundary as a comparable value (a UTC `datetime`), not as RFC 3339 text; the closed-loop run showed a performance writing its own 45-line parser when it was text. `Id` is an opaque identity, `str` in Python, equality only; `UUID` stays a format refinement for when the format matters. Inputs are validated at the boundary too, refinements included, so a generated input never fails it; an output that fails it on a generated input is a finding, not a failure. Accepted non-decisions are `accept` lines in the obligation; there is no separate list to maintain.

**The space, stated (2026-10-23).** v0 is pure transformations over business records: lists of declared records or primitives in; a record, a list, or a `Result` out; every element type declared, as a record or a primitive. No function-typed inputs, no infinite outputs, no element-agnostic functions. The sentence exists so that a reach-for from outside the space is logged as outside and not queued: the reach-for census of the same day ran the draft over 107 functions of more-itertools and found 45 list parameters forced to `Int` and five workarounds for function arguments, which read as a roadmap until the space was said. Read against it, the census's in-space findings are the `Some(n)` binder over an `Option` and the "exactly when" error form; type parameters are out of v0 by definition. An obligation inside the space that never reads its element declares a record it does not read, as problem 13 does with `Item`; one opaque element type with equality only, no parameters and no instantiation at a call, is the direction if a second such problem appears. The library behind the standing claim (a library's API as obligations, reproduced on a stranger's machine) is chosen from inside the space, not sampled by size; the census's second half is the measurement of which libraries are inside.

**Deliberately out of v0**

- Effects and capabilities beyond the all-or-nothing boundary. The evidence-kind slot exists; nothing populates it until v1.
- Pre/post contracts and SMT discharge.
- A second target. TypeScript is the obvious one and it waits until the Python target is measured.
- The Surety-core target, per its triggers.
- A standard library of any kind. Performances use the target's own.
- Importing foreign components. v0 performances are called; they do not call.
- Function-typed inputs, infinite outputs, and element-agnostic functions with type parameters (added 2026-10-23, with the space stated above). A function argument is modelled as a sibling obligation called by name, or not at all.

**v1 candidates, in order:** read-only capabilities (data sources) as the first effect class, which is also the first trigger for the core target; then write capabilities; then service handlers composed from v0 transformations. Each step adds one effect class and tests whether the boundary model holds under it.

## Validation plan

Three experiments gate the project, two first-month checks gate the Python target, and one metric measures the spec layer directly. v1's experiment 3 (model competence in a new language) is retired as a gate and returns when the Surety-core target is built.

| # | Experiment | Measures | Kill if |
| --- | --- | --- | --- |
| 1 | **Context audit.** Instrument real agent sessions on an existing codebase. Break context down: first reads, re-reads, tool output, history, docs | Tokens per accepted change; re-read ratio; share of context that is verifiable by tooling | Re-reads and tool output are not the dominant cost |
| 2 | **Obligation substitution.** For a subset of functions, write obligations and serve those to callers instead of source. Same tasks, same model | Delta in tokens per accepted change; delta in defect rate | No token reduction, or defects rise |
| 4 | **Residual review.** Give reviewers only the projection (evidence, residuals, mutant report, behavioral diff) for v0 changes. Compare to source review | Defects caught; time per review; how often reviewers open source anyway | Reviewers open source on most changes, or miss defects source review catches |

**Status, 2026-10-21.** The first gate under v2 is the closed-loop run, which replaced v1's experiment 3: a model reads the obligations and writes Python; `check` runs it. Its mechanical half ran on 21 October: claude-opus-5-5 wrote a passing performance for all eleven obligations zero-shot, one round, 71 seconds, about 18k tokens. Its reviewer half, whether the evidence table replaces reading for a person, was first judged "not yet, by design" on reports that carried only example results, and 1 of 3 on a second read of reports carrying run counts, slack, imports, determinism and shape, with both no's correctly attributable to obligation holes rather than the performance. Slice 2's own kill criterion (stop extending the detector if it finds nothing beyond the predicted case; rewrite the corpus if most obligations report many kangaroos) landed on the first side: five obligations had holes, all closed by one rule each, and the corpus now reports zero findings and passes strict, with the caveat that the library is blind on transform-shaped obligations until the skeleton family and its successors exist. Experiment 1 ran on 21 October over 230 sessions and 67.7k turns: not killed. Reading is about 29% of an agent's context and the largest category, most of it through the shell; re-reads are 80% of all file reads and between 10% and 23% of context; test and build output is 1.4%; cache reads are 40 times new input. The token thesis stands with its mechanism corrected: re-reading happens through `cat` and `grep`, not a read tool. Experiment 2 stays as written, aimed at the reading share, one project for one week. Experiment 4 waits on the projection.

**Status, 2026-10-21, evening.** Slice 4, author tooling (`surety lsp`, the VS Code extension with check on save, the two-stage draft), landed the same day; its mechanical acceptance tests pass in CI. The draft's first two cold runs (problems 2 and 8, the author's obligations withheld) passed the tool's own test and found two gaps in this document rather than in the notation: the fragment summary did not say that `len` and `in` apply to `Text`, nor that a bodiless predicate is a graph reference, so the model reached for text operations it already had; and it read `leftover` under value semantics and reached for a count-where form. Both are fixed in Layers above. The optical recall cache (obligations rendered as images for an agent to carry) was measured and killed as a token saver; the text recall projection it implied, index line, signature plus sentences, full text, keyed by hash, is what experiment 2's harness serves.

**Status, 2026-10-23, afternoon.** Slice 5, integer arithmetic and the positional slice (commits 6cc6aeb to c31b7ce on main, CI green on the last), landed the engineer's half: `+ - *` and `abs` on `Int`, the clamped slice, the generator's base 0 to 4, the scalar mutant family, and the eleven re-pinned once with nine records changed and `verify --rerun` reproducing 11 of 11. Acceptance tests 1 to 4 and 9 to 11 pass in the suite; 5 to 8 are the author's and the closed loop's and are the gate: problem 13 written in the editor with the draft tool and pinned against a closed-loop performance, 11 and 20 through the closed loop, the draft re-run on all three. Two findings from the build. The metamorphic hole reading 106 predicted is real: on the invoice fixture the mutant that forgets the tax passes the self-call and both bounds, and only the examples catch it; with the `abs` tolerance line beside the metamorphic line every scalar mutant is caught, which is the measurement behind "always needs companions that pin values". And on a pagination scratch the search found an `Err` with an empty code passing an obligation that said "an error exactly when out of range": an error of another kind satisfies neither side of an iff over one error kind, so a `Result` output needs its `Ok` side stated, and the search named the missing line with no example written. The second is the underdetermination check doing the job it was built for. Slice 6, the draft flow, was briefed the same night from both findings; its first piece, the examples run against the lines on every check, is decided on 2026-10-23 and built in slice 6, so the Architecture paragraph above is ahead of the binary until it lands, and today's `check` does not do it.

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
- Verified intent per human-hour (added 2026-10-22): behaviours, properties and examples, in accepted obligations, where accepted means pinned and reproduced by `verify`, divided by the human minutes spent authoring and reviewing them. The baseline is owned: the first eleven took two days by hand. Measured on problems 9 to 22 as they are written, the draft's run records carrying the model's side and the author noting start and stop for the human side
- Reading share (added 2026-10-22; the source-open rate made specific): how much of a performance a reviewer read to accept it, and whether, when they read, the report pointed them to the right place. Should fall as the evidence grows. The slice 1 read is the starting point, nearly all of it

**Gate to the core target:** one of the four triggers in the Build section, with the measurement that fired it. **Gate to continuing at all:** experiments 1 and 2 show the token thesis holds, and experiment 4 shows the projection replaces reading for a reviewer where the evidence is strong and directs it where the evidence is thin. If 4 fails, the spec layer has failed and no language fixes it.

## Risks and open questions

The spec layer is the risk that kills the project; the rest delay it.

| Risk | Why it matters | Mitigation |
| --- | --- | --- |
| Specs are unsafe without the check (renamed 2026-10-21 from "humans won't write specs") | 4GLs, model-driven development, and Dafny stalled on humans writing specs. The first corpus showed something narrower: humans can write them (eleven obligations in two days), but a correct sentence sat above an incorrect property for a day and two readers missed it. The detector found it on its first run. 1 of 11 obligations by the language's author carried a silent hole. The risk is workflow, not skill, and it is cheaper. | An obligation is accepted only after check's generated tier and mutants have run on it and every finding is answered: a fix, a refinement on a type, or an accept line. Reading is not a check; running is. The model may draft the obligation and the human reviews; examples stay hand-written and cheap; v0 keeps a domain where properties are natural to state. Throughput rises only as reading becomes directed rather than abolished (2026-10-22): the reviewer-half test grades the report on whether, when the reviewer did read, it pointed them to the right place, and the checker, which never reads a performance, points by input, at the clauses no mutant can violate, the accepts that fired, the boundary cases. |
| The mutant library is blind on transforms (added 2026-10-21) | The slice 2 library builds wrong performances from the input's shape, so on an obligation whose output has a shape no input has (group, flatten, join, diff) only the empty mutant applies and a clean result is blindness, not cleanliness. Problems 9 to 22 are mostly that shape. | Report an applicability line beside every rate. The skeleton family built from the frame pairs is the first operator for transforms; the next mutates a computed field plausibly. Target on problems 9 to 22: at most 3 of 13 with a slipping mutant or a missing refinement on first run, 0 after one fix pass; 8 or more of 13 reporting only empty applicable means the search grows before the graph. |
| Part of the context cost is harness waste (added 2026-10-21) | Experiment 1 found that most re-reading goes through the shell. A context-aware harness that returned "unchanged since turn 12" instead of the file would shrink that bucket without any language, so the economics argument has a competing explanation. | Say so. The obligation wins only on reading-to-understand, not reading-to-re-find, and the audit cannot separate the two. Experiment 2 is designed to: obligations served instead of source on one project for a week, measured per change cycle. |
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
| Scope creep toward a code-context product | A spec language and checker over existing code is nearer "tooling over existing languages" than v1 was | Reread the prior engagement's clause with this framing before any code; the category (nearer Hypothesis or Dafny than documentation) is defensible but must be checked |
| Reading code stays good enough | The stable end state may be humans who read code fluently without writing it, because code is the best bridge between intent and deterministic behavior we have. If that holds, the projection never earns their trust and the evidence table is a curiosity beside the source view | This is what experiment 4 tests, and it is a different failure from "humans won't write specs." Keep source one query away so the design does not depend on winning the argument; measure source-open rate from the first session; and make the projection show what reading cannot (the mutant report, the behavioral diff), so it is additive to reading rather than a replacement someone has to accept |

**Open questions**

- [ ] Which Python sandbox: a WASM-hosted Python, or a locked subprocess with seccomp? Isolation first, then speed.
- [ ] What does the boundary do with values the obligation's types cannot express (NaN, oversized ints, non-UTF-8 text)? Reject at the boundary, or widen the type language?
- [ ] Does the spec language need its own surface syntax beyond the `.sur` grammar, or is one grammar enough for human and model readers?
- [ ] Who merges: can the checker auto-merge when the residual set and the mutant report are both empty, or does a human always approve?
- [ ] Objectives. Every obligation so far is a constraint. Some real specifications are objectives (the valid output that minimizes X). An `optimize` clause is outside v0, and property testing cannot confirm optimality, so admitting it later means a new evidence kind or an honest "not checkable" status.
- [x] Contract scope for the author: how far does a prior engagement restrict building this? Read the clause before any code is written. Read on 2026-10-21 against the v2 shape (graph store with `show`, obligations served instead of source, projections, model-drafted obligations): no conflict.
- [x] The evidence part of a record must be a function of the obligation hash, the performance hash and the budget, and of nothing else. Done 2026-10-22 (commit 2ff1bf3, record format 2): clauses and examples by canonical ordinal with the print as label, the generated domain sorted, the format named in provenance, the invariant in the kernel's doc and a two-presentation test. As written then: As of 2026-10-21 it carries presentation line numbers, so two presentations of one node would pin different records for one performance; the fix (clauses identified by ordinal in canonical order, a record format version in provenance, a two-presentation test) is queued before the first drafted obligation is pinned.
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

**Changelog within v2**

| Date | Change |
| --- | --- |
| 2026-10-21 | Validation plan: status paragraph with the closed-loop run (11 of 11 zero-shot), both reviewer reads, slice 2's kill-criterion outcome, and experiment 1's result; the token thesis restated to match the mechanism (re-reading through the shell). |
| 2026-10-21 | Architecture, checker queries: underdetermination defined; boundary failures on generated inputs are findings; `--strict`; the report names which party a finding belongs to. |
| 2026-10-21 | Data flow: the acceptance gate (check and mutants before an obligation enters the graph). |
| 2026-10-21 | Layers: the property fragment as consolidated from the corpus; occurrence semantics for binders; the list-relation family; refinements in brackets; preconditions relating two things are error cases; `accept`. |
| 2026-10-21 | v0 scope: boundary contract amended (`Time` as a comparable value, `Id`, input validation, `accept` as the accepted list). |
| 2026-10-21 | Risks: "humans won't write specs" renamed "specs are unsafe without the check" with the 1-of-11 number; two rows added, the library's blindness on transforms and harness waste as a competing explanation for the context cost. |
| 2026-10-21 | Layers: `len` and `in` on `Text`; bodiless predicates as graph references; `leftover` with `drawn_from` pins multiplicity. Found by the draft tool's first cold runs, which read this document and reached for what it did not say. |
| 2026-10-21 | Architecture: the generated domain carries each integer literal's neighbours; examples seed the domain, not the run. |
| 2026-10-21 | Validation plan: slice 4 status; the optical recall cache killed as a token saver and the text recall projection kept for experiment 2. Risks: the prior-engagement item closed; the record's presentation-line gap opened with its fix queued. |
| 2026-10-21 | Layers: named values are declarations the model may draft, marked like types; examples and `accept` lines stay the human's. Decided when the draft of problem 8 reached for its error values six times. |
| 2026-10-22 | Why now: the ceiling paragraph (generation scales, verification throughput does not, unless the form of saying and confirming changes; reading directed rather than abolished). Validation plan: verified intent per human-hour and the reading share as metrics; the gate to continuing reworded from "replaces reading" to "replaces where strong, directs where thin". Risks: the throughput line on the specs-unsafe row. |
| 2026-10-22 | Data flow: a change is three derived deltas (obligation, evidence, behaviour); approval signs the obligation delta. Human-facing surfaces: review by difference as the projection of those deltas, coverage first, next actions from each delta, the words ruling any picture, measured before hosted. From the internal note of the same name. |
| 2026-10-22 | Data flow: the checked boundary moves one definition at a time; unchecked graph references are bounded, named, pinned by examples until a sibling obligation exists, and stated on the verdict line. From the author's reading of the editor's hover on `well_formed_email`. |
| 2026-10-22 | Layers: a property may call its own obligation (metamorphic form), with its limit under mutants; example-wanted lines carry the case and the method, never an expected output. |
| 2026-10-22 | Architecture: what the checker is (a program, not a model; its six pieces; what it never does) and where the mutants come from (three mechanical sources; mutation testing inverted; the limit). Prompted by the author: readers will assume an AI does the checking. |
| 2026-10-22 | Layers: one sentence per property; consecutive `#` lines are its continuation; wrapping is the renderer's, not the grammar's. |
| 2026-10-23 | Layers: the fragment after slice 5 (integer `+ - *` and `abs`, exact, with overflow the obligation's own; `/` and `%` refused on `Int`; a tolerance is a line and `==` never approximate; the clamped positional slice; `true` and `false` as values; the Float slice's opening). Architecture: the scalar mutant family, and mutant identity as a name plus parameters. From the slice 5 brief and its amendment. |
| 2026-10-23 | Validation plan: slice 5 status, the gate on problem 13, and the two findings (the metamorphic hole measured; an iff over one error kind leaves a `Result` open). Risks: the record's presentation-line item closed. |
| 2026-10-23 | Architecture: examples run against the lines on every check, with no performance, as the obligation's finding when they disagree; an open pin refuses on one; the verdict's four counts and its no-pass-without-an-example rule; no credit for a catch on an input with no passing output, and the bounded wording. From the slice 6 brief and the engineer's read of it. |
| 2026-10-23 | Human-facing surfaces: the draft flow (sentences, the author's examples, lines; the input literal on an example-wanted line; the lines stage's refusal without an example; the unreached-clause hint). |
| 2026-10-23 | Architecture: the not-judged state for a mutant, the search outcome per input in the record (the next format move), and which finding keeps the word "unsatisfiable". From the engineer's second read of the slice 6 brief. |
| 2026-10-23 | v0 scope: the space stated in one sentence (pure transformations over business records, every element type declared); function-typed inputs, infinite outputs and type parameters listed as deliberately out; the opaque element type as a direction; the claim's library chosen from inside the space. After the reach-for census's first spot-check. |
| 2026-10-23 | Validation plan: piece 1 of slice 6 decided and not yet built, so the Architecture paragraph is read as ahead of the binary. Architecture: the definition of an example reaching a line. From the engineers' reads of the slice 6 brief. |
| 2026-10-23 | Architecture: a bounded search never withdraws a clause's credit; a catch on an input with no passing output found is unconfirmed, the search is seeded with the outputs in hand, and only the proved input-only case withdraws credit. Reading 129, after piece 1 of slice 6 landed. |
| 2026-10-23 | Layers: a self-call is set aside in both searches, with or without a performance, by the referee rule; found on problem 11's ties after piece 5 of slice 6 (reading 140). |
| 2026-10-23 | Layers: `%` on `Int`, exact, floored like Python's, `% 0` the obligation's error; decided on problem 11's ties, 35 of 508 inputs the tolerance line could not settle. `/` stays refused. |
