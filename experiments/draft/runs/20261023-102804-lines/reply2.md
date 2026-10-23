```surety
# The output is the uneven error exactly when strict is true and chunking the same iterable with
# strict false ends with a chunk of fewer than n elements.
	all Ok(full) in chunked(iterable, n, false): strict and len(full) > 0 and len(flatten(full[len(full) - 1 .. len(full)])) < n <-> any Err(e) in out: e == uneven_error

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
	all Ok(cs) in out where strict: all c in cs: len(c) == n
```