more_itertools.more.partitions (from more-itertools 11.2.1)

Signature, in words: `partitions` takes the required argument `iterable`.

The docstring:

Yield all possible order-preserving partitions of *iterable*.

>>> iterable = 'abc'
>>> for part in partitions(iterable):
...     print([''.join(p) for p in part])
['abc']
['a', 'bc']
['ab', 'c']
['a', 'b', 'c']

This is unrelated to :func:`partition`.
