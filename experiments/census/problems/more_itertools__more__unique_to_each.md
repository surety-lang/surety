more_itertools.more.unique_to_each (from more-itertools 11.2.1)

Signature, in words: `unique_to_each` takes any number of positional arguments `iterables`.

The docstring:

Return the elements from each of the input iterables that aren't in the
other input iterables.

For example, suppose you have a set of packages, each with a set of
dependencies::

    {'pkg_1': {'A', 'B'}, 'pkg_2': {'B', 'C'}, 'pkg_3': {'B', 'D'}}

If you remove one package, which dependencies can also be removed?

If ``pkg_1`` is removed, then ``A`` is no longer necessary - it is not
associated with ``pkg_2`` or ``pkg_3``. Similarly, ``C`` is only needed for
``pkg_2``, and ``D`` is only needed for ``pkg_3``::

    >>> unique_to_each({'A', 'B'}, {'B', 'C'}, {'B', 'D'})
    [['A'], ['C'], ['D']]

If there are duplicates in one input iterable that aren't in the others
they will be duplicated in the output. Input order is preserved::

    >>> unique_to_each("mississippi", "missouri")
    [['p', 'p'], ['o', 'u', 'r']]

It is assumed that the elements of each iterable are hashable.
