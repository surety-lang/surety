```surety
# The values of all the groups, taken in group order and within each group in order, are exactly the input items' values in input order.
	flatten(out.values) == items.value
# Every group has at least one value.
	all g in out: len(g.values) >= 1
# Every group's values are the values of a run of consecutive input items, all of which carry that group's key.
	reaching for: for each group g in out, some start position p in items such that items[p .. p + len(g.values)].value == g.values and every item in that slice has key == g.key. The slice needs an integer start, and the notation has no binder over positions and no way to take the position of an occurrence. `g.values same_order_as items.value` says subsequence, not consecutive, so it would widen the sentence.
# No group has the same key as the group right after it.
	all a in out: all b in out where a before b in out: a.key != b.key or any c in out: a before c in out and c before b in out
```