```surety
# The result is Ok exactly when the element is drawn from the iterable: every item of the element is an occurrence of an item of the iterable, and no occurrence is used twice.
	element drawn_from iterable <-> exactly_one Ok(_) in out
# An error result is the not-a-permutation error.
	all Err(e) in out: e == not_a_permutation_error
# An Ok index is at least 0.
	all Ok(i) in out: i >= 0
# An Ok index is 0 exactly when the element is the iterable's first len(element) items, in order.
	all Ok(i) in out: i == 0 <-> element == iterable[0 .. len(element)]
# If the element's first item is the iterable's first item, the result is the result for the element without its first item over the iterable without its first item.
	len(element) > 0 and element[0 .. 1] == iterable[0 .. 1] -> permutation_index(element[1 .. len(element)], iterable[1 .. len(iterable)]) == out
```