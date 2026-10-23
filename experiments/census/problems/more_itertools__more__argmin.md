more_itertools.more.argmin (from more-itertools 11.2.1)

Signature, in words: `argmin` takes the required argument `iterable`; the keyword-only argument `key` (default none).

The docstring:

Index of the first occurrence of a minimum value in an iterable.

    >>> argmin('efghabcdijkl')
    4
    >>> argmin([3, 2, 1, 0, 4, 2, 1, 0])
    3

For example, look up a label corresponding to the position
of a value that minimizes a cost function::

    >>> def cost(x):
    ...     "Days for a wound to heal given a subject's age."
    ...     return x**2 - 20*x + 150
    ...
    >>> labels =  ['homer', 'marge', 'bart', 'lisa', 'maggie']
    >>> ages =    [  35,      30,      10,      9,      1    ]

    # Fastest healing family member
    >>> labels[argmin(ages, key=cost)]
    'bart'

    # Age with fastest healing
    >>> min(ages, key=cost)
    10
