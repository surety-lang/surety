def get_small_orders(orders):
    return [dict(o) for o in orders if o["amount"] <= 100]
