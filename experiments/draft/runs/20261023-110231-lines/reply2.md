```surety
# The output is Ok exactly when the count is at least 0 and every argument from start up to but not including start plus count has an entry in the table.
	reaching for: a list of the integers from start up to but not including start + count to quantify over, so the line could read `(count >= 0 and all a in <start .. start + count>: a in func.arg) <-> exactly_one Ok(_) in out`; the notation has no integer range and no count-where, so whether every integer in that range is covered cannot be stated over the table alone
# An error output is the negative-count error exactly when the count is below 0.
	all Err(e) in out: e == negative_count_error <-> count < 0
# An error output is the uncovered-argument error exactly when the count is at least 0 and some argument from start up to but not including start plus count has no entry in the table.
	reaching for: `all Err(e) in out: e == uncovered_arg_error <-> (count >= 0 and any a in <start .. start + count>: a not in func.arg)`; it needs a list of the integers from start up to but not including start + count, and the notation has no integer range
# An Ok output has exactly count elements.
	all Ok(r) in out: len(r) == count
# No two elements of an Ok output share an arg.
	all Ok(r) in out: unique r.arg
# Every element of an Ok output has an arg that is at least start and less than start plus count.
	all Ok(r) in out: all e in r: e.arg >= start and e.arg < start + count
# An Ok output is sorted by arg, ascending.
	all Ok(r) in out: r sorted_by (arg asc)
# Every element of an Ok output is an entry of the table, so its value is func's value at its arg.
	all Ok(r) in out: all e in r: e in func
```