more_itertools.more.chunked (from more-itertools 11.2.1)

Signature, in words: `chunked` takes the required arguments `iterable`, `n`; the optional argument `strict` (default `False`).

The docstring:

Break *iterable* into lists of length *n*:

    >>> list(chunked([1, 2, 3, 4, 5, 6], 3))
    [[1, 2, 3], [4, 5, 6]]

By the default, the last yielded list will have fewer than *n* elements
if the length of *iterable* is not divisible by *n*:

    >>> list(chunked([1, 2, 3, 4, 5, 6, 7, 8], 3))
    [[1, 2, 3], [4, 5, 6], [7, 8]]

To use a fill-in value instead, see the :func:`grouper` recipe.

If the length of *iterable* is not divisible by *n* and *strict* is
``True``, then ``ValueError`` will be raised before the last
list is yielded.
