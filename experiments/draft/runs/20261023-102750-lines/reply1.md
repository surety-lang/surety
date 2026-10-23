```surety
# The output is an error exactly when the iterable is empty and no default is supplied.
	(len(iterable) == 0 and default == None) <-> exactly_one Err(_) in out
# If the iterable has at least one item, the output is Ok with the iterable's item at position 0.
	len(iterable) >= 1 -> exactly_one Ok(x) in out: x in iterable[0 .. 1]
# If the iterable is empty and a default is supplied, the output is Ok with the default.
	(len(iterable) == 0 and default != None) -> exactly_one Ok(x) in out: x == default
# Every error output is the empty-iterable error.
	all Err(e) in out: e == empty_error
```