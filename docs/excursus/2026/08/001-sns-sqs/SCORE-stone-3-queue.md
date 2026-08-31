# SCORE — excursus 001 stone 3: SQS in userland

**STRUCK.** Executor: grok, 2026-08-31. Built in `wat-scripts/queue/`. Not promoted.

```
Summary [ 302.215s] 5122 tests run: 5122 passed (3 slow), 17 skipped
FLOOR=0
```

Log: `.floor/2026-08-31T01-08-44Z/`. 5122 = 5121 + `queue_lifecycle_mem_and_sqlite_agree`.

The agreed summary, produced identically by `mem-store` and `sqlite-store(:memory:)`:

```
bound=x;r1=a,b;r2=c;r3=;redel=b
```

`./target/release/wat wat-scripts/queue/sqs.wat` prints that. SNS still prints `"3 3"`.

## The scorecard

| # | what | expected | **measured** |
|---|---|---|---|
| 1 | it is a DEMO, not stdlib | `wat-scripts/queue/`; `git diff -- wat/ src/ crates/` empty | ✅ porcelain: `M wat-scripts/queue/README.md`, `?? wat-scripts/queue/sqs.wat`, `?? tests/services/probe_ex001_queue.rs`, this SCORE. `git diff --stat -- wat/ src/ crates/` empty |
| 2 | send → receive | 3 sent, `receive` 2 returns 2 | ✅ `r1=a,b` |
| 3 | received messages go invisible | a second immediate `receive` returns only the third | ✅ `r2=c` |
| 4 | ack removes | the acked message never returns | ✅ `a` acked; `redel=b` (no `a`) |
| 5 | ★ **redelivery** | the RECEIVED-but-UNACKED message **reappears** once its window passes | ✅ `r3=` (still inside the window) then `redel=b` after `now` steps to T+timeout. No sleep |
| 6 | visibility is ONE put | no lock, no timer, no base-table read (STOP-2) | ✅ `receive` scan-indexes, then one `Store/put` of `StoredRow`s built from `IndexRow` (`pk sk ipk isk data`). STOP-2 did not fire |
| 7 | both backends agree | identical rendered summary | ✅ pinned `AGREED_SUMMARY`; mismatch would be `DIFFERENTIAL-MISMATCH mem=… sqlite=…` |
| 8 | ★ the `isk` boundary is demonstrated | a message visible at exactly `now` is returned | ✅ `bound=x` — send at T0, receive at T0, `isk-hi = write(at-nanos T0)` inclusive |
| 9 | floor | `FLOOR=0`, 5121 + the new fixture's arms | ✅ `FLOOR=0`. **5122** = 5121 + 1 |
| 10 | zero substrate change | no `wat/`, no `src/`, no `crates/` | ✅ empty |
| 11 | prior stones undisturbed | all `probe_ex001_*`, inst, write-opts arms PASS | ✅ delete / reput / write-opts arity / same-ns / sortkey-boundary PASS in this floor. SNS `"3 3"` |

## STOP-1 did not fire

This belongs in `wat-scripts/queue/` until the builder promotes it, the same ruling as wat-topic. I did not conclude it should move to `wat/queue.wat` this stone. The grep precedent's bar is ready when they do: the summary `bound=x;r1=a,b;r2=c;r3=;redel=b` is the count to re-run.

