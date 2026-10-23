```surety
# The output is Ok exactly when the items are not empty or a default is given.
	(len(items) > 0 or default != None) <-> exactly_one Ok(_) in out
# Every error output is the empty error.
	all Err(e) in out: e == empty_error
# If the items are empty and a default is given, the output is Ok carrying the default unchanged.
	(len(items) == 0 and default != None) -> exactly_one Ok(m) in out: m == default
# If the items are not empty, the smallest and the largest in an Ok output are both items from the input.
	all Ok(m) in out where len(items) > 0: m.smallest in items and m.largest in items
# No input item has a key less than the smallest's key.
	all Ok(m) in out: none i in items: i.key < m.smallest.key
# No input item has a key greater than the largest's key.
	all Ok(m) in out: none i in items: i.key > m.largest.key
```