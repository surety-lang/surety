```surety
# The output has exactly as many elements as the input has items.
	len(out) == len(items)

# Mapping the input with its first item left off gives the output with its first element left off.
	map_if(items[1 .. len(items)], else_given) == out[1 .. len(out)]

# If pred holds of the input's first item, the output's first element is func of that item.
	all x in items[0 .. 1] where pred(x): any y in out[0 .. 1]: y == func(x)

# If pred does not hold of the input's first item and func_else is given, the output's first element is func_else of that item.
	all x in items[0 .. 1] where not pred(x) and else_given: any y in out[0 .. 1]: y == func_else(x)

# If pred does not hold of the input's first item and no func_else is given, the output's first element is that item unchanged.
	all x in items[0 .. 1] where not pred(x) and not else_given: any y in out[0 .. 1]: y == x
```