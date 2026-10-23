```surety
# The output is Ok exactly when n is at least 1.
	n >= 1 <-> exactly_one Ok(_) in out
# An Err output carries size_error, meaning n is less than 1.
	all Err(e) in out: e == size_error and n < 1
# The chunks' items, joined in the order of the chunks, are exactly the iterable's elements in the iterable's order.
	all Ok(cs) in out: flatten(cs.items) == iterable
# Every chunk has at most n items.
	all Ok(cs) in out: all c in cs: len(c.items) <= n
# Every chunk has at least one item.
	all Ok(cs) in out: all c in cs: len(c.items) >= 1
# A chunk with fewer than n items comes after every other chunk.
	all Ok(cs) in out: all c in cs where len(c.items) < n: none d in cs: c before d in cs
```