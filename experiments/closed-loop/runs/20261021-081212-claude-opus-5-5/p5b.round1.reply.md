```python
def get_first_occurrence_per_email(orders):
    seen = set()
    out = []
    for o in orders:
        email = o["customer_email"]
        if email in seen:
            continue
        seen.add(email)
        out.append({"amount": o["amount"], "customer_email": email})
    return out
```