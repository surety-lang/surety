def flatten_order(order):
    return [
        {
            "line_id": item["id"],
            "order_id": order["id"],
            "customer_email": order["customer_email"],
            "amount": item["amount"],
            "item_id": item["item_id"],
        }
        for item in order["items"]
    ]
