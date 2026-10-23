more_itertools.recipes.prepend (from itertools recipes (more_itertools.recipes) 11.2.1)

Signature, in words: `prepend` takes the required arguments `value`, `iterable`.

The docstring:

Yield *value*, followed by the elements in *iterable*.

    >>> value = '0'
    >>> iterable = ['1', '2', '3']
    >>> list(prepend(value, iterable))
    ['0', '1', '2', '3']

To prepend multiple values, see :func:`itertools.chain`
or :func:`value_chain`.
