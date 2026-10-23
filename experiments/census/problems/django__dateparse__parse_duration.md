django.utils.dateparse.parse_duration (from django 6.1.2)

Signature, in words: `parse_duration` takes the required argument `value`.

The docstring:

Parse a duration string and return a datetime.timedelta.

The preferred format for durations in Django is '%d %H:%M:%S.%f'.

Also supports ISO 8601 representation (excluding years and months) and
PostgreSQL's day-time interval format.
