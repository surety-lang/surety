# Decision: an obligation is accepted only after `check` and `mutants` have run on it (2026-10-21)

On the detector's first run over the corpus it found that P5b's first property was written in the out-to-in direction while the sentence above it said in-to-out, which admitted the always-empty performance. The author knew the nothing-lost rule from day one and wrote the sentence correctly. A reviewer read the file the same day. Neither saw it. The prose-beside-notation convention was in place and did not help.

## Rule

An obligation is accepted into the graph only after:

1. `check` has run its generated tier over it against at least one passing performance, and
2. `mutants` has run over it, and
3. the author has answered every finding: with a fix to the obligation, a refinement on a type, or an explicit accepted non-decision (the way `unordered` says "no claim here" for order).

Reading is not a check. Running is. A `#` sentence above a property is for the reader; it is not evidence that the property says what the sentence says, and the lint that asks for a sentence on nested quantifiers stays a lint.

## What this changes

- The risk formerly named "humans won't write specs" is renamed **"specs are unsafe without the check."** It is a workflow risk, not a skills risk, and it is cheaper: the workflow is three commands. The number for the risk table: 1 of 11 obligations written by the language's author carried a silent hole on first run.
- Spec v2's data flow already puts the mutant library at the first step of a change; this makes it a gate on acceptance rather than a step in the flow.
- Accepted non-decisions need a place to live. Until the graph store exists (slice 3), the accepted list is a line in the obligation's `note` and a row in the findings report; P5a's tie on `ordered_at` is the first entry.
