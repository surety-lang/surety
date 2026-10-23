```surety
# The output is Ok exactly when n is at least 0 and at most the length of the iterable.
	(n >= 0 and n <= len(iterable)) <-> exactly_one Ok(_) in out
# The output is the negative-n error exactly when n is less than 0.
	n < 0 <-> exactly_one Err(e) in out: e == negative_n_err
# The output is the n-too-large error exactly when n is more than the length of the iterable.
	n > len(iterable) <-> exactly_one Err(e) in out: e == n_too_large_err
# An Ok output has one more window than the length of the iterable minus n.
	all Ok(ws) in out: len(ws) == len(iterable) - n + 1
# In an Ok output, every window's middle has exactly n items.
	all Ok(ws) in out: all w in ws: len(w.middle) == n
# In an Ok output, every window's beginning is the iterable's items from position 0 up to but not including the length of that beginning.
	all Ok(ws) in out: all w in ws: w.beginning == iterable[0 .. len(w.beginning)]
# In an Ok output, every window's middle is the iterable's items from the position where its beginning ends up to but not including that position plus n.
	all Ok(ws) in out: all w in ws: w.middle == iterable[len(w.beginning) .. len(w.beginning) + n]
# In an Ok output, every window's end is the iterable's items from the position where its middle ends to the end of the iterable.
	all Ok(ws) in out: all w in ws: w.end == iterable[len(w.beginning) + len(w.middle) .. len(iterable)]
# In an Ok output, of any two windows, the one that comes first has the shorter beginning.
	all Ok(ws) in out: all a in ws: all b in ws where a before b in ws: len(a.beginning) < len(b.beginning)
```