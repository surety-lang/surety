toolz.itertoolz.topk (from toolz 1.2.0)

Signature, in words: `topk` takes the required arguments `k`, `seq`; the optional argument `key` (default none).

The docstring:

Find the k largest elements of a sequence

Operates lazily in ``n*log(k)`` time

>>> topk(2, [1, 100, 10, 1000])
(1000, 100)

Use a key function to change sorted order

>>> topk(2, ['Alice', 'Bob', 'Charlie', 'Dan'], key=len)
('Charlie', 'Alice')

See also:
    heapq.nlargest
