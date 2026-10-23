django.utils.http.parse_header_parameters (from django 6.1.2)

Signature, in words: `parse_header_parameters` takes the required argument `line`; the optional argument `max_length` (default `10000`).

The docstring:

Parse a Content-type like header.
Return the main content-type and a dictionary of options.

If `line` is longer than `max_length`, `ValueError` is raised.
