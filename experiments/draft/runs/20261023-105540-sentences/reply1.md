```surety
// draft
// drafted
type Item = { value: Int, key: Int }
// drafted
type MinMax = { smallest: Item, largest: Item }
// drafted
type MinmaxError = { code: Text, display: Text }
// drafted
empty_error: MinmaxError = { code: "ValueError", display: "`minmax()` argument is an empty iterable. Provide a `default` value to suppress this error." }

minmax(items: Item[], default: Option<MinMax>) -> Result<MinMax, MinmaxError>
	note "return the item with the smallest key and the item with the largest key, where an item's key is the key function's result (its value when no key is given), the default when there are no items, and an error when there is no default either; among tied keys, which item is returned is deliberately not decided"
	# The output is Ok exactly when the items are not empty or a default is given.

	# Every error output is the empty error.

	# If the items are empty and a default is given, the output is Ok carrying the default unchanged.

	# If the items are not empty, the smallest and the largest in an Ok output are both items from the input.

	# No input item has a key less than the smallest's key.

	# No input item has a key greater than the largest's key.

	// example wanted: items [3, 1, 5] with keys equal to values and no default: the smallest is the lowest key and the largest the highest
	// example wanted: one item and no default: the smallest and the largest are that same item
	// example wanted: items 5 and 30 with keys that rank 30 below 5 (as str does): the key, not the value, decides which is smallest
	// example wanted: empty items with default (0, 0): the default is returned
	// example wanted: empty items with no default: the empty error is reported
	// example wanted: non-empty items with a default given: the default is ignored and the items decide
	// example wanted: two items with different values and the same key: either may be the smallest or the largest, the tie the docstring leaves open
```