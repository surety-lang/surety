```surety
// draft

// the iterables are finite lists of Int: the fragment has no generic element type and no infinite list
interleave(iterables: Int[][]) -> Int[]
	note "yield one element from each iterable in turn, round after round, until the shortest iterable is exhausted"
	# The output has as many elements as the number of iterables times the length of the shortest iterable.

	# For every position k below the length of the shortest iterable, the output's elements from position k times the number of iterables up to but not including k plus one times the number of iterables are the element at position k of each iterable, in the order the iterables are given.

	// example wanted: no iterables at all; the length sentence has no shortest iterable to measure, and the output is decided by what zero rounds mean
	// example wanted: [[1, 2, 3], [4, 5], [6, 7, 8]], the docstring case; the second iterable is the shortest and decides how many rounds there are
	// example wanted: [[1, 2], [], [3]]; one iterable is empty, so there are zero rounds
	// example wanted: a single iterable [1, 2, 3]; each round has one element, so the rounds are that iterable's positions
	// example wanted: [[1, 2], [3, 4]], equal lengths; every element of every iterable is reached
	// example wanted: [[5, 5], [5, 9]]; equal values in different iterables, so only position decides which iterable each output element came from
```