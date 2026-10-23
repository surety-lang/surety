boltons.timeutils.strpdate (from boltons 26.2.0)

Signature, in words: `strpdate` takes the required arguments `string`, `format`.

The docstring:

Parse the date string according to the format in `format`.  Returns a
:class:`date` object.  Internally, :meth:`datetime.strptime` is used to
parse the string and thus conversion specifiers for time fields (e.g. `%H`)
may be provided;  these will be parsed but ignored.

Args:
    string (str): The date string to be parsed.
    format (str): The `strptime`_-style date format string.
Returns:
    datetime.date

.. _`strptime`: https://docs.python.org/2/library/datetime.html#strftime-strptime-behavior

>>> strpdate('2016-02-14', '%Y-%m-%d')
datetime.date(2016, 2, 14)
>>> strpdate('26/12 (2015)', '%d/%m (%Y)')
datetime.date(2015, 12, 26)
>>> strpdate('20151231 23:59:59', '%Y%m%d %H:%M:%S')
datetime.date(2015, 12, 31)
>>> strpdate('20160101 00:00:00.001', '%Y%m%d %H:%M:%S.%f')
datetime.date(2016, 1, 1)
