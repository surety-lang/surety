```surety
// draft
// drafted
type Item = { value: Text, pred: Bool }

strip(iterable: Item[]) -> Item[]
	note "yield the items of iterable with every leading and trailing item for which pred is true removed, each item carrying pred's verdict on it"
	# The output is drawn from the input.

	# The output keeps the input's order.

	# Every output item whose pred is true comes after some output item whose pred is false and before some output item whose pred is false.

	# Every input item left out of the output comes before every input item whose pred is false, or after every one of them.

	// example wanted: the empty iterable; no item at all decides it
	// example wanted: every item has pred true; nothing lies between two pred-false items, so whether anything survives is decided by the interior rule
	// example wanted: no item has pred true; nothing is at either end to strip
	// example wanted: the docstring case (None, False, None, 1, 2, None, 3, False, None), pred true for None and False; the inner None sits between 2 and 3
	// example wanted: a single item with pred false surrounded by pred-true items; first and last pred-false item are the same item
	// example wanted: the same pred-true value at the start, in the middle and at the end, e.g. [N, 1, N, 2, N]; which copies are kept is decided by position, not value
	// example wanted: pred-true items only at the start, none at the end; the strip is one-sided
```