```surety
# Every window has exactly as many slots as there are offsets.
	all w in out: len(w.slots) == len(offsets)

# In the window at position k, the slot for an offset holds the item at position k plus that offset whenever that position lies within the iterable.
	reaching for: a quantifier over positions and indexing by position, so I can say: for every window position k in out and every offset position i, when 0 <= k + offsets[i] < len(iterable), the i-th slot of out[k] holds iterable[k + offsets[i]]

# In the window at position k, the slot for an offset holds the fill value whenever position k plus that offset falls before the start of the iterable or at or past its end.
	reaching for: a quantifier over positions and indexing by position, so I can say: for every window position k in out and every offset position i, when k + offsets[i] < 0 or k + offsets[i] >= len(iterable), the i-th slot of out[k] equals fillvalue

# Without longest, if the output is not empty, every offset plus the output's length is at most the item count.
	not longest and len(out) > 0 -> all d in offsets: d + len(out) <= len(iterable)

# Without longest, if the output is not empty, some offset plus the output's length equals the item count.
	not longest and len(out) > 0 -> any d in offsets: d + len(out) == len(iterable)

# Without longest, the output is empty exactly when there are no offsets or some offset is at least the item count.
	not longest and len(out) == 0 <-> not longest and len(offsets) == 0 or not longest and any d in offsets: d >= len(iterable)

# With longest, if the output is not empty, every offset plus the output's length is at least the item count.
	longest and len(out) > 0 -> all d in offsets: d + len(out) >= len(iterable)

# With longest, if the output is not empty, some offset plus the output's length equals the item count.
	longest and len(out) > 0 -> any d in offsets: d + len(out) == len(iterable)

# With longest, the output is empty exactly when there are no offsets or every offset is at least the item count.
	longest and len(out) == 0 <-> longest and len(offsets) == 0 or longest and all d in offsets: d >= len(iterable)
```