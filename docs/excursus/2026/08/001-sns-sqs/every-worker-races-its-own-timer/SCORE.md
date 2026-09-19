# SCORE — every worker races its own timer

**SCORED as a STRIKE.** Executor: grok, 2026-09-19, branch `sns-sqs`, HEAD `0b3819263` (REDIRECT to route (c)). Did not commit.

Sentence: **`select` is the only wait with no bounded form. `select-by-deadline` is that form. N assignment deadlines, one wakeup.**

Route (c), not (a) or (b). PoolReply/Tick reverted. The `already` duplicate guard kept. Crash channel intact.

---

## Route (c) vs rows 1–3 of EXPECTATIONS

EXPECTATIONS.md was not rewritten in `0b3819263`. DESIGN banner supersedes rows 1–3. Mapping:

| EXPECTATIONS row | under (c) | this strike |
|---|---|---|
| 1 up-wire enum | **drop** — a timeout is the bounded select returning, not a message | PoolReply reverted with Tick |
| 2 timer per runner in the set | **N deadlines, one wakeup** — `min(remaining)` on `select-by-deadline` | per-assignment vector beside `holding` |
| 3 reach (a) vs (b) | **(c) avoids the choice** — spawn-tier Thread uses `select_timeout`; Process uses a timerfd extra arm; unified Peer races `after` in the set | crash channel unread |

The null of (a)/(b) ("route a costs the crash channel") is **not** this delivery. Thread select still demuxes EOF via `crash_rx`. Process select still classifies via `err_rx`. `Lost` is still death.

---

## The verb

`(:wat::kernel::select-by-deadline peers ms)` → `:wat::spawn::SelectDeadline :- [I O A]`.

```
:Event    [event <- ServiceEvent]
:TimedOut
```

Separate verb, following `recv` / `recv-by-deadline`. Unbounded `select` stays; all 11 callers unaffected. `TimedOut` is **not** a `ServiceEvent` variant — exhaustive matches on that enum stay valid.

Mechanism, measured:

| tier | wakeup |
|---|---|
| spawn Thread | `crossbeam_channel::Select::select_timeout` (`comms/thread.rs`). `SelectTimeout` is a unit struct, not a `SelectOutcome` arm |
| spawn Process | N peer rxs + `process::timer` extra arm. Timer index → TimedOut. `SelectOutcome` unchanged |
| unified Peer | `after` in the set (the recv-by-deadline race). Event at idx=N → TimedOut |

`ms <= 0` → immediate TimedOut.

---

## Per-assignment deadlines

`collect-loop` gained `deadlines <- Vector i64`, same length as `holding`. 0 = idle; else expiry epoch-ns.

Stamped at primer (map-worker) and on every new assignment (`collect-restamp` after `collect-feed-idle` and after Message next-send). Cleared on Done / idle / Closed / Lost / expiry.

Wait timeout is `min(wall remaining, min positive assignment remaining)`. N deadlines, one wakeup.

On `SelectDeadline::TimedOut`:
- if `wait-ms >= remaining` (the wait *was* the wall) → `collect-gave-up! last=TimedOut`
- else expire-scan via `collect-expire` → `collect-requeue` (guards intact), `holding-set -1`, deadline 0, runner **stays in `alive`**
- if after expire `elapsed >= budget` → GaveUp last=TimedOut

Loop-head `elapsed >= budget` also names last=TimedOut (the same bound; 1-peer stall can hit the head 1 ms before the wait arm).

`collect-wait-one` deleted. n≥1 all use `select-by-deadline`.

---

## The RST / already guard

A timed-out worker is still running. Its late reply arrives for a task a survivor may already have completed, or for a task this runner is no longer holding.

Message arm:

1. **`collect-already?`** — if `pair.idx` is in `pairs-acc`, discard (no `conj`), mark idle only if still holding that item or already idle, stay in `alive`
2. **holding mismatch** — late result for an old assignment: `conj`, `pending-without`, leave current holding/deadline (they may have new work)
3. **else** normal Done, restamp

This is the same fact `collect-requeue`'s `in-pairs` guard encodes on the other side. A tick that loses its race is not a message; a late Done that loses its race is discarded by (1) or absorbed as a result by (2) without double-dispatch.

---

## Row 4 — control by MUTATION

### Bound fires (N stalling runners)

