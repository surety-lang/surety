```surety
# The output is drawn from the arguments' contributions, where a non-iterable argument contributes its value and an iterable argument contributes its items.
	reaching for: out drawn_from the list made of each non-iterable argument's value and each iterable argument's items, as a multiset. `flatten(args.items)` also takes the items field of non-iterable arguments, and the notation has no filtered list or per-element choice of field to build the right list. `all x in out: any a in args: ...` would keep membership but lose multiplicity, which widens the sentence.
# The output has as many elements as there are non-iterable arguments plus the items of the iterable arguments together.
	reaching for: len(out) == (the number of arguments with iterable false) + (the sum of len(a.items) over the arguments with iterable true). The notation has no count or sum over a filtered list, and `len(flatten(args.items))` would also count the ignored items of non-iterable arguments.
# Everything an earlier argument contributes comes before everything a later argument contributes.
	reaching for: for arguments a before b in args, every output occurrence that a contributes comes before every output occurrence that b contributes. `x before y in out` on contributed values resolves to first occurrences in out, so the same value contributed twice (the wanted example) makes the line false on the correct output. The notation also cannot choose a contribution by case (a value or the items) inside one quantifier.
# An iterable argument's items keep their order in the output.
	all a in args where a.iterable: a.items same_order_as out
```