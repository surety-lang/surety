```surety
# The output is Ok exactly when every index is at least 0 and less than the iterable's length, and, when monotonic is true, no index is less than the index before it.
	reaching for: one line joining two quantified operands with <->, "(all i in indices: i >= 0 and i < len(iterable) and all j in indices where monotonic and i before j in indices: i <= j) <-> exactly_one Ok(_) in out". Comparing each index with every earlier one says the same as comparing it with the one just before it. A quantifier has to be the last operand on a line, so this needs the condition as an obligation-level abbreviation, like apply_diff's `checks`, and that abbreviation is the author's to name.
# An Ok output has as many elements as there are indices.
	all Ok(res) in out: len(res) == len(indices)
# For every position in indices, the Ok output's element at that position is the iterable's element at the index found at that position of indices.
	reaching for: a binder over the positions of a list, so the line could say "for every position k of indices, res[k .. k + 1] == iterable[i .. i + 1], where i is the element of indices at position k". Binders range over occurrences but never expose their position as an Int, and nothing pairs the k-th element of one list with the k-th element of another.
# When monotonic is false and some index is less than 0, the error is the negative-index error, whatever else is wrong with the indices.
	all i in indices where not monotonic and i < 0: all Err(e) in out: e == negative_index_error
# When monotonic is false, the error is the beyond-the-iterable error exactly when every index is at least 0 and some index is at least the iterable's length.
	all Err(e) in out where not monotonic: e == index_beyond_error <-> any j in indices: j >= len(iterable) and all i in indices: i >= 0
# When monotonic is true, the error reported is the one for the first index in indices that is negative, at least the iterable's length, or less than the index before it.
	all Err(e) in out where monotonic: any f in indices: any p in indices: ((f < 0 and e == negative_index_error) or (f >= len(iterable) and e == index_beyond_error) or (p before f in indices and f < p and e == decreasing_index_error)) and all g in indices where g before f in indices: 0 <= g and g < len(iterable) and all q in indices where q before g in indices: q <= g
```