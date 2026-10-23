more_itertools.more.value_chain (from more-itertools 11.2.1)

Signature, in words: `value_chain` takes any number of positional arguments `args`.

The docstring:

Yield all arguments passed to the function in the same order in which
they were passed. If an argument itself is iterable then iterate over its
values.

    >>> list(value_chain(1, 2, 3, [4, 5, 6]))
    [1, 2, 3, 4, 5, 6]

Binary and text strings are not considered iterable and are emitted
as-is:

    >>> list(value_chain('12', '34', ['56', '78']))
    ['12', '34', '56', '78']

Pre- or postpend a single element to an iterable:

    >>> list(value_chain(1, [2, 3, 4, 5, 6]))
    [1, 2, 3, 4, 5, 6]
    >>> list(value_chain([1, 2, 3, 4, 5], 6))
    [1, 2, 3, 4, 5, 6]

Multiple levels of nesting are not flattened.
