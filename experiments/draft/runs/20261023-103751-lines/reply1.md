```surety
# The output has count elements, and none when count is below zero.
	len(out) == max(count, 0)

# The output starts with the items in their input order, as far as count reaches: the output from position 0 up to count is the items from position 0 up to count.
	out[0 .. len(items)] == items[0 .. count]
	// the gloss after the colon, read literally (out[0 .. count] == items[0 .. count]), is false whenever count > len(items): out[0 .. count] has count elements and items[0 .. count] has len(items), so it contradicts the repeat sentences below. This line pins "as far as count reaches". The author may want to reword the gloss to "the output up to the number of items is the items up to count".

# When there is at least one item, every output element at a position at or past the number of items is the last item.
	len(items) > 0 -> all o in out[len(items) .. len(out)]: o in items[len(items) - 1 .. len(items)]

# When there are no items, every output element is the default.
	len(items) == 0 -> all o in out: o == default
```