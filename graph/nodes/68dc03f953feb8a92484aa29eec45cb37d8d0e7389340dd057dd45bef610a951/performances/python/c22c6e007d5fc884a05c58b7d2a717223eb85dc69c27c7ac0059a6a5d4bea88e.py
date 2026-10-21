def enrich_orders(orders, users):
    names = {u["id"]: u["user_name"] for u in users}
    return [
        {**o, "user_name": names.get(o["user_id"]) if o["user_id"] is not None else None}
        for o in orders
    ]
