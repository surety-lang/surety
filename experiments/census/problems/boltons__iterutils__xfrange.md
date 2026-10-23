boltons.iterutils.xfrange (from boltons 26.2.0)

Signature, in words: `xfrange` takes the required argument `stop`; the optional arguments `start` (default none), `step` (default `1.0`).

The docstring:

Same as :func:`frange`, but generator-based instead of returning a
list.

>>> tuple(xfrange(1, 3, step=0.75))
(1.0, 1.75, 2.5)

See :func:`frange` for more details.
