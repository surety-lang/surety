# Spec: in-toto evidence emission

2026-10-23. A design, not an implementation. Status: draft for discussion.

## BLUF

Surety already produces the thing an attestation is supposed to carry: a verdict computed by a program about one performance against one signed obligation, with the run log that lets anyone recompute it. Today that record lives only in Surety's own format under `graph/`. This spec adds one derived projection and nothing else: each admitted evidence record can be emitted as an in-toto Statement (v1), wrapped in a DSSE envelope, so that tooling Surety does not build (Sigstore and `cosign`, GitHub artifact attestations, Conftest and OPA, gittuf, in-toto policy engines) can require "a passing Surety verdict for every changed performance" without understanding Surety.

Three rules keep it small. The record in `graph/` stays the source of truth; the statement is a pure function of it, so two emissions of one record produce byte-identical payloads. The checker's identity signs the evidence, and a person's identity signs the obligation; the two signatures never stand in for each other. Nothing is attested that the kernel has not admitted.

## What exists already

The store (`crates/surety-graph/src/store.rs`) holds, per obligation node `<ob>` under `graph/nodes/<ob>/`: `canonical.sur` (the canonical form; `<ob>` is its SHA-256), `presentations/<text hash>.sur` (the file as the author wrote it), `performances/<target>/<perf>.py` (the performance bytes as pinned; `<perf>` is a structural hash over the normalized Python AST, itself a SHA-256 hex string), and `evidence/<target>/<perf>.json` (the record).

A record has two parts (`record.rs`). The `evidence` part is a function of the obligation hash, the performance hash and the budget, and of nothing else: no timestamp, no absolute path, no machine name; it is compared byte for byte by `verify`. The `provenance` part (checker version, Python version, source paths, presentation hash, sibling performances, record `format`) is not compared. The kernel (`admit`) checks bookkeeping, never logic: hashes are the content's, the run log holds exactly the inputs the obligation and budget produce, every count in the evidence is a count over recorded runs. `verify --rerun` runs the check again and requires the evidence byte for byte.

Everything the attestation needs is therefore already pinned and already reproducible. The gap is only the wire format and a signature.

## What we are and are not doing

Doing: a serializer from record to Statement; a canonical byte form for the payload; an envelope with a signature from a key the checker's operator controls; a check in `verify` that an emitted payload is the record's projection; a compatibility projection onto the vetted `test-result` predicate so consumers that only accept registered predicates can still gate on it.

Not doing: a new evidence kind, a change to the record, a hosted log or registry, a human-signature workflow (the obligation delta is signed by a person through git commit signing today, and that is where it stays), a merge gate of our own, or registration of the predicate with in-toto (that is a later decision, see open questions). Nothing here blocks on any in-toto proposal merging; whichever shape lands, this record maps onto it because it carries more than any of them ask for.

## The Statement

An in-toto Statement v1 has four fields: `_type`, `subject`, `predicateType`, `predicate`. Subjects are matched by digest only. The design decisions are what the subject is, what the predicate type is, and what the predicate carries.

### Subject: the performance

One statement per record; the subject is the performance. A consumer holding a PR can hash the changed `.py` file and look up a statement by that digest, which is the whole point.

| Field | Value | Why |
| --- | --- | --- |
| `name` | the `performance_source` path from provenance, repo-relative (for the corpus, `crates/surety/tests/corpus/p6.py`), or `_` if unknown | A stable, human-readable identifier; the pinned copy under `graph/` has the same bytes |
| `digest.sha256` | SHA-256 of the pinned performance bytes | The one algorithm every consumer accepts; `pin` copies the source file's bytes unchanged, so the source file and the pinned copy hash alike |
| `digest.gitBlob` | git blob id of the same bytes | Lets git-native consumers (gittuf, forge APIs) match the subject against a tree without rehashing |
| `digest.suretyPerformance` | the structural hash `<perf>` | A custom algorithm name, allowed by the DigestSet rules when its encoding is documented: SHA-256 hex over the normalized AST dump (locals renamed in order of appearance, formatting erased). Consumers that do not know it ignore it; Surety-aware consumers use it to recognise the same performance under a different spelling |

The obligation is not a subject. It is the policy the subject was judged against, and it belongs in the predicate the way `test-result` carries `configuration` and `svr` carries `verifier.policies`. A commit is not a subject either: the statement says "this performance satisfies this obligation", not "this commit was reviewed"; binding to a commit is the consumer's join (the file's blob is in the commit's tree) or a separate attestation.

### Predicate type

