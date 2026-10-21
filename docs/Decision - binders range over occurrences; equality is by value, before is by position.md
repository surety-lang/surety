# Decision: binders range over occurrences; equality is by value, `before` is by position (2026-10-21)

Came out of slice 2's first run. P5b (keep the first occurrence per email) is a dedupe, and its central input is two identical orders. On `orders = [X, X]` the property

```
all o in out:
  all l leftover in orders where l.customer_email == o.customer_email:
    o before l in orders
```

was false under the slice 1 semantics: `leftover` subtracted by value and `before` found positions by first occurrence, so `o` and `l` both resolved to position 0 and "the leftover one is later" failed. The intent was satisfiable; the notation could not say "the second copy." A dedupe obligation that is false on duplicates is not acceptable, so this is a language decision, not a residue.

## Rule

- A binder introduced by `in L` or `leftover in L` ranges over **occurrences** of `L`: a position and the value at it.
- `==`, `in`, `not in`, and every list relation (`drawn_from`, `same_order_as`, `same_elements_as`, `unique`) compare **values**. Nothing about positions leaks into them.
- `leftover in L` removes, for each output element, the **earliest occurrence** of an equal value in `L` not yet removed. What remains are occurrences.
- `a before b in L` compares **positions**. An operand that is an occurrence of `L` has its position. An operand that is not (an element of `out`, say) stands for the occurrence of its value that `leftover` assigned to it, i.e. the earliest occurrence not left over; if no `leftover` is in scope, its first occurrence.

Under this, `orders = [X, X]`, `out = [X]`: `out`'s `X` is position 0, the leftover is position 1, `0 before 1` holds, and the obligation is true on its own core input. The observable consequence in the P5b case is the same as the smaller reading the build proposed (`a before b` with `a == b` is true when the value occurs twice), but stated as a rule about what a binder is rather than a special case of one operator.

## What it does not change

Every property in the corpus except P5b's `before` line evaluates identically, because none of them compares positions. The hash is unchanged (occurrence vs value is evaluation, not syntax). `unique` still means "no two occurrences share the key," which it always did.

## Why not the smaller reading

"`a before b` is true for equal values when the list has duplicates" is true as a consequence but wrong as a definition: it would also hold when `out` kept the *second* copy, and it says nothing about three copies. The occurrence rule gives the same answer on P5b and a correct one everywhere else, and it is one sentence about binders rather than one exception about `before`.
