def snapshot_diff(old, new):
    old_by_sku = {item["sku"]: item for item in old}
    new_by_sku = {item["sku"]: item for item in new}

    added = []
    changed = []
    for n in new:
        sku = n["sku"]
        if sku not in old_by_sku:
            added.append({"sku": sku, "qty": n["qty"]})
        else:
            o = old_by_sku[sku]
            if o["qty"] != n["qty"]:
                changed.append({"sku": sku, "old_qty": o["qty"], "new_qty": n["qty"]})

    removed = []
    for o in old:
        if o["sku"] not in new_by_sku:
            removed.append({"sku": o["sku"], "qty": o["qty"]})

    return {"added": added, "removed": removed, "changed": changed}
