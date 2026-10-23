```surety
# The output is Ok exactly when r is at least 0 and the index is at least minus the number of arrangements and less than it, where the number of arrangements is the length of permutations(iterable, r), the sibling obligation that lists every ordered selection of r elements in lexicographic order of their positions in the iterable.
	(r >= 0 and index + len(permutations(iterable, r)) >= 0 and index < len(permutations(iterable, r))) <-> exactly_one Ok(_) in out
# The output is negative_r_error exactly when r is negative.
	out == Err(negative_r_error) <-> r < 0
# The output is index_error exactly when r is at least 0 and the index is either less than minus the number of arrangements or at least the number of arrangements.
	out == Err(index_error) <-> (r >= 0 and (index + len(permutations(iterable, r)) < 0 or index >= len(permutations(iterable, r))))
# An Ok output has exactly r elements.
	all Ok(p) in out: len(p) == r
# An Ok output is drawn from the iterable: every element is an occurrence of an iterable element, and no occurrence is used twice.
	all Ok(p) in out: p drawn_from iterable
# When the index is at least 0, an Ok output is the arrangement at position index of permutations(iterable, r).
	index >= 0 -> all Ok(p) in out: p in permutations(iterable, r)[index .. index + 1]
# When the index is negative, the output is the output of nth_permutation on the same iterable and r with the index plus the number of arrangements.
	index < 0 -> out == nth_permutation(iterable, r, index + len(permutations(iterable, r)))
```