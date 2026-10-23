```surety
# The rows' x values are exactly the elements of xs, in the order of xs, a duplicate in xs giving a row of its own.
	out.x == xs
# In every row, the cells' y values are exactly the elements of ys, in the order of ys.
	all r in out: r.cells.y == ys
# Every cell carries the x of the row it is in.
	all r in out: all c in r.cells: c.x == r.x
# Every cell's value is what func gives for that cell's x and y, as the sibling obligation for func decides.
	all r in out: all c in r.cells: c.value == func_value(c.x, c.y)
```