`https://surety-lang.org/attestation/evidence/v1` (domain to be confirmed; see open questions). A TypeURI need not resolve, but it must be a namespace we control. The version is the predicate's, and it moves with the record `format`: a record `format` bump that changes the predicate's shape bumps the predicate version; the predicate also carries `recordFormat` so a consumer can tell the two apart.

`payloadType` in the envelope is the generic `application/vnd.in-toto+json`, since the predicate is not registered under the in-toto namespace.

### Predicate contents

Field names in lowerCamelCase per the in-toto predicate guidelines. Everything is copied or derived from the record; nothing is computed fresh. The predicate deliberately omits the run log and the full report, which together are most of the record (about 114 KB for `flatten_order`); instead it carries a digest of the full evidence part, so the attestation binds to the whole record without embedding it.

| Field | Type | Source | Meaning |
| --- | --- | --- | --- |
| `obligation` | ResourceDescriptor | node | `name`: the obligation's name; `digest.sha256`: `<ob>` (SHA-256 of `canonical.sur`); `uri`: repo-relative path `graph/nodes/<ob>/canonical.sur`; `mediaType`: `text/x.surety-canonical` |
| `presentation` | ResourceDescriptor | provenance | `digest.sha256`: the presentation text hash; `uri`: its path under the node. This is the file a person signed; a consumer that wants "the spec a human approved" uses this, not the canonical form |
| `target` | string | evidence | `python` |
| `checker` | object | provenance | `name`: `surety`; `version`: checker version; `python`: interpreter version. Identity, not authority: authority is the envelope's key |
| `recordFormat` | integer | provenance | the record `format` |
| `budget` | object | evidence | `generated`, `mutants`, `searchCandidates`, `searchInputs`, `timeoutSecs`. Part of what the evidence is a function of, so a consumer can require a minimum |
| `verdict` | object | evidence | `line`: the verdict line as printed; `status`: `ACCEPTED` or the record's other verdict words; `passed`, `strictPassed`, `findings` |
| `examples` | object | evidence | `passed`, `total` |
| `properties` | object | evidence | `passed`, `total`, `runs`, `findings`, and `each[]` with `clause` (canonical ordinal), `canonical` (the canonical print, as the label), `runs`, `findings` |
| `determinism` | object | evidence | `matched`, `total` |
| `totality` | object | evidence | `answered`, `raised`, `timeouts` (derived: `total - answered - raised` or as the record states), `total` |
| `boundary` | object | evidence `types` | `boundaryFailures`, `exampleInputs`, `generatedInputs`, `outputs` |
| `underdetermination` | object | evidence | `total`, `applicable`, `caught`, `slipped`, `accepted`, `open`, `underdeterminedInputs`, `causes` |
| `purity` | string | evidence | `enforced`, `unverified`, or `proven`, as the record says; never upgraded |
| `imports`, `accepts`, `dependencies`, `uncheckedReferences` | arrays | evidence | Copied; these are the things the verdict leans on |
| `evidenceDigest` | DigestSet | derived | `sha256` over the JCS (RFC 8785) serialization of the record's `evidence` object. Binds the statement to the full record; `verify` recomputes it |
| `record` | ResourceDescriptor | store | `uri`: `graph/nodes/<ob>/evidence/<target>/<perf>.json`; no digest (the file also holds provenance, which is allowed to differ) |
| `reproduce` | object | fixed | `command`: `surety verify --rerun`; `requires`: `checker.version` and `python`. States how a third party re-derives the verdict; not a promise about toolchain availability |