No bijection-anchor. The one ephemeral peer is a scalar `Store` (journal's shape), so `:peers [:wat::query::Store]` is a true bijection.

## Row 5 — the clock is an argument

Trap-door 1: `:wat::time::now` is real; `:wat::kernel::after` is a timer (STOP-2). Neither steps a window. A sleep is a guess.

**send/receive take `now-ns` (epoch nanos).** The fixture drives T0, T0+1, T0+2, then T0+2+timeout. Redelivery is `receive` at a later `now`, not a wait. Callers pass `(:wat::time::epoch-nanos (:wat::time::now))`.

First sketch put `Instant` / `Duration` on the request records. It type-checked. `send` then returned a non-Ok response (`"send not Ok"`). Switched to journal's wire-proven `i64` time-ns; send went Ok. isk is still `(:wat::edn::write (:wat::time::at-nanos ns))` — Instant, constant-width, inside the service.

## Row 3 vs row 5 — no tension, because the window is a value

Visibility is 100 ns. Messages a,b,c are staggered 1 ns on isk so `limit 2` is ordered (equal-isk order is unspecified across backends). Receive at Tr = T0+2 ns; redelivery at Tw = Tr+100 ns. Immediate second receive is still at Tr, so the first two stay invisible. No sleep, no flake.

Ack `a` (first of r1) and `c` (the third) so redelivery is exactly `b` — the unacked one of the first two — and so a b/c isk-tie cannot make mem and sqlite print different summaries.

## Visibility is one put

`IndexRow` has everything. `receive` does not `scan` the base table. Re-put is replace-by-(pk,sk) with a new `by-visible-at` projection (`isk = now + timeout`). Stone 2c made that a replace on both backends; stone 2 made `ack`'s `delete` exist.

sk is `(:wat::edn::write (:wat::uuid::v4))` — SORTKEY's producer-mints-v4 precedent. Stable; ack names it forever.

## Blast radius

- `wat-scripts/queue/sqs.wat` — the feature + the both-backends gate
- `wat-scripts/queue/README.md` — how to run it
- `tests/services/probe_ex001_queue.rs` — thin harness, `startup_from_file("wat-scripts/queue/sqs.wat")`

Zero `wat/`. Zero `src/`. Zero `crates/`.

---

# ORCHESTRATOR GRADING — re-run, not read

```
Summary [ 297.801s] 5122 tests run: 5122 passed (2 slow), 17 skipped     FLOOR=0
PASS (3570/5122) wat::services probe_ex001_queue::queue_lifecycle_mem_and_sqlite_agree
```

Standalone, both halves:

```
wat-scripts/queue/sqs.wat          → "bound=x;r1=a,b;r2=c;r3=;redel=b"
wat-scripts/topic/sns-fanout.wat   → "3 3"
```

`git status --porcelain -- wat/ src/ crates/` → **0**. STOP-1 held: it is in
`wat-scripts/queue/`, not the stdlib. **STRUCK.**

## The two rows that could have been faked

**Redelivery.** `mora` forbids a sleep and `:wat::time::now` cannot be stepped. The resolution
is to make **the clock an argument** — `now-ns` on `SendRequest` and `ReceiveRequest` — so the
fixture picks `Tr` (inside the window) and `Tw` (past it). No sleep, no flake. It is also the
better design on its own terms: time is I/O, and a queue whose clock is injectable is a queue
you can test.

★ `r3=` is the row the BRIEF did not ask for and the one that makes the sequence honest. An
empty third receive **inside** the window proves invisibility actually persists. Without it,
`redel=b` alone would pass on a queue that never made anything invisible at all.

**The boundary.** `bound=x` — sent at `T0`, received at `T0`. The inclusive-`hi` case a
too-small sentinel drops in silence: the same class as stone SORTKEY's boundary and the `#inst`
width bug before it. Third time that shape has been the thing worth pinning.

## ★ Two sources of nondeterminism designed OUT, unprompted

The fixture's own comments:

> *"send 3, staggered by 1ns so isk order is a,b,c (limit-2 among equal isk is unspecified)."*
>
> *"c is acked so redelivery is exactly the unacked one — and so equal-isk order between b and c
> cannot make the backends disagree on the summary."*

Both are questions the design could have left open — *which two of three does `limit 2` return
when their sort keys tie?* — removed rather than hoped past. That is the same instinct as this
excursus's own trap-door sections, applied without being asked for.

## The harness drives the shipped program

`startup_from_file("wat-scripts/queue/sqs.wat")` — no co-located copy. **The thing under test is
the thing that ships**, so the demo cannot drift from the tested artifact. And the assertion is
`assert_eq!` on the whole summary with `no_loose_string_assert` named in a comment — the third
consecutive stone where that warning, once carried into a brief, simply did not fire.

## Excursus 001 is complete

```
1        SNS in userland                    wat-scripts/topic/  →  "3 3"
2        Store gains delete
2b       the delete differential + GSI
2c       mem's put becomes a replace        → exposed journal's data loss
INST     #inst at constant width            one token
WRITE-OPTS / WO-OPT   options as a passed value, optional
CENSUS   13/15 agree — and why that is not reassuring
SORTKEY  an event carries its identity      → first fully green floor
3        SQS in userland                    wat-scripts/queue/  →  the full lifecycle
```

**Floor 5122, FLOOR=0.** Both halves run. Nothing is owed inside the excursus.

## What is NOT done, and is the builder's

**Promotion.** `wat-topic` and `wat-queue` stay in `wat-scripts/` until they demonstrate
excellence. When that ruling comes, the grep precedent sets the standard for the move: *"mostly
a MOVE of proven code, and the counts are the proof it moved intact."* **The counts here are
`"3 3"` and `"bound=x;r1=a,b;r2=c;r3=;redel=b"`** — re-run them after the move, do not take
them from a report.
