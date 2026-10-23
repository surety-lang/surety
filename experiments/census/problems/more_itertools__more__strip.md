more_itertools.more.strip (from more-itertools 11.2.1)

Signature, in words: `strip` takes the required arguments `iterable`, `pred`.

The docstring:

Yield the items from *iterable*, but strip any from the
beginning and end for which *pred* returns ``True``.

For example, to remove a set of items from both ends of an iterable:

    >>> iterable = (None, False, None, 1, 2, None, 3, False, None)
    >>> pred = lambda x: x in {None, False, ''}
    >>> list(strip(iterable, pred))
    [1, 2, None, 3]

This function is analogous to :func:`str.strip`.
