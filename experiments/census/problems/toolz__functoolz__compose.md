toolz.functoolz.compose (from toolz 1.2.0)

Signature, in words: `compose` takes any number of positional arguments `funcs`.

The docstring:

Compose functions to operate in series.

Returns a function that applies other functions in sequence.

Functions are applied from right to left so that
``compose(f, g, h)(x, y)`` is the same as ``f(g(h(x, y)))``.

If no arguments are provided, the identity function (f(x) = x) is returned.

>>> inc = lambda i: i + 1
>>> compose(str, inc)(3)
'4'

See Also:
    compose_left
    pipe
