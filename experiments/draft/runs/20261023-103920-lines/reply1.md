```surety
# The output is Ok exactly when there are as many offsets as iterables.
	len(offsets) == len(iterables) <-> exactly_one Ok(_) in out
# Every error output is the mismatch error.
	all Err(e) in out: e == mismatch_error
# Every row has one cell per iterable, in the order of the iterables.
	reaching for: for every row r in an Ok output and every position i below len(iterables), cell i of r belongs to iterables[i]. The count half alone is a line (`all Ok(rows) in out: all r in rows: len(r.cells) == len(iterables)`), but that drops the ordering, and pairing cell i with iterable i needs a binder that carries a position as an Int, which the fragment does not have.
# Cell i of row k is the item of iterable i at position k plus offset i when that position is within iterable i, and the fillvalue otherwise.
	reaching for: all Ok(rows) in out: for every row position k and every cell position i: if 0 <= k + offsets[i] < len(iterables[i].items) then rows[k].cells[i] is iterables[i].items[k + offsets[i]], else rows[k].cells[i] == fillvalue. This needs index binders over 0 .. len(rows) and 0 .. len(iterables), and element access at a computed position. A slice gives back a list, not the element.
# There are rows only if there is at least one iterable.
	all Ok(rows) in out: len(rows) > 0 -> len(iterables) > 0
# If longest is false and the output has rows, every iterable has at least the number of rows plus its offset items.
	reaching for: all Ok(rows) in out: longest == false and len(rows) > 0 -> for every position i below len(iterables), len(iterables[i].items) >= len(rows) + offsets[i]. Pairing an iterable with "its offset" pairs iterables[i] with offsets[i] by position, and the fragment has no index binder or zip to do that.
# If longest is false and there is at least one iterable, some iterable has at most the number of rows plus its offset items.
	reaching for: all Ok(rows) in out: longest == false and len(iterables) > 0 -> there is some position i below len(iterables) with len(iterables[i].items) <= len(rows) + offsets[i]. This needs the same positional pairing of iterables[i] with offsets[i], under an existential over positions.
# If longest is true, every iterable has at most the number of rows plus its offset items.
	reaching for: all Ok(rows) in out: longest == true -> for every position i below len(iterables), len(iterables[i].items) <= len(rows) + offsets[i]. This needs the positional pairing of iterables[i] with offsets[i].
# If longest is true and the output has rows, some iterable has at least the number of rows plus its offset items.
	reaching for: all Ok(rows) in out: longest == true and len(rows) > 0 -> there is some position i below len(iterables) with len(iterables[i].items) >= len(rows) + offsets[i]. This needs the positional pairing of iterables[i] with offsets[i], under an existential over positions.
```