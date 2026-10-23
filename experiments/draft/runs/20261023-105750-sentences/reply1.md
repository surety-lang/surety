```surety
// draft
// drafted
type Cell = { x: Int, y: Int, value: Int }
// drafted
type Row = { x: Int, cells: Cell[] }

outer_product(xs: Int[], ys: Int[]) -> Row[]
	note "apply func, with any extra args and kwargs passed through to it, to every pair of an element of xs and an element of ys, one row per element of xs and one cell per element of ys; func and its extra arguments are a sibling obligation's, named here as func_value"
	# The rows' x values are exactly the elements of xs, in the order of xs, a duplicate in xs giving a row of its own.

	# In every row, the cells' y values are exactly the elements of ys, in the order of ys.

	# Every cell carries the x of the row it is in.

	# Every cell's value is what func gives for that cell's x and y, as the sibling obligation for func decides.

	// example wanted: xs [1, 2, 3], ys [1, 2, 3, 4, 5] with func as multiplication: the docstring's multiplication table, three rows of five cells
	// example wanted: xs empty, ys non-empty: the output has no rows
	// example wanted: xs [1, 2], ys empty: the docstring promises len(xs) rows of zero columns, while the implementation batches by len(ys); which of the two holds decides it
	// example wanted: xs [2, 2], ys [3]: a repeated element of xs gives a repeated row, not a merged one
	// example wanted: xs [1, 2], ys [5, 7] with a func that is not symmetric, such as subtraction: x is the first argument and y the second, so swapping them changes every off-diagonal cell
```