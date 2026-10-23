more_itertools.more.count_cycle (from more-itertools 11.2.1)

Signature, in words: `count_cycle` takes the required argument `iterable`; the optional argument `n` (default none).

The docstring:

Cycle through the items from *iterable* up to *n* times, yielding
the number of completed cycles along with each item. If *n* is omitted the
process repeats indefinitely.

>>> list(count_cycle('AB', 3))
[(0, 'A'), (0, 'B'), (1, 'A'), (1, 'B'), (2, 'A'), (2, 'B')]
