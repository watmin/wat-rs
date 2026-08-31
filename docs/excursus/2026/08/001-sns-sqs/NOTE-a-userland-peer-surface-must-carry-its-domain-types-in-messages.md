# ⛔ NOTE (excursus 001) — a USERLAND peer surface must declare its domain types INSIDE `:messages`; the stdlib exemption does not transfer

**Found 2026-08-31, by stone 4 hitting it and STOP-5 reporting it rather than working around it.**

## The failure

`wat-scripts/queue/sqs.wat` declares `:queue::Envelope` at line 36 — **beside**
`:queue::Queue`, whose `defsurface` begins at line 40. `Queue::ReceiveResponse::Ok` carries a
`(Vector :- [:queue::Envelope])`.

A worker service declaring `:peers [:queue::Queue]` and forked to a **process** cannot resolve
`:queue::Envelope/id`. Measured: the parent freeze type-checks; the child dies with
`unknown callee: :queue::Envelope/id`. **Process workers cannot consume the queue at all.**

## Why — and the examples were all correct

`wat/service.wat:792`: *"MANIFEST: for each `:peers` surface S, `(S::surface-forms)` is
concatenated into the child."* A type not inside `:messages` is not in `surface-forms`, so it
does not cross the fork.

`wat/query.wat:497-500` states the shape the queue copied, and — read carefully — states its
**precondition**:

> *"The SHARED domain vocabulary they are built from (StoredRow/Row/IndexRow/IndexKey/Page/
> IndexPage/TableSchema/IndexSchema) + the error records … stay top-level: **they cross via
> stdlib**, are not per-op messages."*

**`Store` is stdlib.** Its domain types are baked into every child by the stdlib manifest, so
they reach the fork without `surface-forms`. **`wat-queue` is userland.** `Envelope` crosses via
*neither* door — not baked, not in `:messages`.

★ **The queue copied a pattern whose justification is a precondition it does not satisfy.**

## The census — why nothing caught it sooner

| surface | domain types its messages carry | crosses via | forked consumer works |
|---|---|---|---|
| `:demo::Sub` (SNS, userland) | **only builtins** — `String`, `i64`, `Vector` | n/a | ✅ |
| `:wat::query::Store` | top-level `StoredRow`, `Row`, `IndexRow`, … | **stdlib** | ✅ |
| `:wat::telemetry::Journal` | top-level `Metric`, `Log` | **stdlib** | ✅ |
| `:probe::Echo` (the s2s probes) | **only builtins** — `String` | n/a | ✅ |
| **`:queue::Queue` (userland)** | **`Envelope`** — top-level | **nothing** | ❌ |

Every existing example is correct, and **none of them exercises the failing combination.** The
userland ones carry only builtins; the ones with real domain vocabulary are stdlib. `wat-queue`
is the first userland surface whose messages carry a userland type, and it broke immediately.

## The rule

> **For a userland peer surface, every type its messages are built from must be declared inside
> `:messages`. A userland record crosses via `surface-forms` or not at all.**

The stdlib exemption is real but non-transferable, and `wat/query.wat:500` should say so — it
currently reads as general guidance ("stay top-level") with the reason attached almost in
passing. A reader copying that shape into userland inherits the words and not the condition.

## What is owed

1. **`Envelope` moves inside `:queue::Queue`'s `:messages`.** Small, and it is the fix.
   ⚠ **Not verified as sufficient.** A grading experiment moved it on a scratch copy and the
   circuit went from silent-drain to `peer crashed` — but that run left stone 4's foreign-read
   workaround in place, so it changed one variable with its compensator still installed. **The
   clean experiment is: move `Envelope` AND remove the workaround, then run.** Untested.
2. **`wat/query.wat:500`'s note gains the condition explicitly** — *"…stay top-level: they cross
   via stdlib. **A userland surface has no such carrier; its domain types belong in
   `:messages`.**"*
3. **A gate**, so this class cannot return: a forked-process consumer of a *userland* peer
   surface whose messages carry a *userland* type. No such test exists today, which is exactly
   why this reached a working-looking circuit before surfacing.

## A second finding, from the same run

The scratch experiment produced:

```
"peer crashed (abnormal far-side crash — no reason; the crash reason is administrative
 and travels only to the owner's crash channel)"
```

**A forked child died and its reason did not reach the caller.** That is the law
`tests/services/probe_arc278_dead_child_speaks.rs` exists to enforce — *"wat NEVER HIDES A
FAILURE … the caller's error must CARRY the reason"* — and this path does not. Whether it is
the same defect that test closed or a sibling on a different door is **unexamined**; it is
recorded here because it was seen, not because it was diagnosed.

## Kin

- `wat/service.wat:792`, `:2523` — the `surface-forms` manifest that carries `:messages` across
  a fork, and nothing else.
- `wat-scripts/topic/sns-fanout.wat` — the userland surface that dodged this by carrying only
  builtins.
- `SCORE-stone-4-fanout-circuit.md` — the strike that found it and correctly stopped.
