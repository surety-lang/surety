toolz.itertoolz.iterate (from toolz 1.2.0)

Signature, in words: `iterate` takes the required arguments `func`, `x`.

The docstring:

Repeatedly apply a function func onto an original input

Yields x, then func(x), then func(func(x)), then func(func(func(x))), etc..

>>> def inc(x):  return x + 1
>>> counter = iterate(inc, 0)
>>> next(counter)
0
>>> next(counter)
1
>>> next(counter)
2

>>> double = lambda x: x * 2
>>> powers_of_two = iterate(double, 1)
>>> next(powers_of_two)
1
>>> next(powers_of_two)
2
>>> next(powers_of_two)
4
>>> next(powers_of_two)
8
