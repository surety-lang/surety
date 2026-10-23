```surety
// draft
// drafted
type TooLongError = { kind: Text }
// drafted
value_error: TooLongError = { kind: "ValueError" }

only(iterable: Int[], default: Option<Int>, too_long: Option<TooLongError>) -> Result<Option<Int>, TooLongError>
	note "return the iterable's one item, the default when it has none, and the too_long error when it has more than one"
	# The output is Ok exactly when the iterable has at most one item.

	# Every item of the iterable is the Ok value.

	# An Ok value that is not an item of the iterable is the default.

	# An Err output is the too_long error the caller gave, or the ValueError when the caller gave none.

	// example wanted: an empty iterable with a default given; the empty case decides it, so the default is returned
	// example wanted: an empty iterable with no default given; the default's own default (None) decides it
	// example wanted: an iterable of one item with no default given; the single item decides it
	// example wanted: an iterable of one item with a default given that differs from the item; this decides that the item, not the default, is returned
	// example wanted: an iterable of exactly two items with no too_long given; two items cross the threshold of one, so the ValueError is reported
	// example wanted: an iterable of two items with too_long given as kind "TypeError"; the caller's error, not the ValueError, is reported
	// example wanted: an iterable of three or more items; this decides that the error does not depend on how far past one the length goes
```