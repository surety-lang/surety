```surety
# The output is an error exactly when window_size is less than 1.
	window_size < 1 <-> exactly_one Err(_) in out
# Every error output is the window size error.
	all Err(e) in out: e == window_size_error
# When no item's window matches, or count is at most 0, the output is the items' values in their order.
	all Ok(vs) in out: (true not in items.window_matches or (count != None and count <= 0)) -> vs == items.value
# When the first item's window does not match, or count is at most 0, the output starts with the first item's value and continues as replace over the items after it, with the same substitutes, count and window size.
	all Ok(vs) in out: (false in items[0 .. 1].window_matches or (count != None and count <= 0)) -> vs[0 .. 1] == items[0 .. 1].value and exactly_one Ok(rest) in replace(items[1 .. len(items)], substitutes, count, window_size): vs[1 .. len(vs)] == rest
# When the first item's window matches and count is None or more than 0, the output starts with the substitutes.
	all Ok(vs) in out: (true in items[0 .. 1].window_matches and (count == None or count > 0)) -> vs[0 .. len(substitutes)] == substitutes
# When the first item's window matches and count is None or more than 0, the output after the substitutes is replace over the items from position window_size on, with the same substitutes and window size and with one fewer replacement allowed.
	reaching for: when the first item's window matches and count is None or above 0, the rest of the Ok output after the substitutes, vs[len(substitutes) .. len(vs)], equals the Ok of replace(items[window_size .. len(items)], substitutes, <one fewer>, window_size). Here <one fewer> is None when count is None and count - 1 otherwise. The notation has no Some pattern to unwrap an Option<Int> and no arithmetic on one. The two cases would need two recursive-call quantifiers, and only one quantifier can be the last operand on a line.
# Every output value is the value of some item or one of the substitutes.
	all Ok(vs) in out: all v in vs: v in items.value or v in substitutes
```