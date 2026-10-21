**Date:** 2026-10-19
**Surfaced by:** Problem 7, sentence 2

## The case

Two phrasings of the same property:

- Explicit: *user_name is the name of the input user whose id equals the order's user_id, if such a user exists; otherwise None.*
- Natural: *user_name is the name of the input user if the order's user_id matches, otherwise user_name is None.*

The explicit form is complete but reads stiffly ("whose" was the sticking point). The natural form reads well and leaves "matches *what*" implicit: a human fills in "an input user's id" without noticing.

## The decision

Prose sentences in an obligation are written the natural way. They are for the human: the author checking their own thinking, and a reviewer reading the obligation later. Ambiguity that any reader resolves the same way is acceptable in prose.

The compressed notation is held to the stricter standard. Every comparison names both sides; every quantifier names its range. Nothing is left for a reader to fill in, because the reader is a checker and a model, and neither one gets to guess.

## Consequence

This is a division of labor between the two layers of the spec, not a defect in either. The prose carries intent at reading speed; the notation carries the exact claim. When the consolidation pass compresses the sentence corpus into notation, the implicit phrases in the prose ("matches," "corresponding," "the first") are exactly the places where the notation has to add an operand. Expect each one to show up as a small expansion.

Follow-on: the prose and the notation for the same property should sit next to each other in the obligation, so the expansion is visible and the prose doesn't drift into saying something the notation doesn't check.