# Spot-check 1 of the reach-for census (2026-10-23)

Run stopped by hand at about 11:06. At that point `results.jsonl` held 110 lines for 107 ids, all complete: 96 from more-itertools, 11 from the itertools recipes, and no other source reached. Read-only check, except this file. Ids below drop the `more_itertools__` prefix.

## 1. Recording

I checked all 107 records against disk by script, not just ten: rounds against `parse*.txt` and `reply*.md`; `parse_ok` against the last parse file; `stage1.sentences` and `examples_wanted` against the stage-1 `after.sur` and against `out/<id>.sur`; `stage2.lines` against every single-tab property line in `after.sur` and against the checker's own `props=N`; `stage2.reach` against the `// reaching for:` comments (continuation lines joined); `mutants_summary` against the first line of `mutants.txt`; `mutants` verbatim; tokens against the log; `out/<id>.sur` byte-equal to the stage-2 `after.sur`. I also compared twelve by hand, side by side: `more__first`, `more__chunked`, `more__distinct_permutations`, `more__always_iterable`, `more__interleave_evenly`, `more__minmax`, `more__filter_map`, `more__gray_product`, `more__constrained_batches`, `more__idft`, `recipes__tabulate`, `recipes__nth`.

**No field mismatches.** Every sentence has exactly one line or one reach (434 + 121 = 555 sentences).

Things the fields do not say:
- `stage2.lines` in the 6 stage-2 parse failures counts lines in a file that never parsed (`more__distinct_permutations` 7, `more__intersperse` 7, `more__distribute` 6, `more__circular_shifts` 7, `more__zip_broadcast` 5, `more__constrained_batches` 8). `mutants_summary` is null for these, and `mutants` holds the parse error.
- `stage2.parse_ok` is draft.py's exit code, which is `surety mutants`' exit, not the parse result. It agrees with the last parse file on all 107 records today.
- `underdetermined` is not part of `applicable` (`more__minmax`: "mutants 50 of 61 apply  slipped 0  caught 50  underdetermined 6 in 2 gaps").
- Tokens are uncached input only (`msg.usage.input_tokens`). Cache reads and writes on the stage-1 system prompt (~6.5k characters) are not recorded. If `fallbacks: "default"` sent a request to another model, the record would not say so.

## 2. Reach-for capture

- `out/*.sur`: 121 `// reaching for:` comments across 107 recorded drafts (48 drafts have at least one). All 121 are in their record's `stage2.reach`, in order. None was made up by the tool ("a line the model gave no answer for" or "…as an example or an accept": 0).
- Raw stage-2 replies (`reply*.md`, every round): no text outside the block, no reach-for in another wording, none written as a comment, no pair line that is prose. **Missed in pair form: 0.**
- One author-facing comment the tool dropped (it skips `//` lines in a reply). `more__repeat_last`, beneath `out[0 .. len(items)] == items[0 .. count]`: "// the gloss after the colon, read literally (out[0 .. count] == items[0 .. count]), is false whenever count > len(items) … The author may want to reword the gloss". It is not a reach-for, but it is the only place the model says its line differs from its sentence.
- Where reach-fors go uncounted instead:
  - **The six stage-2 failures are reach-fors in all but name.** In each, the model wrote a form the fragment lacks, and the record files it under `lines`:
    - `more__zip_broadcast`: `out == Err(length_error) <-> strict and any a in objects where not a.scalar: …` (pattern_as_value)
    - `more__intersperse`: `(n >= 1 <-> exactly_one Ok(_) in out) and all Err(x) in out: x == n_error` (quantifier_mid_line)
    - `more__distinct_permutations`: `r != None and r >= 0 and r <= len(iterable) -> …` (Option<Int> compared with Int)
  - **Retry rounds move lines into reach-fors.** Each retry regenerates every pair, not just the failing one. Over the 24 multi-round stage-2 runs: 20 sentences went line→reach, 0 went reach→line, and 59 lines were rewritten. So 20 of the 121 final reach-fors (17%) were lines in round 1. Examples: `more__filter_map` r1 `out same_order_as iterable.result` → "reaching for: `out same_order_as` the list of the Int inside each non-None result…"; `recipes__tabulate` r1 `all Err(err) in out: err == uncovered_arg_error <-> …` → "reaching for: … it needs a list of the integers from start up to but not including…". The record keeps only the final round. Earlier rounds are on disk.

