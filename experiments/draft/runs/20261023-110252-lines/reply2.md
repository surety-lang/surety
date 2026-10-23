```surety
# The output is the negative-index error exactly when n is less than 0.
	n < 0 <-> any Err(e) in out: e == negative_index_error
# If n is at least 0 and less than the length of the iterable, the output is the item at position n.
	n >= 0 and n < len(iterable) -> any Ok(v) in out: v in iterable[n .. n + 1]
# If n is at least the length of the iterable, the output is the default.
	n >= len(iterable) -> any Ok(v) in out: v == default
```