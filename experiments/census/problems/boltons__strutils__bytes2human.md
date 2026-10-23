boltons.strutils.bytes2human (from boltons 26.2.0)

Signature, in words: `bytes2human` takes the required argument `nbytes`; the optional argument `ndigits` (default `0`).

The docstring:

Turns an integer value of *nbytes* into a human readable format. Set
*ndigits* to control how many digits after the decimal point
should be shown (default ``0``).

>>> bytes2human(128991)
'126K'
>>> bytes2human(100001221)
'95M'
>>> bytes2human(0, 2)
'0.00B'
>>> bytes2human(1024)
'1K'
>>> bytes2human(1024 ** 8)
'1Y'
