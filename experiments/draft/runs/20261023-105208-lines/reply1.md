```surety
# The result is an index exactly when the element is a subsequence of the iterable, its items appearing in the iterable in the same order.
	element same_order_as iterable <-> exactly_one Ok(_) in out
# An error result is always the not-a-combination error.
	all Err(e) in out: e == not_a_combination_err
# The index is never negative.
	all Ok(i) in out: i >= 0
# The index is 0 exactly when the element is the iterable's first items, as many as the element has.
	all Ok(i) in out: i == 0 <-> element == iterable[0 .. len(element)]
# If the element and the iterable start with the same item, the index is the index of the element without its first item within the iterable without its first item.
	len(element) > 0 and element[0 .. 1] == iterable[0 .. 1] -> all Ok(i) in out: exactly_one Ok(j) in combination_index(element[1 .. len(element)], iterable[1 .. len(iterable)]): i == j
# If the element is a combination of the iterable and does not start with the iterable's first item, the index is greater than the element's index within the iterable without its first item.
	len(element) > 0 and element same_order_as iterable and element[0 .. 1] != iterable[0 .. 1] -> all Ok(i) in out: exactly_one Ok(j) in combination_index(element, iterable[1 .. len(iterable)]): i > j
```