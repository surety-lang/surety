```surety
# The leaves of the output pieces, taken together, are the same leaves as the input's, with none lost, none added and none repeated.
	flatten(out.leaves) same_elements_as iterable

# The leaves of the output pieces, taken together, keep the input's order.
	flatten(out.leaves) same_order_as iterable

# Every piece with no root holds exactly one leaf.
	all p in out where p.root == None: len(p.leaves) == 1

# A leaf is in a piece with no root exactly when none of its ancestors is of the base type and either levels is not given or the leaf has at most levels ancestors.
	reaching for: a way to compare against the Int inside an Option<Int> (an unwrap, or a `Some(n)` binder over levels). With one, the line would be: for every leaf l of iterable, (some piece p with p.root == None has l in p.leaves) <-> ((none a in l.ancestors: a.type_name == base_type) and (levels == None or len(l.ancestors) <= the Int in levels)). As written, `len(l.ancestors) <= levels` compares Int with Option<Int> and is refused.

# A piece with a root holds every input leaf that has that root among its ancestors, and no other leaf.
	all p in out where p.root != None: p.leaves drawn_from iterable and all l in iterable: l in p.leaves <-> any a in l.ancestors: a == p.root

# No two pieces share a root.
	all p in out where p.root != None: exactly_one q in out: q.root == p.root

# Every piece's root is of the base type, or it sits at position levels in each of its leaves' ancestors.
	reaching for: field access through an Option (the type_name of the Container inside p.root), a way to name an element's position in a list (the index of p.root within l.ancestors), and a comparison of that position with the Int inside levels. With these, the line would be: every piece p with a root has p.root's type_name == base_type, or, for every l in p.leaves, the position of p.root in l.ancestors equals levels.

# In each leaf of a piece with a root, no ancestor before the root is of the base type, and the root sits at a position no greater than levels.
	reaching for: `before` with an Option<Container> operand (p.root), and an ordered comparison of the root's position in l.ancestors with the Int inside levels (an unwrap of levels, plus a position-of form). With these, the line would be: for every piece p with a root and every l in p.leaves, no a in l.ancestors that comes before p.root has a.type_name == base_type, and either levels == None or the position of p.root in l.ancestors <= the Int in levels.
```