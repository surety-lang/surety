```python
import re
from datetime import datetime, timedelta, timezone

_RFC3339 = re.compile(
    r"^(\d{4})-(\d{2})-(\d{2})[Tt ](\d{2}):(\d{2}):(\d{2})(\.(\d+))?"
    r"([Zz]|[+-]\d{2}:\d{2})?$"
)


def _time_key(s):
    """Return a sortable key for an RFC 3339 timestamp; falls back to raw string."""
    if isinstance(s, str):
        m = _RFC3339.match(s.strip())
        if m:
            try:
                year, month, day, hour, minute, second = (int(m.group(i)) for i in range(1, 7))
                frac = m.group(8) or ""
                tz = m.group(9)
                leap = 0
                if second == 60:
                    second = 59
                    leap = 1
                if tz is None or tz in ("Z", "z"):
                    tzinfo = timezone.utc
                else:
                    sign = 1 if tz[0] == "+" else -1
                    oh, om = int(tz[1:3]), int(tz[4:6])
                    tzinfo = timezone(sign * timedelta(hours=oh, minutes=om))
                dt = datetime(year, month, day, hour, minute, second, tzinfo=tzinfo)
                dt = dt + timedelta(seconds=leap)
                utc = dt.astimezone(timezone.utc)
                base = (utc - datetime(1, 1, 1, tzinfo=timezone.utc))
                total_seconds = base.days * 86400 + base.seconds
                frac_key = frac.rstrip("0")
                # pad to compare fractional parts numerically
                return (0, total_seconds, frac_key.ljust(30, "0"), s)
            except (ValueError, OverflowError):
                pass
    return (1, 0, "", str(s))


def get_first_customer_orders(orders):
    best = {}
    order_of_emails = []
    for o in orders:
        email = o["customer_email"]
        key = _time_key(o["ordered_at"])
        if email not in best:
            best[email] = (key, o)
            order_of_emails.append(email)
        else:
            if key < best[email][0]:
                best[email] = (key, o)
    return [dict(best[e][1]) for e in order_of_emails]
```