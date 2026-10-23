```surety
# The groups' items, taken together in the order of the groups, are exactly the input items in their input order.
	flatten(out.items) == iterable
# No group is empty.
	all g in out: len(g.items) >= 1
# Within a group, every item after the first has a position one more than the position of the item before it in that group.
	all g in out: all a in g.items: all b in g.items where a before b in g.items: b.position == a.position + 1 or any c in g.items: a before c in g.items and c before b in g.items
# For every two groups next to each other in the output, the first item of the later group does not have a position one more than the last item of the earlier group.
	all g in out: all h in out where g before h in out: all f in h.items[0 .. 1]: all l in g.items[len(g.items) - 1 .. len(g.items)]: f.position != l.position + 1 or any k in out: g before k in out and k before h in out
```