## 3. Failures

- complete=false / error, all superseded by a later complete record (load_results keeps the last line per id, but any reader of the raw file must dedupe):
  - `more__chunked`, `more__first`: "FileExistsError: [Errno 17] File exists: '…/20261023-102629-sentences'"
  - `more__ilen`: the same, on '…/20261023-102629-2-sentences'
  - Cause: two draft.py starts in one second racing on the run directory, fixed by be59dc6. Their logs were overwritten by the re-runs. 9 orphan run directories remain from 10:26:29–37.
- Stage 1 never parsed: **none.** Two needed a second round: `more__collapse` "type definition is circular (type_error)" and `more__always_iterable` "unexpected character `\` (syntax)".
- Stage 2 still failing after 3 rounds, by the final error:
  - pattern_as_value (`out == Err(x)`): `more__circular_shifts` l.13, `more__zip_broadcast` l.16, `more__constrained_batches` l.17
  - quantifier_mid_line: `more__intersperse` l.11, `more__distribute` l.11
  - type_error, `>=` on Option<Int>: `more__distinct_permutations` l.27
  - Five of the six are one sentence shape: "The output is <error> exactly when <condition>". The three-round sequences alternate, e.g. `more__zip_broadcast` pattern_as_value → quantifier_mid_line → pattern_as_value.
- All stage-2 errors, every round: pattern_as_value 18, quantifier_mid_line 8, type_error 6 (Option<Int> 5, `sort key items is not ordered` 1), syntax 2, else_without_where 1, element_type_mismatch 1.
- Left by the stop at about 11:06 (no record): `recipes__repeatfunc`, `recipes__grouper` (stage 1 done, stage 2 KeyboardInterrupt), `recipes__partition`, `recipes__powerset` (stage 1 KeyboardInterrupt), and `recipes__unique_everseen`, `recipes__unique_justseen`, `recipes__unique`, `recipes__first_true`, all started *after* the interrupt (see the end). `recipes__unique_everseen` finished both stages (logs say exit 0; run dirs 110622-sentences and 110634-lines) but has no record. `out/` holds 4 unrecorded files; `out/recipes__repeatfunc.sur` is a half-written stage-2 file carrying one reach-for.

## 4. Draft habits

- **Arbitrary elements become Int.** List-typed parameters in the 107 signatures: Int 45, Text 20, a drafted record `Item` 19, `Element` 4, `Id` once (`more__interleave_evenly`). `more__interleave`: "the iterables are finite lists of Int: the fragment has no generic element type and no infinite list". `more__distinct_permutations`: "elements are drafted as Int, so the input is always sortable; the docstring's equal-but-not-identical case (1 and True) has no counterpart here". `more__first`: "the element type is fixed to Int, so the docstring's Text default is out of scope".
- **Higher-order arguments, five ways:**
  - The predicate's verdict as a field. 17 drafts have a record with a Bool field. `more__adjacent`: `type Item = { value: Int, satisfies: Bool }`, note "the predicate is called once per item, so its result arrives as each item's satisfies".
  - The function as a table. `more__iterate`: `func: Step[unique from]` with `type Step = { from: Int, to: Int }`. `recipes__tabulate`: `func: Entry[unique arg]`.
  - A sibling obligation called by name. 17 drafts name one. `more__map_if`: `all x in items[0 .. 1] where pred(x): any y in out[0 .. 1]: y == func(x)`. `more__nth_prime`: `is_prime(out)`.
  - The function fixed. `more__difference`: "func is fixed to subtraction: the fragment has no function-typed input, so a func other than sub would be a sibling obligation".
  - The function dropped. `more__sort_together`: "the key function argument is not modeled".
