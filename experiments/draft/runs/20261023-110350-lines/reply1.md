```surety
# The count is at least 0.
	out >= 0
# The count is at most the number of items.
	out <= len(iterable)
# The count is 0 exactly when no item holds.
	out == 0 <-> none t in iterable: t.holds
# The count is 1 exactly when exactly one item holds.
	out == 1 <-> exactly_one t in iterable: t.holds
# The count is the number of items exactly when every item holds.
	out == len(iterable) <-> all t in iterable: t.holds
```