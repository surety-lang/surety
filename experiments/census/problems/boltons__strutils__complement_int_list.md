boltons.strutils.complement_int_list (from boltons 26.2.0)

Signature, in words: `complement_int_list` takes the required argument `range_string`; the optional arguments `range_start` (default `0`), `range_end` (default none), `delim` (default `','`), `range_delim` (default `'-'`).

The docstring:

Returns range string that is the complement of the one provided as
*range_string* parameter.

These range strings are of the kind produce by :func:`format_int_list`, and
parseable by :func:`parse_int_list`.

Args:
    range_string (str): String of comma separated positive integers or
       ranges (e.g. '1,2,4-6,8'). Typical of a custom page range string
       used in printer dialogs.
    range_start (int): A positive integer from which to start the resulting
       range. Value is inclusive. Defaults to ``0``.
    range_end (int): A positive integer from which the produced range is
       stopped. Value is exclusive. Defaults to the maximum value found in
       the provided ``range_string``.
    delim (char): Defaults to ','. Separates integers and contiguous ranges
       of integers.
    range_delim (char): Defaults to '-'. Indicates a contiguous range of
       integers.

>>> complement_int_list('1,3,5-8,10-11,15')
'0,2,4,9,12-14'

>>> complement_int_list('1,3,5-8,10-11,15', range_start=0)
'0,2,4,9,12-14'

>>> complement_int_list('1,3,5-8,10-11,15', range_start=1)
'2,4,9,12-14'

>>> complement_int_list('1,3,5-8,10-11,15', range_start=2)
'2,4,9,12-14'

>>> complement_int_list('1,3,5-8,10-11,15', range_start=3)
'4,9,12-14'

>>> complement_int_list('1,3,5-8,10-11,15', range_end=15)
'0,2,4,9,12-14'

>>> complement_int_list('1,3,5-8,10-11,15', range_end=14)
'0,2,4,9,12-13'

>>> complement_int_list('1,3,5-8,10-11,15', range_end=13)
'0,2,4,9,12'

>>> complement_int_list('1,3,5-8,10-11,15', range_end=20)
'0,2,4,9,12-14,16-19'

>>> complement_int_list('1,3,5-8,10-11,15', range_end=0)
''

>>> complement_int_list('1,3,5-8,10-11,15', range_start=-1)
'0,2,4,9,12-14'

>>> complement_int_list('1,3,5-8,10-11,15', range_end=-1)
''

>>> complement_int_list('1,3,5-8', range_start=1, range_end=1)
''

>>> complement_int_list('1,3,5-8', range_start=2, range_end=2)
''

>>> complement_int_list('1,3,5-8', range_start=2, range_end=3)
'2'

>>> complement_int_list('1,3,5-8', range_start=-10, range_end=-5)
''

>>> complement_int_list('1,3,5-8', range_start=20, range_end=10)
''

>>> complement_int_list('')
''
