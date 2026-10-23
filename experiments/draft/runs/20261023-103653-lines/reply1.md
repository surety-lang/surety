```surety
# Joining the pieces' items end to end gives back the input items, in the input's order.
	flatten(out.items) == items
# If the input has at least one item, no piece is empty.
	len(items) > 0 -> none p in out: len(p.items) == 0
# Every piece except the first begins with an item marked split_before.
	all p in out[1 .. len(out)]: any i in p.items[0 .. 1]: i.split_before
# In every piece except the last, no item other than the piece's first is marked split_before.
	all p in out[0 .. len(out) - 1]: none i in p.items[1 .. len(p.items)]: i.split_before
# If the last piece holds a marked item other than its first, then maxsplit is at least 0 and the output has exactly maxsplit plus one pieces.
	maxsplit < 0 or len(out) != maxsplit + 1 -> none p in out[len(out) - 1 .. len(out)]: any i in p.items[1 .. len(p.items)]: i.split_before
# If maxsplit is at least 0, the output has at most maxsplit plus one pieces.
	maxsplit >= 0 -> len(out) <= maxsplit + 1
```