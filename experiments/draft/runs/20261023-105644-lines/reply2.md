```surety
# The output is too_few_items_err exactly when repeat is at least 1 and some iterable has fewer than two items.
	reaching for: an iff with a quantifier on each side, "repeat is at least 1 and some iterable has fewer than two items" if and only if "out is the Err side holding too_few_items_err"; Err(...) cannot be compared as a value, and a line can end in only one quantifier
# In an Ok output, every tuple's length is the number of iterables times repeat, or 0 when repeat is below 1.
	all Ok(res) in out: all t in res: len(t.values) == len(iterables) * max(repeat, 0)
# In an Ok output, every value in a tuple is an item of the iterable its position belongs to, with the positions running through the iterables in order and starting over for each repeat.
	reaching for: a quantifier over the positions p of a tuple, and the iterable at index p modulo the number of iterables, so that the value at p is in that iterable's items
# In an Ok output, every combination that takes one item from each position's iterable appears as some tuple.
	reaching for: a quantifier over every combination that takes one item from each position's iterable (the cartesian product, with the iterables repeated repeat times), each found among the tuples' values
# If no iterable repeats an item, no two tuples in an Ok output are equal.
	all Ok(res) in out: unique res.values or any f in iterables: not unique f.items
# In an Ok output, the first tuple takes the first item of every position's iterable.
	reaching for: the list of each iterable's first item, written out repeat times in sequence (list repetition), equal to the values of the tuple at position 0
# If no iterable repeats an item, each tuple in an Ok output differs from the tuple before it in exactly one position.
	reaching for: a quantifier over pairs of consecutive tuples, and a count of the positions at which their values differ, required to be exactly 1
# In an Ok output, the value that changes from one tuple to the next moves to an item next to it in its iterable.
	reaching for: for consecutive tuples, the position where they differ and that position's iterable, with the old and new values at adjacent indexes of the iterable's items
# The output for a repeat of at least 1 is the output for the iterables written out that many times in sequence with a repeat of 1.
	reaching for: list repetition, the iterables written out repeat times in sequence, so the line reads: when repeat is at least 1, out equals gray_product(<iterables repeated repeat times>, 1)
```