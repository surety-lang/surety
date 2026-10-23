humanize.number.metric (from humanize 4.16.0)

Signature, in words: `metric` takes the required argument `value`; the optional arguments `unit` (default `''`), `precision` (default `3`). It returns a value of type `str`.

The docstring:

Return a value with a metric SI unit-prefix appended.

Examples:
    ```pycon
    >>> metric(1500, "V")
    '1.50 kV'
    >>> metric(2e8, "W")
    '200 MW'
    >>> metric(220e-6, "F")
    '220 μF'
    >>> metric(1e-14, precision=4)
    '10.00 f'

    ```

The unit prefix is always chosen so that non-significant zero digits are required.
i.e. `123,000` will become `123k` instead of `0.123M` and `1,230,000` will become
`1.23M` instead of `1230K`. For numbers that are either too huge or too tiny to
represent without resorting to either leading or trailing zeroes, it falls back to
`scientific()`.
```pycon
>>> metric(1e40)
'1.00 x 10⁴⁰'

```

Args:
    value (int, float): Input number.
    unit (str): Optional base unit.
    precision (int): The number of digits the output should contain.

Returns:
    str:
