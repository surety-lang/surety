```surety
# The output has as many columns as the first row of the iterable has cells, and no columns when the iterable has no rows.
	len(out) == len(flatten(iterable[0 .. 1].cells))
# For every column position i and every position j in column i, the j-th cell of column i is the i-th cell of the j-th row of the iterable.
	reaching for: a quantifier over integer positions, so I could write "for every i from 0 up to len(out), and every j from 0 up to the length of column i, flatten(out[i .. i + 1].cells)[j .. j + 1] == flatten(iterable[j .. j + 1].cells)[i .. i + 1]". The slices can pick out single cells, but nothing binds i or j, and a binder over out gives me an occurrence, never its position as an Int
# Every row of the iterable before position len(column i) has more than i cells.
	reaching for: a quantifier over the integer position i of each column, so I could write "for every i from 0 up to len(out), every r in iterable[0 .. len(flatten(out[i .. i + 1].cells))] has len(r.cells) > i". The slice and the length comparison are expressible, but nothing binds the column's position i as an Int
# If column i has fewer cells than the iterable has rows, the row at position len(column i) has at most i cells.
	reaching for: a quantifier over the integer position i of each column, so I could write "for every i from 0 up to len(out), with k = len(flatten(out[i .. i + 1].cells)): k < len(iterable) -> every r in iterable[k .. k + 1] has len(r.cells) <= i". The slice picks out the row at position k, but nothing binds the column's position i as an Int
```