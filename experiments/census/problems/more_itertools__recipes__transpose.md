more_itertools.recipes.transpose (from itertools recipes (more_itertools.recipes) 11.2.1)

Signature, in words: `transpose` takes the required argument `matrix`.

The docstring:

Swap the rows and columns of the input matrix.

>>> list(transpose([(1, 2, 3), (11, 22, 33)]))
[(1, 11), (2, 22), (3, 33)]

The caller should ensure that the dimensions of the input are compatible.
If the input is empty, no output will be produced.
