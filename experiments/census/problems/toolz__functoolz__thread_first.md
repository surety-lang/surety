toolz.functoolz.thread_first (from toolz 1.2.0)

Signature, in words: `thread_first` takes the required argument `val`; any number of positional arguments `forms`.

The docstring:

Thread value through a sequence of functions/forms

>>> def double(x): return 2*x
>>> def inc(x):    return x + 1
>>> thread_first(1, inc, double)
4

If the function expects more than one input you can specify those inputs
in a tuple.  The value is used as the first input.

>>> def add(x, y): return x + y
>>> def pow(x, y): return x**y
>>> thread_first(1, (add, 4), (pow, 2))  # pow(add(1, 4), 2)
25

So in general
    thread_first(x, f, (g, y, z))
expands to
    g(f(x), y, z)

See Also:
    thread_last
