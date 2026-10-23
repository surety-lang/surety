```surety
// draft

split_into(iterable: Int[], sizes: Option<Int>[]) -> Int[][]
	note "cut the iterable into consecutive chunks, one per size, a None size taking everything that remains"
	# If no size is None, the output has exactly one chunk for each size, in the order of the sizes.

	# If some size is None, the output has one chunk for each size up to and including the first None, and none for the sizes after it.

	# The chunks joined in order are the items of the iterable from its start, in the iterable's order, with no item skipped or repeated.

	# No chunk is longer than the whole-number size it was cut for.

	# If a chunk is shorter than its whole-number size, the chunks joined in order are the whole iterable.

	# If some size is None, the chunks joined in order are the whole iterable.

	// example wanted: iterable [1,2,3,4,5,6], sizes [1,2,3]: the sizes sum exactly to the length, so every chunk is full
	// example wanted: iterable [1,2,3,4,5,6], sizes [2,3]: the sizes sum to less than the length, so the tail is not returned
	// example wanted: iterable [1,2,3,4], sizes [1,2,3,4]: the third chunk overruns the iterable and the fourth comes back empty
	// example wanted: iterable [1,2,3,4,5,6,7,8,9,0], sizes [2,3,None]: the None chunk takes everything that remains
	// example wanted: iterable [1,2,3,4,5], sizes [2,None,1]: the size after the first None gets no chunk at all
	// example wanted: iterable [1,2,3], sizes []: no sizes, so no chunks
	// example wanted: iterable [], sizes [2,None]: every chunk comes back empty, including the None chunk
	// example wanted: iterable [1,2,3], sizes [1,0,2]: a size of 0 is an empty chunk in the middle and uses up nothing
	// example wanted: iterable [1,2,3], sizes [-1]: islice refuses a negative size; the author decides whether this is an error case or a refinement on sizes
```