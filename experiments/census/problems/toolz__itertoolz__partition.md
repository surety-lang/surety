toolz.itertoolz.partition (from toolz 1.2.0)

Signature, in words: `partition` takes the required arguments `n`, `seq`; the optional argument `pad` (default `'__no__pad__'`).

The docstring:

Partition sequence into tuples of length n

>>> list(partition(2, [1, 2, 3, 4]))
[(1, 2), (3, 4)]

If the length of ``seq`` is not evenly divisible by ``n``, the final tuple
is dropped if ``pad`` is not specified, or filled to length ``n`` by pad:

>>> list(partition(2, [1, 2, 3, 4, 5]))
[(1, 2), (3, 4)]

>>> list(partition(2, [1, 2, 3, 4, 5], pad=None))
[(1, 2), (3, 4), (5, None)]

See Also:
    partition_all
