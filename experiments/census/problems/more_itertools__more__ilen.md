more_itertools.more.ilen (from more-itertools 11.2.1)

Signature, in words: `ilen` takes the required argument `iterable`.

The docstring:

Return the number of items in *iterable*.

For example, there are 168 prime numbers below 1,000:

    >>> ilen(sieve(1000))
    168

Equivalent to, but faster than::

    def ilen(iterable):
        count = 0
        for _ in iterable:
            count += 1
        return count

This fully consumes the iterable, so handle with care.
