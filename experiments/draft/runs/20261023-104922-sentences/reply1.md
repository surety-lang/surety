```surety
// draft
// drafted
type Window = { beginning: Int[], middle: Int[], end: Int[] }
// drafted
type WindowError = { code: Text, display: Text }
// drafted
negative_n_err: WindowError = { code: "e-n-negative", display: "n must be at least 0" }
// drafted
n_too_large_err: WindowError = { code: "e-n-too-large", display: "n must be at most the length of iterable" }

windowed_complete(iterable: Int[], n: Int) -> Result<Window[], WindowError>
	note "slide a window of n items across the iterable, yielding at each position the items before it, the window, and the items after it"
	# The output is Ok exactly when n is at least 0 and at most the length of the iterable.

	# The output is the negative-n error exactly when n is less than 0.

	# The output is the n-too-large error exactly when n is more than the length of the iterable.

	# An Ok output has one more window than the length of the iterable minus n.

	# In an Ok output, every window's middle has exactly n items.

	# In an Ok output, every window's beginning is the iterable's items from position 0 up to but not including the length of that beginning.

	# In an Ok output, every window's middle is the iterable's items from the position where its beginning ends up to but not including that position plus n.

	# In an Ok output, every window's end is the iterable's items from the position where its middle ends to the end of the iterable.

	# In an Ok output, of any two windows, the one that comes first has the shorter beginning.

	// example wanted: the docstring's case, iterable [0, 1, 2, 3, 4, 5, 6] with n 3: five windows, the middle sliding one position each time
	// example wanted: n 0 on a non-empty iterable: every middle is empty, and the beginning takes every length from 0 to the full iterable
	// example wanted: n equal to the length of the iterable: the boundary where exactly one window remains, with an empty beginning and end
	// example wanted: the empty iterable with n 0: the only n allowed, decided by the zero-length boundary
	// example wanted: n of -1: one below the lower bound, decided by the negative-n error
	// example wanted: n one more than the length of the iterable: one past the upper bound, decided by the n-too-large error
	// example wanted: an iterable with repeated values such as [1, 1, 2] with n 2: windows are told apart by position, not value
```