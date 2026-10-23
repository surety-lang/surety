humanize.number.fractional (from humanize 4.16.0)

Signature, in words: `fractional` takes the required argument `value`. It returns a value of type `str`.

The docstring:

Convert to fractional number.

There will be some cases where one might not want to show ugly decimal places for
floats and decimals.

This function returns a human-readable fractional number in form of fractions and
mixed fractions.

Pass in a string, or a number or a float, and this function returns:

* a string representation of a fraction
* or a whole number
* or a mixed fraction
* or the str output of the value, if it could not be converted

Examples:
    ```pycon
    >>> fractional(0.3)
    '3/10'
    >>> fractional(1.3)
    '1 3/10'
    >>> fractional(float(1/3))
    '1/3'
    >>> fractional(1)
    '1'
    >>> fractional("ten")
    'ten'
    >>> fractional(None)
    'None'

    ```

Args:
    value (int, float, str): Integer to convert.

Returns:
    str: Fractional number as a string.
