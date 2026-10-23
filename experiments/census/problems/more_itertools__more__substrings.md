more_itertools.more.substrings (from more-itertools 11.2.1)

Signature, in words: `substrings` takes the required argument `iterable`.

The docstring:

Yield all of the substrings of *iterable*.

    >>> [''.join(s) for s in substrings('more')]
    ['m', 'o', 'r', 'e', 'mo', 'or', 're', 'mor', 'ore', 'more']

Note that non-string iterables can also be subdivided.

    >>> list(substrings([0, 1, 2]))
    [(0,), (1,), (2,), (0, 1), (1, 2), (0, 1, 2)]

Like subslices() but returns tuples instead of lists
and returns the shortest substrings first.
