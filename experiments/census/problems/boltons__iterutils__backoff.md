boltons.iterutils.backoff (from boltons 26.2.0)

Signature, in words: `backoff` takes the required arguments `start`, `stop`; the optional arguments `count` (default none), `factor` (default `2.0`), `jitter` (default `False`).

The docstring:

Returns a list of geometrically-increasing floating-point numbers,
suitable for usage with `exponential backoff`_. Exactly like
:func:`backoff_iter`, but without the ``'repeat'`` option for
*count*. See :func:`backoff_iter` for more details.

.. _exponential backoff: https://en.wikipedia.org/wiki/Exponential_backoff

>>> backoff(1, 10)
[1.0, 2.0, 4.0, 8.0, 10.0]
