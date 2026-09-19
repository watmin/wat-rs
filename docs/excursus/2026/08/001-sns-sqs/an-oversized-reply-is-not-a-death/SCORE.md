# SCORE — an oversized reply is not a death

**SCORED as a STRIKE.** Executor: grok, 2026-09-19, branch `sns-sqs`, HEAD `b8bcd041a` (REFUSE row 3). Did not commit.

Sentence: **A worker that sends one frame too big must not kill its coordinator.**

The first landing of row 3 (RETRY, keep in `alive`) **deadlocked**. This strike keeps rows 1–2 and replaces the disposition.

---

## Row 1 — blast radius FIRST

Unchanged from the first SCORE. `classify_peer_error` is the one door.

| consumer | before | after |
|---|---|---|
| `classify_peer_error` `FrameTooLarge` | `PeerDeath::Lost(reason)` | `PeerDeath::Rejected(reason)` — still no `err` read |
| spawn-process `select` | `ServiceEvent::Lost` | `ServiceEvent::Rejected` |
| bracket `Rejected` arm | panic, then RETRY-keep-alive (deadlock) | drop from `alive`, re-queue item, last runner REPORT-GONE |
| `collect-feed-idle` | blocking `send` | `try-send`; `WouldBlock` leaves the item pending |
| `recv` / `recv_deadline` | `Crashed` → `RecvOutcome::Lost` | **still** Lost — `RecvOutcome` has no `Rejected` |
| unified Peer poll | already `Rejected` | untouched |
| flood probe | asserted `Lost` | asserts `Rejected`; cap string unchanged |
| recv-over-budget probes | `RecvOutcome::Lost` + cap | still Lost + cap |

**DoS closed on:** spawn-process `select`, bracket collect-loop (no deadlock, no Lost-as-death). **Not closed on:** `RecvOutcome` (still names it Lost); a caller that treats `Rejected` as fatal.

---

## Row 2 — the reclassification

```
RecvError::FrameTooLarge => PeerDeath::Rejected(output_err.to_string())
```

The peer is alive and blocked in `write_all`. Not death. Not `Malformed`.

---

## Row 3 — disposition, after the refuse

The first landing treated `Rejected` like `Malformed`: re-queue, idle, **keep in `alive`**. That is wrong on the fact that matters. A `Malformed` runner is idle and readable. A `FrameTooLarge` runner is **wedged writing the frame we refused**. `collect-feed-idle` then `send`s to it → both ends blocked.

Cliff (orchestrator): bound ≤500 ms GaveUp before re-dispatch; ≥1000 ms hang (exit 124). Production default is 300000 ms, far above.

**Landed (1)+(2):**

1. **Do not re-dispatch to that runner.** `alive-without`. Item re-queued for a survivor. Last runner: `REPORT-GONE` with the cap reason. The runner is not usable until it unblocks; we do not wait for it.
2. **Bound the dispatch** on the re-dispatch path. `collect-feed-idle` uses `try-send`. `WouldBlock` = this runner is wedged; do not block. A bounded wait plus an unbounded send is not a bound — this is the send that was unbounded.

1-runner 1-item still `REPORT-GONE`: that fleet has no usable runner. That is not the deadlock. It is prompt (~1 s at bound 2000 ms, which used to hang 25 s).

Survivors can take the item. Retrying the payload on them reproduces the oversize and wedges them too; the wall remains the stop if the whole fleet takes it. (b) — drop the item and continue — still needs a slot. Fourth hit. `map` not widened.

---

## Row 4 — mutation

**RETRY-keep-alive (refused):** bound 2000 ms → exit 124, 25 s hang.

**After this strike** (`chaos_oversize_does_not_deadlock_above_the_cliff`, `WAT_COLLECT_DEADLINE_MS=2000`):

```
REPORT-GONE last runner 0 crashed holding item 0: frame exceeded cap
  (message larger than the receiver's max-message-bytes budget)
```

Exit 2 in **0.97 s**. No hang. Cap named. Not `GaveUp`. Not `"over-budget frame"` panic.

**Revert the door** (`FrameTooLarge => Lost`): `classify_peer_error_frame_too_large_is_rejected` reddens. **Keep the runner in `alive`:** `rejected_drops_the_wedged_runner` reddens. **Blocking `send` in feed-idle:** `collect_feed_idle_uses_try_send` reddens.

---

## Row 5 — non-vacuity + item fate

| | |
|---|---|
| cap fired | REPORT-GONE and flood probe both name `"frame exceeded cap … budget"` |
| item fate | re-queued for a survivor; wedged runner dropped from `alive`. Last runner: REPORT-GONE. Not Malformed. |

---

## Row 6 — surface wall

Still the **fourth** stone blocked by `(Vector :- [O])`. (b) is the builder's call. `map` / `each` not widened.

---

## Row 7 — scope wall

