```surety
// draft
// drafted
type Item = { value: Int, pred: Bool }
// drafted
type Group = { items: Item[] }

split_before(items: Item[], maxsplit: Int) -> Group[]
	note "split the items into groups, each ending just before an item whose pred is true, doing at most maxsplit splits when maxsplit is at least 0 and no limit when it is below 0; each item carries pred's answer for it, since the fragment has no callables"
	# The groups' items, taken group after group, are exactly the input items in their input order.

	# No group is empty.

	# Every group after the first starts with an item whose pred is true.

	# In every group but the last, no item after the group's first has pred true.

	# When maxsplit is at least 0, the output has at most maxsplit plus one groups.

	# When maxsplit is below 0, or the output has fewer than maxsplit plus one groups, no item after the first in the last group has pred true.

	// example wanted: no items, maxsplit -1: no buffer is ever filled, so decide whether the output is no groups or one empty group
	// example wanted: the first item has pred true, maxsplit -1: no split falls before position 0, so there is no empty leading group
	// example wanted: two consecutive items with pred true, maxsplit -1: each one starts its own group
	// example wanted: values 0 to 9 with pred true on multiples of 3, maxsplit 2: the third and later splits are not done, so the remaining items stay in the last group
	// example wanted: values 0 to 9 with pred true on multiples of 3, maxsplit 0: no split is done at all
	// example wanted: maxsplit exactly the number of pred-true items after position 0: the limit is reached and every split is still done
	// example wanted: maxsplit -2: any value below 0 means no limit, not only -1
	// example wanted: no item has pred true, maxsplit -1: nothing splits, so all the items are in one group
```