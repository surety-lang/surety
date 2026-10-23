more_itertools.more.product_index (from more-itertools 11.2.1)

Signature, in words: `product_index` takes the required argument `element`; any number of positional arguments `iterables`; the keyword-only argument `repeat` (default `1`).

The docstring:

Equivalent to ``list(product(*iterables, repeat=repeat)).index(tuple(element))``

The products of *iterables* can be ordered lexicographically.
:func:`product_index` computes the first index of *element* without
computing the previous products.

    >>> product_index([8, 2], range(10), range(5))
    42

The *repeat* keyword argument specifies the number of repetitions
of the iterables::

    >>> product_index([8, 0, 7], range(10), repeat=3)
    807

``ValueError`` will be raised if the given *element* isn't in the product
of *args*.
