humanize.number.scientific (from humanize 4.16.0)

Signature, in words: `scientific` takes the required argument `value`; the optional argument `precision` (default `2`). It returns a value of type `str`.

The docstring:

Return number in string scientific notation z.wq x 10ⁿ.

Examples:
    ```pycon
    >>> scientific(float(0.3))
    '3.00 x 10⁻¹'
    >>> scientific(int(500))
    '5.00 x 10²'
    >>> scientific(-1000)
    '-1.00 x 10³'
    >>> scientific(1000, 1)
    '1.0 x 10³'
    >>> scientific(1000, 3)
    '1.000 x 10³'
    >>> scientific("99")
    '9.90 x 10¹'
    >>> scientific("foo")
    'foo'
    >>> scientific(None)
    'None'

    ```

Args:
    value (int, float, str): Input number.
    precision (int): Number of decimal for first part of the number.

Returns:
    str: Number in scientific notation z.wq x 10ⁿ.
