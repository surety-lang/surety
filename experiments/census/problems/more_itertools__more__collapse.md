more_itertools.more.collapse (from more-itertools 11.2.1)

Signature, in words: `collapse` takes the required argument `iterable`; the optional arguments `base_type` (default none), `levels` (default none).

The docstring:

Flatten an iterable with multiple levels of nesting (e.g., a list of
lists of tuples) into non-iterable types.

    >>> iterable = [(1, 2), ([3, 4], [[5], [6]])]
    >>> list(collapse(iterable))
    [1, 2, 3, 4, 5, 6]

Binary and text strings are not considered iterable and
will not be collapsed.

To avoid collapsing other types, specify *base_type*:

    >>> iterable = ['ab', ('cd', 'ef'), ['gh', 'ij']]
    >>> list(collapse(iterable, base_type=tuple))
    ['ab', ('cd', 'ef'), 'gh', 'ij']

Specify *levels* to stop flattening after a certain level:

>>> iterable = [('a', ['b']), ('c', ['d'])]
>>> list(collapse(iterable))  # Fully flattened
['a', 'b', 'c', 'd']
>>> list(collapse(iterable, levels=1))  # Only one level flattened
['a', ['b'], 'c', ['d']]
