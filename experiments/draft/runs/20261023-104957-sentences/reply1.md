```surety
// draft
// drafted
type Pool = { items: Int[] }
// drafted
type IndexError = { code: Text }
// drafted
index_error: IndexError = { code: "IndexError" }

nth_product(index: Int, iterables: Pool[], repeat: Int) -> Result<Int[], IndexError>
	note "return the product at sort position index in the lexicographic order of the products of the iterables taken repeat times, without computing the products before it"
	# The result is the index error exactly when the index is at least the number of products or below minus the number of products, the number of products being the product of the iterables' lengths, taken repeat times over.

	# A product has one element for each iterable in each repetition: its length is the number of iterables times repeat.

	# Each element of a product is an item of the iterable at its position, with the iterables taken in their given order and that sequence repeated repeat times.

	# The result with a repeat is the result with the iterables listed that many times in a row and a repeat of 1.

	# When the index and the index plus one are both in range, the product at the index plus one is the product at the index advanced by one place. The last position not yet at its iterable's final item moves to that iterable's next item, and every position after it goes back to its iterable's first item. If every position is already at its final item, every position goes back to its first item.

	# A negative index counts back from the end: the result at a negative index is the result at the index plus the number of products.

	// example wanted: index 8 over four iterables [0, 1] with repeat 1, and again over one iterable [0, 1] with repeat 4: decided by reading 8 as the positions in each iterable, most significant first, so both calls must agree
	// example wanted: index 0 over iterables of different lengths: every position is at its iterable's first item
	// example wanted: index -1 over the same iterables: every position is at its iterable's last item
	// example wanted: index equal to the number of products: just past the end, decided by the in-range sentence
	// example wanted: index equal to minus the number of products, and one below it: the first is the first product, the second is out of range
	// example wanted: one empty iterable among non-empty ones, index 0: the number of products is 0, so no index is in range
	// example wanted: no iterables, or repeat 0, at index 0 and at index 1: decided by whether the empty product counts as one product
	// example wanted: repeat -1: the docstring is silent; the author decides whether this is the index error, the empty product, or out of scope
	// example wanted: an iterable with a repeated item, such as [5, 5]: an item's position is what is counted, not its value, so the same product appears at two indexes
```