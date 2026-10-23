requests.utils.from_key_val_list (from requests 2.34.2)

Signature, in words: `from_key_val_list` takes the required argument `value`. It returns a value of type `dict[Any, Any] | None`.

The docstring:

Take an object and test to see if it can be represented as a
dictionary. Unless it can not be represented as such, return an
OrderedDict, e.g.,

::

    >>> from_key_val_list([('key', 'val')])
    OrderedDict([('key', 'val')])
    >>> from_key_val_list('string')
    Traceback (most recent call last):
    ...
    ValueError: cannot encode objects that are not 2-tuples
    >>> from_key_val_list({'key': 'val'})
    OrderedDict([('key', 'val')])

:rtype: OrderedDict
