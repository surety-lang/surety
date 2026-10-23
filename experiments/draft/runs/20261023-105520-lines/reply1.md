```surety
# The output has exactly as many entries as the input has elements.
	len(out) == len(items)
# The output's elements, taken in order, are the input elements in the input's order.
	out.element == items
# An entry's unique_justseen flag is true exactly when it is the first entry or its element's key differs from the key of the entry immediately before it.
	all c in out: c.unique_justseen <-> none p in out where p before c in out and p.element.key == c.element.key: none q in out: p before q in out and q before c in out
# An entry's unique_everseen flag is true exactly when no input element that comes before its element has the same key.
	all c in out: c.unique_everseen <-> none p in out where p before c in out: p.element.key == c.element.key
```