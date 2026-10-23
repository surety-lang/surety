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
	reaching for: the list made by taking each non-empty group's first item (g[0 .. 1]) in the order the groups appear in gs, compared with == to items[0 .. n]; the notation has no way to map a slice over every element of a list to build that list

# Distributing the input without its first n items gives each group without its first item.
	reaching for: for a successful output gs, distribute(n, items[n .. len(items)]) is Ok(rest) where, for every position i, rest's group at i equals gs's group at i with its first item dropped (g[1 .. len(g)]); the notation has no positional pairing of two lists and no map over a list, so "each group without its first item" cannot be built or compared group by group
```