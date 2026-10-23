boltons.timeutils.daterange (from boltons 26.2.0)

Signature, in words: `daterange` takes the required arguments `start`, `stop`; the optional arguments `step` (default `1`), `inclusive` (default `False`).

The docstring:

In the spirit of :func:`range` and :func:`xrange`, the `daterange`
generator that yields a sequence of :class:`~datetime.date`
objects, starting at *start*, incrementing by *step*, until *stop*
is reached.

When *inclusive* is True, the final date may be *stop*, **if**
*step* falls evenly on it. By default, *step* is one day. See
details below for many more details.

Args:
    start (datetime.date): The starting date The first value in
        the sequence.
    stop (datetime.date): The stopping date. By default not
        included in return. Can be `None` to yield an infinite
        sequence.
    step (int): The value to increment *start* by to reach
        *stop*. Can be an :class:`int` number of days, a
        :class:`datetime.timedelta`, or a :class:`tuple` of integers,
        `(year, month, day)`. Positive and negative *step* values
        are supported. A step that does not advance (e.g. ``0`` or a
        self-cancelling tuple like ``(0, 1, -31)``) raises
        :exc:`ValueError`; a step pointed away from *stop* yields
        nothing, like ``range(1, 5, -1)``.
    inclusive (bool): Whether or not the *stop* date can be
        returned. *stop* is only returned when a *step* falls evenly
        on it.

>>> christmas = date(year=2015, month=12, day=25)
>>> boxing_day = date(year=2015, month=12, day=26)
>>> new_year = date(year=2016, month=1,  day=1)
>>> for day in daterange(christmas, new_year):
...     print(repr(day))
datetime.date(2015, 12, 25)
datetime.date(2015, 12, 26)
datetime.date(2015, 12, 27)
datetime.date(2015, 12, 28)
datetime.date(2015, 12, 29)
datetime.date(2015, 12, 30)
datetime.date(2015, 12, 31)
>>> for day in daterange(christmas, boxing_day):
...     print(repr(day))
datetime.date(2015, 12, 25)
>>> for day in daterange(date(2017, 5, 1), date(2017, 8, 1),
...                      step=(0, 1, 0), inclusive=True):
...     print(repr(day))
datetime.date(2017, 5, 1)
datetime.date(2017, 6, 1)
datetime.date(2017, 7, 1)
datetime.date(2017, 8, 1)

*Be careful when using stop=None, as this will yield an infinite
sequence of dates.*
