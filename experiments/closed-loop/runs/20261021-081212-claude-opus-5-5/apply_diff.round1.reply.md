```python
def apply_diff(old, diff):
    added = diff["added"]
    removed = diff["removed"]
    changed = diff["changed"]

    old_skus = set(item["sku"] for item in old)

    removed_ok = all(r["sku"] in old_skus for r in removed)
    changed_ok = all(c["sku"] in old_skus for c in changed)
    added_ok = all(a["sku"] not in old_skus for a in added)

    if not (removed_ok and changed_ok and added_ok):
        errs = []
        if not removed_ok:
            errs.append({
                "code": "e1-missing-removed-sku",
                "display": "Item in diff.removed not present in old",
            })
        if not changed_ok:
            errs.append({
                "code": "e1-missing-changed-sku",
                "display": "Item in diff.changed not present in old",
            })
        if not added_ok:
            errs.append({
                "code": "e1-added-present-sku",
                "display": "Item in diff.added already present in old",
            })
        return {"err": errs}

    removed_skus = set(r["sku"] for r in removed)
    changed_map = {}
    for c in changed:
        changed_map[c["sku"]] = c["new_qty"]

    result = []
    seen = set()
    for item in old:
        sku = item["sku"]
        if sku in removed_skus:
            continue
        if sku in seen:
            continue
        if sku in changed_map:
            result.append({"sku": sku, "qty": changed_map[sku]})
        else:
            result.append({"sku": sku, "qty": item["qty"]})
        seen.add(sku)

    for a in added:
        sku = a["sku"]
        if sku in seen:
            continue
        result.append({"sku": sku, "qty": a["qty"]})
        seen.add(sku)

    return {"ok": result}
```