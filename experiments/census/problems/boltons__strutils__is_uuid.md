boltons.strutils.is_uuid (from boltons 26.2.0)

Signature, in words: `is_uuid` takes the required argument `obj`; the optional argument `version` (default `4`).

The docstring:

Check the argument is either a valid UUID object or string.

Args:
    obj (object): The test target. Strings and UUID objects supported.
    version (int): The target UUID version, set to 0 to skip version check.

>>> is_uuid('e682ccca-5a4c-4ef2-9711-73f9ad1e15ea')
True
>>> is_uuid('0221f0d9-d4b9-11e5-a478-10ddb1c2feb9')
False
>>> is_uuid('0221f0d9-d4b9-11e5-a478-10ddb1c2feb9', version=1)
True
