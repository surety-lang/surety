```surety
# The output keeps the input's order.
	out same_order_as items
# Every output item has an earlier input item with the same key.
	reaching for: for each output item o, some input item with o's key sits at an earlier position than the input occurrence o was kept from (all o in out: any i in items where i.key == o.key: i before o in items). But `before` places an output element at the earliest input occurrence of its value, and `leftover` assigns earliest occurrences too. This function keeps the later copies, so on [X, X] -> [X], or the second 's' in 'mississippi', that line is false on the correct output. The notation has no way to bind an output item to the later occurrence it came from.
# No two input items left out of the output share a key.
	all a leftover in items: exactly_one b leftover in items: b.key == a.key
# Every input item's key is the key of some input item left out of the output.
	all i in items: any l leftover in items: l.key == i.key
```