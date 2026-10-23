more_itertools.more.zip_offset (from more-itertools 11.2.1)

Signature, in words: `zip_offset` takes any number of positional arguments `iterables`; the keyword-only arguments `offsets`, `longest` (default `False`), `fillvalue` (default none).

The docstring:

``zip`` the input *iterables* together, but offset the `i`-th iterable
by the `i`-th item in *offsets*.

    >>> list(zip_offset('0123', 'abcdef', offsets=(0, 1)))
    [('0', 'b'), ('1', 'c'), ('2', 'd'), ('3', 'e')]

This can be used as a lightweight alternative to SciPy or pandas to analyze
data sets in which some series have a lead or lag relationship.

By default, the sequence will end when the shortest iterable is exhausted.
To continue until the longest iterable is exhausted, set *longest* to
``True``.

    >>> list(zip_offset('0123', 'abcdef', offsets=(0, 1), longest=True))
    [('0', 'b'), ('1', 'c'), ('2', 'd'), ('3', 'e'), (None, 'f')]

By default, ``None`` will be used to replace offsets beyond the end of the
sequence. Specify *fillvalue* to use some other value.
