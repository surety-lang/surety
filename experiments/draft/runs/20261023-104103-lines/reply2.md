```surety
# If obj is None, the output is empty, whatever base_type is.
	obj.is_none -> len(out) == 0

# If obj is not None, base_type is given, and some name in obj's type_names is in base_type, the output is obj's id alone.
	reaching for: a binder that opens an Option the way Ok(x) opens a Result (`Some(bt)` is not accepted in the quantifier head), so I could write: all Some(bt) in base_type: all n in obj.type_names where not obj.is_none and n in bt: len(out) == 1 and obj.id in out

# If obj is not None and is not an instance of base_type, or base_type is None, and obj is iterable, the output has the same elements as obj's items, in the same order.
	reaching for: a way to name the list inside base_type when it is Some, and to use "no name in obj.type_names is in that list" as a condition that is not the last operand on its line, so I could write: (not obj.is_none and obj.iterable and (base_type == None or no name in obj.type_names is in the given base_type list)) -> (out same_order_as obj.items and obj.items same_order_as out)

# If obj is not None and is not an instance of base_type, or base_type is None, and obj is not iterable, the output is obj's id alone.
	reaching for: the same opened Option and non-final quantifier, so I could write: (not obj.is_none and not obj.iterable and (base_type == None or no name in obj.type_names is in the given base_type list)) -> (len(out) == 1 and obj.id in out)
```