more_itertools.more.subfactorial (from more-itertools 11.2.1)

Signature, in words: `subfactorial` takes the required argument `n`.

The docstring:

Number of permutations of *n* elements with no fixed points.

The :func:`subfactorial` function computes the length of
:func:`derangements`.  For example, there are 1,854 ways to
rearrange the letters in word "epsilon" without leaving any
letter in its original position:

    >>> from more_itertools import derangements, ilen
    >>> ilen(derangements('epsilon'))
    1854
    >>> subfactorial(len('epsilon'))
    1854

Reference:  https://oeis.org/A000166
