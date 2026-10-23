```surety
# The output is a tail of the input: the input's items from position (the input's length minus the output's length) up to the end.
	out == iterable[len(iterable) - len(out) .. len(iterable)]
# Every input item left out of the output satisfies pred.
	all x leftover in iterable: x.matches_pred == true
# The output is empty or its first item does not satisfy pred.
	all x in out[0 .. 1]: x.matches_pred == false
```