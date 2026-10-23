more_itertools.recipes.reshape (from itertools recipes (more_itertools.recipes) 11.2.1)

Signature, in words: `reshape` takes the required arguments `matrix`, `shape`.

The docstring:

Change the shape of a *matrix*.

If *shape* is an integer, the matrix must be two dimensional
and the shape is interpreted as the desired number of columns:

    >>> matrix = [(0, 1), (2, 3), (4, 5)]
    >>> cols = 3
    >>> list(reshape(matrix, cols))
    [(0, 1, 2), (3, 4, 5)]

If *shape* is a tuple (or other iterable), the input matrix can have
any number of dimensions. It will first be flattened and then rebuilt
to the desired shape which can also be multidimensional:

    >>> matrix = [(0, 1), (2, 3), (4, 5)]    # Start with a 3 x 2 matrix

    >>> list(reshape(matrix, (2, 3)))        # Make a 2 x 3 matrix
    [(0, 1, 2), (3, 4, 5)]

    >>> list(reshape(matrix, (6,)))          # Make a vector of length six
    [0, 1, 2, 3, 4, 5]

    >>> list(reshape(matrix, (2, 1, 3, 1)))  # Make 2 x 1 x 3 x 1 tensor
    [(((0,), (1,), (2,)),), (((3,), (4,), (5,)),)]

Each dimension is assumed to be uniform, either all arrays or all scalars.
Flattening stops when the first value in a dimension is a scalar.
Scalars are bytes, strings, and non-iterables.
The reshape iterator stops when the requested shape is complete
or when the input is exhausted, whichever comes first.
