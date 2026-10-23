django.utils.dateparse.parse_datetime (from django 6.1.2)

Signature, in words: `parse_datetime` takes the required argument `value`.

The docstring:

Parse a string and return a datetime.datetime.

This function supports time zone offsets. When the input contains one,
the output uses a timezone with a fixed offset from UTC.

Raise ValueError if the input is well formatted but not a valid datetime.
Return None if the input isn't well formatted.
