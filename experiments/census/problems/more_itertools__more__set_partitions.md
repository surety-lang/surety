more_itertools.more.set_partitions (from more-itertools 11.2.1)

Signature, in words: `set_partitions` takes the required argument `iterable`; the optional arguments `k` (default none), `min_size` (default none), `max_size` (default none).

The docstring:

Yield the set partitions of *iterable* into *k* parts. Set partitions are
not order-preserving.

>>> iterable = 'abc'
>>> for part in set_partitions(iterable, 2):
...     print([''.join(p) for p in part])
['a', 'bc']
['ab', 'c']
['b', 'ac']


If *k* is not given, every set partition is generated.

>>> iterable = 'abc'
>>> for part in set_partitions(iterable):
...     print([''.join(p) for p in part])
['abc']
['a', 'bc']
['ab', 'c']
['b', 'ac']
['a', 'b', 'c']

if *min_size* and/or *max_size* are given, the minimum and/or maximum size
per block in partition is set.

>>> iterable = 'abc'
>>> for part in set_partitions(iterable, min_size=2):
...     print([''.join(p) for p in part])
['abc']
>>> for part in set_partitions(iterable, max_size=2):
...     print([''.join(p) for p in part])
['a', 'bc']
['ab', 'c']
['b', 'ac']
['a', 'b', 'c']
