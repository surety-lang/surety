```surety
# The output is an error exactly when times is given and is more than the number of results.
	reaching for: one line saying out is an Err if and only if times holds a value t with t > len(results). Both sides need a quantifier: `exactly_one Err(_) in out` on one, `any Some(t) in times: t > len(results)` on the other. A quantifier has to be the last operand on a line, so they cannot meet across `<->`. Moving the iff inside `all Some(t) in times` drops the None case, where the output must not be an error. It fits once the times side is an abbreviation (`too_many = any Some(t) in times: t > len(results)`, then `too_many <-> exactly_one Err(_) in out`), and that is the author's line to add. It also assumes `Some(t)` is accepted as the binder for the present side of an Option.
# Every error output is too_few_results_err.
	all Err(e) in out: e == too_few_results_err
# When times is given and the output is Ok, the output is the results from position 0 up to but not including times.
	all Some(t) in times: all Ok(res) in out: res == results[0 .. t]
# When times is None and the output is Ok, the output is the results, all of them, in the order the calls produced them.
	times == None -> all Ok(res) in out: res == results
```