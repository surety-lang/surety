more_itertools.more.join_mappings (from more-itertools 11.2.1)

Signature, in words: `join_mappings` takes any keyword arguments `field_to_map`.

The docstring:

Joins multiple mappings together using their common keys.

>>> user_scores = {'elliot': 50, 'claris': 60}
>>> user_times = {'elliot': 30, 'claris': 40}
>>> join_mappings(score=user_scores, time=user_times)
{'elliot': {'score': 50, 'time': 30}, 'claris': {'score': 60, 'time': 40}}
