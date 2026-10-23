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
	reaching for: out sorted_by (items asc), where two Int[] keys compare element by element from position 0 and the first differing element decides. The notation defines no order on a list-valued key, so `sorted_by` over `items` has no stated meaning, and there is no element indexing to spell the comparison out by hand.
# When r is given and is at least 0 and at most the input length, the first r input elements, in input order, are the items of some permutation in the output.
	r != None and r >= 0 and r <= len(iterable) -> iterable[0 .. r] in out.items
# When r is not given, the input itself, in input order, is the items of some permutation in the output.
	r == None -> iterable in out.items
# When r is not given, running on the items of any output permutation, with no r, gives the same output.
	r == None -> all p in out: distinct_permutations(p.items, None) == out
```