What is left out, and why: no timestamp (the record has none on purpose; the guideline's RFC 3339 field would make two emissions of one record differ, so the time of attestation lives in the signature layer: a transparency-log entry or the signing certificate); no absolute paths or machine names (same rule as the record); no model or session identifiers (who drafted the obligation or wrote the performance is provenance of authorship, which is a different predicate family, the one the in-toto proposals #600, #601 and #604 are about; mixing it in would make the evidence statement non-monotonic and would tempt consumers to trust a verdict because of who produced the code).

Monotonic principle: the predicate states only facts about recorded runs. A `FAILED` or `REJECTED` verdict is attested too (the kernel admits records of failing checks), but a policy must require the presence of a passing statement, never the absence of a failing one, because any bundle can have lines deleted.

### Worked example

From the committed record for `flatten_order` (obligation `1501313…`, performance `3f3c77d…`). Digests below are real, computed from the checkout on 2026-10-23.

```json
{
  "_type": "https://in-toto.io/Statement/v1",
  "subject": [
    {
      "name": "crates/surety/tests/corpus/p6.py",
      "digest": {
        "sha256": "83ac8db86de152657fdb6bedb2261d74fba7cd5f7676e22c41b7e31eeb74d7ff",
        "gitBlob": "30d524cb8ed0d161728dbd088d41b5f5c166061e",
        "suretyPerformance": "3f3c77da4d7657b1340785abad4f99b099975c6d37d1c94c2baac258b0f5105d"
      }
    }
  ],
  "predicateType": "https://surety-lang.org/attestation/evidence/v1",
  "predicate": {
    "obligation": {
      "name": "flatten_order",
      "uri": "graph/nodes/150131328a400b324de692005af855135d96915582d15624b2568174b51ce0f5/canonical.sur",
      "digest": { "sha256": "150131328a400b324de692005af855135d96915582d15624b2568174b51ce0f5" },
      "mediaType": "text/x.surety-canonical"
    },
    "presentation": {
      "uri": "graph/nodes/150131328a400b324de692005af855135d96915582d15624b2568174b51ce0f5/presentations/f105f63dd5f242494dcd2d96c636a86aca78bd63f5d8c67164dd54a1c97b4251.sur",
      "digest": { "sha256": "f105f63dd5f242494dcd2d96c636a86aca78bd63f5d8c67164dd54a1c97b4251" }
    },
    "target": "python",
    "checker": { "name": "surety", "version": "0.1.0", "python": "3.14.0" },
    "recordFormat": 2,
    "budget": { "generated": 500, "mutants": true, "searchCandidates": 600, "searchInputs": 8, "timeoutSecs": 10 },
    "verdict": {
      "line": "flatten_order  ACCEPTED  examples 2/2; 7 clauses clean over 502 runs; 7 of 13 mutants apply, none slipped",
      "status": "ACCEPTED", "passed": true, "strictPassed": true, "findings": 0
    },
    "examples": { "passed": 2, "total": 2 },
    "properties": { "passed": 7, "total": 7, "runs": 502, "findings": 0,
      "each": [
        { "clause": 0, "canonical": "(== (. out customer_email) (. order customer_email))", "runs": 502, "findings": 0 },
        { "clause": 1, "canonical": "(== (. out order_id) (. order id))", "runs": 502, "findings": 0 }
      ]
    },
    "determinism": { "matched": 502, "total": 502 },
    "totality": { "answered": 502, "raised": 0, "timeouts": 0, "total": 502 },
    "boundary": { "boundaryFailures": 0, "exampleInputs": 2, "generatedInputs": 500, "outputs": 502 },
    "underdetermination": { "total": 13, "applicable": 7, "caught": 7, "slipped": 0, "accepted": 0, "open": 0, "underdeterminedInputs": 0, "causes": 0 },
    "purity": "unverified",
    "imports": [], "accepts": [], "dependencies": [], "uncheckedReferences": [],
    "evidenceDigest": { "sha256": "41880081401a2ec4d75e88f8e33aafcd930d97198b7e82f1fa9fe83187d1bb5e" },
    "record": { "uri": "graph/nodes/150131328a400b324de692005af855135d96915582d15624b2568174b51ce0f5/evidence/python/3f3c77da4d7657b1340785abad4f99b099975c6d37d1c94c2baac258b0f5105d.json" },
    "reproduce": { "command": "surety verify --rerun", "requires": { "surety": "0.1.0", "python": "3.14.0" } }
  }
}
```

(The `each` list is cut to two entries here; the real one carries every clause the record lists.)

## Canonical bytes and the envelope

**Payload bytes.** The Statement is serialized with JSON Canonicalization Scheme (RFC 8785): sorted keys, no whitespace, canonical number and string forms. This is what makes the payload a pure function of the record, and it is what `verify` compares. Any JSON library that sorts keys and strips whitespace produces it for this data (no floats except `timeoutSecs`, which the record already stores as a plain number).

**Envelope.** DSSE v1: `payloadType` `application/vnd.in-toto+json`, `payload` the base64 of the canonical bytes, `signatures[]` each with `keyid` and `sig`. The signed message is the DSSE pre-authentication encoding over the payload type and payload. Standard; every Sigstore and in-toto verifier reads it.

**Bundles.** Several envelopes in one file use the in-toto bundle form: JSON Lines, one envelope per line, suffix `.intoto.jsonl`, media type `application/vnd.in-toto.bundle`. Consumers must ignore lines they do not recognise and must not depend on order; policies therefore look for the statement they need and never count lines.

**File naming.** One envelope per record: `<perf>.<target>.intoto.jsonl` beside the record it projects, or one bundle for the whole graph, `graph.intoto.jsonl`. Both forms carry the same lines; the per-record form diffs cleanly in git, the single bundle is what a consumer downloads. Default: per record.

## Signing: who, and with what

Two different things get signed in this system, and the spec keeps them apart.

| What | Signed by | How, today | How, with this spec |
| --- | --- | --- | --- |
| The obligation delta (the presentation, the examples, the `accept` lines) | A person | Git commit signature on the commit that changes the `.sur` file (gitsign or GPG) | Unchanged. The statement points at the presentation hash; a consumer that wants the human approval checks the commit that introduced that presentation |
| The evidence record | The checker, as operated by whoever ran `pin` | Nothing; the record is plain JSON in git | The DSSE signature on the envelope |

Three signing modes, in order of increasing trust and increasing setup:

1. **Unsigned.** `signatures: []`. Still useful: the payload is canonical and `verify` checks it against the record, so the statement is as trustworthy as the checkout it sits in. This is the default for local work and for the committed form, since a signature is not reproducible and would churn the graph on every emission.
2. **Local key.** An Ed25519 key the operator holds; `keyid` is the key's fingerprint. For a team that pins on one machine and wants consumers elsewhere to check the key. This is the only mode the checker signs itself in v0; the dependency is one small crate.
3. **Keyless (Sigstore).** In CI, the workflow's OIDC identity signs through Fulcio and the signature is logged in Rekor, which also supplies the timestamp the payload lacks. Not implemented by Surety: `surety attest --statement` prints the canonical payload and an external signer (`cosign`, GitHub's attest action) wraps it. Note that `cosign attest-blob` builds its own Statement around a blob digest and would not reproduce ours; the tool to use is one that signs a supplied DSSE payload.

