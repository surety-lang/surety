```surety
# The output is Ok exactly when the count is at least 0 and every value among the first count minus one values of the iteration from start has a step in func.
	reaching for: the iteration from start as a list the line can quantify over: position 0 is start, and each later position is the to of func's step from the value before it. With that I would write "the output is Ok iff count >= 0 and every value at positions 0 up to count - 1 of that list is some step's from". The values exist only inside an Ok output, so an Err output gives nothing to range over. The recursive restatement through a self-call needs a quantifier on both sides of the <->, and a quantifier must be the last operand on a line.
# The output is the negative-count error exactly when the count is below 0.
	count < 0 <-> any Err(e) in out: e == negative_count_error
# The output is the missing-step error exactly when the count is at least 0 and some value among the first count minus one values of the iteration from start has no step in func.
	reaching for: the same iteration list from start, independent of the output, so I could write "the output is Err(missing_step_error) iff count >= 0 and some value at positions 0 up to count - 1 of that list is no step's from". An Err output carries none of the values, and the recursive form needs two quantifiers on one line.
# An Ok output has exactly count elements.
	all Ok(xs) in out: len(xs) == count
# If the count is at least 1, an Ok output's first element is start.
	all Ok(xs) in out: count >= 1 -> all x in xs[0 .. 1]: x == start
# If the count is at least 1 and start has a step in func, an Ok output's elements after the first are the Ok output of iterate on the same func from that step's to, with a count one less.
	all s in func where s.from == start: all Ok(xs) in out: count >= 1 -> any Ok(rest) in iterate(func, s.to, count - 1): xs[1 .. len(xs)] == rest
```