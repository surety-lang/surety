humanize.time.precisedelta (from humanize 4.16.0)

Signature, in words: `precisedelta` takes the required argument `value`; the optional arguments `minimum_unit` (default `'seconds'`), `suppress` (default `()`), `format` (default `'%0.2f'`). It returns a value of type `str`.

The docstring:

Return a precise representation of a timedelta or number of seconds.

```pycon
>>> import datetime as dt
>>> from humanize.time import precisedelta

>>> delta = dt.timedelta(seconds=3633, days=2, microseconds=123000)
>>> precisedelta(delta)
'2 days, 1 hour and 33.12 seconds'

```

A custom `format` can be specified to control how the fractional part
is represented:

```pycon
>>> precisedelta(delta, format="%0.4f")
'2 days, 1 hour and 33.1230 seconds'

```

Instead, the `minimum_unit` can be changed to have a better resolution;
the function will still readjust the unit to use the greatest of the
units that does not lose precision.

For example setting microseconds but still representing the date with milliseconds:

```pycon
>>> precisedelta(delta, minimum_unit="microseconds")
'2 days, 1 hour, 33 seconds and 123 milliseconds'

```

If desired, some units can be suppressed: you will not see them represented and the
time of the other units will be adjusted to keep representing the same timedelta:

```pycon
>>> precisedelta(delta, suppress=['days'])
'49 hours and 33.12 seconds'

```

Note that microseconds precision is lost if the seconds and all
the units below are suppressed:

```pycon
>>> delta = dt.timedelta(seconds=90, microseconds=100)
>>> precisedelta(delta, suppress=['seconds', 'milliseconds', 'microseconds'])
'1.50 minutes'

```

If the delta is too small to be represented with the minimum unit,
a value of zero will be returned:

```pycon
>>> delta = dt.timedelta(seconds=1)
>>> precisedelta(delta, minimum_unit="minutes")
'0.02 minutes'

>>> delta = dt.timedelta(seconds=0.1)
>>> precisedelta(delta, minimum_unit="minutes")
'0 minutes'

```
