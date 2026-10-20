MISSING_REMOVED = {
    "code": "e1-missing-removed-sku",
    "display": "Item in diff.removed not present in old",
}
MISSING_CHANGED = {
    "code": "e1-missing-changed-sku",
    "display": "Item in diff.changed not present in old",
}
ADDED_PRESENT = {
    "code": "e1-added-present-sku",
    "display": "Item in diff.added already present in old",
}


def apply_diff(old, diff):
    old_skus = {item["sku"] for item in old}
    errors = []
    if any(r["sku"] not in old_skus for r in diff["removed"]):
        errors.append(MISSING_REMOVED)
    if any(c["sku"] not in old_skus for c in diff["changed"]):
        errors.append(MISSING_CHANGED)
    if any(a["sku"] in old_skus for a in diff["added"]):
        errors.append(ADDED_PRESENT)
    if errors:
        return {"err": errors}
    removed = {r["sku"] for r in diff["removed"]}
    new_qty = {c["sku"]: c["new_qty"] for c in diff["changed"]}
    result = []
    for item in old:
        if item["sku"] in removed:
            continue
        if item["sku"] in new_qty:
            result.append({"sku": item["sku"], "qty": new_qty[item["sku"]]})
        else:
            result.append(item)
    result.extend(diff["added"])
    return {"ok": result}
