boltons.strutils.strip_ansi (from boltons 26.2.0)

Signature, in words: `strip_ansi` takes the required argument `text`.

The docstring:

Strips ANSI escape codes from *text*. Useful for the occasional
time when a log or redirected output accidentally captures console
color codes and the like.

>>> strip_ansi('[0m[1;36mart[46;34m')
'art'

Supports str, bytes and bytearray content as input. Returns the
same type as the input.

There's a lot of ANSI art available for testing on `sixteencolors.net`_.
This function does not interpret or render ANSI art, but you can do so with
`ansi2img`_ or `escapes.js`_.

.. _sixteencolors.net: http://sixteencolors.net
.. _ansi2img: http://www.bedroomlan.org/projects/ansi2img
.. _escapes.js: https://github.com/atdt/escapes.js
