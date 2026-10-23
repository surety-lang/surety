boltons.strutils.is_ascii (from boltons 26.2.0)

Signature, in words: `is_ascii` takes the required argument `text`.

The docstring:

Check if a string or bytestring, *text*, is composed of ascii
characters only. Raises :exc:`ValueError` if argument is not text.

Args:
    text (str): The string to be checked.

>>> is_ascii('Beyoncé')
False
>>> is_ascii('Beyonce')
True
