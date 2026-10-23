```surety
// draft
// drafted
type CombinationError = { code: Text, display: Text }
// drafted
not_a_combination_err: CombinationError = { code: "ValueError", display: "element is not a combination of iterable" }

combination_index(element: Text[], iterable: Text[]) -> Result<Int, CombinationError>
	note "the position of element in the list of combinations of iterable of its length, in lexicographic order of positions; the first such position when the values repeat"
	# The result is an index exactly when the element is a subsequence of the iterable, its items appearing in the iterable in the same order.

	# An error result is always the not-a-combination error.

	# The index is never negative.

	# The index is 0 exactly when the element is the iterable's first items, as many as the element has.

	# If the element and the iterable start with the same item, the index is the index of the element without its first item within the iterable without its first item.

	# If the element is a combination of the iterable and does not start with the iterable's first item, the index is greater than the element's index within the iterable without its first item.

	// example wanted: element "adf" in iterable "abcdefg", the docstring's case: count the length-3 combinations that come before a, d, f in position order
	// example wanted: an empty element in a non-empty iterable: the single empty combination is the first one
	// example wanted: an empty element in an empty iterable: the empty combination still exists
	// example wanted: element "da" in iterable "abcd": the items are present but out of order, so it is not a combination
	// example wanted: element "abcde" in iterable "abc": the element is longer than the iterable
	// example wanted: element "a" in iterable "aa": two combinations carry the same value, and the first position is reported
	// example wanted: element equal to the whole iterable: the only combination of that length
	// example wanted: element "fg" in iterable "abcdefg": the last combination, which every other combination of length 2 comes before
```