- **Generators and infinite outputs** get a bound added, or the infinite case cut. `more__repeat_last`: "the Python function yields forever; the obligation models the first `count` values, as `islice(repeat_last(items, default), count)` does". `more__padded`: "n of None (fillvalue emitted indefinitely) gives an infinite output and is not modelled". `more__count_cycle` note: "the indefinite case with n omitted is not modelled here". `recipes__repeatfunc` note: "with times None the endless iterable is the results drawn so far". These are stage-1 reach-fors (no infinite list, no function type) that sit in `//` comments and notes, which the census does not count.
- **Sentences that are instances:** essentially none. 1 of 555 sentences has a literal beyond 0 or 1 (`more__subfactorial`, a recurrence). Related: 7 sentences only *withhold* a claim, all as `unordered` lines (`more__map_reduce` "The order of the categories is not claimed." → `out unordered`; also `more__distinct_combinations`, `more__join_mappings`, `more__set_partitions`, `more__powerset_of_sets`).
- **`// example wanted` carrying an output:** 0 of 761 use `->`, so the tool's check fires on none. About 17 still state the output in prose. `more__windowed`: "seq [1, 2], n 0, step 0: … the output is one empty window". `more__split_at`: "the empty input, maxsplit -1, no keep: the output is still one group, and it is empty". `more__mark_ends`: "the empty list; the output is empty." `more__powerset_of_sets`: "an empty iterable: the output is the single subset with no members". Others leave it open, as asked: `more__split_when`: "so decide whether the output is no pieces or one empty piece".
- **e: blocks or accept lines stripped:** none. `stage1.notes` is empty in all 107 records, and `stage2.refused` is empty in all 107.
- **Recovering from `out == Err(x)`:** the model rewrites it as an Err binder. `more__windowed` r1 `n < 0 <-> out == Err(negative_n_error)` → `n < 0 <-> exactly_one Err(e) in out: e == negative_n_error`. Also `more__nth_or_last`, `more__windowed_complete`, `more__nth_permutation`, `recipes__nth` (as `any Err(e)`).
- **Unwrapping an Option** is the commonest named reach. `more__collapse`: "a way to compare against the Int inside an Option<Int> (an unwrap, or a `Some(n)` binder over levels)". The same reach appears in `more__set_partitions`, `more__replace`, `more__interleave_evenly` and `more__always_iterable`.
- **A line that plainly differs from its sentence:** `more__repeat_last`. Sentence: "the output from position 0 up to count is the items from position 0 up to count"; line: `out[0 .. len(items)] == items[0 .. count]`. The model said so in the comment the tool dropped (§2). A heuristic sweep (iff without `<->`, "order" without an order form, "no two" without `unique`, "every" as `any`) turned up nothing else that is plainly off.

## 5. Cost and pace

107 complete records, effort high, `claude-opus-5-5`:

| | tokens_in mean / max | tokens_out mean / max | seconds mean / max | more than one round |
|---|---|---|---|---|
| stage 1 | 15,776 / 31,468 (`more__always_iterable`) | 3,003 / 11,310 (`more__split_at`) | 32 / 117 (`more__split_at`) | 2 of 107 (2%), none needed 3 |
| stage 2 | 20,455 / 49,074 (`more__constrained_batches`) | 3,082 / 23,931 (`more__extract`) | 32 / 241 (`more__extract`) | 24 of 107 (22%), 6 needed 3 |

- Spent on the 107: 3.88M in, 0.65M out, about **$28.5** at $4 / $20 per MTok, about $0.27 per function. The unrecorded runs after the stop and the cache tokens add a little.
- Wall time with `--jobs 3`: 38.7 min for 107 (21.7 s per function; 64 s per function serial).
- Projected for 333: about 12.1M in, 2.0M out, **about $89**, about **2.0 h** at `--jobs 3`. The other 226 are other libraries (boltons, toolz, django, requests, humanize); recipes and more-itertools may not be typical of them.

## Before resuming, in run_census.py

1. On KeyboardInterrupt, cancel queued futures (`shutdown(cancel_futures=True)`). After the 11:06:20 interrupt, the pool started five more draft.py runs, and `recipes__unique_everseen` finished unrecorded, so a resume pays for it twice.
2. Record every round's parse error and each round's reach count, not just `last_error`. 20 of the 121 reach-fors were lines in round 1, and the 6 failures' unparsed lines are reach-fors the record files under `lines`.
3. Take `stage2.parse_ok` from the last `parse*.txt` (": ok "), not from draft.py's exit, which is the exit of `surety mutants`.
4. Set `stage2.lines` to null, or add `lines_parsed: false`, when the file never parsed.
