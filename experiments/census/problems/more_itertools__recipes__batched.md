more_itertools.recipes.batched (from itertools recipes (more_itertools.recipes) 11.2.1)

Signature, in words: `batched` takes the required arguments `iterable`, `n`; the keyword-only argument `strict` (default `False`).

The docstring:

Batch data into tuples of length *n*. If the number of items in
*iterable* is not divisible by *n*:
* The last batch will be shorter if *strict* is ``False``.
* :exc:`ValueError` will be raised if *strict* is ``True``.

>>> list(batched('ABCDEFG', 3))
[('A', 'B', 'C'), ('D', 'E', 'F'), ('G',)]

On Python 3.13 and above, this is an alias for :func:`itertools.batched`.
