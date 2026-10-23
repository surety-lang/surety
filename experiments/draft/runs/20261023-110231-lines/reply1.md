```surety
# The output is Ok exactly when the count is at least 0 and every argument from start up to but not including start plus count has an entry in the table.
	exactly_one Ok(_) in out <-> count >= 0 and (count > 0 -> start in func.arg) and all x in func where start <= x.arg and x.arg + 1 < start + count: x.arg + 1 in func.arg
# An error output is the negative-count error exactly when the count is below 0.
	all Err(err) in out: err == negative_count_error <-> count < 0
# An error output is the uncovered-argument error exactly when the count is at least 0 and some argument from start up to but not including start plus count has no entry in the table.
	all Err(err) in out: err == uncovered_arg_error <-> count > 0 and start not in func.arg or any x in func where start <= x.arg and x.arg + 1 < start + count: x.arg + 1 not in func.arg
# An Ok output has exactly count elements.
	all Ok(res) in out: len(res) == count
# No two elements of an Ok output share an arg.
	all Ok(res) in out: unique res.arg
# Every element of an Ok output has an arg that is at least start and less than start plus count.
	all Ok(res) in out: all r in res: start <= r.arg and r.arg < start + count
# An Ok output is sorted by arg, ascending.
	all Ok(res) in out: res sorted_by (arg asc)
# Every element of an Ok output is an entry of the table, so its value is func's value at its arg.
	all Ok(res) in out: all r in res: r in func
```