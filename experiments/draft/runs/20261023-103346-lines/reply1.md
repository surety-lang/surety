```surety
# The output is Ok exactly when lengths is None, or lengths has one entry per iterable and each entry equals the length of the iterable at its position.
	reaching for: out is Ok iff lengths is None, or the list inside a given lengths has len equal to len(iterables) and, at every position i, its entry i equals len(iterables[i]). Two pieces are missing. No binder takes the Int[] out of a given Option (no Some pattern beside Ok and Err), and nothing pairs two lists by position: there is no index binder and no zip, and a slice needs a position that no quantifier supplies.
# The output is the count error exactly when lengths is given and its number of entries differs from the number of iterables.
	reaching for: (lengths is given and len(the list inside it) != len(iterables)) <-> any Err(e) in out: e == count_error. The missing piece is a binder for the Int[] inside a given Option, or a defined len on Option<Int[]>; neither is in the notation.
# The output is the length error exactly when lengths is given with one entry per iterable and some entry differs from the length of the iterable at its position.
	reaching for: (lengths is given, len(the list inside it) == len(iterables), and at some position i its entry i != len(iterables[i])) <-> any Err(e) in out: e == length_error. This needs the same two missing pieces as the Ok sentence: unwrapping a given Option, and pairing lengths with iterables by position.
# An Ok output has the same elements as all the iterables taken together.
	all Ok(res) in out: res same_elements_as flatten(iterables)
# An Ok output keeps each iterable's elements in that iterable's order.
	all Ok(res) in out: all it in iterables: it same_order_as res
# A non-empty Ok output starts with the first element of the longest iterable, the earliest-listed one when several share the longest length.
	all Ok(res) in out: all lead in iterables where len(res) > 0 and len(lead) > 0 and res[0 .. 1] != lead[0 .. 1]: any o in iterables: len(o) > len(lead) or (len(o) == len(lead) and o before lead in iterables)
# In an Ok output, between any two consecutive elements of an iterable other than that leading longest one, some element of the leading iterable comes.
	reaching for: for every iterable occurrence that is not the leader (the leader is the longest, and the earliest-listed among the longest), and every pair a, b consecutive in it, some element of the leader sits strictly between a's and b's places in res. Three pieces are missing. First, naming the leader takes a quantifier inside a where clause, or an abbreviation that names a list rather than a boolean, and either needs more than one line. Second, a and b must be placed at their own occurrences in res. Third, `before` on values resolves each one to its first occurrence in res, and an Id can repeat across iterables, so that occurrence may belong to another iterable.
```