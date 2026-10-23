toolz.itertoolz.partition_all (from toolz 1.2.0)

Signature, in words: `partition_all` takes the required arguments `n`, `seq`.

The docstring:

Partition all elements of sequence into tuples of length at most n

The final tuple may be shorter to accommodate extra elements.

>>> list(partition_all(2, [1, 2, 3, 4]))
[(1, 2), (3, 4)]

>>> list(partition_all(2, [1, 2, 3, 4, 5]))
[(1, 2), (3, 4), (5,)]

See Also:
    partition
