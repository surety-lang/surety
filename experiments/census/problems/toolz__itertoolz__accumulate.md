toolz.itertoolz.accumulate (from toolz 1.2.0)

Signature, in words: `accumulate` takes the required arguments `binop`, `seq`; the optional argument `initial` (default `'__no__default__'`).

The docstring:

Repeatedly apply binary function to a sequence, accumulating results

>>> from operator import add, mul
>>> list(accumulate(add, [1, 2, 3, 4, 5]))
[1, 3, 6, 10, 15]
>>> list(accumulate(mul, [1, 2, 3, 4, 5]))
[1, 2, 6, 24, 120]

Accumulate is similar to ``reduce`` and is good for making functions like
cumulative sum:

>>> from functools import partial, reduce
>>> sum    = partial(reduce, add)
>>> cumsum = partial(accumulate, add)

Accumulate also takes an optional argument that will be used as the first
value. This is similar to reduce.

>>> list(accumulate(add, [1, 2, 3], -1))
[-1, 0, 2, 5]
>>> list(accumulate(add, [], 1))
[1]

See Also:
    itertools.accumulate :  In standard itertools for Python 3.2+
