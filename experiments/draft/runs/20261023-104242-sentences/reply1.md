```surety
// draft
// drafted
type Item = { value: Text, pred_true: Bool }

rstrip(iterable: Item[], ) -> Item[]
	note "yield the items of iterable with the trailing run of items for which pred returns True stripped from the end; each item's pred_true carries what pred returns for it"
	# The output is the input's items from the start up to but not including position len(out), so it is a prefix of the input in the input's order.

	# Every input item left out of the output is one for which pred returns True.

	# The output's last item, if there is one, is one for which pred returns False.

	// example wanted: an empty iterable; nothing to strip, decided by the prefix claim alone
	// example wanted: every item has pred True; the whole run is trailing, so nothing survives the strip
	// example wanted: no item has pred True; the output is decided by nothing being left out
	// example wanted: the docstring's (None, False, None, 1, 2, None, 3, False, None) with pred true for None and False; the leading and middle matches stay because a pred-False item comes after them
	// example wanted: pred True only on the first items and False on the last; unlike strip, the leading run is kept
	// example wanted: two items with the same value, the first pred False and the last pred True; only the trailing occurrence is stripped
```