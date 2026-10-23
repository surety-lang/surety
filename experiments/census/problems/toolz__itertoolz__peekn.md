toolz.itertoolz.peekn (from toolz 1.2.0)

Signature, in words: `peekn` takes the required arguments `n`, `seq`.

The docstring:

Retrieve the next n elements of a sequence

Returns a tuple of the first n elements and an iterable equivalent
to the original, still having the elements retrieved.

>>> seq = [0, 1, 2, 3, 4]
>>> first_two, seq = peekn(2, seq)
>>> first_two
(0, 1)
>>> list(seq)
[0, 1, 2, 3, 4]
