more_itertools.more.chunked_even (from more-itertools 11.2.1)

Signature, in words: `chunked_even` takes the required arguments `iterable`, `n`.

The docstring:

Break *iterable* into lists of approximately length *n*.
Items are distributed such the lengths of the lists differ by at most
1 item.

>>> iterable = [1, 2, 3, 4, 5, 6, 7]
>>> n = 3
>>> list(chunked_even(iterable, n))  # List lengths: 3, 2, 2
[[1, 2, 3], [4, 5], [6, 7]]
>>> list(chunked(iterable, n))  # List lengths: 3, 3, 1
[[1, 2, 3], [4, 5, 6], [7]]
