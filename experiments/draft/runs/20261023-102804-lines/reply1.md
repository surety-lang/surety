```surety
# The output is the uneven error exactly when strict is true and chunking the same iterable with strict false ends with a chunk of fewer than n elements.
	out == Err(uneven_error) <-> strict and any Ok(cs) in chunked(iterable, n, false): any c in cs[len(cs) - 1 .. len(cs)]: len(c) < n
# An error output is always the uneven error and never any other error.
	all Err(e) in out: e == uneven_error
# In an Ok output, the chunks joined end to end in order are exactly the iterable.
	all Ok(cs) in out: flatten(cs) == iterable
# In an Ok output, every chunk except the last has exactly n elements.
	all Ok(cs) in out: all c in cs[0 .. len(cs) - 1]: len(c) == n
# In an Ok output, the number of chunks times n is at least the length of the iterable.
	all Ok(cs) in out: len(cs) * n >= len(iterable)
# In an Ok output, the number of chunks times n is less than the length of the iterable plus n.
	all Ok(cs) in out: len(cs) * n < len(iterable) + n
# If strict is true, every chunk of an Ok output has exactly n elements.
	strict -> all Ok(cs) in out: all c in cs: len(c) == n
```