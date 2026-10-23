```surety
// draft
// drafted
type Source = { items: Int[] }

interleave_longest(iterables: Source[]) -> Int[]
	note "take one item from each iterable in turn, skipping any that are exhausted, until all are exhausted"
	# The output is the items of all the iterables taken together: every item of every iterable appears in it exactly once, and nothing else does.

	# An item at an earlier position in its iterable comes before every item at a later position in any iterable, its own included.

	# Of two items at the same position in their iterables, the one from the earlier iterable comes before the one from the later iterable.

	// example wanted: no iterables at all: nothing to take from
	// example wanted: several iterables, every one empty: all are exhausted from the start
	// example wanted: a single iterable: each turn has only one iterable to take from
	// example wanted: the docstring case [1, 2, 3], [4, 5], [6, 7, 8]: the middle one is exhausted first, and the later turns skip it
	// example wanted: an empty iterable between two non-empty ones: it is skipped from the first turn
	// example wanted: the same value in two iterables, e.g. [1, 2] and [2, 1]: occurrences, not values, decide each position
```