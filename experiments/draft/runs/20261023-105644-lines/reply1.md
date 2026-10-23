```surety
# The output is too_few_items_err exactly when repeat is at least 1 and some iterable has fewer than two items.
	out == Err(too_few_items_err) <-> repeat >= 1 and any f in iterables: len(f.items) < 2
# In an Ok output, every tuple's length is the number of iterables times repeat, or 0 when repeat is below 1.
	all Ok(combos) in out: all c in combos: len(c.values) == max(0, len(iterables) * repeat)
# In an Ok output, every value in a tuple is an item of the iterable its position belongs to, with the positions running through the iterables in order and starting over for each repeat.
	reaching for: a quantifier over positions in a list, so that for every tuple c and every position i below len(c.values), the value of c.values at i is among the items of the iterable at position i - k * len(iterables) of iterables, for the k that puts that position between 0 and len(iterables); the notation binds elements but never their positions as Ints, and has no remainder
# In an Ok output, every combination that takes one item from each position's iterable appears as some tuple.
	reaching for: the cartesian product of the position iterables, the iterables written out repeat times with one item chosen from each, as a list the line can range over: every combination in that product is equal to the values of some tuple in the Ok output
# If no iterable repeats an item, no two tuples in an Ok output are equal.
	all Ok(combos) in out: unique combos or any f in iterables: not unique f.items
# In an Ok output, the first tuple takes the first item of every position's iterable.
	reaching for: a quantifier over positions, so that for the tuple in combos[0 .. 1] and every position i in its values, the value at i equals the first item of the iterable that position i belongs to; the notation can take the first tuple by a slice but cannot pair each of its positions with an iterable's first item
# If no iterable repeats an item, each tuple in an Ok output differs from the tuple before it in exactly one position.
	reaching for: a quantifier over consecutive pairs of tuples, every tuple with the tuple at the position before it, and a count of the positions at which their values differ, claimed to be exactly 1 when every iterable's items are unique
# In an Ok output, the value that changes from one tuple to the next moves to an item next to it in its iterable.
	reaching for: a quantifier over consecutive pairs of tuples and the position at which they differ, with the positions of the old and new values within that position's iterable's items, claimed to differ by 1, as abs(new position - old position) == 1
# The output for a repeat of at least 1 is the output for the iterables written out that many times in sequence with a repeat of 1.
	reaching for: a list built from iterables repeated repeat times end to end, so the line could be repeat >= 1 -> out == gray_product(<iterables written out repeat times>, 1); the notation has no repetition or concatenation of lists
```