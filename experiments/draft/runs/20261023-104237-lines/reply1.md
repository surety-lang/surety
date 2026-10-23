```surety
# For every iterable, its elements from position 0 up to but not including the output's length are exactly the output, in order.
	all it in iterables: it[0 .. len(out)] == out
# Every iterable either has exactly as many elements as the output, or some iterable's elements from position 0 up to but not including one past the output's length differ from that iterable's own.
	all a in iterables: len(a) == len(out) or any b in iterables: b[0 .. len(out) + 1] != a[0 .. len(out) + 1]
```