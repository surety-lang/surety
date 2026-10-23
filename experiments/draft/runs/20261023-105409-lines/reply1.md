```surety
# The result is the window-size error exactly when n is less than 1.
	n < 1 <-> out == Err(window_error)
# On success, the output is drawn from the items and keeps their order.
	all Ok(res) in out: res same_order_as items
# On success, every output item's key matches no key among the n minus 1 items just before it in the input.
	reaching for: the input position of the occurrence each output element was taken from, aligned by the subsequence match rather than by first occurrence (in [0, 1, 0, 2, 3, 0] with n = 3 the kept second 0 is position 5, not 0), so that I could write `all Ok(res) in out: all o in res: none p in items[pos(o) - (n - 1) .. pos(o)]: p.key == o.key`; the notation has no way to name a binder's position, so a slice cannot be anchored to it
# On success, every item left out of the output has its key among the n minus 1 items just before it in the input, whether or not those items were kept.
	reaching for: the occurrences of items that were actually dropped, each with its input position, so that I could write `all Ok(res) in out: all l dropped in items: any p in items[pos(l) - (n - 1) .. pos(l)]: p.key == l.key`; `leftover` cannot stand in for "dropped" because it assigns each output element the earliest equal occurrence (in [0, 1, 0, 2, 3, 0] with n = 3 it would call position 5 the leftover when position 2 is the one dropped), and there is no form that names a binder's position to anchor the slice
```