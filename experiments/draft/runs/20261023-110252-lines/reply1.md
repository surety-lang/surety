```surety
# The output is the negative-index error exactly when n is less than 0.
	out == Err(negative_index_error) <-> n < 0
# If n is at least 0 and less than the length of the iterable, the output is the item at position n.
	0 <= n and n < len(iterable) -> any x in iterable[n .. n + 1]: out == Ok(x)
# If n is at least the length of the iterable, the output is the default.
	n >= len(iterable) -> out == Ok(default)
```