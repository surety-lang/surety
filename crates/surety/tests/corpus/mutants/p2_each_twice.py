def get_small_orders(orders):
    out = []
    for o in orders:
        if o["amount"] <= 100:
            out.append(o)
            out.append(o)
    return out
