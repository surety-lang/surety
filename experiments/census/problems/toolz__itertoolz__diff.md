toolz.itertoolz.diff (from toolz 1.2.0)

Signature, in words: `diff` takes any number of positional arguments `seqs`; any keyword arguments `kwargs`.

The docstring:

Return those items that differ between sequences

>>> list(diff([1, 2, 3], [1, 2, 10, 100]))
[(3, 10)]

Shorter sequences may be padded with a ``default`` value:

>>> list(diff([1, 2, 3], [1, 2, 10, 100], default=None))
[(3, 10), (None, 100)]

A ``key`` function may also be applied to each item to use during
comparisons:

>>> list(diff(['apples', 'bananas'], ['Apples', 'Oranges'], key=str.lower))
[('bananas', 'Oranges')]
