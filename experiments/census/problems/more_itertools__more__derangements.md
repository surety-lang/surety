more_itertools.more.derangements (from more-itertools 11.2.1)

Signature, in words: `derangements` takes the required argument `iterable`; the optional argument `r` (default none).

The docstring:

Yield successive derangements of the elements in *iterable*.

A derangement is a permutation in which no element appears at its original
index. In other words, a derangement is a permutation that has no fixed points.

Suppose Alice, Bob, Carol, and Dave are playing Secret Santa.
The code below outputs all of the different ways to assign gift recipients
such that nobody is assigned to himself or herself:

    >>> for d in derangements(['Alice', 'Bob', 'Carol', 'Dave']):
    ...    print(', '.join(d))
    Bob, Alice, Dave, Carol
    Bob, Carol, Dave, Alice
    Bob, Dave, Alice, Carol
    Carol, Alice, Dave, Bob
    Carol, Dave, Alice, Bob
    Carol, Dave, Bob, Alice
    Dave, Alice, Bob, Carol
    Dave, Carol, Alice, Bob
    Dave, Carol, Bob, Alice

If *r* is given, only the *r*-length derangements are yielded.

    >>> sorted(derangements(range(3), 2))
    [(1, 0), (1, 2), (2, 0)]
    >>> sorted(derangements([0, 2, 3], 2))
    [(2, 0), (2, 3), (3, 0)]

Elements are treated as unique based on their position, not on their value.

Consider the Secret Santa example with two *different* people who have
the *same* name. Then there are two valid gift assignments even though
it might appear that a person is assigned to themselves:

    >>> names = ['Alice', 'Bob', 'Bob']
    >>> list(derangements(names))
    [('Bob', 'Bob', 'Alice'), ('Bob', 'Alice', 'Bob')]

To avoid confusion, make the inputs distinct:

    >>> deduped = [f'{name}{index}' for index, name in enumerate(names)]
    >>> list(derangements(deduped))
    [('Bob1', 'Bob2', 'Alice0'), ('Bob2', 'Alice0', 'Bob1')]

The number of derangements of a set of size *n* is known as the
"subfactorial of n".  For n > 0, the subfactorial is:
``round(math.factorial(n) / math.e)``.   The more-itertools function
:func:`subfactorial` computes this directly.

References:

* Article:  https://www.numberanalytics.com/blog/ultimate-guide-to-derangements-in-combinatorics
* Sizes:    https://oeis.org/A000166
