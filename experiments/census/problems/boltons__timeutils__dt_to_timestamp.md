boltons.timeutils.dt_to_timestamp (from boltons 26.2.0)

Signature, in words: `dt_to_timestamp` takes the required argument `dt`.

The docstring:

Converts from a :class:`~datetime.datetime` object to an integer
timestamp, suitable interoperation with :func:`time.time` and
other `Epoch-based timestamps`.

.. _Epoch-based timestamps: https://en.wikipedia.org/wiki/Unix_time

>>> timestamp = int(time.time())
>>> utc_dt = datetime.fromtimestamp(timestamp, timezone.utc)
>>> timestamp - dt_to_timestamp(utc_dt)
0.0

``dt_to_timestamp`` supports both timezone-aware and naïve
:class:`~datetime.datetime` objects. Note that it assumes naïve
datetime objects are implied UTC, such as those generated with
:meth:`datetime.datetime.utcnow`. If your datetime objects are
local time, such as those generated with
:meth:`datetime.datetime.now`, first convert it using the
:meth:`datetime.datetime.replace` method with ``tzinfo=``
:class:`LocalTZ` object in this module, then pass the result of
that to ``dt_to_timestamp``.
