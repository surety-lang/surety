toolz.functoolz.thread_last (from toolz 1.2.0)

Signature, in words: `thread_last` takes the required argument `val`; any number of positional arguments `forms`.

The docstring:

Thread value through a sequence of functions/forms

>>> def double(x): return 2*x
>>> def inc(x):    return x + 1
>>> thread_last(1, inc, double)
4

If the function expects more than one input you can specify those inputs
in a tuple.  The value is used as the last input.

>>> def add(x, y): return x + y
>>> def pow(x, y): return x**y
>>> thread_last(1, (add, 4), (pow, 2))  # pow(2, add(4, 1))
32

So in general
    thread_last(x, f, (g, y, z))
expands to
    g(y, z, f(x))

>>> def iseven(x):
...     return x % 2 == 0
>>> list(thread_last([1, 2, 3], (map, inc), (filter, iseven)))
[2, 4]

See Also:
    thread_first
