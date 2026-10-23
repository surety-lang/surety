boltons.iterutils.strip (from boltons 26.2.0)

Signature, in words: `strip` takes the required argument `iterable`; the optional argument `strip_value` (default none).

The docstring:

Strips values from the beginning and end of an iterable. Stripped items
will match the value of the argument strip_value. Functionality is
analogous to that of the method str.strip. Returns a list.

>>> strip(['Fu', 'Foo', 'Bar', 'Bam', 'Fu'], 'Fu')
['Foo', 'Bar', 'Bam']
