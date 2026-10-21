# Decision: preconditions are refinement types (2026-10-20)

Came out of problem 14. `snapshot_diff(old, new)` only makes sense if each input has unique skus. That is a claim about the inputs, and the fragment had no place for one.

## Rule

A precondition that is structural goes in the input's type, as a refinement, and the boundary validator enforces it. There is no `requires` slot.

```
snapshot_diff(old: Item[unique sku], new: Item[unique sku]) -> Diff
```

If a precondition cannot be stated as a type on one input (it relates two inputs, e.g. `page * size <= len(items)`), that is the sign the function should return a `Result` and make it an error case. Problem 13 is the first test of this.

This is the same move as `Score[0..3]` on an output: a value-level fact about a list, carried by the type, checked at the boundary. It says nothing a property would otherwise need to say; `len(out) == min(3, len(scores))` still has to be stated. A refinement is a bound, not a count.

## Brackets

- Angle brackets hold **types**: `Option<T>`, `Result<T, E>`. These are constructors with type parameters.
- Square brackets hold **list refinements**: `Item[]`, `Item[0..3]`, `Item[unique sku]`, and they compose: `Item[0..3, unique sku]`. These are the same list type with a constraint on it, not a new type.

Writing `List<Item>` next to `Item[]`, or `Unique<Item>[]` next to `Item[unique sku]`, would be two spellings of one thing; same objection as `=` vs `==`. `[]` is the list constructor everywhere.

`Unique<Item::sku>[]` was the first attempt and is wrong for the same reason `all g in out: unique g.customer_email` was wrong: it puts uniqueness on the element. Uniqueness is a property of the list, so the refinement wraps the list.

`unique` inside a type is the same word as `unique` in a property, in a type position. `[0..3]` is an inclusive range, not slice notation.

## Reached for

- `Set<T>`: not needed. A `Set` says unordered, and inputs are lists whose order may matter elsewhere (5a, 5b). `Item[unique sku]` is a list with a key constraint, which is what every case so far has meant.

## Addendum (2026-10-21): a precondition that relates two things is an error case

Slice 2 found the case the rule predicted. `apply_diff` is contradictory when one sku is in both `diff.removed` and `diff.changed`: the obligation requires it absent from the result and present in it. No refinement on one list can exclude that input; it relates two fields of one input.

The rule stands as written, with its scope stated more carefully: a precondition that can be expressed as a refinement on **one** list or field goes in the type; a precondition that relates **two** things (two inputs, or two fields of one input) is not a type and becomes an error case in a `Result`. `apply_diff` gained `conflicting_sku_err`, a `disjoint` abbreviation (`none s in diff.removed.sku: s in diff.changed.sku`), and a fourth conjunct in `checks`. Overlaps with `added` are already caught by `added_ok` (added requires the sku absent from `old`; removed and changed require it present), so only the removed/changed pair needed a new error.

Problem 13 (pagination, `page * size <= len(items)`) is the same shape and gets the same answer.
