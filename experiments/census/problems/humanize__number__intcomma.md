humanize.number.intcomma (from humanize 4.16.0)

Signature, in words: `intcomma` takes the required argument `value`; the optional argument `ndigits` (default none). It returns a value of type `str`.

The docstring:

Converts an integer to a string containing commas every three digits.

For example, 3000 becomes "3,000" and 45000 becomes "45,000". To maintain some
compatibility with Django's `intcomma`, this function also accepts floats.

Examples:
    ```pycon
    >>> intcomma(100)
    '100'
    >>> intcomma("1000")
    '1,000'
    >>> intcomma(1_000_000)
    '1,000,000'
    >>> intcomma(1_234_567.25)
    '1,234,567.25'
    >>> intcomma(1234.5454545, 2)
    '1,234.55'
    >>> intcomma(14308.40, 1)
    '14,308.4'
    >>> intcomma("14308.40", 1)
    '14,308.4'
    >>> intcomma(None)
    'None'

    ```

Args:
    value (int, float, str): Integer or float to convert.
    ndigits (int, None): Digits of precision for rounding after the decimal point.

Returns:
    str: String containing commas every three digits.
