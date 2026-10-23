toolz.itertoolz.isdistinct (from toolz 1.2.0)

Signature, in words: `isdistinct` takes the required argument `seq`.

The docstring:

All values in sequence are distinct

>>> isdistinct([1, 2, 3])
True
>>> isdistinct([1, 2, 1])
False

>>> isdistinct("Hello")
False
>>> isdistinct("World")
True
