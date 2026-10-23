more_itertools.more.circular_shifts (from more-itertools 11.2.1)

Signature, in words: `circular_shifts` takes the required argument `iterable`; the optional argument `steps` (default `1`).

The docstring:

Yield the circular shifts of *iterable*.

>>> list(circular_shifts(range(4)))
[(0, 1, 2, 3), (1, 2, 3, 0), (2, 3, 0, 1), (3, 0, 1, 2)]

Set *steps* to the number of places to rotate to the left
(or to the right if negative).  Defaults to 1.

>>> list(circular_shifts(range(4), 2))
[(0, 1, 2, 3), (2, 3, 0, 1)]

>>> list(circular_shifts(range(4), -1))
[(0, 1, 2, 3), (3, 0, 1, 2), (2, 3, 0, 1), (1, 2, 3, 0)]
