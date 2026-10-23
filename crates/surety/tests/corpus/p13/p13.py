def paginate(items, page, size):
    errs = []
    if page < 0:
        errs.append({"code": "e-page-negative", "display": "Page number must be at least 0"})
    if size < 1:
        errs.append({"code": "e-page-size", "display": "Page size must be at least 1"})
    if errs:
        return {"err": errs}
    n = len(items)
    if page * size >= max(n, 1):
        return {"err": [{"code": "e-page-past-end", "display": "Page is past the end of the list"}]}
    start = page * size
    end = (page + 1) * size
    page_items = [dict(it) for it in items[start:end]]
    total_pages = (n + size - 1) // size
    has_next = end < n
    return {"ok": {"items": page_items, "total_pages": total_pages, "has_next": has_next}}