The consumer decides which mode it accepts; the spec does not. A policy that requires mode 3 for merges to main and accepts mode 1 on feature branches is a reasonable first shape.

## Interfaces

**Command.**

```
surety attest [<name | hash>...] [--target python] [--out <dir>] [--bundle <file>]
              [--statement] [--sign <key file>] [--compat test-result]
```

With no names, every admitted record in the graph. `--out` writes one `<perf>.<target>.intoto.jsonl` per record under the given directory (default: beside the record); `--bundle` writes one JSON Lines file instead. `--statement` prints canonical payloads, one per line, unsigned and unwrapped, for an external signer. `--sign` is mode 2. `--compat test-result` adds, for each record, a second line carrying the vetted `test-result` predicate (below). A record the kernel does not admit is not attested and is reported, since attesting a record the kernel would reject is the one way this feature could lie.

**Verify.** `surety verify [--attestations <dir | bundle>]` adds, per record: there is a statement whose subject digest is the pinned performance's and whose `evidenceDigest` equals the SHA-256 of the record's evidence part in canonical form; and the statement's canonical bytes equal the projection recomputed from the record. Signature checking is not `verify`'s job; `cosign verify-blob-attestation` and its peers do that against the key or identity the consumer trusts. `verify` says "projection matches" or names the first field that differs, the same way it reports evidence differences today.

**Library.** Two functions in `surety-graph`, module `attest`: `statement(store, node, target, perf) -> Result<Json>` builds the Statement from the stored record and node files (never from a live check, so the function is pure over the store); `envelope(statement, signer) -> Result<Json>` serializes canonically and wraps. The CLI and the LSP call these; nothing else learns about in-toto.

**Data flow.**

```
pin ─────────▶ record in graph/ (source of truth; kernel-admitted)
                     │
                     ▼
attest ──────▶ Statement (pure function of record) ──▶ canonical bytes ──▶ DSSE envelope
                                                                              │
                      ┌───────────────────────────────────────────────────────┤
                      ▼                                   ▼                   ▼
             committed beside the record        uploaded as a GitHub      Rekor / registry
             (unsigned or local key)            artifact attestation      (keyless, CI)
                      │
                      ▼
verify --attestations: payload == projection(record); evidenceDigest == sha256(evidence)
                      │
                      ▼
consumer policy (Conftest/OPA, gittuf, in-toto layout, branch rule):
  for each changed performance file in the PR:
    exists statement S with S.subject.digest.sha256 == sha256(file)
      and S.predicate.verdict.status == "ACCEPTED" and S.predicate.verdict.strictPassed
      and S.predicate.obligation.digest.sha256 == sha256(graph/nodes/<ob>/canonical.sur at PR head)
      and S.predicate.budget.generated >= 500
      and signature by an accepted key or identity
```

