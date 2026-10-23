more_itertools.more.map_reduce (from more-itertools 11.2.1)

Signature, in words: `map_reduce` takes the required arguments `iterable`, `keyfunc`; the optional arguments `valuefunc` (default none), `reducefunc` (default none).

The docstring:

Return a dictionary that maps the items in *iterable* to categories
defined by *keyfunc*, transforms them with *valuefunc*, and
then summarizes them by category with *reducefunc*.

*valuefunc* defaults to the identity function if it is unspecified.
If *reducefunc* is unspecified, no summarization takes place:

    >>> keyfunc = lambda x: x.upper()
    >>> result = map_reduce('abbccc', keyfunc)
    >>> sorted(result.items())
    [('A', ['a']), ('B', ['b', 'b']), ('C', ['c', 'c', 'c'])]

Specifying *valuefunc* transforms the categorized items:

    >>> keyfunc = lambda x: x.upper()
    >>> valuefunc = lambda x: 1
    >>> result = map_reduce('abbccc', keyfunc, valuefunc)
    >>> sorted(result.items())
    [('A', [1]), ('B', [1, 1]), ('C', [1, 1, 1])]

Specifying *reducefunc* summarizes the categorized items:

    >>> keyfunc = lambda x: x.upper()
    >>> valuefunc = lambda x: 1
    >>> reducefunc = sum
    >>> result = map_reduce('abbccc', keyfunc, valuefunc, reducefunc)
    >>> sorted(result.items())
    [('A', 1), ('B', 2), ('C', 3)]

You may want to filter the input iterable before applying the map/reduce
procedure:

    >>> all_items = range(30)
    >>> items = [x for x in all_items if 10 <= x <= 20]  # Filter
    >>> keyfunc = lambda x: x % 2  # Evens map to 0; odds to 1
    >>> categories = map_reduce(items, keyfunc=keyfunc)
    >>> sorted(categories.items())
    [(0, [10, 12, 14, 16, 18, 20]), (1, [11, 13, 15, 17, 19])]
    >>> summaries = map_reduce(items, keyfunc=keyfunc, reducefunc=sum)
    >>> sorted(summaries.items())
    [(0, 90), (1, 75)]

Note that all items in the iterable are gathered into a list before the
summarization step, which may require significant storage.

The returned object is a :obj:`collections.defaultdict` with the
``default_factory`` set to ``None``, such that it behaves like a normal
dictionary.

.. seealso:: :func:`bucket`, :func:`groupby_transform`

If storage is a concern, :func:`bucket` can be used without consuming the
entire iterable right away. If the elements with the same key are already
adjacent, :func:`groupby_transform` or :func:`itertools.groupby` can be
used without any caching overhead.
