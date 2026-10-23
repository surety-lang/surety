```surety
# The output is None exactly when the values are empty.
	out == None <-> len(values) == 0
# A returned index is a position in the values: at least 0 and less than the number of values.
	reaching for: a binder for the present side of an Option, the way `Ok(i)` binds one side of a Result, so I could write `all Some(i) in out: i >= 0 and i < len(values)`; the notation has no way to name the Int inside `out`
# No value is greater than the value at the returned index.
	reaching for: the same Option binder, then the one-element slice at the index, as in `all Some(i) in out: all m in values[i .. i + 1]: none v in values: v > m`; without a name for the Int inside `out`, the value at the returned index can't be reached
# Every value before the returned index is less than the value at the returned index.
	reaching for: the same Option binder, then the slice before the index against the one-element slice at it, as in `all Some(i) in out: all m in values[i .. i + 1]: all v in values[0 .. i]: v < m`; without a name for the Int inside `out`, neither slice can be bounded
```