```surety
# The result is Ok exactly when the iterable has exactly one item.
	len(iterable) == 1 <-> exactly_one Ok(_) in out
# An Ok result is an item of the iterable.
	all Ok(x) in out: x in iterable
# If the iterable is empty and too_short is given, the error is too_short.
	len(iterable) == 0 and too_short != None -> all Err(e) in out: e == too_short
# If the iterable is empty and too_short is not given, the error is the default too-short
# ValueError.
	len(iterable) == 0 and too_short == None -> all Err(e) in out: e == default_too_short_err
# If the iterable has more than one item and too_long is given, the error is too_long.
	len(iterable) > 1 and too_long != None -> all Err(e) in out: e == too_long
# If the iterable has more than one item and too_long is not given, the error's kind is
# ValueError and its message is the too-long message for the first two items, which a sibling
# obligation decides.
	len(iterable) > 1 and too_long == None -> all Err(e) in out: e.kind == "ValueError" and default_too_long_message(e.message, iterable[0 .. 2])
```