django.utils.http.parse_etags (from django 6.1.2)

Signature, in words: `parse_etags` takes the required argument `etag_str`.

The docstring:

Parse a string of ETags given in an If-None-Match or If-Match header as
defined by RFC 9110. Return a list of quoted ETags, or ['*'] if all ETags
should be matched.

ETags values containing a comma are not supported, as the comma is used as
list separator.
