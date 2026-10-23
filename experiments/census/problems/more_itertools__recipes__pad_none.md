more_itertools.recipes.pad_none (from itertools recipes (more_itertools.recipes) 11.2.1)

Signature, in words: `pad_none` takes the required argument `iterable`.

The docstring:

Returns the sequence of elements and then returns ``None`` indefinitely.

    >>> take(5, pad_none(range(3)))
    [0, 1, 2, None, None]

Useful for emulating the behavior of the built-in :func:`map` function.

See also :func:`padded`.
