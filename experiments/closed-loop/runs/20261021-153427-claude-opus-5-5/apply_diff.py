def apply_diff(old, diff):
    old_skus = {i["sku"] for i in old}
    removed_skus = {i["sku"] for i in diff["removed"]}
    changed = {c["sku"]: c for c in diff["changed"]}
    added_skus = [i["sku"] for i in diff["added"]]

    removed_ok = all(s in old_skus for s in removed_skus)
    changed_ok = all(s in old_skus for s in changed)
    added_ok = all(s not in old_skus for s in added_skus)
    disjoint = not any(s in changed for s in removed_skus)

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
    if not disjoint:
        errs.append({
            "code": "e1-conflicting-sku",
            "display": "A sku is in both diff.removed and diff.changed",
        })
    if errs:
        return {"err": errs}

    res = []
    for i in old:
        sku = i["sku"]
        if sku in removed_skus:
            continue
        if sku in changed:
            res.append({"sku": sku, "qty": changed[sku]["new_qty"]})
        else:
            res.append({"sku": sku, "qty": i["qty"]})
    for i in diff["added"]:
        res.append({"sku": i["sku"], "qty": i["qty"]})
    return {"ok": res}
