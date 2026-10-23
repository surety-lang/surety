```surety
// draft
// drafted
type Element = { value: Text, key: Text }

all_unique(elements: Element[]) -> Bool
	note "true when no two elements compare equal; each element's key is the key function applied to its value, or the value itself when no key function is given"
	# The output is true exactly when no two elements share a key.

	// example wanted: the empty list; there are no two elements, so none can share a key
	// example wanted: one element; there is no second element to compare it with
	// example wanted: 'ABCB' with no key function (each key is its value); the second and fourth elements share the key "B"
	// example wanted: 'ABCb' with no key function; "B" and "b" are different keys, so no two elements share a key
	// example wanted: 'ABCb' with str.lower as the key function (keys "a", "b", "c", "b"); "B" and "b" differ in value but share the key "b"
	// example wanted: 'ABAC' with no key function; the shared key is at positions 0 and 2, which are not adjacent
```