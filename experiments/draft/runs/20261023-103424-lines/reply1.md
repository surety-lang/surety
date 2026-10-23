```surety
# The leaves of the output pieces, taken together, are the same leaves as the input's, with none lost, none added and none repeated.
	flatten(out.leaves) same_elements_as iterable

# The leaves of the output pieces, taken together, keep the input's order.
	flatten(out.leaves) same_order_as iterable

# Every piece with no root holds exactly one leaf.
	all p in out where p.root == None: len(p.leaves) == 1

# A leaf is in a piece with no root exactly when none of its ancestors is of the base type and either levels is not given or the leaf has at most levels ancestors.
	all l in iterable: base_type not in l.ancestors.type_name and (levels == None or len(l.ancestors) <= levels) <-> any p in out where p.root == None: l in p.leaves

# A piece with a root holds every input leaf that has that root among its ancestors, and no other leaf.
	all p in out where p.root != None: p.leaves drawn_from iterable and all l in iterable: l in p.leaves <-> p.root in l.ancestors

# No two pieces share a root.
	all p in out where p.root != None: exactly_one q in out: q.root == p.root

# Every piece's root is of the base type, or it sits at position levels in each of its leaves' ancestors.
	all p in out where p.root != None: p.root.type_name == base_type or all l in p.leaves: levels != None and p.root in l.ancestors[levels .. levels + 1]

# In each leaf of a piece with a root, no ancestor before the root is of the base type, and the root sits at a position no greater than levels.
	all p in out where p.root != None: all l in p.leaves: (levels == None or p.root in l.ancestors[0 .. levels + 1]) and all a in l.ancestors where a before p.root in l.ancestors: a.type_name != base_type
```