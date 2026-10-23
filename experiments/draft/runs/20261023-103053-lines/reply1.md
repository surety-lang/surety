```surety
# For every input iterable there is exactly one output entry with the same position.
	all i in iterables: exactly_one o in out: o.position == i.position
# The output entries are in the same order as the input iterables, by position.
	out.position same_order_as iterables.position
# Each output entry's elements are drawn from the elements of the input iterable at the same position.
	all o in out: all i in iterables where i.position == o.position: o.elements drawn_from i.elements
# Each output entry's elements keep the order they have in the input iterable at the same position.
	all o in out: all i in iterables where i.position == o.position: o.elements same_order_as i.elements
# No element of an output entry appears in any input iterable at a different position.
	all o in out: all e in o.elements: none j in iterables where j.position != o.position: e in j.elements
# Every element of an input iterable that is left out of the output entry at the same position appears in some input iterable at a different position.
	reaching for: a `leftover` taken against a nested output list rather than against `out`. For each input iterable i and the output entry o with o.position == i.position, range over the occurrences in i.elements that remain after removing one occurrence for each element of o.elements (earliest first). Then require every such remaining occurrence e to satisfy `any j in iterables where j.position != i.position: e in j.elements`. Writing `e not in o.elements` instead would lose multiplicity: it would pass an output that drops one copy of a repeated element while keeping another.
```