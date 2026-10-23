```surety
# The output is drawn from the input elements' results.
	reaching for: `out drawn_from` the list of the Int inside each non-None result in `iterable.result`, kept in input order and with duplicates kept, so `out` (Int) and the results (Option<Int>) are lists of one element type. `drawn_from` refuses Int against Option<Int>, and the notation has no binder pattern for Option the way `Ok(x)` matches a Result.
# Every input element whose result is left out of the output has a result of None.
	reaching for: `all a leftover in iterable: a.result == None`, where `leftover` takes away, for each Int in `out`, the earliest unremoved input element whose result holds that Int. Today `leftover` matches by value against `out`'s Int elements, so it cannot match them against Applied or Option<Int> elements. `a.result != None -> a.result in out` loses the occurrence count: with two results of 5, it is met when only one 5 is kept.
# The output keeps the input's order of results.
	reaching for: `out same_order_as` the list of the Int inside each non-None result in `iterable.result`. It is the same unwrapping of Option<Int> to Int as the first sentence, and `same_order_as` refuses mismatched element types for the same reason `drawn_from` does.
```