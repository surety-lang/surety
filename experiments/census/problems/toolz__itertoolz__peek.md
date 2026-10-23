toolz.itertoolz.peek (from toolz 1.2.0)

Signature, in words: `peek` takes the required argument `seq`.

The docstring:

Retrieve the next element of a sequence

Returns the first element and an iterable equivalent to the original
sequence, still having the element retrieved.

>>> seq = [0, 1, 2, 3, 4]
>>> first, seq = peek(seq)
>>> first
0
>>> list(seq)
[0, 1, 2, 3, 4]
