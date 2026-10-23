humanize.number.ordinal (from humanize 4.16.0)

Signature, in words: `ordinal` takes the required argument `value`; the optional argument `gender` (default `'male'`). It returns a value of type `str`.

The docstring:

Converts an integer to its ordinal as a string.

For example, 1 is "1st", 2 is "2nd", 3 is "3rd", etc. Works for any integer or
anything `int()` will turn into an integer. Anything else will return the output
of str(value).

Examples:
    ```pycon
    >>> ordinal(1)
    '1st'
    >>> ordinal(1002)
    '1002nd'
    >>> ordinal(103)
    '103rd'
    >>> ordinal(4)
    '4th'
    >>> ordinal(12)
    '12th'
    >>> ordinal(101)
    '101st'
    >>> ordinal(111)
    '111th'
    >>> ordinal("something else")
    'something else'
    >>> ordinal([1, 2, 3]) == "[1, 2, 3]"
    True

    ```

Args:
    value (int, str, float): Integer to convert.
    gender (str): Gender for translations. Accepts either "male" or "female".

Returns:
    str: Ordinal string.
