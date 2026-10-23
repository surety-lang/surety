```surety
# The chunks, joined in order, are the input list.
	flatten(out) == iterable
# Every chunk has at most n items.
	all c in out: len(c) <= n
# Any two chunks' lengths differ by at most one.
	all a in out: all b in out: abs(len(a) - len(b)) <= 1
# One fewer chunk than the output has, at n items each, could not hold the input: the output's chunk count minus one, times n, is less than the input's length.
	(len(out) - 1) * n < len(iterable)
# No chunk is longer than the chunk before it.
	all a in out: all b in out where a before b in out: len(b) <= len(a)
```