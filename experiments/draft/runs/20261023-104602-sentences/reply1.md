```surety
// draft
// drafted
type Block = { items: Text[] }
// drafted
type Partition = { blocks: Block[] }
// drafted
type PartitionError = { code: Text, display: Text }
// drafted
k_error: PartitionError = { code: "e-k-nonpositive", display: "Can't partition in a negative or zero number of groups" }

set_partitions(iterable: Text[], k: Option<Int>, min_size: Option<Int>, max_size: Option<Int>) -> Result<Partition[], PartitionError>
	note "Yield the set partitions of the iterable into k blocks, or into any number of blocks when k is not given, with each block's size bounded by min_size and max_size when they are given."
	# The output is k_error exactly when k is given and is less than 1.

	# In every output partition, the elements of its blocks taken together are the same elements as the iterable.

	# No block in any output partition is empty.

	# If k is given, every output partition has exactly k blocks.

	# If min_size is given, every block in every output partition has at least min_size elements.

	# If max_size is given, every block in every output partition has at most max_size elements.

	# No two output partitions group the elements into the same blocks.

	# Every partition returned by the call with no k, min_size or max_size that meets the given k, min_size and max_size is in the output.

	# The order of the output partitions is not claimed.

	# The order of the blocks within a partition, and of the elements within a block, is not claimed.

	// example wanted: iterable "a","b","c" with k 2: the three two-block partitions from the docstring
	// example wanted: iterable "a","b","c" with nothing else given: all five partitions, from one block up to three singletons
	// example wanted: iterable "a","b","c" with min_size 2: only the one-block partition qualifies
	// example wanted: iterable "a","b","c" with max_size 2: every partition except the one-block one
	// example wanted: empty iterable with no k: whether the empty partition (zero blocks) is yielded or nothing is
	// example wanted: iterable of two elements with k 3: k exceeds the element count, so no partition has k non-empty blocks
	// example wanted: k 0: the threshold of the error case, against k 1 just above it
	// example wanted: min_size 3 with max_size 2 on any iterable: no block can satisfy both bounds
	// example wanted: iterable "a","a" with nothing else given: whether the two elements are treated as distinct positions, and how many partitions that yields
```