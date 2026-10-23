```surety
# The output is k_error exactly when k is given and is less than 1.
	(k != None and k < 1) <-> any Err(e) in out: e == k_error
# In every output partition, the elements of its blocks taken together are the same elements as the iterable.
	all Ok(ps) in out: all p in ps: flatten(p.blocks.items) same_elements_as iterable
# No block in any output partition is empty.
	all Ok(ps) in out: all p in ps: all b in p.blocks: len(b.items) > 0
# If k is given, every output partition has exactly k blocks.
	k != None -> all Ok(ps) in out: all p in ps: len(p.blocks) == k
# If min_size is given, every block in every output partition has at least min_size elements.
	min_size != None -> all Ok(ps) in out: all p in ps: all b in p.blocks: len(b.items) >= min_size
# If max_size is given, every block in every output partition has at most max_size elements.
	max_size != None -> all Ok(ps) in out: all p in ps: all b in p.blocks: len(b.items) <= max_size
# No two output partitions group the elements into the same blocks.
	reaching for: `unique` over the output partitions, where two partitions count as equal when their blocks are equal as a multiset of multisets, so the order of blocks and the order of elements within a block are ignored. `unique ps` compares by value, order included. That would let two orderings of one grouping pass as distinct. Checking that every block of one partition appears among the other's blocks, in both directions, is not exact when the iterable has duplicates: {[a,a]×5, [a,a,a]} and {[a,a]×2, [a,a,a]×3} pass it and are different groupings.
# Every partition returned by the call with no k, min_size or max_size that meets the given k, min_size and max_size is in the output.
	reaching for: all Ok(ps) in out: all Ok(all_ps) in set_partitions(iterable, None, None, None): all p in all_ps where (k == None or len(p.blocks) == k): <some q in ps groups the elements into the same blocks as p> or any b in p.blocks: (min_size != None and len(b.items) < min_size) or (max_size != None and len(b.items) > max_size). The bracketed part is partition equality up to the order of blocks and of elements within a block. `p in ps` compares by value with order, which narrows the sentence, because the two calls need not order their blocks alike. The one-direction block-matching approximation is not exact on duplicate elements.
# The order of the output partitions is not claimed.
	all Ok(ps) in out: ps unordered
# The order of the blocks within a partition, and of the elements within a block, is not claimed.
	all Ok(ps) in out: all p in ps: p.blocks unordered and all b in p.blocks: b.items unordered
```