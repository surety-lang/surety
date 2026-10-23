```surety
# The output is an error exactly when n is less than 1, and that error is n_error.
	(n < 1 <-> exactly_one Err(_) in out) and all Err(e) in out: e == n_error
# A successful output has exactly n groups.
	all Ok(gs) in out: len(gs) == n
# The groups taken together hold the same elements as the input, each occurrence once.
	all Ok(gs) in out: flatten(gs) same_elements_as items
# Every group keeps the input's order.
	all Ok(gs) in out: all g in gs: g same_order_as items
# Every group has at least as many items as each group after it.
	all Ok(gs) in out: all a in gs: all b in gs where a before b in gs: len(a) >= len(b)
# The first group has at most one item more than the last group.
	all Ok(gs) in out: all f in gs[0 .. 1]: all l in gs[len(gs) - 1 .. len(gs)]: len(f) <= len(l) + 1
# The first items of the non-empty groups, taken in group order, are the input's first items in order.
	reaching for: the list made of each non-empty group's first item, in group order, compared with ==, so the line can say it is items[0 .. k], where k is the number of non-empty groups. The notation has no map over the groups and no binder for a group's position. Comparing the heads pairwise with `before in items` resolves equal values to their first occurrence, so the line would be false on [7, 7, 7].
# Distributing the input without its first n items gives each group without its first item.
	reaching for: exactly_one Ok(hs) in distribute(n, items[n .. len(items)]), where the group at position i of hs equals gs[i][1 .. len(gs[i])] for every position i. That needs pairing the groups of two lists by position, or a map that drops each group's first item. A membership test (`g[1 .. len(g)] in hs`) does not pin the position, so it would admit the groups in the wrong order.
```