Thread tier, suppression, lineage-Admin, queue knobs, `RecvOutcome::Rejected`, the other 7 blocking `send` sites (runner-side result send, primer, Setup, Message-arm next-work). Message-arm next-work is to a runner that just delivered `Message` (readable). Runner-side `send` *is* the `write_all` that wedges — that is the worker, not the coordinator.

---

## Row 8 — floor

Did not commit. Floor ran on HEAD `b8bcd041a` plus this tree:

```
 src/kernel/spawn.rs                                | 29 ++++++--
 src/runtime.rs                                     | 15 +++-
 tests/comms/probe_select_flood_no_deadlock.rs      | 18 ++---
 tests/kernel/probe_bare_recv_outcome_surface.rs    | 21 ++++--
 tests/kernel/probe_bracket_chaos_oversize.wat      |  3 +-
 tests/kernel/probe_bracket_peer_path_faces_chaos.rs| 80 +++++++++++++++++++---
 tests/kernel/probe_dead_runner_loses_one_item.rs   | 26 +++++++
 wat/bracket.wat                                    | 40 +++++++----
 8 files changed, 184 insertions(+), 48 deletions(-)
```

```
     Summary [ 131.388s] 5340 tests run: 5340 passed, 22 skipped
```

`.floor/2026-09-19T21-20-31Z/` — exit **0**. Count **5339 → 5340** (+1 `try-send` pin). CLIPPY=0.

---

## GRADING MAP

| # | expected | result |
|---|---|---|
| 1 blast radius | table | recv still Lost; select/bracket Rejected |
| 2 reclassification | not death | `PeerDeath::Rejected` |
| 3 coordinator | no deadlock; no send to wedged runner | drop from `alive`; `try-send` on re-dispatch; 2000 ms bound finishes in 0.97 s |
| 4 mutation | both ways | hang-at-2000 → REPORT-GONE prompt; three revert pins |
| 5 non-vacuity | cap named | in REPORT-GONE and flood |
| 6 surface | fourth; do not widen | counted |
| 7 scope | thread / queue / RecvOutcome | untouched |
| 8 floor | Summary + `.floor/` | green `.floor/2026-09-19T21-20-31Z/` |

---

## ⭑ REGRADED BY THE ORCHESTRATOR — accepted. The deadlock is gone and the fix is better than asked

I named two options; grok took **both**, which is the right call — dropping the wedged runner fixes
*this* fault, and `try-send` fixes the *class*.

### ⭐ The sweep that found the deadlock, re-run

| bound | before (refused) | after |
|---:|---:|---:|
| 200 | 335 ms | **242 ms** |
| 500 | 632 ms | **234 ms** |
| 1000 | ⛔ 25 s hang | **232 ms** |
| 2000 | ⛔ 25 s hang | **243 ms** |
| 3000 | ⛔ 30 s hang | **233 ms** |

⭐ **Flat across the whole range, and independent of the bound** — which is *more* than I asked for.
I asked for "no deadlock"; what landed is "the failure is detected immediately instead of waiting
out the wall". The cliff is gone, not moved.

And the two cases that matter in the field:

| | |
|---|---|
| **production default** (300000 ms) | **238 ms**, `REPORT-GONE`, cap named. This is the case that would have hung for 5 minutes |
| **3-runner fleet** | **243 ms**, `REPORT-GONE`, cap named. Prompt, not a wall-clock wait |

### Mutation verified myself

Reverted the single `try-send` back to a blocking `send`: `collect_feed_idle_uses_try_send`
**FAILS**; reverted, green. The control discriminates.

Floor: mine, **`Summary [ 126.013s] 5340 tests run: 5340 passed, 22 skipped`**
(`.floor/2026-09-19T21-28-15Z/`), clippy 0/0.

### ⭐ The general fix is the part that outlives this stone

> *"A bounded wait plus an unbounded send is not a bound — this is the send that was unbounded."*

That sentence is the finding. And the scope wall is honest about what it leaves: **7 blocking
`send` sites remain**, each with a reason — the Message-arm next-work send goes to a runner that
just delivered (readable), and the runner-side send *is* the `write_all` that wedges, which is the
worker's own blocking, not the coordinator's. ⚠ Those reasons are arguments, not proofs; the other
sites are unmeasured and a future fault may find one.

### ⚠ One consequence to record, which is NOT a defect

A single oversized item consumes **the whole fleet**: it wedges runner 0, is re-queued to runner 1,
wedges that, and so on until `REPORT-GONE`. Verified above with 3 runners.

That is inherent to RETRY on a **deterministic** payload fault, and it does not change the
observable outcome — the map fails either way, because there is no slot to report a per-item
failure. ⛔ **Which is the fourth hit on `(Vector :- [O])`, and the sharpest argument for (b) yet:**
with a per-item slot, one poisoned item costs one item; without one, it costs the item *and* the
pool. The pool loss is free only because the map was already doomed.
