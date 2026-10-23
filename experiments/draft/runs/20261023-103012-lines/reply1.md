```surety
# Every derangement has r items when r is given, and as many items as the iterable when it is not.
	all d in out: (r == None -> len(d.items) == len(iterable)) and (r != None -> len(d.items) == r)
# Every derangement's items are drawn from the iterable: each item is an occurrence of an input element, and no occurrence is used twice.
	all d in out: d.items drawn_from iterable
# No derangement places an input occurrence at its own index: the item at each index of a derangement comes from a different index of the iterable, even when the two have the same value.
	reaching for: for each derangement, a one-to-one assignment of input positions to its items in which no item at index i came from input position i. A derangement is a list of Text and does not record which input position each item came from. The notation has no binder over indices and no existential over assignments. So it can only compare values, and that would wrongly reject ["Bob", "Alice", "Bob"] from ["Alice", "Bob", "Bob"].
# Every arrangement of the right length drawn from the iterable that leaves no occurrence at its own index is in the output.
	reaching for: a quantifier over every arrangement of r input positions (or all of them when r is None) with no position at its own index, saying the arrangement of values it picks out is in out. The notation quantifies only over lists that exist in the inputs or out, not over all permutations of a list.
# Each arrangement of input positions appears in the output exactly once, so two derangements may be equal by value when the iterable repeats a value.
	reaching for: the multiplicity of each value arrangement in out equals the number of fixed-point-free arrangements of input positions that produce it. This needs each derangement's source positions, which the output does not carry, and a count over position arrangements. `unique` and `drawn_from` compare values, so they would wrongly forbid the repeats.
# The derangements come in lexicographic order of the input positions they draw from.
	reaching for: out sorted lexicographically by each derangement's sequence of source input positions. `sorted_by` takes field keys, not a key that is a list of positions, and a derangement's source positions are not recorded in it. Sorting by the item values would be wrong whenever the iterable is not already sorted.
```