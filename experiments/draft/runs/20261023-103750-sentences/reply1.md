```surety
// draft
// drafted
type Window = { slots: Option<Int>[] }

stagger(iterable: Int[], offsets: Int[], longest: Bool, fillvalue: Option<Int>) -> Window[]
	note "yield windows over the iterable whose i-th slot is offset from the window's position by the i-th offset, padding with the fill value"
	# Every window has exactly as many slots as there are offsets.

	# In the window at position k, the slot for an offset holds the item at position k plus that offset whenever that position lies within the iterable.

	# In the window at position k, the slot for an offset holds the fill value whenever position k plus that offset falls before the start of the iterable or at or past its end.

	# Without longest, if the output is not empty, every offset plus the output's length is at most the item count.

	# Without longest, if the output is not empty, some offset plus the output's length equals the item count.

	# Without longest, the output is empty exactly when there are no offsets or some offset is at least the item count.

	# With longest, if the output is not empty, every offset plus the output's length is at least the item count.

	# With longest, if the output is not empty, some offset plus the output's length equals the item count.

	# With longest, the output is empty exactly when there are no offsets or every offset is at least the item count.

	// example wanted: iterable [0, 1, 2, 3], offsets [-1, 0, 1], not longest, fill None: the -1 offset puts the fill in the first window's first slot, and the +1 offset ends the output
	// example wanted: iterable [0, 1, 2, 3], offsets [-1, 0, 1], longest, fill None: the -1 offset decides the length, so the trailing windows are padded
	// example wanted: iterable 0 through 7, offsets [0, 2, 4], not longest: the largest offset, 4, decides how many windows there are
	// example wanted: empty iterable, offsets [-1, 0, 1], longest: the negative offset still gives the output a window, and every slot in it is the fill
	// example wanted: empty iterable, offsets [-1, 0, 1], not longest: the offset 0 is already at the item count
	// example wanted: iterable [0, 1], offsets [], longest and not longest: with no offsets nothing is yielded
	// example wanted: iterable [0, 1, 2], offsets [5], not longest: the only offset is past the end
	// example wanted: iterable [0, 1, 2], offsets [1, -1], longest: the offsets are unsorted, so the smallest offset decides the length and not the first one
	// example wanted: iterable [0, 1, 2], offsets [0, 0], not longest: repeated offsets give each window two equal slots
	// example wanted: iterable [7, 8], offsets [-1, 0], not longest, fill 7: the fill value equals an item, so the slot does not show where it came from
```