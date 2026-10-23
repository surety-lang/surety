boltons.urlutils.parse_host (from boltons 26.2.0)

Signature, in words: `parse_host` takes the required argument `host`.

The docstring:

Low-level function used to parse the host portion of a URL.

Returns a tuple of (family, host) where *family* is a
:mod:`socket` module constant or ``None``, and host is a string.

>>> parse_host('googlewebsite.com') == (None, 'googlewebsite.com')
True
>>> parse_host('[::1]') == (socket.AF_INET6, '::1')
True
>>> parse_host('192.168.1.1') == (socket.AF_INET, '192.168.1.1')
True

Odd doctest formatting above due to py3's switch from int to enums
for :mod:`socket` constants.
