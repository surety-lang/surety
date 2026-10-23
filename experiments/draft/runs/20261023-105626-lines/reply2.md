```surety
# The count error is reported exactly when max_count is given and is less than one.
	(max_count != None and max_count < 1) <-> any Err(e) in out: e == count_error
# The oversize error is reported exactly when max_count is absent or at least one, strict is true, and some item's size is more than max_size.
	all i in items where i.size > max_size:
		(max_count == None or max_count >= 1) and strict <-> any Err(e) in out: e == oversize_error
	else none Err(e) in out: e == oversize_error
# The output is a list of batches exactly when neither error is reported.
	all Err(e) in out: e == count_error or e == oversize_error
# The batches' items, taken batch by batch, are the input items in their input order, with none dropped, none added and none repeated.
	all Ok(bs) in out: flatten(bs.items) == items
# No batch is empty.
	all Ok(bs) in out: all b in bs: len(b.items) >= 1
# Every batch with more than one item has a combined size of at most max_size.
	all Ok(bs) in out: all b in bs where len(b.items) > 1: sum(b.items.size) <= max_size
# If max_count is given, every batch has at most max_count items.
	all Ok(bs) in out: all b in bs: max_count == None or len(b.items) <= max_count
# For every batch except the last, the next batch's first item would not have fit in it: the batch's combined size plus that item's size is more than max_size, or max_count is given and the batch already has max_count items.
	all Ok(bs) in out: all b in bs: all c in bs where b before c in bs: all f in c.items[0 .. 1]: sum(b.items.size) + f.size > max_size or (max_count != None and len(b.items) == max_count) or any d in bs: b before d in bs and d before c in bs
```