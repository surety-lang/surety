more_itertools.more.all_unique (from more-itertools 11.2.1)

Signature, in words: `all_unique` takes the required argument `iterable`; the optional argument `key` (default none).

The docstring:

Returns ``True`` if all the elements of *iterable* are unique (no two
elements are equal).

    >>> all_unique('ABCB')
    False

If a *key* function is specified, it will be used to make comparisons.

    >>> all_unique('ABCb')
    True
    >>> all_unique('ABCb', str.lower)
    False

The function returns as soon as the first non-unique element is
encountered. Iterables with a mix of hashable and unhashable items can
be used, but the function will be slower for unhashable items.
