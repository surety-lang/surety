```surety
# The output is an error exactly when the iterable is empty and no default is supplied.
	(len(iterable) == 0 and default == None) <-> exactly_one Err(_) in out
# An error output is the empty-iterable error.
	all Err(e) in out: e == empty_error
# If the iterable is not empty, the output is the item at its last position, whether or not a default is supplied.
	len(iterable) > 0 -> exactly_one Ok(x) in out: x in iterable[len(iterable) - 1 .. len(iterable)]
# If the iterable is empty and a default is supplied, the output is the default.
	(len(iterable) == 0 and default != None) -> exactly_one Ok(x) in out: x == default
```