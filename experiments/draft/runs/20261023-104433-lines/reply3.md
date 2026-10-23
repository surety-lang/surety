```surety
# The output is the steps error exactly when steps is 0, and is a list of shifts otherwise.
	(steps == 0 and out == Err(steps_error)) or steps != 0 and exactly_one Ok(_) in out
# Every shift has the same elements as the input.
	all Ok(shifts) in out: all s in shifts: s.items same_elements_as iterable
# The list of shifts is empty exactly when the input is empty.
	all Ok(shifts) in out: len(shifts) == 0 <-> len(iterable) == 0
# When the input is not empty, the first shift is the input in its own order.
	all Ok(shifts) in out: len(iterable) > 0 -> any f in shifts[0 .. 1]: f.items == iterable
# Each shift after the first is the shift before it rotated by steps places, to the left when steps is positive and to the right when it is negative, where a sibling obligation decides the rotation.
	all Ok(shifts) in out: all p in shifts: all q in shifts where p before q in shifts: rotated(p.items, q.items, steps) or any r in shifts: p before r in shifts and r before q in shifts
# The last shift rotated by steps places is the first shift.
	all Ok(shifts) in out: all l in shifts[len(shifts) - 1 .. len(shifts)]: all f in shifts[0 .. 1]: rotated(l.items, f.items, steps)
# There are never more shifts than the input has elements.
	all Ok(shifts) in out: len(shifts) <= len(iterable)
```