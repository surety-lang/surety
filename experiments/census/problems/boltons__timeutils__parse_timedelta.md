boltons.timeutils.parse_timedelta (from boltons 26.2.0)

Signature, in words: `parse_timedelta` takes the required argument `text`.

The docstring:

Robustly parses a short text description of a time period into a
:class:`datetime.timedelta`. Supports weeks, days, hours, minutes,
and seconds, with or without decimal points:

Args:
    text (str): Text to parse.
Returns:
    datetime.timedelta
Raises:
    ValueError: on parse failure.

>>> parse_td('1d 2h 3.5m 0s') == timedelta(days=1, seconds=7410)
True

Also supports full words and whitespace.

>>> parse_td('2 weeks 1 day') == timedelta(days=15)
True

Negative times are supported, too:

>>> parse_td('-1.5 weeks 3m 20s') == timedelta(days=-11, seconds=43400)
True
