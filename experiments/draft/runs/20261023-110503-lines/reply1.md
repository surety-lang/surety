```surety
# The dot product is the product of the first elements of vec1 and vec2, with a missing first element counting as 0, plus the dot product of the two vectors with their first elements dropped.
	out == sum(vec1[0 .. 1]) * sum(vec2[0 .. 1]) + dotproduct(vec1[1 .. len(vec1)], vec2[1 .. len(vec2)])
# If every element of vec2 is 1 and vec2 is at least as long as vec1, the dot product is the sum of vec1.
	len(vec2) >= len(vec1) -> out == sum(vec1) or any x in vec2: x != 1
```