boltons.mathutils.clamp (from boltons 26.2.0)

Signature, in words: `clamp` takes the required argument `x`; the optional arguments `lower` (default `-inf`), `upper` (default `inf`).

The docstring:

Limit a value to a given range.

Args:
    x (int or float): Number to be clamped.
    lower (int or float): Minimum value for x.
    upper (int or float): Maximum value for x.

The returned value is guaranteed to be between *lower* and
*upper*. Integers, floats, and other comparable types can be
mixed.

>>> clamp(1.0, 0, 5)
1.0
>>> clamp(-1.0, 0, 5)
0
>>> clamp(101.0, 0, 5)
5
>>> clamp(123, upper=5)
5

Similar to `numpy's clip`_ function.

.. _numpy's clip: http://docs.scipy.org/doc/numpy/reference/generated/numpy.clip.html
