```surety
# The output is None exactly when the list of costs is empty.
	out == None <-> len(costs) == 0
# Every index the output gives is a position in the list: at least 0 and less than the number of costs.
	reaching for: a binder for the value inside an Option. The binder patterns are Ok and Err for a Result and none for Option's present side, so the index cannot be named. Wanted: all Some(i) in out: 0 <= i and i < len(costs)
# No cost in the list is less than the cost at the output position.
	reaching for: the same Option binder, with the one-element slice standing in for indexing. Wanted: all Some(i) in out: all c in costs: all m in costs[i .. i + 1]: c >= m
# Every cost before the output position is greater than the cost at the output position.
	reaching for: the same Option binder, with slices for "before the position" and "at the position". Wanted: all Some(i) in out: all c in costs[0 .. i]: all m in costs[i .. i + 1]: c > m
```