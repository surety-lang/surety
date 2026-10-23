```surety
# Every output index is a position in verdicts at which the verdict is true.
	all i in out: true in verdicts[i .. i + 1]
# Every position in verdicts at which the verdict is true is in the output.
	reaching for: a range over the positions of verdicts, 0 up to len(verdicts) - 1 (or the position of a binder's occurrence as an Int), so the line could say: all p in positions of verdicts where true in verdicts[p .. p + 1]: p in out
# The output is sorted ascending.
	all a in out: all b in out where a before b in out: a <= b
# No two output indexes are equal.
	unique out
```