def get_small_orders(orders):
    return [o for o in orders if o["amount"] <= 100]
