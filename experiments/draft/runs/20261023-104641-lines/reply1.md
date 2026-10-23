```surety
# The output is Ok exactly when the iterable has at most one item.
	len(iterable) <= 1 <-> exactly_one Ok(_) in out
# Every item of the iterable is the Ok value.
	all i in iterable: all Ok(v) in out: v == i
# An Ok value that is not an item of the iterable is the default.
	all Ok(v) in out: v not in iterable -> v == default
# An Err output is the too_long error the caller gave, or the ValueError when the caller gave none.
	all Err(e) in out: (too_long != None -> e == too_long) and (too_long == None -> e == value_error)
```