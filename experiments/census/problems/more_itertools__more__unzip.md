more_itertools.more.unzip (from more-itertools 11.2.1)

Signature, in words: `unzip` takes the required argument `iterable`.

The docstring:

The inverse of :func:`zip`, this function disaggregates the elements
of the zipped *iterable*.

The ``i``-th iterable contains the ``i``-th element from each element
of the zipped iterable. The first element is used to determine the
length of the remaining elements.

    >>> iterable = [('a', 1), ('b', 2), ('c', 3), ('d', 4)]
    >>> letters, numbers = unzip(iterable)
    >>> list(letters)
    ['a', 'b', 'c', 'd']
    >>> list(numbers)
    [1, 2, 3, 4]

This is similar to using ``zip(*iterable)``, but it avoids reading
*iterable* into memory. Note, however, that this function uses
:func:`itertools.tee` and thus may require significant storage.
