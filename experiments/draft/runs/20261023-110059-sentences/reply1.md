```surety
// draft
// drafted
type ExtractError = { code: Text, display: Text }
// drafted
index_beyond_error: ExtractError = { code: "IndexError", display: "An index lies beyond the iterable" }
// drafted
negative_index_error: ExtractError = { code: "ValueError", display: "An index is negative" }
// drafted
decreasing_index_error: ExtractError = { code: "ValueError", display: "An index is less than the one before it while monotonic is true" }

extract(iterable: Text[], indices: Int[], monotonic: Bool) -> Result<Text[], ExtractError>
	note "yield the iterable's values at the given indices, in the order the indices are given, with repeats allowed"
	# The output is Ok exactly when every index is at least 0 and less than the iterable's length, and, when monotonic is true, no index is less than the index before it.

	# An Ok output has as many elements as there are indices.

	# For every position in indices, the Ok output's element at that position is the iterable's element at the index found at that position of indices.

	# When monotonic is false and some index is less than 0, the error is the negative-index error, whatever else is wrong with the indices.

	# When monotonic is false, the error is the beyond-the-iterable error exactly when every index is at least 0 and some index is at least the iterable's length.

	# When monotonic is true, the error reported is the one for the first index in indices that is negative, at least the iterable's length, or less than the index before it.

	// example wanted: the docstring's alphabet with indices [7, 4, 11, 11, 14], monotonic false: the indices are out of order and repeated, and neither is an error when monotonic is false
	// example wanted: empty indices over a non-empty iterable, and empty indices over an empty iterable: no index means nothing is checked
	// example wanted: an empty iterable with indices [0]: index 0 is already beyond an empty iterable
	// example wanted: a three-element iterable with indices [2], then with indices [3]: the exact threshold, last position against one past the end
	// example wanted: indices [-1] with monotonic false: a negative index never counts from the end
	// example wanted: monotonic true with indices [1, 1, 2]: equal neighbours are non-decreasing, so no decreasing error
	// example wanted: monotonic true with indices [2, 1]: a decrease that is reported only because monotonic is true
	// example wanted: monotonic false with indices [5, -1] over a three-element iterable: the beyond index comes first but the negative check decides
	// example wanted: monotonic true with indices [5, 2] over a three-element iterable: beyond the iterable and then a decrease, and the earlier index decides
	// example wanted: monotonic true with indices [1, -1]: the second index is both negative and a decrease, so the implementation's check order decides which ValueError is reported
```