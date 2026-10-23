```surety
// draft

locate(verdicts: Bool[]) -> Int[]
	note "yield the index of each position at which pred returned true, where verdicts is pred's answer on each item, or on each window when window_size is given, in order"
	# Every output index is a position in verdicts at which the verdict is true.

	# Every position in verdicts at which the verdict is true is in the output.

	# The output is sorted ascending.

	# No two output indexes are equal.

	// example wanted: verdicts empty; no position exists, so nothing can be located
	// example wanted: verdicts all false; no position holds a true verdict
	// example wanted: verdicts all true; every position from 0 to the last is located
	// example wanted: the docstring's [0, 1, 1, 0, 1, 0, 0] under bool, as verdicts [false, true, true, false, true, false, false]; positions are counted from 0
	// example wanted: the docstring's window case, 12 items with window_size 3, as one verdict per window starting at each position, true where the window is (1, 2, 3); the index is the window's starting position
```