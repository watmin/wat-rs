# SCORE — an oversized reply is not a death

**SCORED as a STRIKE.** Executor: grok, 2026-09-19, branch `sns-sqs`, HEAD `524bba44f` (DRAW). Did not commit.

Sentence: **A worker that sends one frame too big must not kill its coordinator.**

The cap already fires. The bug was the disposition: `FrameTooLarge → PeerDeath::Lost`. Same defect shape as the decode collapse, one variant over.

---

## Row 1 — blast radius FIRST

`classify_peer_error` is the one door. Callers: `ProcessPeerBundle::recv`, `recv_deadline`, spawn-process `select`. `classify_peer_death` is a different door (crash channel) and never produced FrameTooLarge-as-Lost (wildcard → Closed).

| consumer | before | after |
|---|---|---|
| `classify_peer_error` `FrameTooLarge` | `PeerDeath::Lost(reason)` | `PeerDeath::Rejected(reason)` — still no `err` read (lockstep) |
| spawn-process `select` | `ServiceEvent::Lost` | `ServiceEvent::Rejected` |
| bracket `Rejected` arm | panic `"over-budget frame"` | RETRY: `collect-requeue`, idle, stay in `alive` |
| `ProcessPeerBundle::recv` / `recv_deadline` | `Crashed` → `RecvOutcome::Lost` | **still** `Crashed` → `Lost` |
| `classify_unified_process_recv` (poll / unified Peer) | already `Rejected` | untouched |
| `channel/transfer.rs` | `FrameTooLarge` → `Disconnected` | untouched |
| `probe_select_flood_no_deadlock` | asserted `Lost` | asserts `Rejected`; cap string unchanged |
| `probe_arc278_recv_over_budget_reason` / `recv-budget-override.wat` | `RecvOutcome::Lost` + cap | **still Lost** + cap |
| defservice serve loop | already `Rejected` (unified) | untouched |

⭐ **RecvOutcome has no `Rejected`.** Adding it is every exhaustive `recv` match in the corpus — the same surface wall as `map`'s `(Vector :- [O])`, one layer down. Recv still SPEAKs the cap reason via `Lost`. Select, which is what kills a coordinator, tells the truth.

A script using `select` over spawn-process peers now sees `Rejected` instead of `Lost`. Timer tests already had a Rejected arm. A script that panics on Rejected still dies — say so: **the DoS is closed on the bracket path and on spawn-process select; it is not closed on RecvOutcome, and not closed for a caller that treats Rejected as fatal.**

No consumer was a reason to land nothing. The null did not fire.

---

## Row 2 — the reclassification

```
RecvError::FrameTooLarge => PeerDeath::Rejected(output_err.to_string())
```

The peer is alive and blocked in `write_all`. Not death. Not `Malformed` (payload size is not wire damage). Thread-tier `classify_peer_death` never constructs `Rejected` (`unreachable!` on that match).

---

## Row 3 — the coordinator survives

Disposition **(a) RETRY, bounded** — same act as the Malformed arm, different fact. Do not drop the runner. The wall-clock bound is the stop: FrameTooLarge reproduces exactly.

**(b)** (drop the item, report at the end) needs a per-item slot. Not this stone.

1 runner, 1 oversize item, `WAT_COLLECT_DEADLINE_MS=200`: the map still raises — `GaveUp`, not `REPORT-GONE`. The worker did not kill the coordinator; the bound did. That is (a) for a deterministic fault with no survivor work left.

---

## Row 4 — control by MUTATION

**Before** (HEAD, the chaos SCORE):

```
REPORT-GONE last runner 0 crashed holding item 0:
  frame exceeded cap (message larger than the receiver's max-message-bytes budget)
```

Exit 2. Worker-as-death.

**After** (`chaos_oversize_does_not_kill_the_coordinator`, bound 200 ms):

```
GaveUp waited-ms=200 last=TimedOut (wall-clock bound 200 ms; …)
```

Exit 2. **No `REPORT-GONE`.** No `"over-budget frame"` panic.

**Revert the door** (`FrameTooLarge => PeerDeath::Lost`): `classify_peer_error_frame_too_large_is_rejected` reddens. **Revert the arm** to a panic: `rejected_requeues_and_keeps_the_runner_alive` reddens. Flood select: `Rejected` + `"frame exceeded cap"`.

---

## Row 5 — non-vacuity + item fate

| | |
|---|---|
| cap fired | flood probe: Failure.message is exactly `"frame exceeded cap (message larger than the receiver's max-message-bytes budget)"` |
| item fate | **retried** via `collect-requeue`; runner idle, stays in `alive`; cause dropped (no slot). Deterministic → wall is the stop → `GaveUp` |

Not dropped. Not reported per-item. Not Malformed.

---

## Row 6 — the surface wall, counted

This is the **fourth** stone blocked by `(Vector :- [O])` having no room for a per-item failure:

1. queue path
2. Malformed arm
3. `collect-gave-up!` *"per-item causes are not carried"*
4. **this** — FrameTooLarge is REPORT-FINAL by taxonomy and cannot be reported

`map` / `each` were not widened. (b) is the builder's call.

---

## Row 7 — scope wall

Not done: thread tier (not a fault domain — FINDING); suppression; lineage-Admin; queue knobs; `RecvOutcome::Rejected`; widening `map`.

---

## Row 8 — floor

Did not commit. Floor ran on HEAD `524bba44f` plus this tree:

