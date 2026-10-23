```surety
// draft
// drafted
type Permutation = { items: Int[] }

distinct_permutations(iterable: Int[], r: Option<Int>) -> Permutation[]
	note "yield each distinct arrangement of r of the input's elements once, r defaulting to the input length, in sorted order"
	// elements are drafted as Int, so the input is always sortable; the docstring's equal-but-not-identical case (1 and True) has no counterpart here
	# Every permutation's items are drawn from the input: each item is an input element, and no input occurrence is used twice within one permutation.

	# When r is given, every permutation has exactly r items.

	# When r is not given, every permutation has as many items as the input.

	# No two permutations in the output have the same items.

	# The permutations are in ascending lexicographic order of their items.

	# When r is given and is at least 0 and at most the input length, the first r input elements, in input order, are the items of some permutation in the output.

	# When r is not given, the input itself, in input order, is the items of some permutation in the output.

	# When r is not given, running on the items of any output permutation, with no r, gives the same output.

	// example wanted: iterable [1, 0, 1] with no r: the docstring's case, and the count is the multinomial 3! / (2! * 1!), which no sentence states
	// example wanted: iterable [1, 0, 1] with r 2: the duplicate 1 may fill both positions, but only once per arrangement
	// example wanted: iterable [0, 1, 2] with r 2: distinct elements, so every ordered pair of different positions counts
	// example wanted: empty iterable with no r: r defaults to the input length, 0, which is a zero-length request
	// example wanted: iterable [1, 0, 1] with r 0: a zero-length request on a non-empty input
	// example wanted: iterable [1, 0, 1] with r 4: r is past the input length, so no permutation can be drawn
	// example wanted: iterable [1, 0, 1] with r -1: a negative r, which no permutation can have as its length
	// example wanted: iterable [2, 2, 2] with no r: every element is equal, so every arrangement has the same items
```