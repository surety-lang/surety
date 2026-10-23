more_itertools.recipes.convolve (from itertools recipes (more_itertools.recipes) 11.2.1)

Signature, in words: `convolve` takes the required arguments `signal`, `kernel`.

The docstring:

Discrete linear convolution of two iterables.
Equivalent to polynomial multiplication.

For example, multiplying ``(x² -x - 20)`` by ``(x - 3)``
gives ``(x³ -4x² -17x + 60)``.

    >>> list(convolve([1, -1, -20], [1, -3]))
    [1, -4, -17, 60]

Examples of useful kernels:

* The kernel ``[0.25, 0.25, 0.25, 0.25]`` computes a moving average.
  For image data, this blurs the image and reduces noise.
* The kernel ``[1/2, 0, -1/2]`` estimates the first derivative of
  a function evaluated at evenly spaced inputs.
* The kernel ``[1, -2, 1]`` estimates the second derivative of a
  function evaluated at evenly spaced inputs.

Convolutions are mathematically commutative. However, the input iterables are
evaluated differently by this function. *signal* is consumed lazily and can be
infinite. *kernel* is fully consumed before calculations begin.

Supports all numeric types: int, float, complex, Decimal, Fraction.
Note that empty input iterables will produce meaningless output.

References:

* Article:  https://betterexplained.com/articles/intuitive-convolution/
* Video by 3Blue1Brown:  https://www.youtube.com/watch?v=KuXjwB4LzSA
