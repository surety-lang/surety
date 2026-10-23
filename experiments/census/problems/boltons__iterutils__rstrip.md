boltons.iterutils.rstrip (from boltons 26.2.0)

Signature, in words: `rstrip` takes the required argument `iterable`; the optional argument `strip_value` (default none).

The docstring:

Strips values from the end of an iterable. Stripped items will
match the value of the argument strip_value. Functionality is analogous
to that of the method str.rstrip. Returns a list.

>>> rstrip(['Foo', 'Bar', 'Bam'], 'Bam')
['Foo', 'Bar']
