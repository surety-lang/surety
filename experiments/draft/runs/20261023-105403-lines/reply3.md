```surety
# The output is the length error exactly when strict is set and two iterable arguments have different numbers of items.
	out == Err(length_error) <-> strict and any a in objects where not a.scalar: any b in objects where not b.scalar: len(a.items) != len(b.items)
# Every row has as many values as there are arguments.
	all Ok(rows) in out: all r in rows: len(r.values) == len(objects)
# In every row, the value at a scalar argument's position is that argument's value.
	reaching for: a binder for an argument's position in objects, so that for every row r and every scalar argument a at position i, r.values at position i == a.value (e.g. `all i, a in objects where a.scalar: all r in rows: r.values[i .. i + 1] == [a.value]`)
# In row k, the value at an iterable argument's position is that argument's item at position k.
	reaching for: binders for positions in two lists at once, so that for every row at position k in rows and every iterable argument a at position i in objects, the row's value at position i == a.items at position k (e.g. `all k, r in rows: all i, a in objects where not a.scalar: r.values[i .. i + 1] == a.items[k .. k + 1]`)
# Every iterable argument has at least as many items as the output has rows.
	all Ok(rows) in out: all a in objects where not a.scalar: len(a.items) >= len(rows)
# If some argument is iterable, some iterable argument has exactly as many items as the output has rows.
	all Ok(rows) in out: all a in objects where not a.scalar: any b in objects where not b.scalar: len(b.items) == len(rows)
# If no argument is iterable, the output has one row when some argument is a scalar and no rows otherwise.
	all Ok(rows) in out: len(rows) != min(1, len(objects)) -> any a in objects: not a.scalar
```