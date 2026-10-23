toolz.itertoolz.sliding_window (from toolz 1.2.0)

Signature, in words: `sliding_window` takes the required arguments `n`, `seq`.

The docstring:

A sequence of overlapping subsequences

>>> list(sliding_window(2, [1, 2, 3, 4]))
[(1, 2), (2, 3), (3, 4)]

This function creates a sliding window suitable for transformations like
sliding means / smoothing

>>> mean = lambda seq: float(sum(seq)) / len(seq)
>>> list(map(mean, sliding_window(2, [1, 2, 3, 4])))
[1.5, 2.5, 3.5]
