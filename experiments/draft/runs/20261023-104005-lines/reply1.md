```surety
# The output is Ok exactly when n is at least 1.
	n >= 1 <-> exactly_one Ok(_) in out
# An error output is the n-too-small error.
	all Err(e) in out: e == n_too_small_error
# An Ok output has exactly n parts.
	all Ok(parts) in out: len(parts) == n
# The items of the parts, taken in part order, are the iterable element for element and in its order.
	all Ok(parts) in out: flatten(parts.items) == iterable
# No two parts of an Ok output differ in length by more than one.
	all Ok(parts) in out: all a in parts: all b in parts: abs(len(a.items) - len(b.items)) <= 1
# No part of an Ok output is longer than a part before it.
	all Ok(parts) in out: all a in parts: all b in parts where a before b in parts: len(b.items) <= len(a.items)
```