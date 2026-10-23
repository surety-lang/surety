```surety
// draft
// drafted
type Item = { value: Int, split_before: Bool }
// drafted
type Piece = { items: Item[] }

split_when(items: Item[], maxsplit: Int) -> Piece[]
	note "split the items into pieces between successive pairs where pred holds (an item's split_before is pred(previous item, this item); the first item's is never consulted), doing at most maxsplit splits when maxsplit is at least 0 and any number when it is negative, the default being -1"
	# Joining the pieces' items end to end gives back the input items, in the input's order.

	# If the input has at least one item, no piece is empty.

	# Every piece except the first begins with an item marked split_before.

	# In every piece except the last, no item other than the piece's first is marked split_before.

	# If the last piece holds a marked item other than its first, then maxsplit is at least 0 and the output has exactly maxsplit plus one pieces.

	# If maxsplit is at least 0, the output has at most maxsplit plus one pieces.

	// example wanted: the docstring's run, values [1, 2, 3, 3, 2, 5, 2, 4, 2] with split_before true where the previous value is larger, maxsplit -1: every mark begins a piece
	// example wanted: the same items at maxsplit 2: the third mark stays inside the last piece
	// example wanted: no items, maxsplit -1: the source returns before taking any item, so decide whether the output is no pieces or one empty piece
	// example wanted: no items, maxsplit 0: the source yields list(iterable) before looking, so decide between no pieces and one empty piece
	// example wanted: three items with every item marked, maxsplit 0: no split is allowed, so all items stay in one piece
	// example wanted: one item marked split_before, maxsplit -1: the first item's mark is never consulted
	// example wanted: the docstring's items at maxsplit -2: any negative maxsplit counts down without reaching 1, so there is no limit
	// example wanted: the docstring's items at maxsplit 10, more than there are marks: the limit is never reached
	// example wanted: items with no marks after the first, maxsplit -1: the whole input is one piece
```