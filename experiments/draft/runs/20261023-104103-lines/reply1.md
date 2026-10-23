```surety
# If obj is None, the output is empty, whatever base_type is.
	obj.is_none -> len(out) == 0
# If obj is not None, base_type is given, and some name in obj's type_names is in base_type, the output is obj's id alone.
	not obj.is_none -> len(out) == 1 and obj.id in out or none Some(bt) in base_type: any n in obj.type_names: n in bt
# If obj is not None and is not an instance of base_type, or base_type is None, and obj is iterable, the output has the same elements as obj's items, in the same order.
	not obj.is_none and obj.iterable -> out == obj.items or any Some(bt) in base_type: any n in obj.type_names: n in bt
# If obj is not None and is not an instance of base_type, or base_type is None, and obj is not iterable, the output is obj's id alone.
	not obj.is_none and not obj.iterable -> len(out) == 1 and obj.id in out or any Some(bt) in base_type: any n in obj.type_names: n in bt
```