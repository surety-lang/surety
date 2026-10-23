```surety
// draft
// drafted
type Value = { id: Id, is_none: Bool, iterable: Bool, type_names: Text[], items: Id[] }

always_iterable(obj: Value, base_type: Option<Text[]>) -> Id[]
	note "Iterate over obj's items, giving nothing for None and obj alone when obj is not iterable or is an instance of base_type; the Python default for base_type is str and bytes, passed here as Some of those two names."
	# If obj is None, the output is empty, whatever base_type is.

	# If obj is not None, base_type is given, and some name in obj's type_names is in base_type, the output is obj's id alone.

	# If obj is not None and is not an instance of base_type, or base_type is None, and obj is iterable, the output has the same elements as obj's items, in the same order.

	# If obj is not None and is not an instance of base_type, or base_type is None, and obj is not iterable, the output is obj's id alone.

	// example wanted: the tuple (1, 2, 3) with the default base_type; obj is iterable and neither str nor bytes, so the output is its items in order
	// example wanted: the int 1 with the default base_type; obj is not iterable, so it is wrapped alone
	// example wanted: None with the default base_type; the None case decides it before any other check
	// example wanted: None with a base_type naming NoneType; the None check comes first, so the base_type match does not wrap it
	// example wanted: an empty list, not None, with the default base_type; it is iterable with no items, which is distinct from the None case
	// example wanted: the text foo with the default base_type; str is in base_type, so it is treated as a unit
	// example wanted: the text foo with base_type None; no special handling, so it is iterated character by character
	// example wanted: the dict with key a, once with the default base_type and once with base_type naming dict; the first iterates its keys, the second treats it as a unit
```