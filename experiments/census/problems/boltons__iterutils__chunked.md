boltons.iterutils.chunked (from boltons 26.2.0)

Signature, in words: `chunked` takes the required arguments `src`, `size`; the optional argument `count` (default none); any keyword arguments `kw`.

The docstring:

Returns a list of *count* chunks, each with *size* elements,
generated from iterable *src*. If *src* is not evenly divisible by
*size*, the final chunk will have fewer than *size* elements.
Provide the *fill* keyword argument to provide a pad value and
enable padding, otherwise no padding will take place.

>>> chunked(range(10), 3)
[[0, 1, 2], [3, 4, 5], [6, 7, 8], [9]]
>>> chunked(range(10), 3, fill=None)
[[0, 1, 2], [3, 4, 5], [6, 7, 8], [9, None, None]]
>>> chunked(range(10), 3, count=2)
[[0, 1, 2], [3, 4, 5]]

See :func:`chunked_iter` for more info.
