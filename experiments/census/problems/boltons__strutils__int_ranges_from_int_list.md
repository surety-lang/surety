boltons.strutils.int_ranges_from_int_list (from boltons 26.2.0)

Signature, in words: `int_ranges_from_int_list` takes the required argument `range_string`; the optional arguments `delim` (default `','`), `range_delim` (default `'-'`).

The docstring:

Transform a string of ranges (*range_string*) into a tuple of tuples.

Args:
    range_string (str): String of comma separated positive integers or
       ranges (e.g. '1,2,4-6,8'). Typical of a custom page range string
       used in printer dialogs.
    delim (char): Defaults to ','. Separates integers and contiguous ranges
       of integers.
    range_delim (char): Defaults to '-'. Indicates a contiguous range of
       integers.

>>> int_ranges_from_int_list('1,3,5-8,10-11,15')
((1, 1), (3, 3), (5, 8), (10, 11), (15, 15))

>>> int_ranges_from_int_list('1')
((1, 1),)

>>> int_ranges_from_int_list('')
()
