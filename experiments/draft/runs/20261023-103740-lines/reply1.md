```surety
# The output begins with the items, in their order and with nothing left out.
	out[0 .. len(items)] == items
# Every output element after the items is the fillvalue.
	all x in out[len(items) .. len(out)]: x == fillvalue
# If next_multiple is false, the output's length is the larger of n and the number of items.
	not next_multiple -> len(out) == max(n, len(items))
# If next_multiple is true and n is at least 1, the output has at least as many elements as the items and fewer than the number of items plus n.
	next_multiple and n >= 1 -> len(out) >= len(items) and len(out) < len(items) + n
# If next_multiple is true and n is at least 1, the output's length is a multiple of n (a sibling obligation decides what a multiple is).
	next_multiple and n >= 1 -> multiple_of(len(out), n)
```