django.utils.http.is_same_domain (from django 6.1.2)

Signature, in words: `is_same_domain` takes the required arguments `host`, `pattern`.

The docstring:

Return ``True`` if the host is either an exact match or a match
to the wildcard pattern.

Any pattern beginning with a period matches a domain and all of its
subdomains. (e.g. ``.example.com`` matches ``example.com`` and
``foo.example.com``). Anything else is an exact string match.
