```surety
// draft
// drafted
type Event = { id: Id, occurred_at: Text }
// drafted
type MonthBucket = { month: Text, events: Event[] }

bucket_by_month(events: Event[]) -> MonthBucket[]
	note "group events into one bucket per month, the month read from each event's ISO-8601 occurred_at string"
	# For every input event, exactly one bucket has that event's month, where a sibling obligation decides the month of an ISO-8601 string.
	# No two buckets share a month.
	# Every event in a bucket has that bucket's month.
	# The events across all buckets, taken together, are the same elements as the input events.
	# Every bucket holds at least one event.
	# The order of the buckets is not claimed.
	# The order of the events within a bucket is not claimed.
	// example wanted: no events at all; no input event exists to give any bucket a month
	// example wanted: two events on the same month of different years, "2025-10-01T00:00:00Z" and "2026-10-01T00:00:00Z"; decided by whether the month includes the year
	// example wanted: one event at "2026-10-31T23:30:00-02:00"; decided by whether the month is read from the written local date or taken after converting to UTC
	// example wanted: two events with identical id and occurred_at; decided by "same elements as" keeping both copies
```