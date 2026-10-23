humanize.number.apnumber (from humanize 4.16.0)

Signature, in words: `apnumber` takes the required argument `value`. It returns a value of type `str`.

The docstring:

Converts an integer to Associated Press style.

Examples:
  ```pycon
  >>> apnumber(0)
  'zero'
  >>> apnumber(5)
  'five'
  >>> apnumber(10)
  '10'
  >>> apnumber("7")
  'seven'
  >>> apnumber("foo")
  'foo'
  >>> apnumber(None)
  'None'

  ```

Args:
    value (int, float, str): Integer to convert.

Returns:
    str: For numbers 0-9, the number spelled out. Otherwise, the number. This always
        returns a string unless the value was not `int`-able, then `str(value)`
        is returned.
