```surety
# The output keeps the input's order.
	out same_order_as items
# No two output items share a key.
	unique out.key
# Every input item's key is the key of some output item.
	all i in items: i.key in out.key
# Every input item left out of the output comes after the output item with the same key in the input.
	all l leftover in items: all o in out where o.key == l.key: o before l in items
```