more_itertools.more.doublestarmap (from more-itertools 11.2.1)

Signature, in words: `doublestarmap` takes the required arguments `func`, `iterable`.

The docstring:

Apply *func* to every item of *iterable* by dictionary unpacking
the item into *func*.

The difference between :func:`itertools.starmap` and :func:`doublestarmap`
parallels the distinction between ``func(*a)`` and ``func(**a)``.

>>> iterable = [{'a': 1, 'b': 2}, {'a': 40, 'b': 60}]
>>> list(doublestarmap(lambda a, b: a + b, iterable))
[3, 100]

``TypeError`` will be raised if *func*'s signature doesn't match the
mapping contained in *iterable* or if *iterable* does not contain mappings.
