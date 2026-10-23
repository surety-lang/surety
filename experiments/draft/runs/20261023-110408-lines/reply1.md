```surety
# The output has exactly taken elements.
	len(out) == taken
# The output's elements before position len(iterable) are the iterable's elements before position taken, in the same order.
	out[0 .. len(iterable)] == iterable[0 .. taken]
# Every output element at or past position len(iterable) is None.
	all x in out[len(iterable) .. len(out)]: x == None
```