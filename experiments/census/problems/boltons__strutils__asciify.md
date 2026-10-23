boltons.strutils.asciify (from boltons 26.2.0)

Signature, in words: `asciify` takes the required argument `text`; the optional argument `ignore` (default `False`).

The docstring:

Converts a unicode or bytestring, *text*, into a bytestring with
just ascii characters. Performs basic deaccenting for all you
Europhiles out there.

Also, a gentle reminder that this is a **utility**, primarily meant
for slugification. Whenever possible, make your application work
**with** unicode, not against it.

Args:
    text (str): The string to be asciified.
    ignore (bool): Configures final encoding to ignore remaining
        unasciified string instead of replacing it.

>>> asciify('Beyoncé') == b'Beyonce'
True
