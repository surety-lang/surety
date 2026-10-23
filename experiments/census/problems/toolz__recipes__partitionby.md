toolz.recipes.partitionby (from toolz 1.2.0)

Signature, in words: `partitionby` takes the required arguments `func`, `seq`.

The docstring:

Partition a sequence according to a function

Partition `s` into a sequence of lists such that, when traversing
`s`, every time the output of `func` changes a new list is started
and that and subsequent items are collected into that list.

>>> is_space = lambda c: c == " "
>>> list(partitionby(is_space, "I have space"))
[('I',), (' ',), ('h', 'a', 'v', 'e'), (' ',), ('s', 'p', 'a', 'c', 'e')]

>>> is_large = lambda x: x > 10
>>> list(partitionby(is_large, [1, 2, 1, 99, 88, 33, 99, -1, 5]))
[(1, 2, 1), (99, 88, 33, 99), (-1, 5)]

See also:
    partition
    groupby
    itertools.groupby
