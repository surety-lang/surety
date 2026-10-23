```surety
// draft

takewhile_inclusive(items: Int[], limit: Int) -> Int[]
	note "take items while each is below the limit, and also keep the first item that is not (the predicate is drawn here as item < limit)"
	# The output is the items from position 0 up to but not including position len(out), so it is a prefix of the input in the input's order.

	# Every output element before the last one is less than the limit.

	# If every input item is less than the limit, the output is the whole input.

	# If some input item is at least the limit, the last output element is at least the limit.

	// example wanted: the docstring case, items [1, 4, 6, 4, 1] with limit 5: 6 is the first item not below 5, and the 4 and 1 after it are not taken
	// example wanted: the empty list: there is no item, so there is no first failing item
	// example wanted: the first item already at or above the limit: the output holds only that one item
	// example wanted: an item exactly equal to the limit: equal is not less than, so the predicate fails there and that item ends the output
	// example wanted: every item below the limit: nothing fails, so nothing extra is yielded
	// example wanted: only the last item fails: the inclusive item is the final input item, and no item comes after it
```