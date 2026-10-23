```surety
// draft
// drafted
type LastError = { code: Text, display: Text }
// drafted
empty_error: LastError = { code: "ValueError", display: "iterable is empty and no default was provided" }

last(iterable: Int[], default: Option<Int>) -> Result<Int, LastError>
	note "return the last item of the iterable, or the default when the iterable is empty, and fail when it is empty and no default was supplied"
	# The output is an error exactly when the iterable is empty and no default is supplied.

	# An error output is the empty-iterable error.

	# If the iterable is not empty, the output is the item at its last position, whether or not a default is supplied.

	# If the iterable is empty and a default is supplied, the output is the default.

	// example wanted: iterable [0, 1, 2, 3] with no default: the last position decides
	// example wanted: iterable [0, 1, 2, 3] with a default supplied: the default is ignored when items exist
	// example wanted: empty iterable with a default supplied: the default decides
	// example wanted: empty iterable with no default: the empty-iterable error decides
	// example wanted: a single-item iterable with no default: the only item is also the last
	// example wanted: iterable [7, 3, 7] with no default: the last position decides even though its value also occurs earlier
	// example wanted: a non-empty iterable whose last item equals the supplied default: either reading gives the same value, so this one does not tell them apart
```