more_itertools.more.gray_product (from more-itertools 11.2.1)

Signature, in words: `gray_product` takes any number of positional arguments `iterables`; the keyword-only argument `repeat` (default `1`).

The docstring:

Like :func:`itertools.product`, but return tuples in an order such
that only one element in the generated tuple changes from one iteration
to the next.

    >>> list(gray_product('AB','CD'))
    [('A', 'C'), ('B', 'C'), ('B', 'D'), ('A', 'D')]

The *repeat* keyword argument specifies the number of repetitions
of the iterables.  For example, ``gray_product('AB', repeat=3)`` is
equivalent to ``gray_product('AB', 'AB', 'AB')``.

This function consumes all of the input iterables before producing output.
If any of the input iterables have fewer than two items, ``ValueError``
is raised.

For information on the algorithm, see
`this section <https://www-cs-faculty.stanford.edu/~knuth/fasc2a.ps.gz>`__
of Donald Knuth's *The Art of Computer Programming*.
