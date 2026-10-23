```surety
# The output has exactly as many elements as the input has items.
	len(out) == len(items)
# The output's items are the input items, in the input's order.
	out.item == items
# The first output element is marked first.
	all m in out[0 .. 1]: m.is_first
# No output element after the first is marked first.
	none m in out[1 .. len(out)]: m.is_first
# The last output element is marked last.
	all m in out[len(out) - 1 .. len(out)]: m.is_last
# No output element before the last is marked last.
	none m in out[0 .. len(out) - 1]: m.is_last
```