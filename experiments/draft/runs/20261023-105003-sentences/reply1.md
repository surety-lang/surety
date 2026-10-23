```surety
// draft
// drafted
type PermutationError = { kind: Text }
// drafted
negative_r_error: PermutationError = { kind: "ValueError" }
// drafted
index_error: PermutationError = { kind: "IndexError" }

nth_permutation(iterable: Text[], r: Int, index: Int) -> Result<Text[], PermutationError>
	note "return the arrangement of r elements of the iterable at sort position index, equal to list(permutations(iterable, r))[index], without computing the earlier arrangements"
	# The output is Ok exactly when r is at least 0 and the index is at least minus the number of arrangements and less than it, where the number of arrangements is the length of permutations(iterable, r), the sibling obligation that lists every ordered selection of r elements in lexicographic order of their positions in the iterable.

	# The output is negative_r_error exactly when r is negative.

	# The output is index_error exactly when r is at least 0 and the index is either less than minus the number of arrangements or at least the number of arrangements.

	# An Ok output has exactly r elements.

	# An Ok output is drawn from the iterable: every element is an occurrence of an iterable element, and no occurrence is used twice.

	# When the index is at least 0, an Ok output is the arrangement at position index of permutations(iterable, r).

	# When the index is negative, the output is the output of nth_permutation on the same iterable and r with the index plus the number of arrangements.

	// example wanted: iterable "ghijk" as five letters, r 2, index 5: the docstring's own case, the sixth arrangement in position order
	// example wanted: iterable "ghijk", r 2, index -1: a negative index counts from the end, so it is the last of the 20 arrangements
	// example wanted: iterable "ghijk", r 2, index 20: the first index equal to the number of arrangements, the upper boundary
	// example wanted: iterable "ghijk", r 2, index -20 and index -21: the lower boundary crossed from both sides
	// example wanted: iterable "ghijk", r 0, index 0: there is exactly one arrangement of zero elements, the empty one
	// example wanted: empty iterable, r 0, index 0: the empty iterable still has its one empty arrangement
	// example wanted: iterable "ghijk", r 5, index 0: r equal to the length, the full arrangements
	// example wanted: iterable "ghijk", r -1, any index: r negative decides the error before the index is looked at
	// example wanted: iterable "aab", r 2: arrangements are by position, not by value, so equal letters give repeated arrangements and each counts toward the index
	// example wanted: iterable "ghijk", r 6, index 0: the docstring's equivalence gives no arrangements, so the index is invalid, while the 11.2.1 implementation raises ValueError for r past the length; the author decides which
```