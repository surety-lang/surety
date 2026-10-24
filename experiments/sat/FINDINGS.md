# The SAT loop experiment: findings

The first SAT experiment on Surety, run 2026-10-24 against the checker at the commit the run
directory records (`runs/<stamp>/records/checker.json`). The obligation is `sat.sur`, written
in the language the corpus parses today; the performances are `sat_plain.py` (brute force,
the oracle), `sat_dpll.py` (DPLL with unit propagation and pure literals) and `sat_cdcl.py`
(a small CDCL); `harness.py` supplies the hard instance families and the cost measurement the
checker does not have. Nothing under `crates/`, `graph/` or the corpus was touched; the
parser, the checker and the generator are as they were.

The goal was never a fast solver. It was to record exactly what the language could and
could not express about SAT, with the tool's own output as the evidence, and to measure the
performances on instance families where every resolution-based solver is known to be
exponential. Both canaries say exponential, which is the expected outcome.

Run directory for every number below: `runs/<<STAMP>>/`. The checker's text and JSON reports
are under `records/`, the instances under `instances/<family>/`, the per-instance results in
`results.jsonl`, the fits in `fits.json` and `cost.md`.

## 1. What the fragment could not say

The brief's obligation parsed on the first try, nested `any ... where` and all. Two changes
were made to it, neither forced by the parser:

- **`all Ok(a) in out: a.vals unordered` was added.** Examples are compared by equality up
  to the order claims, and without the line the order of the assignment's entries would be a
  claim the obligation never meant to make. Every performance happens to sort by variable,
  so the line changes no verdict here; it is the honest claim. The mutant report says of it:
  `clause (line 17): all Ok(a) in out: a.vals unordered    # no mutant can violate it`.
