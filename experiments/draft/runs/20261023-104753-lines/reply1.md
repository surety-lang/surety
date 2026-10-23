```surety
# The output is Ok exactly when r is at least 0.
	r >= 0 <-> exactly_one Ok(_) in out
# Every error output is the negative-r error.
	all Err(e) in out: e == negative_r_error
# In an Ok output, every combination has exactly r items.
	all Ok(cs) in out: all c in cs: len(c.items) == r
# In an Ok output, every combination's items are drawn from the iterable and keep the iterable's order, so each combination is a sub-sequence of the iterable.
	all Ok(cs) in out: all c in cs: c.items same_order_as iterable
# In an Ok output, no two combinations have the same items.
	all Ok(cs) in out: unique cs.items
# In an Ok output, every sub-sequence of r items of the iterable is the items of some combination.
	reaching for: all Ok(cs) in out: all s in <the sub-sequences of iterable> where len(s) == r: s in cs.items. This needs a quantifier over every sub-sequence of a list, or a built-in that lists them. The notation's binders range only over lists already in the inputs or the output, so no line can range over the sub-sequences the output must cover.
# In an Ok output, the combinations are not claimed in any order.
	all Ok(cs) in out: cs unordered
```