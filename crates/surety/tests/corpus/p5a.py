def get_first_customer_orders(orders):
    earliest = {}
    for o in orders:
        email = o["customer_email"]
        if email not in earliest or o["ordered_at"] < earliest[email]["ordered_at"]:
            earliest[email] = o
    return list(earliest.values())
