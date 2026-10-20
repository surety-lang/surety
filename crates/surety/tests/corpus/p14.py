def snapshot_diff(old, new):
    old_by_sku = {item["sku"]: item for item in old}
    new_by_sku = {item["sku"]: item for item in new}
    added = [item for item in new if item["sku"] not in old_by_sku]
    removed = [item for item in old if item["sku"] not in new_by_sku]
    changed = [
        {"sku": sku, "old_qty": old_item["qty"], "new_qty": new_by_sku[sku]["qty"]}
        for sku, old_item in old_by_sku.items()
        if sku in new_by_sku and old_item["qty"] != new_by_sku[sku]["qty"]
    ]
    return {"added": added, "removed": removed, "changed": changed}
