```surety
// draft
// drafted
type Item = { id: Id }
// drafted
type Page = { items: Item[], total_pages: Int, has_next: Bool }
// drafted
type PageError = { code: Text, display: Text }
// drafted
size_error: PageError = { code: "e-page-size", display: "Page size must be at least 1" }
// drafted
page_error: PageError = { code: "e-page-range", display: "Page is out of range" }
// drafted
valid_errors: PageError[] = [size_error, page_error]

paginate(items: Item[], page: Int, size: Int) -> Result<Page, PageError[]>
	note "return the items on the given 1-based page of the list, with the total number of pages and whether a next page exists"
	# The result is Ok exactly when size is at least 1 and page is in range.
	# Page is in range when page is at least 1 and either items is empty and page is 1, or (page - 1) times size is less than the length of items.
	# An Err carries size_error exactly when size is below 1.
	# If size is at least 1, an Err carries page_error exactly when page is not in range.
	# If size is below 1, an Err carries page_error exactly when page is below 1.
	# Every error in an Err is drawn from valid_errors.
	# The errors in an Err are not claimed in any order.
	# In an Ok, the length of items is at most total_pages times size.
	# In an Ok, the length of items is more than (total_pages - 1) times size.
	# In an Ok, total_pages is 0 exactly when items is empty.
	# In an Ok, has_next is true exactly when page is less than total_pages.
	# In an Ok, the page's items are drawn from items.
	# In an Ok, the page's items keep the input's order.
	# In an Ok, if page is less than total_pages, the length of the page's items is size.
	# In an Ok, if page is total_pages, the length of the page's items is the length of items minus (page - 1) times size.
	# In an Ok, if items is empty, the page's items are empty.
	# In an Ok, every item left out of the page comes before every item on the page or after every item on the page.
	# In an Ok on page 1, every item on the page comes before every item left out of it.
	# In an Ok on a page above 1, every item on the previous page, paginate(items, page - 1, size), comes before every item on this page.
	# In an Ok on a page above 1, every item left out of both this page and the previous page comes before every item on the previous page or after every item on this page.
```