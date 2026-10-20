from datetime import datetime


def _when(order):
    return datetime.fromisoformat(order["ordered_at"].replace("Z", "+00:00"))


def get_first_customer_orders(orders):
    earliest = {}
    for o in orders:
        email = o["customer_email"]
        if email not in earliest or _when(o) < _when(earliest[email]):
            earliest[email] = o
    return list(earliest.values())
