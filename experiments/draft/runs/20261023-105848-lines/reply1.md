```surety
# The output is drawn from the input elements' results.
	out drawn_from iterable.result
# Every input element whose result is left out of the output has a result of None.
	all r leftover in iterable.result: r == None
# The output keeps the input's order of results.
	out same_order_as iterable.result
```