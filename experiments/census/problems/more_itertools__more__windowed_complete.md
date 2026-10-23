more_itertools.more.windowed_complete (from more-itertools 11.2.1)

Signature, in words: `windowed_complete` takes the required arguments `iterable`, `n`.

The docstring:

Yield ``(beginning, middle, end)`` tuples, where:

* Each ``middle`` has *n* items from *iterable*
* Each ``beginning`` has the items before the ones in ``middle``
* Each ``end`` has the items after the ones in ``middle``

>>> iterable = range(7)
>>> n = 3
>>> for beginning, middle, end in windowed_complete(iterable, n):
...     print(beginning, middle, end)
() (0, 1, 2) (3, 4, 5, 6)
(0,) (1, 2, 3) (4, 5, 6)
(0, 1) (2, 3, 4) (5, 6)
(0, 1, 2) (3, 4, 5) (6,)
(0, 1, 2, 3) (4, 5, 6) ()

Note that *n* must be at least 0 and most equal to the length of
*iterable*.

This function will exhaust the iterable and may require significant
storage.
