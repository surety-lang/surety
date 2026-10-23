```surety
# The output is the items from position 0 up to but not including position len(out), so it is a prefix of the input in the input's order.
	out == items[0 .. len(out)]
# Every output element before the last one is less than the limit.
	all x in out[0 .. len(out) - 1]: x < limit
# If every input item is less than the limit, the output is the whole input.
	out == items or any x in items: x >= limit
# If some input item is at least the limit, the last output element is at least the limit.
	all x in items where x >= limit: any y in out[len(out) - 1 .. len(out)]: y >= limit
```