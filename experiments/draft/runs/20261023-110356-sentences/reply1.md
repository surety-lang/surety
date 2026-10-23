```surety
// draft

ncycles(iterable: Text[], n: Int) -> Text[]
	note "return the iterable's elements, in order, repeated n times end to end"
	# If n is at least 0, the output has n times as many elements as the iterable.

	# If n is negative, the output is empty.

	# If n is at least 1, the output's first elements, as many as the iterable has, are the iterable's elements in the iterable's order.

	# The output from position len(iterable) to its end is the same as the output from its start up to but not including the last len(iterable) positions, so each output element repeats the one an iterable's length before it.

	// example wanted: the docstring case, iterable ["a", "b"] with n 3: the iterable written out three times in a row
	// example wanted: iterable ["a", "b"] with n 0: zero repetitions, the n-at-least-0 boundary
	// example wanted: iterable ["a", "b"] with n -1: a negative n, decided by the negative-n sentence rather than the length sentence
	// example wanted: iterable ["a", "b"] with n 1: a single repetition, the n-at-least-1 boundary
	// example wanted: empty iterable with n 3: nothing to repeat however large n is
	// example wanted: iterable ["a", "a", "b"] with n 2: duplicates within the iterable are kept on every repetition
```