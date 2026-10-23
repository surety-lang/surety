toolz.functoolz.compose_left (from toolz 1.2.0)

Signature, in words: `compose_left` takes any number of positional arguments `funcs`.

The docstring:

Compose functions to operate in series.

Returns a function that applies other functions in sequence.

Functions are applied from left to right so that
``compose_left(f, g, h)(x, y)`` is the same as ``h(g(f(x, y)))``.

If no arguments are provided, the identity function (f(x) = x) is returned.

>>> inc = lambda i: i + 1
>>> compose_left(inc, str)(3)
'4'

See Also:
    compose
    pipe
