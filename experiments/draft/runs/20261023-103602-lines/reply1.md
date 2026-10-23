```surety
# The chunks, joined end to end, are the input items in their input order.
	flatten(out.items) == iterable

# Every chunk except the last ends with an item where pred holds.
	all c in out[0 .. len(out) - 1]: any i in c.items[len(c.items) - 1 .. len(c.items)]: i.pred

# In every chunk except the last, pred holds for no item before the chunk's final one.
	all c in out[0 .. len(out) - 1]: none i in c.items[0 .. len(c.items) - 1]: i.pred

# When maxsplit is not negative, the output has at most maxsplit plus one chunks.
	maxsplit >= 0 -> len(out) <= maxsplit + 1

# Unless the output has exactly maxsplit plus one chunks, pred holds for no item of the last
# chunk before its final one.
	len(out) != maxsplit + 1 -> all c in out[len(out) - 1 .. len(out)]: none i in c.items[0 .. len(c.items) - 1]: i.pred

# Some chunk is empty exactly when the input is empty and maxsplit is 0.
	len(iterable) == 0 and maxsplit == 0 <-> any c in out: len(c.items) == 0
```