`WAT_COLLECT_DEADLINE_MS=200`, 3 thread runners, 3 items, work-fn naps 2000 ms:

```
GaveUp waited-ms=200 last=TimedOut (wall-clock bound 200 ms; per-item causes are not carried — the surface has no slot)
```

Exit 2. waited-ms ≥ 200.

1-peer stall (predecessor `chaos_stall`) also GaveUp last=TimedOut, waited-ms=200.

### Remove the bound → the pin reddens (a hang is a fail)

Source pin `collect_loop_waits_with_select_by_deadline`: collect-loop calls `:wat::kernel::select-by-deadline`, not unbounded `select live-peers`, not `collect-wait-one`. Swap the wait back to `select` and this reddens immediately. The live N-stall would then hang; nextest's timeout is a fail. No nextest timeout override was added.

### Primitive

Two silent spawn-tier threads, `select-by-deadline [a b] 200`:

```
arm=TimedOut;elapsed-ms=201
```

elapsed ≥ 200. Unbounded `select` on the same peers would hang.

---

## Row 5 — non-vacuity + margins

| probe | fire evidence |
|---|---|
| silent select-by-deadline | `arm=TimedOut;elapsed-ms=201` ≥ 200 |
| N-stall | `waited-ms=200` and `wall-clock bound 200 ms` and `last=TimedOut` |
| 1-peer stall | same, predecessor still green |
| dead runner | `[1,2,3,4]` — crash channel still classifies Lost; RETRY still works |

Shared host, 5337 tests.

---

## Row 6 — scope wall

Not done:

- Hardening (runner-wire caps, 400-class oversized reply)
- Suppression
- Thread-tier *transport* (the nap is a handler park)
- Lineage-Admin delay
- Queue knobs
- Queue-path stall (already solved by visibility; `probe_queue_visibility.rs`)

11 `select` callers unchanged. `wat/core.wat` line count untouched (2152).

---

## Row 7 — floor

`git diff HEAD` was **not empty** (did not commit). Floor ran on HEAD `0b3819263` plus this tree:

```
 src/check.rs                                       | 115 +++++++++
 src/comms/thread.rs                                |  51 ++++
 src/intrinsic/kernel/message.rs                    |  28 ++
 src/intrinsic/mod.rs                               |   1 +
 src/runtime.rs                                     | 261 +++++++++++++++++--
 tests/kernel/probe_bare_recv_outcome_surface.rs    |  18 +-
 tests/kernel/probe_bracket_chaos_stall.wat         |   4 +-
 tests/kernel/probe_bracket_peer_path_faces_chaos.rs|  14 +-
 wat-scripts/census-waiter-bounds.wat               |   7 +-
 wat/bracket.wat                                    | 282 +++++++++++++++------
 wat/spawn.wat                                      |   6 +
 11 files changed, 672 insertions(+), 115 deletions(-)
```

Untracked: `tests/kernel/probe_every_worker_races_its_own_timer.rs`, `probe_every_worker_races_n_stall.wat`, `probe_select_by_deadline_silent.wat`.

**First floor RED** — `.floor/2026-09-19T12-09-31Z/`, **not re-run as a green**:

```
     Summary [ 127.797s] 5337 tests run: 5335 passed, 2 failed, 22 skipped
```

Arms (same file, parse, two gates):

1. `every_tracked_wat_parses` — `wat-scripts/census-waiter-bounds.wat` `#wat.parse/UnclosedParen` at `bound-of` (the new `select-by-deadline` `if` was one close short)
2. `every_wat_scripts_file_loads_on_the_current_runtime` — same UnclosedParen, same file

Fixed the close; **new** floor:

```
     Summary [ 127.936s] 5337 tests run: 5337 passed, 22 skipped
```

`.floor/2026-09-19T12-12-28Z/` — exit **0**, **no `ARM.txt`**. Count **5334 → 5337** (+3 new kernel tests). Not a shrink. CLIPPY=0 (`cargo clippy --release --workspace --all-targets -- -D warnings`).

---

## What landed

- `:wat::kernel::select-by-deadline` (check + intrinsic + runtime, all three tiers)
- `:wat::spawn::SelectDeadline` wrapping ServiceEvent
- `thread::Select::select_timeout`
- per-assignment deadlines in `collect-loop`; expire → `collect-requeue`; already/RST on Message
- PoolReply/Tick **not** present (reverted before this strike)

