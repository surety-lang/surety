```surety
// draft
// drafted
type Item = { value: Text, key: Text }

unique_everseen(items: Item[]) -> Item[]
	note "keep the first item for each key, in input order; each item carries the key computed for it, which is the value itself when no key function is given"
	# The output keeps the input's order.

	# No two output items share a key.

	# Every input item's key is the key of some output item.

	# Every input item left out of the output comes after the output item with the same key in the input.

	// example wanted: the empty list
	// example wanted: 'AAAABBBCCDAABBB' as items whose key equals the value; decided by keeping the first item of each run and dropping later repeats such as the second A run
	// example wanted: 'ABBCcAD' with the casefolded letter as key; decided by 'c' sharing key 'c' with the earlier 'C', so the earlier one is kept
	// example wanted: two identical items (same value, same key); decided by keeping the first copy
	// example wanted: two items with different values and the same key, the later one listed first in the expected output's candidates; decided by input position, not by value
```