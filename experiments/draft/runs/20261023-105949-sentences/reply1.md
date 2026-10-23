```surety
// draft
// drafted
type Entry = { key: Text, value: Int }
// drafted
type Kwargs = { entries: Entry[unique key] }
// drafted
type CallError = { code: Text, display: Text }
// drafted
type_error: CallError = { code: "TypeError", display: "func's signature doesn't match the mapping contained in iterable" }

doublestarmap(iterable: Kwargs[]) -> Result<Int[], CallError>
	note "apply func to every item of iterable by unpacking the item's entries as keyword arguments; func and whether it accepts an item's keys are sibling obligations"
	# The output is Ok exactly when func accepts every item's keys as its keyword arguments (a sibling obligation decides acceptance).

	# Every Err output is the type error.

	# An Ok output has as many elements as the iterable has items.

	# Each element of an Ok output is func's result on the item at the same position in the iterable, so the output keeps the iterable's order.

	// example wanted: the empty iterable: no item is unpacked, so func's signature never comes into it
	// example wanted: the docstring's case, items {a: 1, b: 2} and {a: 40, b: 60} with func adding a and b: each result comes from its own item, in order
	// example wanted: one item with an extra key c beside a and b, for func taking a and b: a key func does not name is a mismatch
	// example wanted: one item with only key a, for func taking a and b: a missing keyword is a mismatch
	// example wanted: a matching item followed by a mismatching one: the Python generator yields the first result before raising, and this obligation reports only the error
	// example wanted: an item with no entries, for func taking no parameters: an empty mapping matches an empty signature
```