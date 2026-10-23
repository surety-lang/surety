```surety
# Every key of every input mapping has exactly one output record with that key.
	all m in field_to_map: all e in m.entries: exactly_one r in out: r.key == e.key
# Every output record's key is a key of some input mapping.
	all r in out: any m in field_to_map: r.key in m.entries.key
# For every input mapping and every entry in it, the output record with that entry's key carries a field value with the mapping's field and the entry's value.
	all m in field_to_map: all e in m.entries: all r in out where r.key == e.key: any fv in r.fields: fv.field == m.field and fv.value == e.value
# Every field value in an output record names an input mapping whose entry for the record's key has that value.
	all r in out: all fv in r.fields: any m in field_to_map where m.field == fv.field: any e in m.entries where e.key == r.key: e.value == fv.value
# The order of the output records is not claimed.
	out unordered
# The order of the field values within an output record is not claimed.
	all r in out: r.fields unordered
```