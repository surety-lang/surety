```surety
// draft
// drafted
type Entry = { arg: Int, value: Int }
// drafted
type TabulateError = { code: Text, display: Text }
// drafted
negative_count_error: TabulateError = {
	code: "e-negative-count",
	display: "The number of results to take must be at least 0"
}
// drafted
uncovered_arg_error: TabulateError = {
	code: "e-uncovered-arg",
	display: "The function has no value given for an argument the results reach"
}

tabulate(func: Entry[unique arg], start: Int, count: Int) -> Result<Entry[], TabulateError>
	note "the first count results of func(start), func(start + 1), ..., with func given as a table of its value at each argument and each result paired with its argument; start defaults to 0 in the source"
	# The output is Ok exactly when the count is at least 0 and every argument from start up to but not including start plus count has an entry in the table.

	# An error output is the negative-count error exactly when the count is below 0.

	# An error output is the uncovered-argument error exactly when the count is at least 0 and some argument from start up to but not including start plus count has no entry in the table.

	# An Ok output has exactly count elements.

	# No two elements of an Ok output share an arg.

	# Every element of an Ok output has an arg that is at least start and less than start plus count.

	# An Ok output is sorted by arg, ascending.

	# Every element of an Ok output is an entry of the table, so its value is func's value at its arg.

	// example wanted: the docstring's case, a square table covering -3 to 0 with start -3 and count 4; the values are read from the table at -3, -2, -1, 0 in that order
	// example wanted: start 0 (the source's default) with a table covering 0 onward; the first result is the table's value at 0
	// example wanted: count 0 with an empty table; the coverage condition holds vacuously, so Ok or error is decided by sentence one
	// example wanted: count -1; decided by which error the negative count reports
	// example wanted: a table missing only the argument start plus count minus 1, the last one reached; decided by the coverage boundary
	// example wanted: a table with extra entries at start minus 1 and at start plus count; decided by the range in sentence six leaving them out
```