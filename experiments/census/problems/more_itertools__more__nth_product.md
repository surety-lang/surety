more_itertools.more.nth_product (from more-itertools 11.2.1)

Signature, in words: `nth_product` takes the required argument `index`; any number of positional arguments `iterables`; the keyword-only argument `repeat` (default `1`).

The docstring:

Equivalent to ``list(product(*iterables, repeat=repeat))[index]``.

The products of *iterables* can be ordered lexicographically.
:func:`nth_product` computes the product at sort position *index* without
computing the previous products.

    >>> nth_product(8, range(2), range(2), range(2), range(2))
    (1, 0, 0, 0)

The *repeat* keyword argument specifies the number of repetitions
of the iterables.  The above example is equivalent to::

    >>> nth_product(8, range(2), repeat=4)
    (1, 0, 0, 0)

``IndexError`` will be raised if the given *index* is invalid.