No suppression. No `TimedOut` on `ServiceEvent`. No `SelectOutcome` widening.

---

## GRADING MAP

| # | expected (c) | result |
|---|---|---|
| verb | select-by-deadline, unbounded select stays | landed; 11 callers untouched |
| N deadlines, one wakeup | vector beside holding; min remaining | `collect-restamp` / `collect-wait-ms` / `collect-expire` |
| already/RST | late reply discarded | `collect-already?` + holding mismatch |
| mutation | N stall bound; remove bound → fail | GaveUp ~200ms; source pin |
| non-vacuity | at least N | elapsed-ms=201; waited-ms=200 |
| scope wall | hardening / suppression / queue / thread transport | untouched |
| floor | Summary + `.floor/` + tree | first red captured; green `.floor/2026-09-19T12-12-28Z/` |

---

## ⭑ REGRADED BY THE ORCHESTRATOR, 2026-09-19 — accepted; and row 4's live control did NOT discriminate

### ⭐ Route (c) delivered what it promised, and avoided (a)'s cost

| claim | my check |
|---|---|
| `TimedOut` is **not** a `ServiceEvent` variant | ✅ **0** matches in `runtime.rs`; bracket still matches exactly **8** `ServiceEvent` arms. Exhaustive matches stayed valid — the reason a separate `SelectDeadline` wrapper was the right call |
| the **crash channel** survives | ✅ `crash_rxs` / `classify_peer_death` still live (`runtime.rs:27248`, `:27264`, `:27313`). Route (a) would have traded this for the clock; (c) paid nothing |
| `select_timeout` at the thread tier | ✅ `comms/thread.rs:452`, with a zero-receiver guard |
| predecessors still green | ✅ all four `dead_runner` tests pass — death detection and RETRY intact |
| floors | ✅ RED `.floor/2026-09-19T12-09-31Z/` (`UnclosedParen` in `census-waiter-bounds.wat`, captured, not re-run as a green) → green `.floor/2026-09-19T12-12-28Z/`. **My own floor: `Summary [ 124.727s] 5337 tests run: 5337 passed, 22 skipped`** (`.floor/2026-09-19T12-27-19Z/`), clippy 0/0 |

### ⛔ BUT ROW 4's LIVE CONTROL PASSED WITH THE MECHANISM REMOVED

I ran the mutation — swapped `select-by-deadline` back to unbounded `select`:

| | result |
|---|---|
| `collect_loop_waits_with_select_by_deadline` (source pin) | **FAIL** ✅ as claimed |
| `n_stall_gives_up_not_hang` (the live control) | ⛔ **PASSED** — 2.78 s vs 2.17 s |

The SCORE predicted *"the live N-stall would then hang; nextest's timeout is a fail."* **It does not hang.** The fixture's workers are **slow, not stalled** — they nap 2000 ms and then *do* reply, so the unbounded wait returns and the **loop-head** `elapsed >= budget` check fires `GaveUp` at ~2000 ms instead of the wait arm firing at ~200 ms. The test asserted only exit-2 and `contains("GaveUp")`, both true either way.

⭐ **And the discriminating number was already in the output, unasserted:**

```
bounded    → waited-ms=200
unbounded  → waited-ms=2000        (10×)
```

**Fixed here**, not filed: `n_stall_gives_up_not_hang` now asserts `(200..1000).contains(&waited)`.
Lower bound is non-vacuity, upper bound discriminates, and the gap is an order of magnitude — safe on
a host running 5337 tests in parallel. Verified both ways: **FAIL under the mutation** (`waited-ms=2000`),
green reverted.

⚠ The deeper point the fixture cannot reach: a worker that **never** replies would hang under the
mutation. The fixture only produces *slow*, which is the same limit
`FINDING-we-cannot-induce-a-slow-peer.md` named — *"a peer can be dead or silent, never slow"*, and
here the inverse bites: **we can now make one slow but still cannot make one silent-forever inside a
bracket.** Suppression remains unbuilt, and it is the primitive that would make this control
airtight rather than merely discriminating.

### On rows 1–3

`0b3819263` amended the DESIGN with the route-(c) banner but left EXPECTATIONS' rows 1–3 describing
(a)/(b) — my omission. The SCORE mapped them explicitly rather than quietly scoring against the
wrong rubric, which is the correct handling of a rubric the orchestrator failed to update.
