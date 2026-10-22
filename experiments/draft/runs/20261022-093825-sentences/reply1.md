```surety
// draft
// drafted
type Param = { key: Text, value: Text }

serialize(params: Param[unique key]) -> Text
	note "serialize a record of key/value params into a query string that parse reads back unchanged"
	# The output is a well-formed query string.
	# Parsing the output gives exactly one Ok.
	# The params parsed from the output have the same elements as the input params.
	# The params parsed from the output keep the input's order.
	# If the input has no params, then the length of the output is 0.
	# If the input has some param, then the output contains "=".
	# If the length of the input is at least 2, then the output contains "&".
```