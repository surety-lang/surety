```surety
# Every member of every subset is an element of the iterable.
	all s in out: all m in s.members: m in iterable
# No subset has the same member twice, and a subset's members are not claimed in any order.
	all s in out: unique s.members and s.members unordered
# No two subsets in the output have the same members.
	all a in out: exactly_one b in out: b.members same_elements_as a.members
# Some subset in the output has no members.
	any s in out: len(s.members) == 0
# For every subset in the output and every element of the iterable that is not one of its members, some subset in the output has exactly that subset's members plus that element.
	all s in out: all x in iterable where x not in s.members: any t in out: x in t.members and s.members drawn_from t.members and len(t.members) == len(s.members) + 1
# Every subset's frozen flag is baseset_frozen.
	all s in out: s.frozen == baseset_frozen
# A subset with more members never comes before a subset with fewer members.
	all a in out: all b in out where len(a.members) > len(b.members): not (a before b in out)
```