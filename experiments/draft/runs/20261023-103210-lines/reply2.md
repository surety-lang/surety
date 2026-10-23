```surety
# The output is the negative n error exactly when n is less than 0.
	n < 0 <-> exactly_one Err(e) in out: e == negative_n_error
# The output is the step error exactly when n is at least 1 and step is less than 1.
	(n >= 1 and step < 1) <-> exactly_one Err(e) in out: e == step_error
# The output is Ok exactly when neither error applies.
	not (n < 0 or (n >= 1 and step < 1)) <-> exactly_one Ok(_) in out
# Every window in an Ok output has exactly n items.
	all Ok(ws) in out: all w in ws: len(w.items) == n
# When n is at least 1, an Ok output has a window exactly when seq is non-empty.
	all Ok(ws) in out: n >= 1 -> (len(ws) > 0 <-> len(seq) > 0)
# The first window's items are the first n elements of seq, in order, followed by fillvalue in every position past the end of seq.
	all Ok(ws) in out: all w in ws[0 .. 1]: w.items[0 .. len(seq)] == seq[0 .. n] and all f in w.items[len(seq) .. n]: f == fillvalue
# When n is at least the length of seq, an Ok output has at most one window.
	all Ok(ws) in out: n >= len(seq) -> len(ws) <= 1
# When n is at least 1 and less than the length of seq, the windows after the first are, in order, the output of windowed over seq from position step onward, with the same n, fillvalue and step.
	all Ok(ws) in out: (n >= 1 and n < len(seq)) -> exactly_one Ok(rest) in windowed(seq[step .. len(seq)], n, fillvalue, step): ws[1 .. len(ws)] == rest
```