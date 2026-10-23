boltons.iterutils.remap (from boltons 26.2.0)

Signature, in words: `remap` takes the required argument `root`; the optional arguments `visit` (default the `default_visit` function), `enter` (default the `default_enter` function), `exit` (default the `default_exit` function), `cache` (default `True`); any keyword arguments `kwargs`.

The docstring:

The remap ("recursive map") function is used to traverse and
transform nested structures. Lists, tuples, sets, and dictionaries
are just a few of the data structures nested into heterogeneous
tree-like structures that are so common in programming.
Unfortunately, Python's built-in ways to manipulate collections
are almost all flat. List comprehensions may be fast and succinct,
but they do not recurse, making it tedious to apply quick changes
or complex transforms to real-world data.

remap goes where list comprehensions cannot.

Here's an example of removing all Nones from some data:

>>> from pprint import pprint
>>> reviews = {'Star Trek': {'TNG': 10, 'DS9': 8.5, 'ENT': None},
...            'Babylon 5': 6, 'Dr. Who': None}
>>> pprint(remap(reviews, lambda p, k, v: v is not None))
{'Babylon 5': 6, 'Star Trek': {'DS9': 8.5, 'TNG': 10}}

Notice how both Nones have been removed despite the nesting in the
dictionary. Not bad for a one-liner, and that's just the beginning.
See `this remap cookbook`_ for more delicious recipes.

.. _this remap cookbook: http://sedimental.org/remap.html

remap takes four main arguments: the object to traverse and three
optional callables which determine how the remapped object will be
created.

Args:

    root: The target object to traverse. By default, remap
        supports iterables like :class:`list`, :class:`tuple`,
        :class:`dict`, and :class:`set`, but any object traversable by
        *enter* will work.
    visit (callable): This function is called on every item in
        *root*. It must accept three positional arguments, *path*,
        *key*, and *value*. *path* is simply a tuple of parents'
        keys. *visit* should return the new key-value pair. It may
        also return ``True`` as shorthand to keep the old item
        unmodified, or ``False`` to drop the item from the new
        structure. *visit* is called after *enter*, on the new parent.

        The *visit* function is called for every item in root,
        including duplicate items. For traversable values, it is
        called on the new parent object, after all its children
        have been visited. The default visit behavior simply
        returns the key-value pair unmodified.
    enter (callable): This function controls which items in *root*
        are traversed. It accepts the same arguments as *visit*: the
        path, the key, and the value of the current item. It returns a
        pair of the blank new parent, and an iterator over the items
        which should be visited. If ``False`` is returned instead of
        an iterator, the value will not be traversed.

        The *enter* function is only called once per unique value. The
        default enter behavior support mappings, sequences, and
        sets. Strings and all other iterables will not be traversed.
    exit (callable): This function determines how to handle items
        once they have been visited. It gets the same three
        arguments as the other functions -- *path*, *key*, *value*
        -- plus two more: the blank new parent object returned
        from *enter*, and a list of the new items, as remapped by
        *visit*.

        Like *enter*, the *exit* function is only called once per
        unique value. The default exit behavior is to simply add
        all new items to the new parent, e.g., using
        :meth:`list.extend` and :meth:`dict.update` to add to the
        new parent. Immutable objects, such as a :class:`tuple` or
        :class:`namedtuple`, must be recreated from scratch, but
        use the same type as the new parent passed back from the
        *enter* function.
    cache (bool): Controls whether to cache transformed
        objects. Uses object identity for the cache. For example
        this is turned off for applications like `research` which
        need to traverse all trees.
    reraise_visit (bool): A pragmatic convenience for the *visit*
        callable. When set to ``False``, remap ignores any errors
        raised by the *visit* callback. Items causing exceptions
        are kept. See examples for more details.
    trace (bool): Pass ``trace=True`` to print out the entire
        traversal. Or pass a tuple of ``'visit'``, ``'enter'``,
        or ``'exit'`` to print only the selected events.

remap is designed to cover the majority of cases with just the
*visit* callable. While passing in multiple callables is very
empowering, remap is designed so very few cases should require
passing more than one function.

When passing *enter* and *exit*, it's common and easiest to build
on the default behavior. Simply add ``from boltons.iterutils import
default_enter`` (or ``default_exit``), and have your enter/exit
function call the default behavior before or after your custom
logic. See `this example`_.

Duplicate and self-referential objects (aka reference loops) are
automatically handled internally, `as shown here`_.

.. _this example: http://sedimental.org/remap.html#sort_all_lists
.. _as shown here: http://sedimental.org/remap.html#corner_cases
