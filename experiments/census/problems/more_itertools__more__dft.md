more_itertools.more.dft (from more-itertools 11.2.1)

Signature, in words: `dft` takes the required argument `xarr`.

The docstring:

Discrete Fourier Transform. *xarr* is a sequence of complex numbers.
Yields the components of the corresponding transformed output vector.

>>> import cmath
>>> xarr = [1, 2-1j, -1j, -1+2j]  # time domain
>>> Xarr = [2, -2-2j, -2j, 4+4j]  # frequency domain
>>> magnitudes, phases = zip(*map(cmath.polar, Xarr))
>>> all(map(cmath.isclose, dft(xarr), Xarr))
True

Inputs are restricted to numeric types that can add and multiply
with a complex number.  This includes int, float, complex, and
Fraction, but excludes Decimal.

See :func:`idft` for the inverse Discrete Fourier Transform.
