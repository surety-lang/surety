```surety
# The output has as many elements as the number of iterables times the length of the shortest iterable.
	all s in iterables: len(out) == len(iterables) * len(s) or any t in iterables: len(t) < len(s)
# For every position k below the length of the shortest iterable, the output's elements from position k times the number of iterables up to but not including k plus one times the number of iterables are the element at position k of each iterable, in the order the iterables are given.
	reaching for: a quantifier over integer positions, k from 0 up to but not including the shortest iterable's length, and a way to build the list of each iterable's element at position k in the order the iterables are given (indexing mapped over iterables), so I could write: for every such k, out[k * len(iterables) .. (k + 1) * len(iterables)] == [it[k] for each it in iterables]; the notation binds only list occurrences, never a bare position, and has no index or map to form that list.
```