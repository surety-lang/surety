humanize.number.intword (from humanize 4.16.0)

Signature, in words: `intword` takes the required argument `value`; the optional argument `format` (default `'%.1f'`). It returns a value of type `str`.

The docstring:

Converts a large integer to a friendly text representation.

Works best for numbers over 1 million. For example, 1_000_000 becomes "1.0 million",
1_200_000 becomes "1.2 million" and "1_200_000_000" becomes "1.2 billion". Supports
up to decillion (33 digits) and googol (100 digits).

Examples:
    ```pycon
    >>> intword("100")
    '100'
    >>> intword("12400")
    '12.4 thousand'
    >>> intword("1000000")
    '1.0 million'
    >>> intword(1_200_000_000)
    '1.2 billion'
    >>> intword(8100000000000000000000000000000000)
    '8.1 decillion'
    >>> intword(None)
    'None'
    >>> intword("1234000", "%0.3f")
    '1.234 million'

    ```

Args:
    value (int, float, str): Integer to convert.
    format (str): To change the number of decimal or general format of the number
        portion.

Returns:
    str: Friendly text representation as a string, unless the value passed could not
        be coaxed into an `int`.