- **Example 1 was changed to a formula with one model.** The brief's example, `(x1 or x2) and
  (not x1 or x3) and (not x2 or not x3)` with the model `{1: true, 2: false, 3: true}`, failed
  on the plain performance, which returns the lexicographically smallest model:

      FAIL  example 1 (line 21, "three clauses, satisfiable: (x1 or x2) and (not x1 or x3) and (not x2 or not x3)"): out.Ok.vals: expected element {value: true, var: 1} is missing (order is unspecified here): got [{value: false, var: 1}, {value: true, var: 2}, {value: false, var: 3}]

  An example pins one output. For a relational function a hand-written example of a formula
  with several models is a claim about which model, which the properties never make. The
  example is now `(x1) and (not x1 or x2) and (not x2 or x3)`, whose only model is all true,
  and its note says why.

The lines the fragment could not take. Each is kept under `attempts/` with the parser's
output in `records/attempts/`:

### 1a. "No assignment exists" has no domain

The claim `sat.sur` needs and cannot make: when the answer is `Err(unsat)`, no assignment
satisfies every clause. The fragment's quantifiers range over lists in the input and the
output; there is no list of all assignments to range over. The line tried
(`attempts/a_no_assignment_exists.sur`):

    all Err(_) in out: none b in Assignment: all c in f.clauses: any l in c.lits: any v in b.vals where v.var == l.var: (v.value == true) <-> (l.neg == false)

The parser:

    experiments/sat/attempts/a_no_assignment_exists.sur: line 21: `Assignment` is not an input, `out`, a binder here, a named value, or an abbreviation (unbound_name)

Spelled with a lowercase name (`none b in assignments: ...`, `attempts/a2_...`) the message is
the same. (The first spelling, `all Err(e) in out: none b in ...`, was refused one step
earlier with `binder `e` is never used (unused_binder)`; a binder a body never mentions must
be `_`.)

So nothing in `sat.sur` forces `Ok` on a satisfiable formula. The checker says so in three
places, none of them a bug:

1. **The search for two passing outputs** finds the gap on every obligation here, before any
   performance runs:

       UNDERDETERMINED  out can be Ok({vals: []}) or Err({code: "unsat"}); 8 inputs show it; the smallest:
             in: {clauses: [], n: 0}
             out: Ok({vals: []})    # example 3
             out: Err({code: "unsat"})    # also passes every clause

   All eight underdetermined inputs the search reports are `Ok`-versus-`Err` pairs (the
   JSON `underdetermined` records in `records/mutants_sat.json`), and they collapse into the
   one gap named above: `out` can be a model or `unsat`.

2. **The mutant library has no "always `Err(unsat)`" mutant.** Its two `Err` mutants carry
   an empty error (`empty`: `Err({code: ""})`) and a fabricated one (`err_fabricated`:
   `Err({code: "fab"})`), and both are caught, by the one line that reads the error:

       mutant empty caught on 504 of 504 inputs by line 19: all Err(e) in out: e == unsat (504 inputs)
         slipped through anyway: lines 13, 15, 17
       mutant err_fabricated caught on 504 of 504 inputs by line 19: all Err(e) in out: e == unsat (504 inputs)
         slipped through anyway: lines 13, 15, 17

   So `surety mutants` reports `slipped 0`, and the headline slip is not visible in the
   library verdict. It is visible one step over: a hand-written performance that answers
   `unsat` to everything (`sat_always_unsat.py`) passes every property:

       sat  NOT ACCEPTED  obligation: 8 underdetermined inputs, 1 gap  performance: fails examples 1, 3
         FAIL  example 1 (line 21, ...): out: expected Ok({vals: [{value: true, var: 1}, {value: true, var: 2}, {value: true, var: 3}]}), got Err({code: "unsat"})
         pass  example 2 (line 30, "unsatisfiable: (x) and (not x)")
         FAIL  example 3 (line 35, "the empty formula is satisfied by the empty assignment"): out: expected Ok({vals: []}), got Err({code: "unsat"})
         pass  example 4 (line 40, "a formula with an empty clause has no model")
         pass  property (line 13): ...  (504 runs, 0 findings)
         pass  property (line 15): ...  (504 runs, 0 findings)
         pass  property (line 17): ...  (504 runs, 0 findings)
         pass  property (line 19): all Err(e) in out: e == unsat  (504 runs, 0 findings)
           properties:       4 of 4 clean over 504 runs

   Only the two examples whose output is `Ok` stop it. Over 500 generated inputs, zero
   findings.

3. **The checker's own suggested passing output is `unsat` on a satisfiable formula.** When
   `sat_dpll.py` fails the canonical obligation's minimality line (section 5), the report
   offers the output that would have passed:

       PERFORMANCE FAILS  cause 1: property (line 21) on 60 of 500 generated inputs; an output that passes every clause exists
             in: {clauses: [{lits: [{neg: false, var: 0}, {neg: true, var: 2}]}, {lits: [{neg: false, var: 3}, {neg: false, var: 3}, {neg: false, var: 1}]}, {lits: [{neg: false, var: 3}, {neg: false, var: 2}, {neg: true, var: 1}]}], n: 3}
             out: Ok({vals: [{value: true, var: 0}, {value: false, var: 1}, {value: false, var: 2}, {value: true, var: 3}]})    # the performance's
             out: Err({code: "unsat"})    # passes every clause

### 1b. Lexicographic order needs a position

To say "the lexicographically smallest model" one needs the entry at a position. The line
tried (`attempts/b_position.sur`):

    all Ok(a) in out: a.vals[0].value == false

The parser:

    experiments/sat/attempts/b_position.sur: line 21: expected `..` in a slice: `xs[a .. b]` is the elements from position a up to but not including b, and an element at a position is the one-element slice `xs[i .. i + 1]` (syntax)

The one-element slice exists, but the comparison itself needs the other model to compare
with, which is 1a again. No line in the fragment states lexicographic minimality.

### 1c. The self-call cannot pin the class

The metamorphic form, "the obligation called on the same formula agrees with `out`"
(`attempts/c_self_call.sur`):

    all Ok(a) in out: all Ok(b) in sat(f): a == b

It parses and it passes, trivially: in a check the call runs the performance under test, so
the line compares the performance with itself. The mutant report names it as such:

    clause (line 21): all Ok(a) in out: all Ok(b) in sat(f): a == b    # no mutant can violate it

as the spec said it would (a self-call is set aside in both searches by the referee rule).

### 1d. `accept` is per input, not per pair of outputs

The non-decision "which model is returned" cannot be marked either. `accept` takes a
boolean over the inputs; "the formula has several models" is 1a again. The nearest
over-approximation that is a boolean over the inputs (`attempts/e_accept_the_choice.sur`):

    accept any c in f.clauses: len(c.lits) > 1

(a formula with two models has a clause of two or more literals, so the condition covers
every input where the choice arises). It parses and it fires on 444 of 504 inputs, and it
does what was asked of the model-versus-model pairs: the diff between plain and DPLL goes
from `84 within the obligation (0 accepted, 84 undecided)` to
`84 within the obligation (84 accepted, 0 undecided)`. But it covers the wrong pairs too.
The underdetermination report under the accept:

    ACCEPTED  two outputs pass every clause; accept (line 22)
          in: {clauses: [{lits: [{neg: false, var: 1}]}, {lits: [{neg: true, var: 1}, {neg: false, var: 2}]}, {lits: [{neg: true, var: 2}, {neg: false, var: 3}]}], n: 3}
          out: Ok({vals: [{value: true, var: 1}, {value: true, var: 2}, {value: true, var: 3}]})    # example 1
          out: Err({code: "unsat"})    # also passes every clause

The accept declares `unsat` on a satisfiable formula a deliberate non-decision. An accept
line cannot say "any model, but a model", because it speaks about inputs and the distinction
is between outputs.

### 1e. What was written instead

`sat.sur` keeps the brief's three claims plus the order claim: a returned model satisfies
every clause, names every variable, and the only error is `unsat`. `sat_canonical.sur` adds
the one narrowing the fragment can state (section 5). The "no assignment exists" half of
SAT is not in either file, and no example can supply it for more than one formula at a time.

## 2. Records

The verdict lines, from `records/verdicts.md` in the run directory.

<<RECORDS>>

## 3. Cost by family

<<COST>>

## 4. The canaries

<<CANARIES>>

## 5. The relational form

`sat_canonical.sur` is `sat.sur` with one more line, the one narrowing the fragment can
express without a domain of assignments: a model is locally minimal, every true variable is
the only true literal of some clause, so no true variable can be made false on its own.

    all Ok(a) in out: all v in a.vals where v.value == true: any c in f.clauses: any l in c.lits where l.var == v.var and l.neg == false: none m in c.lits where m.var != v.var: any w in a.vals where w.var == m.var: (w.value == true) <-> (m.neg == false)

What it changed:

- **It tells the performances apart.** The lexicographically smallest model is locally
  minimal, so `sat_plain.py` passes. DPLL branches true first and takes pure literals as
  they come: `sat_dpll.py` fails the line on 60 of 500 generated inputs. CDCL's saved
  phase defaults to false: `sat_cdcl.py` fails on 2 of 500. Neither failure is binding
  without `--strict`, so both checks still exit 0 with `performance: fails property (line 21)
  on N of 500 generated inputs`.
- **The diff reclassifies.** Plain against DPLL under `sat.sur`: `84 within the obligation
  (0 accepted, 84 undecided)`. Under `sat_canonical.sur`: `60 regressions, 24 within the
  obligation`. The 24 are formulas with several locally minimal models, which the line does
  not decide between either.
- **The mutant verdict does not move.** The same 3 of 14 mutants apply, the same 3 are
  caught, the same gap is reported: `out can be Ok({vals: []}) or Err({code: "unsat"})`. The
  new line catches no mutant, as the library has no mutant that returns a non-minimal model,
  and the search still finds `unsat` as the second passing output.

It does not pin the output to one model, and nothing in the fragment does: the minimal model
is not unique, and "smallest" needs the other models (1a, 1b). Lexicographic minimality
is therefore a finding, not a line.

What the language would need to say "the output class is the claim":

1. **A domain that is a type.** `none b in Assignment: ...` is the natural spelling, and the
   checker already enumerates candidate outputs of the output type in its search (600 per
   input, by the budget). The rule that binders range over lists in the input is what refuses
   it. Lifting it for finite, example-sized types would let the author write both halves of
   SAT and would make the brute-force checker the decision procedure, at the cost the brief
   says is not a concern. It would not help the performance's cost at all, which is the
   point of the experiment.
2. **An accept over outputs, or a relation on `out`.** `accept` marks an input region as
   undecided. The SAT non-decision is a region of output pairs: two `Ok` outputs are
   interchangeable, an `Ok` and an `Err` never are. The diff and the search already class
   pairs (`within`, `regression`, `fix`); the language has no line that names which pairs
   are the same answer. Something like `out unordered` for lists, but for "any value the
   properties admit, provided the side of the Result is fixed", would make the diff's 84
   differences `accepted` without accepting `unsat`.
3. **A certificate type for the negative side.** The honest alternative within today's
   language is to change the signature: return a resolution refutation (a DRAT trace) as the
   `Err` payload and check it. The checking is linear in the trace, in the obligation's own
   terms, and "no assignment exists" becomes "this trace derives the empty clause", which the
   fragment can state over lists. The trace is exponential where the solver is, which is the
   cost the harness measures anyway. This is the next step, and it is a different obligation.

## 6. Questions

Decisions the spec and the corpus did not settle, made here so the experiment could run.
`surety-docs/Questions from the build.md` is absent in this checkout, so they are recorded
here only.

1. **A relational function's examples.** An example of a satisfiable formula pins one model.
   The corpus examples all have one admissible output up to order claims. Should an example
   for a relational output be allowed to say `out: Ok(_)` or `out: Ok({ vals: ... })` up to
   the properties, or is the rule "write examples whose output is unique" (what was done
   here)?
2. **`n` is a field no property reads.** `Formula.n` was in the brief's types and nothing
   ties it to the clauses: the generator fills it with values unrelated to the variables
   (`{clauses: [{lits: [{neg: false, var: 3}, {neg: true, var: 4}]}], n: 0}`). The fragment has
   no refinement spanning two fields (`Lit[]` bounded by `n`), and a precondition relating two
   things is an error case in a `Result`, which would make `n` an input the performance
   validates. The performances ignore it and take the variables from the clauses. Should the
   type drop `n`, or should the obligation carry an `Err` for a literal past `n`?
3. **The `Bool` comparison spelling.** `v.value == true` and `l.neg == false` are what the
   brief uses and what parses; `(v.value == true) <-> (l.neg == false)` reads as a double
   negative where `v.value <-> not l.neg` would not. Is a bare `Bool` field a proposition in
   the fragment, or always compared?
4. **Findings on generated inputs are not binding.** `sat_dpll.py` fails the canonical line on
   60 of 500 generated inputs and the check exits 0; the same fails under `--strict`. For an
   obligation whose examples cannot cover the claim (a relational one), the generated tier
   carries the whole weight. Is `--strict` the intended mode for such obligations, or should
   a clean example tier with a failing generated tier read as a failure by default?
5. **The library's `Err` mutants.** Both carry errors of no declared value, so an obligation
   with a named error value catches them with one line and reports `slipped 0` while a
   performance that always returns the declared error passes every property. Should the
   library carry "`Err` of each named error value" as a mutant? That is a question for the
   catch-up note's mutant list; it is not fixed here.
6. **The accept rule's granularity** (1d): an accept over inputs cannot distinguish an
   `Ok`/`Ok` pair from an `Ok`/`Err` pair on the same input. Recorded as a question, not
   proposed as a change.
