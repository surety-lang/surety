boltons.iterutils.lstrip (from boltons 26.2.0)

Signature, in words: `lstrip` takes the required argument `iterable`; the optional argument `strip_value` (default none).

The docstring:

Strips values from the beginning of an iterable. Stripped items will
match the value of the argument strip_value. Functionality is analogous
to that of the method str.lstrip. Returns a list.

>>> lstrip(['Foo', 'Bar', 'Bam'], 'Foo')
['Bar', 'Bam']
