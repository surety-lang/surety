```surety
// draft
// drafted
type NthError = { code: Text, display: Text }
// drafted
empty_error: NthError = { code: "e-empty-no-default", display: "last() was called on an empty iterable, and no default value was provided." }
// drafted
negative_n_error: NthError = { code: "e-negative-n", display: "Indices for islice() must be None or an integer: 0 <= x <= sys.maxsize." }

nth_or_last(iterable: Int[], n: Int, default: Option<Int>) -> Result<Int, NthError>
	note "return the item at position n, or the last item if the iterable is shorter than that, or the default if it is empty"
	# If n is at least 0 and n is less than the number of items, the output is Ok of the item at position n.

	# If n is at least 0, the iterable is not empty, and n is at least the number of items, the output is Ok of the last item.

	# If n is at least 0, the iterable is empty, and a default is given, the output is Ok of the default.

	# If n is at least 0, the iterable is empty, and no default is given, the output is the empty error.

	# If n is less than 0, the output is the negative-n error, whether or not the iterable is empty and whether or not a default is given.

	// example wanted: [0, 1, 2, 3] with n 2 and no default: n falls inside the list, so position 2 decides it
	// example wanted: [0, 1] with n 2 and no default: n is past the end, so the last item decides it
	// example wanted: [0, 1] with n 1: n is the last position, where the nth item and the last item are the same item
	// example wanted: [5, 5, 7] with n 1 and a default given: a non-empty iterable means the default is never used
	// example wanted: [] with n 0 and a default given: the empty iterable with a default
	// example wanted: [] with n 0 and no default: the empty iterable without a default, the empty error
	// example wanted: [0, 1, 2] with n -1: a negative n is an error, not Python's count-from-the-end indexing
	// example wanted: [] with n -1 and a default given: both conditions hold, and the negative-n error takes precedence because islice rejects n before last() runs
```