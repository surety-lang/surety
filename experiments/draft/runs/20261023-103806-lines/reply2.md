```surety
# The output is an error exactly when n is less than 1, and that error is n_error.
	n < 1 and out == Err(n_error) or n >= 1 and exactly_one Ok(_) in out
# A successful output has exactly n groups.
	all Ok(groups) in out: len(groups) == n
# The groups taken together hold the same elements as the input, each occurrence once.
	all Ok(groups) in out: flatten(groups) same_elements_as items
# Every group keeps the input's order.
	all Ok(groups) in out: all g in groups: g same_order_as items
# Every group has at least as many items as each group after it.
	all Ok(groups) in out: all a in groups: all b in groups where a before b in groups: len(a) >= len(b)
# The first group has at most one item more than the last group.
	all Ok(groups) in out: all f in groups[0 .. 1]: all l in groups[len(groups) - 1 .. len(groups)]: len(f) <= len(l) + 1
# The first items of the non-empty groups, taken in group order, are the input's first items in order.
	reaching for: the list built by taking each group's first item (g[0 .. 1] for every g in groups, concatenated in group order) equals items[0 .. len(groups)]; the notation has no way to map a slice over every element of a list, or to name a group's position so that the group at position k can be paired with items[k]
# Distributing the input without its first n items gives each group without its first item.
	reaching for: for every Ok(groups) in out and Ok(rest) in distribute(n, items[n .. len(items)]), len(rest) == len(groups) and, at every position k, rest's group at k equals groups' group at k with its first item dropped (that group's [1 .. len] slice); the notation can call the obligation and slice a list, but it cannot pair two lists' elements by position or map a slice over a list to build "each group without its first item"
```