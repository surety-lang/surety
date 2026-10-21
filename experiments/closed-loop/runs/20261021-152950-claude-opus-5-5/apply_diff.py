def apply_diff(old, diff):
    missing_removed_sku_err = {
        "code": "e1-missing-removed-sku",
        "display": "Item in diff.removed not present in old",
    }
    missing_changed_sku_err = {
        "code": "e1-missing-changed-sku",
        "display": "Item in diff.changed not present in old",
    }
    added_already_present_sku_err = {
        "code": "e1-added-present-sku",
        "display": "Item in diff.added already present in old",
    }
    conflicting_sku_err = {
        "code": "e1-conflicting-sku",
        "display": "A sku is in both diff.removed and diff.changed",
    }

    added = diff.get("added", []) or []
    removed = diff.get("removed", []) or []
    changed = diff.get("changed", []) or []

    old_skus = set(i["sku"] for i in old)
    removed_skus = set(i["sku"] for i in removed)
    changed_skus = set(c["sku"] for c in changed)
    added_skus = set(i["sku"] for i in added)

    removed_ok = all(s in old_skus for s in removed_skus)
    changed_ok = all(s in old_skus for s in changed_skus)
    added_ok = all(s not in old_skus for s in added_skus)
    disjoint = not any(s in changed_skus for s in removed_skus)

    errs = []
    if not removed_ok:
        errs.append(dict(missing_removed_sku_err))
    if not changed_ok:
        errs.append(dict(missing_changed_sku_err))
    if not added_ok:
        errs.append(dict(added_already_present_sku_err))
    if not disjoint:
        errs.append(dict(conflicting_sku_err))
    if errs:
        return {"err": errs}

    new_qty = {c["sku"]: c["new_qty"] for c in changed}
    res = []
    for i in old:
        sku = i["sku"]
        if sku in removed_skus:
            continue
        if sku in new_qty:
            res.append({"sku": sku, "qty": new_qty[sku]})
        else:
            res.append({"sku": sku, "qty": i["qty"]})
    for i in added:
        res.append({"sku": i["sku"], "qty": i["qty"]})
    return {"ok": res}
