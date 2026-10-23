more_itertools.recipes.multinomial (from itertools recipes (more_itertools.recipes) 11.2.1)

Signature, in words: `multinomial` takes any number of positional arguments `counts`.

The docstring:

Number of distinct arrangements of a multiset.

The expression ``multinomial(3, 4, 2)`` has several equivalent
interpretations:

* In the expansion of ``(a + b + c)⁹``, the coefficient of the
  ``a³b⁴c²`` term is 1260.

* There are 1260 distinct ways to arrange 9 balls consisting of 3 reds, 4
  greens, and 2 blues.

* There are 1260 unique ways to place 9 distinct objects into three bins
  with sizes 3, 4, and 2.

The :func:`multinomial` function computes the length of
:func:`distinct_permutations`.  For example, there are 83,160 distinct
anagrams of the word "abracadabra":

    >>> from more_itertools import distinct_permutations, ilen
    >>> ilen(distinct_permutations('abracadabra'))
    83160

This can be computed directly from the letter counts, 5a 2b 2r 1c 1d:

    >>> from collections import Counter
    >>> list(Counter('abracadabra').values())
    [5, 2, 2, 1, 1]
    >>> multinomial(5, 2, 2, 1, 1)
    83160

A binomial coefficient is a special case of multinomial where there are
only two categories.  For example, the number of ways to arrange 12 balls
with 5 reds and 7 blues is ``multinomial(5, 7)`` or ``math.comb(12, 5)``.

Likewise, factorial is a special case of multinomial where
the multiplicities are all just 1 so that
``multinomial(1, 1, 1, 1, 1, 1, 1) == math.factorial(7)``.

Reference:  https://en.wikipedia.org/wiki/Multinomial_theorem
