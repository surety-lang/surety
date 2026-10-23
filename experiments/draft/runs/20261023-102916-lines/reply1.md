```surety
# The yielded items are the iterable's items from position 0 up to but not including n.
	out.yielded == iterable[0 .. n]
# too_short is called exactly when the iterable has fewer than n items.
	out.too_short_called_with != None <-> len(iterable) < n
# Whenever too_short is called, it is called with the number of items in the iterable.
	out.too_short_called_with != None -> out.too_short_called_with == len(iterable)
# too_long is called exactly when the iterable has more than n items.
	out.too_long_called_with != None <-> len(iterable) > n
# Whenever too_long is called, it is called with n plus one.
	out.too_long_called_with != None -> out.too_long_called_with == n + 1
```