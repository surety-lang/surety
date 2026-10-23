```surety
// draft

substrings(iterable: Int[]) -> Int[][]
	note "Every contiguous non-empty run of the input, shortest first, and within one length in order of where it starts."
	# Every output element is the slice of the input from some position up to but not including a later position.

	# Every slice of the input with at least one element is in the output.

	# The output has one element for each pair of start and end positions: twice the output's length is the input's length times one more than the input's length.

	# No output element is longer than an output element that comes after it.

	# The output elements of any one length are the input's slices of that length, in order of their starting position.

	// example wanted: the empty input, where there are no positions to start from
	// example wanted: a one-element input, where the only slice is the whole input
	// example wanted: [0, 1, 2] from the docstring, where the order by length and then by start position decides the layout
	// example wanted: [0, 0], where two slices are equal in value but start at different positions, so the output keeps both rather than deduplicating
```