boltons.mathutils.ceil (from boltons 26.2.0)

Signature, in words: `ceil` takes the required argument `x`; the optional argument `options` (default none).

The docstring:

Return the ceiling of *x*. If *options* is set, return the smallest
integer or float from *options* that is greater than or equal to
*x*.

Args:
    x (int or float): Number to be tested.
    options (iterable): Optional iterable of arbitrary numbers
      (ints or floats).

>>> VALID_CABLE_CSA = [1.5, 2.5, 4, 6, 10, 25, 35, 50]
>>> ceil(3.5, options=VALID_CABLE_CSA)
4
>>> ceil(4, options=VALID_CABLE_CSA)
4
