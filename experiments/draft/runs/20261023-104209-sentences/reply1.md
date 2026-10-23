```surety
// draft

longest_common_prefix(iterables: Text[][]) -> Text[]
	note "yield the elements that every iterable shares from its start, stopping at the first position where they disagree or one runs out"
	# For every iterable, its elements from position 0 up to but not including the output's length are exactly the output, in order.

	# Every iterable either has exactly as many elements as the output, or some iterable's elements from position 0 up to but not including one past the output's length differ from that iterable's own.

	// example wanted: no iterables at all: zip over nothing decides it, and the two sentences above hold vacuously, so only an example pins it
	// example wanted: one iterable ["a", "b", "c"]: every element is shared with itself, so the second sentence's first branch decides it
	// example wanted: the docstring case, ["a","b","c","d"], ["a","b","c"], ["a","b","f"]: the third position is where they first disagree
	// example wanted: ["a","b","c"] and ["a","b"]: they never disagree, so the shorter one running out decides it
	// example wanted: an empty iterable among non-empty ones, [] and ["a"]: the empty one has nothing to share
	// example wanted: iterables that disagree at the first position, ["a"] and ["b"]: no element is shared
	// example wanted: identical iterables ["x","x"] and ["x","x"]: repeated equal values, compared by value at each position
```