```surety
# If no size is None, the output has exactly one chunk for each size, in the order of the sizes.
	len(out) == len(sizes) or any s in sizes: s == None
# If some size is None, the output has one chunk for each size up to and including the first None, and none for the sizes after it.
	all s in sizes where s == None: any t in sizes[len(out) - 1 .. len(out)] where t == None: none u in sizes[0 .. len(out) - 1]: u == None
# The chunks joined in order are the items of the iterable from its start, in the iterable's order, with no item skipped or repeated.
	flatten(out) == iterable[0 .. len(flatten(out))]
# No chunk is longer than the whole-number size it was cut for.
	reaching for: for every position i in out, if sizes at position i is a whole number n, then the length of the chunk at position i of out is at most n. That needs the chunk and the size at the same position, a positional index or a zip of out with sizes, and the fragment has neither.
# If a chunk is shorter than its whole-number size, the chunks joined in order are the whole iterable.
	reaching for: if, at some position i, sizes at i is a whole number n and the chunk at position i of out has length less than n, then flatten(out) == iterable. Like the line above, it pairs each chunk with the size at its own position, which needs a positional index or a zip of out with sizes.
# If some size is None, the chunks joined in order are the whole iterable.
	flatten(out) == iterable or none s in sizes: s == None
```