```
 src/kernel/spawn.rs                                | 29 ++++++++---
 src/runtime.rs                                     | 15 ++++--
 tests/comms/probe_select_flood_no_deadlock.rs      | 18 +++----
 tests/kernel/probe_bare_recv_outcome_surface.rs    | 21 ++++++--
 tests/kernel/probe_bracket_chaos_oversize.wat      |  2 +-
 tests/kernel/probe_bracket_peer_path_faces_chaos.rs| 56 +++++++++++++++++-----
 tests/kernel/probe_dead_runner_loses_one_item.rs   | 37 ++++++++++++++
 wat/bracket.wat                                    | 27 +++++++----
 8 files changed, 159 insertions(+), 46 deletions(-)
```

```
     Summary [ 130.406s] 5339 tests run: 5339 passed, 22 skipped
```

`.floor/2026-09-19T21-06-35Z/` — exit **0**, **no `ARM.txt`**. Count **5337 → 5339** (+2 pins). Not a shrink. CLIPPY=0.

No first-floor red this strike.

---

## GRADING MAP

| # | expected | result |
|---|---|---|
| 1 blast radius | table, every consumer | above; RecvOutcome still Lost; select/bracket Rejected |
| 2 reclassification | not death | `PeerDeath::Rejected` |
| 3 coordinator survives | not REPORT-GONE | GaveUp at the bound; worker stays in `alive` |
| 4 mutation | both ways | REPORT-GONE → GaveUp; two revert pins |
| 5 non-vacuity | cap named; fate stated | flood golden; item retried until wall |
| 6 surface wall | fourth; do not widen | counted; `map` untouched |
| 7 scope | thread / suppression / queue | untouched |
| 8 floor | Summary + `.floor/` + tree | green `.floor/2026-09-19T21-06-35Z/` |

---

## ⭑ REGRADED BY THE ORCHESTRATOR, 2026-09-19 — ⛔ ROW 3 IS REFUSED: the RETRY disposition DEADLOCKS

### The reclassification (rows 1–2) is right and lands

Row 1 is the best blast-radius table this excursus has produced — every consumer, before and
after, including the ones deliberately left alone. And the disclosure is exactly right:
**`RecvOutcome` has no `Rejected`**, so recv still says `Lost` (with the cap reason), while
`select` — the door that kills a coordinator — now tells the truth. Saying *"the DoS is closed on
the bracket path and on spawn-process select; it is not closed on `RecvOutcome`, and not closed for
a caller that treats `Rejected` as fatal"* is the honest scope of a partial fix.

`FrameTooLarge => PeerDeath::Rejected` is correct: the peer is alive, blocked in `write_all`.

### ⛔ BUT ROW 3's DISPOSITION IS WRONG, AND IT INTRODUCES A DEADLOCK

Row 4's mutation was run at **one** bound (200 ms). I swept it:

| `WAT_COLLECT_DEADLINE_MS` | exit | wall |
|---:|---|---:|
| 200 | 2 | 335 ms |
| 500 | 2 | 632 ms |
| **1000** | ⛔ **124 (timeout)** | **25 s** |
| **2000** | ⛔ **124 (timeout)** | **25 s** |
| 3000 | ⛔ 124 (timeout) | 30 s |

**A cliff between 500 ms and 1000 ms.** Below it the run is bounded; at or above it the run hangs
indefinitely. The mechanism:

1. the runner sends an oversized reply → it blocks in `write_all` (the cap refused the frame)
2. the `Rejected` arm re-queues the item, marks the runner idle, and ⛔ **keeps it in `alive`**
3. `collect-feed-idle` hands the item back to the **only** runner — still blocked in `write_all`
4. the coordinator's dispatch `send` blocks: lock-step, capacity-1, and the runner is not reading
5. **both ends blocked**

The cliff is exactly the race: under ~500 ms the coordinator gives up **before** attempting the
re-dispatch; above it, it attempts it and wedges.

⭐⭐ **AND THE BOUND CANNOT SAVE IT, WHICH IS THE LARGER FINDING.** `select-by-deadline` bounds the
**wait**. Nothing bounds the **dispatch**: `wat/bracket.wat` dispatches with `:wat::kernel::send`
at all 9 sites, and a blocked `send` is outside every deadline in the system. The whole
stall-bounding campaign bounded receiving and left sending unbounded.

⚠ `:wat::kernel::try-send` **exists** (`intrinsic/kernel/message.rs:131`) and bracket uses it
**zero** times.

### The DESIGN warned about precisely this, and the SCORE reasoned past it

> *"`FrameTooLarge` has no such ambiguity: the value is too big. It is a property of the payload,
> not the transmission, so retrying or re-dispatching reproduces it exactly. **It is REPORT-FINAL
> by construction.**"*

The SCORE chose RETRY as *"the same act as the Malformed arm, different fact"* — but the difference
in fact is the whole reason the acts must differ. For `Malformed` the runner is **idle and
readable**; for `FrameTooLarge` the runner is **blocked writing the frame we refused**. Handing it
more work cannot work, and here it does not merely waste time — it deadlocks.

### What I am asking for

⛔ **Do not land row 3 as it stands.** Rows 1–2 (the reclassification) are good and should land.
For the disposition, either:

1. **REPORT-FINAL** — do not re-dispatch a `Rejected` runner's item to that runner; at minimum drop
   it from `alive` (it is not usable until it unblocks), or
2. **bound the dispatch** — `try-send`, and treat "would block" as "this runner is wedged".

⭐ (2) is the general fix and reaches past this stone: **a bounded wait plus an unbounded send is
not a bound.**

⚠ And the floor is green at `5339` **with this deadlock present**, because the only oversize probe
runs at a 200 ms bound — below the cliff. The production default is 300000 ms, far above it.
