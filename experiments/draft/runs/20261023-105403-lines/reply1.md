```surety
# The output is the length error exactly when strict is set and two iterable arguments have different numbers of items.
	out == Err(length_error) <-> strict and any a in objects where not a.scalar: any b in objects where not b.scalar: len(a.items) != len(b.items)
# Every row has as many values as there are arguments.
	all Ok(rows) in out: all r in rows: len(r.values) == len(objects)
# In every row, the value at a scalar argument's position is that argument's value.
	reaching for: the position of an argument in objects as an Int (or indexing r.values by the position of a binder in objects), to say: all Ok(rows) in out: all r in rows: all a at position i in objects where a.scalar: r.values[i] == a.value
# In row k, the value at an iterable argument's position is that argument's item at position k.
	reaching for: the positions of a row in rows and of an argument in objects as Ints, to say: all Ok(rows) in out: all r at position k in rows: all a at position i in objects where not a.scalar: r.values[i] == a.items[k]
# Every iterable argument has at least as many items as the output has rows.
	all Ok(rows) in out: all a in objects where not a.scalar: len(a.items) >= len(rows)
# If some argument is iterable, some iterable argument has exactly as many items as the output has rows.
	all Ok(rows) in out: false in objects.scalar -> any a in objects where not a.scalar: len(a.items) == len(rows)
# If no argument is iterable, the output has one row when some argument is a scalar and no rows otherwise.
	false not in objects.scalar -> exactly_one Ok(rows) in out: (true in objects.scalar -> len(rows) == 1) and (true not in objects.scalar -> len(rows) == 0)
```