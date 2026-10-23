boltons.iterutils.frange (from boltons 26.2.0)

Signature, in words: `frange` takes the required argument `stop`; the optional arguments `start` (default none), `step` (default `1.0`).

The docstring:

A :func:`range` clone for float-based ranges.

>>> frange(5)
[0.0, 1.0, 2.0, 3.0, 4.0]
>>> frange(6, step=1.25)
[0.0, 1.25, 2.5, 3.75, 5.0]
>>> frange(100.5, 101.5, 0.25)
[100.5, 100.75, 101.0, 101.25]
>>> frange(5, 0)
[]
>>> frange(5, 0, step=-1.25)
[5.0, 3.75, 2.5, 1.25]
