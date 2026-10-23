```surety
# The output is the items of the iterable from position (the item count minus n) up to the end, in their original order.
	out == iterable[len(iterable) - n .. len(iterable)]
```