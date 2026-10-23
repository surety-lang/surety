django.utils.http.quote_etag (from django 6.1.2)

Signature, in words: `quote_etag` takes the required argument `etag_str`.

The docstring:

If the provided string is already a quoted ETag, return it. Otherwise, wrap
the string in quotes, making it a strong ETag.
