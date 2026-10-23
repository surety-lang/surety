django.utils.http.escape_leading_slashes (from django 6.1.2)

Signature, in words: `escape_leading_slashes` takes the required argument `url`.

The docstring:

If redirecting to an absolute path (two leading slashes), a slash must be
escaped to prevent browsers from handling the path as schemaless and
redirecting to another host.
