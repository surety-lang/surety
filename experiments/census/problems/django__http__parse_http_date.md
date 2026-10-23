django.utils.http.parse_http_date (from django 6.1.2)

Signature, in words: `parse_http_date` takes the required argument `date`.

The docstring:

Parse a date format as specified by HTTP RFC 9110 Section 5.6.7.

The three formats allowed by the RFC are accepted, even if only the first
one is still in widespread use.

Return an integer expressed in seconds since the epoch, in UTC.
