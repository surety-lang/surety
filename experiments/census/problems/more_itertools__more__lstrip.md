more_itertools.more.lstrip (from more-itertools 11.2.1)

Signature, in words: `lstrip` takes the required arguments `iterable`, `pred`.

The docstring:

Yield the items from *iterable*, but strip any from the beginning
for which *pred* returns ``True``.

For example, to remove a set of items from the start of an iterable:

    >>> iterable = (None, False, None, 1, 2, None, 3, False, None)
    >>> pred = lambda x: x in {None, False, ''}
    >>> list(lstrip(iterable, pred))
    [1, 2, None, 3, False, None]

This function is analogous to :func:`str.lstrip`, and is essentially
a wrapper for :func:`itertools.dropwhile`.
