```surety
# If n is at least 0, the output has n times as many elements as the iterable.
	n >= 0 -> len(out) == n * len(iterable)
# If n is negative, the output is empty.
	n < 0 -> len(out) == 0
# If n is at least 1, the output's first elements, as many as the iterable has, are the iterable's elements in the iterable's order.
	n >= 1 -> out[0 .. len(iterable)] == iterable
# The output from position len(iterable) to its end is the same as the output from its start up to but not including the last len(iterable) positions, so each output element repeats the one an iterable's length before it.
	out[len(iterable) .. len(out)] == out[0 .. len(out) - len(iterable)]
```