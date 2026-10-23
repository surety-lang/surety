more_itertools.more.iterate (from more-itertools 11.2.1)

Signature, in words: `iterate` takes the required arguments `func`, `start`.

The docstring:

Return ``start``, ``func(start)``, ``func(func(start))``, ...

Produces an infinite iterator. To add a stopping condition,
use :func:`take`, ``takewhile``, or :func:`takewhile_inclusive`:.

>>> take(10, iterate(lambda x: 2*x, 1))
[1, 2, 4, 8, 16, 32, 64, 128, 256, 512]

>>> collatz = lambda x: 3*x + 1 if x%2==1 else x // 2
>>> list(takewhile_inclusive(lambda x: x!=1, iterate(collatz, 10)))
[10, 5, 16, 8, 4, 2, 1]
