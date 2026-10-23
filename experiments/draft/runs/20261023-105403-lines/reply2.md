```surety
# The output is the length error exactly when strict is set and two iterable arguments have different numbers of items.
	strict and (any a in objects where not a.scalar: any b in objects where not b.scalar: len(a.items) != len(b.items)) <-> any Err(e) in out: e == length_error
# Every row has as many values as there are arguments.
	all Ok(rows) in out: all r in rows: len(r.values) == len(objects)
# In every row, the value at a scalar argument's position is that argument's value.
	reaching for: a binder for an argument's position i in objects, so that for every row r and every position i where objects[i].scalar, r.values at position i == objects[i].value
# In row k, the value at an iterable argument's position is that argument's item at position k.
	reaching for: binders for a row's position k in the output rows and an argument's position i in objects, so that for every row k and every i where not objects[i].scalar, rows[k].values at position i == objects[i].items at position k
# Every iterable argument has at least as many items as the output has rows.
	all Ok(rows) in out: all a in objects where not a.scalar: len(a.items) >= len(rows)
# If some argument is iterable, some iterable argument has exactly as many items as the output has rows.
	all Ok(rows) in out: all a in objects where not a.scalar: any b in objects where not b.scalar: len(b.items) == len(rows)
# If no argument is iterable, the output has one row when some argument is a scalar and no rows otherwise.
	all Ok(rows) in out: len(rows) == min(1, len(objects)) or any a in objects: not a.scalar
```