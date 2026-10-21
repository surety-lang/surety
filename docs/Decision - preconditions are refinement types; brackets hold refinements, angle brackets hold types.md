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
