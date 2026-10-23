```surety
# The output is Ok exactly when r is not negative and the index is a valid position among the combinations with replacement of length r, as a sibling obligation counting them decides.
	(r >= 0 and valid_combination_with_replacement_index(iterable, r, index)) <-> exactly_one Ok(_) in out
# The output is the negative-r error exactly when r is negative.
	r < 0 <-> out == Err(negative_r_error)
# The output is the index error exactly when r is not negative and the index is not a valid position among the combinations.
	(r >= 0 and not valid_combination_with_replacement_index(iterable, r, index)) <-> out == Err(index_error)
# An Ok combination has exactly r elements.
	all Ok(c) in out: len(c) == r
# Every element of an Ok combination is an element of the iterable.
	all Ok(c) in out: all x in c: x in iterable
# The elements of an Ok combination keep the iterable's order: no element of the combination comes before an earlier element of the combination in the iterable.
	all Ok(c) in out: all a in c: all b in c where a before b in c: not (b before a in iterable)
```