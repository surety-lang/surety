```surety
# The output is Ok exactly when every call either returned a value or raised an exception that is among the given exceptions.
	reaching for: the iff `exactly_one Ok(_) in out <-> none c in calls: any Err(e) in c.result: e not in exceptions`, but both sides are quantifiers and only one can be last on the line; the call side wants a named abbreviation like apply_diff's `checks`, which is a line of its own

# An Err output names the exception raised by the first call whose exception is not among the given exceptions.
	all Err(x) in out: any c in calls: any Err(e) in c.result where e not in exceptions: e == x and none d in calls where d before c in calls: any Err(f) in d.result: f not in exceptions

# An Ok output is drawn from the values the calls returned.
	reaching for: `all Ok(vs) in out: vs drawn_from <the values the calls returned>`, where that list is calls.result keeping only the Ok side, unwrapped, in call order; the fragment cannot build that list, and a per-element `any Ok(r) in calls.result: r == v` compares values, so it would admit a value repeated more often than the calls returned it

# No value a call returned is left out of an Ok output.
	reaching for: `all Ok(vs) in out: <the values the calls returned> drawn_from vs`, over the same list of unwrapped Ok values in call order; a per-value `all Ok(r) in calls.result: r in vs` would let one output 5 stand for two calls that each returned 5

# An Ok output keeps the order of the calls that returned its values.
	reaching for: `all Ok(vs) in out: vs same_order_as <the values the calls returned>`, over the same list of unwrapped Ok values in call order, which the fragment has no way to build from calls.result
```