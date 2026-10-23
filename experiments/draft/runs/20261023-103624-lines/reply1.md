```surety
# Without keep_separator, the groups' items, taken in order, are drawn from the input items and keep the input's order.
	not keep_separator -> flatten(out.items) same_order_as iterable
# Without keep_separator, every input item left out of the groups is a separator.
	reaching for: when keep_separator is false, every occurrence left in iterable after removing the items of flatten(out.items) has is_sep true. That is `all i leftover in iterable: i.is_sep`, but with leftover taken against flatten(out.items) and not against out, which is a list of groups. `in` compares by value, so it cannot say this for repeated items.
# Without keep_separator, the output has exactly one more group than there are input items left out of the groups.
	not keep_separator -> len(out) + len(flatten(out.items)) == len(iterable) + 1
# Without keep_separator, no group except the last holds a separator.
	all g in out[0 .. len(out) - 1] where not keep_separator: none i in g.items: i.is_sep
# Without keep_separator, the last group holds a separator only when maxsplit is not negative and the output has exactly maxsplit plus one groups.
	all g in out[len(out) - 1 .. len(out)] where not keep_separator and (maxsplit < 0 or len(out) != maxsplit + 1): none i in g.items: i.is_sep
# Without keep_separator, if maxsplit is not negative, the output has at most maxsplit plus one groups.
	not keep_separator and maxsplit >= 0 -> len(out) <= maxsplit + 1
# Without keep_separator, two input items in the groups are in different groups exactly when an input item left out of the groups lies between them.
	reaching for: when keep_separator is false, take any two occurrences in flatten(out.items), each tied to its position in iterable. They sit in different groups (different occurrences of out, not unequal values) if and only if some occurrence of iterable that is left over against flatten(out.items) lies between their positions in iterable. This needs leftover against a flattened list and group identity by occurrence, and the notation has neither.
# With keep_separator, the groups' items, taken in order, are exactly the input items in the input's order.
	keep_separator -> flatten(out.items) == iterable
# With keep_separator, every group except the last that holds a separator holds only that one item.
	all g in out[0 .. len(out) - 1] where keep_separator: all i in g.items where i.is_sep: len(g.items) == 1
# With keep_separator, the last group holds a separator only when maxsplit is not negative and the output has exactly two times maxsplit plus one groups.
	all g in out[len(out) - 1 .. len(out)] where keep_separator and (maxsplit < 0 or len(out) != 2 * maxsplit + 1): none i in g.items: i.is_sep
# With keep_separator, the output has one fewer than twice as many groups as the output for the same items and maxsplit without keep_separator.
	keep_separator -> len(out) + 1 == 2 * len(split_at(iterable, maxsplit, false))
# The groups of the output without keep_separator appear among the groups of the output with keep_separator, in the same order.
	split_at(iterable, maxsplit, false) same_order_as split_at(iterable, maxsplit, true)
```