toolz.functoolz.complement (from toolz 1.2.0)

Signature, in words: `complement` takes the required argument `func`.

The docstring:

Convert a predicate function to its logical complement.

In other words, return a function that, for inputs that normally
yield True, yields False, and vice-versa.

>>> def iseven(n): return n % 2 == 0
>>> isodd = complement(iseven)
>>> iseven(2)
True
>>> isodd(2)
False
