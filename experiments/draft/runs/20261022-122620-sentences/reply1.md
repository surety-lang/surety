```surety
// draft
// drafted
type Param = { key: Text, value: Text }
// drafted
type Record = { params: Param[unique key] }

serialize(r: Record) -> Text
	note "serialize a record's params into a URL query string that the sibling parse obligation reads back as the same record"
	# The output is a well-formed query string; a sibling obligation decides the shape.
	# Parsing the output with the sibling parse obligation succeeds.
	# The record parsed from the output equals the input record, with its params in the same order.
	# Every key and every value made only of unreserved characters is contained in the output as written; a sibling obligation decides which characters are unreserved.
	# The output is the empty text exactly when the record has no params.
	// example wanted: a record with no params: the empty-output sentence decides it
	// example wanted: one param whose value contains "&" and "=": escaping decides it, and the round trip must survive it
	// example wanted: one param whose value contains a space: the author decides between "%20" and "+"
	// example wanted: one param with an empty value: the author decides between "k=" and a bare "k"
	// example wanted: two plain params given in reverse alphabetical order: the order-kept sentence decides it
	// example wanted: one param whose value has a non-ASCII character: the author decides the percent-encoding of its UTF-8 bytes
```