requests.utils.to_key_val_list (from requests 2.34.2)

Signature, in words: `to_key_val_list` takes the required argument `value`. It returns a value of type `list[tuple[_KT, _VT]] | None`.

The docstring:

Take an object and test to see if it can be represented as a
dictionary. If it can be, return a list of tuples, e.g.,

::

    >>> to_key_val_list([('key', 'val')])
    [('key', 'val')]
    >>> to_key_val_list({'key': 'val'})
    [('key', 'val')]
    >>> to_key_val_list('string')
    Traceback (most recent call last):
    ...
    ValueError: cannot encode objects that are not 2-tuples

:rtype: list
