```surety
# The output is Ok exactly when every call either returned a value or raised an exception that is among the given exceptions.
	exactly_one Ok(_) in out <-> all c in calls: none Err(e) in c.result: e not in exceptions
# An Err output names the exception raised by the first call whose exception is not among the given exceptions.
	all Err(x) in out: any c in calls: any Err(ce) in c.result where ce not in exceptions: ce == x and none d in calls where d before c in calls: any Err(de) in d.result: de not in exceptions
# An Ok output is drawn from the values the calls returned.
	reaching for: `all Ok(vs) in out: vs drawn_from returned`, where `returned` is the list of the Ok-side values of `calls.result` in call order. The notation has no form that turns a list of Results into the list of one side's values. Membership (`all v in vs: any c in calls: any Ok(r) in c.result: r == v`) would drop the multiplicity that drawn_from carries.
# No value a call returned is left out of an Ok output.
	reaching for: `all Ok(vs) in out: returned drawn_from vs`, with each returned occurrence needing its own place in vs, so two calls returning the same value need two copies. This needs the same missing list of the Ok-side values of `calls.result`. A membership line (`all c in calls: all Ok(r) in c.result: r in vs`) would let one copy stand for both.
# An Ok output keeps the order of the calls that returned its values.
	reaching for: `all Ok(vs) in out: vs same_order_as returned`, where `returned` is the Ok-side values of `calls.result` in call order. Without that list, a pairwise `before` line cannot say which call each element of vs came from when values repeat.
```