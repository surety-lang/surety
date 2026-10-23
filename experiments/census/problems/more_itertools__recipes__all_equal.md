more_itertools.recipes.all_equal (from itertools recipes (more_itertools.recipes) 11.2.1)

Signature, in words: `all_equal` takes the required argument `iterable`; the optional argument `key` (default none).

The docstring:

Returns ``True`` if all the elements are equal to each other.

    >>> all_equal('aaaa')
    True
    >>> all_equal('aaab')
    False

A function that accepts a single argument and returns a transformed version
of each input item can be specified with *key*:

    >>> all_equal('AaaA', key=str.casefold)
    True
    >>> all_equal([1, 2, 3], key=lambda x: x < 10)
    True
