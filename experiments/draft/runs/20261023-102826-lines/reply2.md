```surety
# If n is at least 0 and n is less than the number of items, the output is Ok of the item at position n.
	n >= 0 and n < len(iterable) -> exactly_one Ok(v) in out: v in iterable[n .. n + 1]
# If n is at least 0, the iterable is not empty, and n is at least the number of items, the output is Ok of the last item.
	n >= 0 and len(iterable) > 0 and n >= len(iterable) -> exactly_one Ok(v) in out: v in iterable[len(iterable) - 1 .. len(iterable)]
# If n is at least 0, the iterable is empty, and a default is given, the output is Ok of the default.
	n >= 0 and len(iterable) == 0 and default != None -> exactly_one Ok(v) in out: v == default
# If n is at least 0, the iterable is empty, and no default is given, the output is the empty error.
	n >= 0 and len(iterable) == 0 and default == None -> exactly_one Err(e) in out: e == empty_error
# If n is less than 0, the output is the negative-n error, whether or not the iterable is empty and whether or not a default is given.
	n < 0 -> exactly_one Err(e) in out: e == negative_n_error
```