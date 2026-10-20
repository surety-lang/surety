def enrich_orders(orders, users):
    name = users[0]["user_name"] if users else None
    return [{**o, "user_name": name} for o in orders]
