django.utils.http.url_has_allowed_host_and_scheme (from django 6.1.2)

Signature, in words: `url_has_allowed_host_and_scheme` takes the required arguments `url`, `allowed_hosts`; the optional argument `require_https` (default `False`).

The docstring:

Return ``True`` if the url uses an allowed host and a safe scheme.

Always return ``False`` on an empty url.

If ``require_https`` is ``True``, only 'https' will be considered a valid
scheme, as opposed to 'http' and 'https' with the default, ``False``.

Note: "True" doesn't entail that a URL is "safe". It may still be e.g.
quoted incorrectly. Ensure to also use django.utils.encoding.iri_to_uri()
on the path component of untrusted URLs.
