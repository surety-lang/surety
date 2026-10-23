```surety
// draft
// drafted
type Arrangement = { items: Text[] }

derangements(iterable: Text[], r: Option<Int>) -> Arrangement[]
	note "yield every arrangement of r of the iterable's elements, or of all of them when r is not given, in which no element sits at its original index, with elements told apart by position and not by value"
	# Every derangement has r items when r is given, and as many items as the iterable when it is not.

	# Every derangement's items are drawn from the iterable: each item is an occurrence of an input element, and no occurrence is used twice.

	# No derangement places an input occurrence at its own index: the item at each index of a derangement comes from a different index of the iterable, even when the two have the same value.

	# Every arrangement of the right length drawn from the iterable that leaves no occurrence at its own index is in the output.

	# Each arrangement of input positions appears in the output exactly once, so two derangements may be equal by value when the iterable repeats a value.

	# The derangements come in lexicographic order of the input positions they draw from.

	// example wanted: ["Alice", "Bob", "Carol", "Dave"] with no r: the Secret Santa case the docstring prints, order included
	// example wanted: ["0", "1", "2"] with r 2: a partial derangement compares each index of the shorter tuple with the same index of the input
	// example wanted: ["Alice", "Bob", "Bob"] with no r: a "Bob" may sit at index 1 because it came from index 2; identity is by position
	// example wanted: ["a", "a", "b", "b"] with no r: one value arrangement is drawn from several position arrangements; the claim that each position arrangement appears once decides how many times it appears
	// example wanted: the empty iterable with no r: whether the one empty arrangement counts as a derangement (it has no index to fix)
	// example wanted: ["a", "b"] with r 0: the same question for the empty arrangement on a non-empty iterable
	// example wanted: ["a", "b"] with r 3: no arrangement of three items can be drawn from two occurrences
	// example wanted: ["a"] with no r: the only arrangement fixes its one element
	// example wanted: ["a", "b"] with r -1: Python's permutations raises here; the author decides whether this becomes an error case in a Result
```