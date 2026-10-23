```surety
# Every permutation's items are drawn from the input: each item is an input element, and no input occurrence is used twice within one permutation.
	all p in out: p.items drawn_from iterable
# When r is given, every permutation has exactly r items.
	r != None -> all p in out: len(p.items) == r
# When r is not given, every permutation has as many items as the input.
	r == None -> all p in out: len(p.items) == len(iterable)
# No two permutations in the output have the same items.
	unique out.items
# The permutations are in ascending lexicographic order of their items.
	out sorted_by (items asc)
# When r is given and is at least 0 and at most the input length, the first r input elements, in input order, are the items of some permutation in the output.
	reaching for: a way to take the Int inside r: Option<Int> when r is given, so it can be compared (n >= 0 and n <= len(iterable)) and used as a slice bound, as in: r is Some(n) with 0 <= n <= len(iterable) -> any p in out: p.items == iterable[0 .. n]
# When r is not given, the input itself, in input order, is the items of some permutation in the output.
	r == None -> any p in out: p.items == iterable
# When r is not given, running on the items of any output permutation, with no r, gives the same output.
	r == None -> all p in out: distinct_permutations(p.items, None) == out
```