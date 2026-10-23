more_itertools.recipes.before_and_after (from itertools recipes (more_itertools.recipes) 11.2.1)

Signature, in words: `before_and_after` takes the required arguments `predicate`, `it`.

The docstring:

A variant of :func:`takewhile` that allows complete access to the
remainder of the iterator.

     >>> it = iter('ABCdEfGhI')
     >>> all_upper, remainder = before_and_after(str.isupper, it)
     >>> ''.join(all_upper)
     'ABC'
     >>> ''.join(remainder) # takewhile() would lose the 'd'
     'dEfGhI'

Note that the first iterator must be fully consumed before the second
iterator can generate valid results.
