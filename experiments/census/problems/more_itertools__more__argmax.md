more_itertools.more.argmax (from more-itertools 11.2.1)

Signature, in words: `argmax` takes the required argument `iterable`; the keyword-only argument `key` (default none).

The docstring:

Index of the first occurrence of a maximum value in an iterable.

    >>> argmax('abcdefghabcd')
    7
    >>> argmax([0, 1, 2, 3, 3, 2, 1, 0])
    3

For example, identify the best machine learning model::

    >>> models =   ['svm', 'random forest', 'knn', 'naïve bayes']
    >>> accuracy = [  68,        61,          84,       72      ]

    # Most accurate model
    >>> models[argmax(accuracy)]
    'knn'

    # Best accuracy
    >>> max(accuracy)
    84