The last check (obligation digest matches the canonical form at the PR head) is what stops a stale verdict from carrying over after the obligation changes; it is the same rule the graph already applies ("a performance whose obligation hash has changed is stale and is never served as verified"), stated so a policy engine can apply it with two hashes and no Surety binary.

## Compatibility projection: `test-result`

For consumers that accept only registered predicates, each record can also be emitted as `https://in-toto.io/attestation/test-result/v0.1`, which is vetted and understood by in-toto layouts and several policy engines. The mapping is lossy and is documented as such:

| test-result field | From the record |
| --- | --- |
| `result` | `PASSED` if `strictPassed`, `WARNED` if `passed` with findings, else `FAILED` |
| `configuration[]` | the obligation descriptor (canonical form, with digest) and the budget as a `content` descriptor under 1 KB |
| `passedTests[]` / `failedTests[]` | one name per clause (`clause/<ordinal>: <canonical>`), per example (`example/<ordinal>`), plus `determinism`, `totality`, `boundary`, `mutants`; each in the passed or failed list according to its findings |
| `url` | omitted, or the repo-relative record path |

`svr` (Simple Verification Result) was considered and set aside for now: it requires `timeCreated`, which breaks the reproducible payload; it can be added as a third projection for consumers that want a one-line "verified by X" when a timestamp source exists (the Rekor entry).

## Policy questions this predicate answers

Listed because the in-toto guidelines ask for them, and because they are the consumer's view of the product.

- Was this performance checked against a signed obligation, and which one (by hash)?
- Is the obligation it was checked against the one in the tree now?
- Did it pass strictly: examples, every property over N generated inputs, determinism, totality, boundary?
- How tight is the obligation: how many mutants applied, how many slipped, how many inputs were underdetermined, which non-decisions were accepted?
- What does the verdict lean on: imports, unchecked references, accepted non-decisions, purity status?
- Can I recompute it, with what checker and interpreter version, and does the record I would recompute it from have this digest?

## Consequences and non-goals, stated plainly

- Emitting attestations changes nothing about what the checker computes. If the record is wrong, the attestation is wrong in the same way, signed.
- A signature by the checker's key says "this record is what that key's holder produced". It is not an endorsement of the obligation, which has its own signer, and it is not a claim about who wrote the performance.
- The committed form is unsigned by default so that `graph/` keeps reproducing byte for byte; signed envelopes are outputs, like build artifacts, and live in the cache or the forge.
- The predicate is Surety's. We are not asking in-toto to vet it now; we may later. The `test-result` projection is the bridge until then.
- Nothing here is an obligation on consumers. Teams that do not use attestations lose nothing.

## Open questions

- [ ] Namespace for the predicate type: `surety-lang.org` (do we hold it?), the GitHub org URL, or an in-toto community URI. Needs the domain decision.
- [ ] Should the subject also carry the obligation as a second subject so that a consumer can look up "all performances attested for this obligation" by digest? It is cheap, but it muddles "what is attested" and the predicate already carries it. Current answer: no.
- [ ] Commit binding: a separate, optional statement whose subject is the `gitCommit` and whose predicate lists the (obligation, performance) pairs verified at that commit, for consumers that gate on commits rather than files. Not in v0.
- [ ] Whether `verify` should accept an attestation directory by default when one exists beside the records, or only on the flag. Flag only until the format settles.
- [ ] Key management for mode 2: where the key lives, how `keyid` is derived, rotation. Borrow Sigstore's conventions rather than invent.
- [ ] When the second target arrives, one statement per (performance, target) is already implied by the record layout; confirm nothing in the predicate assumes Python beyond `checker.python`, which becomes a per-target runtime field.
- [ ] Whether to register the predicate with in-toto once the shape has been stable for a few months, and whether to comment on proposals #601 and #604 pointing at this document. That is a founder's action, not the tool's.

## Acceptance tests (to write with the slice)

- Two presentations of one node emit byte-identical Statement payloads (the record invariant, carried through).
- `attest` then `verify --attestations` passes on the committed corpus; editing one byte of a record's evidence part makes `verify` name `evidenceDigest`.
- A record the kernel rejects is not attested; `attest` reports it and exits non-zero.
- The `test-result` projection of an `ACCEPTED` record has `result: PASSED` and an empty `failedTests`; of a record with one failing example, `FAILED` with that example named.
- An unsigned envelope and a mode-2 envelope of the same record have identical `payload` fields.
- A statement's `subject.digest.sha256` equals `sha256sum` of both the pinned copy and the source performance file; `gitBlob` equals `git hash-object` of the same file.
