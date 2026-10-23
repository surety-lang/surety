```surety
# If the iterable has at least n items, the output is the iterable's items from position 0 up to but not including position n.
	len(iterable) >= n -> out == iterable[0 .. n]
# If the iterable has fewer than n items, the output is the whole iterable, in its order.
	len(iterable) < n -> out == iterable
```