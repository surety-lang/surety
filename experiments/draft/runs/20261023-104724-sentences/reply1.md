```surety
// draft
// drafted
type Sequence = { items: Text[] }

iequals(iterables: Sequence[]) -> Bool
	note "true when all the given sequences hold the same items in the same order, compared item by item rather than by the sequences' own equality"
	# If the output is true, every two given sequences have the same items in the same order.

	# If every two given sequences have the same items in the same order, the output is true.

	// example wanted: no sequences given at all; the "every two" claims hold vacuously and decide it
	// example wanted: a single sequence ["a", "b", "c"]; every pair is that sequence with itself
	// example wanted: two empty sequences; equal with no items to compare
	// example wanted: ["a", "b", "c"] and ["a", "b"], one a prefix of the other; the length difference decides it
	// example wanted: ["a", "b", "c"] and ["a", "c", "b"], the same items in a different order; order decides it
	// example wanted: four copies of ["a", "b", "c"] (the docstring's string, list, tuple and iterator flattened to one item type); all pairs equal
	// example wanted: ["a", "b"], ["a", "b"], ["a", "x"], with only the last sequence differing; one bad pair decides it
```