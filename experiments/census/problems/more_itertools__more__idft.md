more_itertools.more.idft (from more-itertools 11.2.1)

Signature, in words: `idft` takes the required argument `Xarr`.

The docstring:

Inverse Discrete Fourier Transform. *Xarr* is a sequence of
complex numbers. Yields the components of the corresponding
inverse-transformed output vector.

>>> import cmath
>>> xarr = [1, 2-1j, -1j, -1+2j]  # time domain
>>> Xarr = [2, -2-2j, -2j, 4+4j]  # frequency domain
>>> all(map(cmath.isclose, idft(Xarr), xarr))
True

Inputs are restricted to numeric types that can add and multiply
with a complex number.  This includes int, float, complex, and
Fraction, but excludes Decimal.

See :func:`dft` for the Discrete Fourier Transform.
