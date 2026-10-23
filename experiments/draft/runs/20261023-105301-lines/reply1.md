```surety
# The result is Ok exactly when the element is a combination with replacement of the iterable: its items can be matched, front to back, to positions of the iterable that never go back.
	reaching for: Ok exactly when there is a list of positions p, one for each item of element, with p[k] <= p[k + 1] for every k and iterable at position p[k] equal to element's item k. That needs a quantifier over sequences of positions, and the fragment's quantifiers range only over the occurrences of a given list. A pairwise check over consecutive items is weaker: it accepts ['b','a','b'] in ['a','b','a'].
# An error result is always not_a_combination_err.
	all Err(e) in out: e == not_a_combination_err
# An Ok index is the position of the first combination equal to the element among the iterable's combinations with replacement of the element's length, as the sibling obligation combinations_with_replacement lists them.
	all Ok(i) in out: combinations_with_replacement(iterable, len(element))[i .. i + 1] == [element] and element not in combinations_with_replacement(iterable, len(element))[0 .. i]
# An Ok index is 0 exactly when every item of the element is the iterable's first item.
	all Ok(i) in out: i == 0 <-> all x in element: x in iterable[0 .. 1]
# If the element's first item is the iterable's first item, the result is the result for the element without its first item.
	len(element) > 0 and element[0 .. 1] == iterable[0 .. 1] -> out == combination_with_replacement_index(element[1 .. len(element)], iterable)
# If the element is not empty, no item of it is the iterable's first item, and the result is Ok, then the index is greater than the index for the same element in the iterable without its first item.
	all Ok(i) in out where len(element) > 0: all Ok(j) in combination_with_replacement_index(element, iterable[1 .. len(iterable)]): i > j or any x in element: x in iterable[0 .. 1]
```