toolz.functoolz.apply (from toolz 1.2.0)

Signature, in words: `apply` takes any number of positional arguments `func_and_args`; any keyword arguments `kwargs`.

The docstring:

Applies a function and returns the results

>>> def double(x): return 2*x
>>> def inc(x):    return x + 1
>>> apply(double, 5)
10

>>> tuple(map(apply, [double, inc, double], [10, 500, 8000]))
(20, 501, 16000)
