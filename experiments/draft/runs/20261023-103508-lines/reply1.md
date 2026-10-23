```surety
# The groups' items, taken group after group, are exactly the input items in their input order.
	flatten(out.items) == items
# No group is empty.
	all g in out: len(g.items) > 0
# Every group after the first starts with an item whose pred is true.
	all g in out[1 .. len(out)]: any i in g.items[0 .. 1]: i.pred == true
# In every group but the last, no item after the group's first has pred true.
	all g in out[0 .. len(out) - 1]: none i in g.items[1 .. len(g.items)]: i.pred == true
# When maxsplit is at least 0, the output has at most maxsplit plus one groups.
	maxsplit >= 0 -> len(out) <= maxsplit + 1
# When maxsplit is below 0, or the output has fewer than maxsplit plus one groups, no item after the first in the last group has pred true.
	(maxsplit < 0 or len(out) < maxsplit + 1) -> all g in out[len(out) - 1 .. len(out)]: none i in g.items[1 .. len(g.items)]: i.pred == true
```