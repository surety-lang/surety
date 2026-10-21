# Decision: `accept` marks a deliberate non-decision (2026-10-21)

Slice 2's search reports an input as underdetermined when two outputs that differ pass every clause. Some of those are holes; some are choices the author made on purpose. P5a keeps the earliest order per email, and when two orders share an email and a timestamp, either is the intent. Under `--strict` that intent failed the check, because the obligation had no way to say "I know, and I mean it."

## Rule

`accept <boolean over the inputs>` is a line in the property slot. It names the inputs on which the obligation deliberately admits more than one output. For P5a:

```
accept any a in orders: any b in orders where b != a and b.customer_email == a.customer_email: b.ordered_at == a.ordered_at
```

- An underdetermined input that satisfies an `accept` is reported as ACCEPTED, counted, and does not fail `--strict`. One that satisfies no `accept` is UNDERDETERMINED, as before.
- `accept` is part of the obligation and of its hash. It is a claim ("on these inputs, either answer is right"), not a note, so it cannot be a free-text line, and a reviewer sees it where the properties are.
- An `accept` that no example or generated input satisfies is itself a finding (a dead accept), so the line cannot be used to silence the check with a predicate that never fires.
- `accept true` is refused. The predicate has to mention an input.

## Why not a note

A note is pinned to the obligation and left out of the hash; it is for readers. An `accept` changes what the checker reports and what `--strict` means, so it has to be in the hash and in the properties, or anyone could silence a finding with a comment.

## Why not a keyword per kind of non-decision

`unordered` is the same move for one specific non-decision, order, and it earned a word because every list obligation has to say something about order. `accept` is the general form for the rest; if one kind recurs across the corpus the way order did, it can get its own word then.

## What it gives the graph

When the store exists, the accepted list per obligation is the set of `accept` lines, hashed with it. There is no separate list to maintain.
