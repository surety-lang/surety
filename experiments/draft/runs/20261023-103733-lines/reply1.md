```surety
# When n is at least 0, the output's length is the iterable's length times n.
	n >= 0 -> len(out) == len(iterable) * n
# When n is negative, the output is empty.
	n < 0 -> len(out) == 0
# Every output element is an element of the iterable.
	all x in out: x in iterable
# Each of the first n elements of the output is the iterable's first element.
	all x in out[0 .. n]: x in iterable[0 .. 1]
# The output from position n onward is repeat_each of the iterable from its second element onward, with the same n.
	out[n .. len(out)] == repeat_each(iterable[1 .. len(iterable)], n)
```