from datetime import datetime


def _when(order):
    return datetime.fromisoformat(order["ordered_at"].replace("Z", "+00:00"))


def get_first_customer_orders(orders):
    latest = {}
    for o in orders:
        email = o["customer_email"]
        if email not in latest or _when(o) > _when(latest[email]):
            latest[email] = o
    return list(latest.values())
