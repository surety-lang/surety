def get_first_customer_orders(orders):
    latest = {}
    for o in orders:
        email = o["customer_email"]
        if email not in latest or o["ordered_at"] > latest[email]["ordered_at"]:
            latest[email] = o
    return list(latest.values())
