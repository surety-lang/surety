```surety
# No two categories share a key.
	unique out.key

# Every input item's key is the key of some category.
	all i in items: i.key in out.key

# Every category's key is the key of some input item.
	all c in out: c.key in items.key

# Each category's values are drawn from the values of the input items with that category's key.
	reaching for: all c in out: c.values drawn_from (the values of only those items whose key is c.key). That needs a filtered sublist as the right operand, and the notation has no expression for one. A membership form (any i in items where i.key == c.key: i.value == v) would drop the per-key multiplicity, and so widen the sentence.

# The values across all categories, taken together, are the same elements as the input items' values.
	flatten(out.values) same_elements_as items.value

# Each category's values keep the input's order.
	reaching for: all c in out: c.values same_order_as (the values of only those items whose key is c.key). Writing c.values same_order_as items.value would let an equal value under another key stand in for a position, and so widen the sentence. A filtered sublist is again the missing operand.

# The order of the categories is not claimed.
	out unordered
```