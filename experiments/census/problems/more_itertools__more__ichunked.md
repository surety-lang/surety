more_itertools.more.ichunked (from more-itertools 11.2.1)

Signature, in words: `ichunked` takes the required arguments `iterable`, `n`.

The docstring:

Break *iterable* into sub-iterables with *n* elements each.
:func:`ichunked` is like :func:`chunked`, but it yields iterables
instead of lists.

If the sub-iterables are read in order, the elements of *iterable*
won't be stored in memory.
If they are read out of order, :func:`itertools.tee` is used to cache
elements as necessary.

>>> from itertools import count
>>> all_chunks = ichunked(count(), 4)
>>> c_1, c_2, c_3 = next(all_chunks), next(all_chunks), next(all_chunks)
>>> list(c_2)  # c_1's elements have been cached; c_3's haven't been
[4, 5, 6, 7]
>>> list(c_1)
[0, 1, 2, 3]
>>> list(c_3)
[8, 9, 10, 11]
