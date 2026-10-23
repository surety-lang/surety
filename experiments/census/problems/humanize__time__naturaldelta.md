humanize.time.naturaldelta (from humanize 4.16.0)

Signature, in words: `naturaldelta` takes the required argument `value`; the optional arguments `months` (default `True`), `minimum_unit` (default `'seconds'`). It returns a value of type `str`.

The docstring:

Return a natural representation of a timedelta or number of seconds.

This is similar to `naturaltime`, but does not add tense to the result.

The timedelta will be rounded to the nearest unit that makes sense.

Args:
    value (datetime.timedelta, int or float): A timedelta or a number of seconds.
    months (bool): If `True`, then a number of months (based on 30.5 days) will be
        used for fuzziness between years.
    minimum_unit (str): The lowest unit that can be used.

Returns:
    str (str or `value`): A natural representation of the amount of time
        elapsed unless `value` is not datetime.timedelta or cannot be
        converted to int (cannot be float due to 'inf' or 'nan').
        In that case, a `value` is returned unchanged.

Raises:
    OverflowError: If `value` is too large to convert to datetime.timedelta.

Examples:
    Compare two timestamps in a custom local timezone::

    ```pycon
    >>> import datetime as dt
    >>> from dateutil.tz import gettz

    >>> berlin = gettz("Europe/Berlin")
    >>> now = dt.datetime.now(tz=berlin)
    >>> later = now + dt.timedelta(minutes=30)

    >>> assert naturaldelta(later - now) == "30 minutes"
    True
    ```
