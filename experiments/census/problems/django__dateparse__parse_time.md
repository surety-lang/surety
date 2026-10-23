django.utils.dateparse.parse_time (from django 6.1.2)

Signature, in words: `parse_time` takes the required argument `value`.

The docstring:

Parse a string and return a datetime.time.

This function doesn't support time zone offsets.

Raise ValueError if the input is well formatted but not a valid time.
Return None if the input isn't well formatted, in particular if it
contains an offset.
