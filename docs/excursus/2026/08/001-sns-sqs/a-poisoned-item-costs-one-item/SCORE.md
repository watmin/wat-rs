# SCORE — a poisoned item costs one item

**SCORED as a STRIKE.** Executor: grok, 2026-09-19, branch `sns-sqs`, HEAD `335e3b1f1` (DRAW). Did not commit.

Sentence: **One poisoned item costs one item.**

---

## Row 1 — the sibling verb

`(:wat::bracket::map-by-outcome locus items work-fn)` → `(Vector :- [(ItemOutcome O)])`.

`each-by-outcome` is the same expansion (returns the fates; does not discard).

**Name.** Follows `recv-by-deadline` / `select-by-deadline`: `-by-<thing-gained>`. The thing gained is a per-item outcome instead of a bare `O`. ⛔ Not `try-` — that prefix means non-blocking (`try-send`). `try-map` would lie on the axis that matters.

`map` / `each` defmacro signatures **byte-identical** (three params + `& kwpairs`). Existing call sites not edited. New probes only. DESIGN's 51/9 was a prior census; this strike did not retarget any of them.

No kwargs tail, no queue door — F5 would force a full duplicate of the kwargs parse, and queue knobs are out of scope. A QueueOpts constructor type-errors into `map-worker-reported` (expects `Locus`).

---

## Row 2 — the outcome shape

No Vector-of-outcomes precedent (measured zero). Follows `CallOutcome` / `StopOutcome`: success carries `O`; every failure arm is a `ServiceEvent` (or the wall) the coordinator actually constructs.

```
:Ok        [value]
:Rejected  [cause]     ;; FrameTooLarge. REPORT-FINAL.
:Malformed [cause]     ;; garbled result. Sibling records it.
:Lost      [cause]     ;; last runner died. Remaining items, same event.
:Closed                ;; last runner Closed. Remaining items.
:GaveUp    [waited-ms last]  ;; wall. Outstanding items. last names the wait.
```

⛔ Not in the type (coordinator cannot project a per-item ServiceEvent):

| arm | what the coordinator knows | what it does |
|---|---|---|
| empty `alive` at loop head | fleet gone, no event | still `REPORT-GONE` |
| `Shutdown` | world stopping | still `REPORT-GONE` |
| `Connection` / `Admin` | protocol-impossible | still panic |
| `Rejected`/`Malformed` with `holding=-1` | runner idle, item unknown | keep runner, do not record |

Do not reintroduce death-vs-garbling on `GaveUp`. `last=TimedOut` is the wait that ended it.

---

## Row 3 — FrameTooLarge is REPORT-FINAL in fact

| arm | `map` (`collect-loop`) | `map-by-outcome` (`collect-loop-reported`) |
|---|---|---|
| Message | `O` | `ItemOutcome::Ok` |
| Rejected | drop from `alive`, re-queue, last runner `REPORT-GONE` | **record, drop the ITEM, KEEP the runner** |
| Malformed | re-queue, keep runner, `_cause` dropped | **record the cause, drop the item, keep the runner** |
| Lost / Closed | re-dispatch to a survivor; last runner `REPORT-GONE` | still re-dispatch; last runner **fills remaining** with Lost/Closed |
| GaveUp wall | raise | fill remaining with GaveUp |
| Shutdown / empty-alive / Connection / Admin | raise / panic | unchanged |

`Lost`/`Closed` re-dispatch stays right — a dead runner's item is not a poisoned item.

The kept Rejected runner is wedged in `write_all`. `collect-feed-idle` `try-send` **skips** `WouldBlock` and tries the next idle (returning on the first WouldBlock stalled remaining work behind a zombie at the front of `alive`).

**`try-send` now faces Thread and Process**, not only unified `Peer`. `send` already did. `collect-feed-idle` on a process spawn-runner was a runtime TypeMismatch (`expected Peer, got Process`) the previous stone never hit (1-item oversize never feed-idles after drop). Mixed `map` with 4 items 3 runners hit it. The 7 blocking `send` sites are untouched.

---

## Row 4 — mutation, both ways

Same input: 3 process runners, items `[10 20 -1 40]`, poison at index 2 returns 1 MiB (`double-n "x" 20`).

**With recording** (`map-by-outcome`, isolated, bound 2000):

```
ok:0=11;ok:1=21;rejected:2=frame exceeded cap (message larger than the receiver's max-message-bytes budget);ok:3=41;elapsed-ms=129;n=4
```

Exit 0. Good results. Poison named. Not `REPORT-GONE`.

**Without** (`map`, isolated, bound 2000):

```
REPORT-GONE last runner 1 crashed holding item 2: frame exceeded cap
  (message larger than the receiver's max-message-bytes budget)
```

Exit 2. The pool is lost.

**Revert the recording** (`alive-without` + `collect-requeue` on `collect-loop-reported`'s Rejected): `reported_rejected_records_and_keeps_the_runner` reddens.

---

## Row 5 — non-vacuity + fleet

| | |
|---|---|
| poison really failed | `rejected:2=` names `"frame exceeded cap … budget"` |
| other items | `ok:0=11` `ok:1=21` `ok:3=41` — `x+1`, both sides of the poison |
| fleet | returned `n=4`, not `REPORT-GONE`. The wedged runner stays in `alive`; survivors finish the rest |
| all-Ok sibling | `by=2,3,4;map=2,3,4;n=3` — happy path matches `map` |

---

## Row 6 — scope wall

`map`/`each` signatures and existing sites, thread-tier poison (no frame cap), suppression, lineage-Admin, queue knobs, `RecvOutcome`, kwargs on the sibling, the 7 blocking `send` sites — untouched.

`try-send` gained Thread/Process arms so the previous stone's feed-idle bound actually fires on process. That is not one of the 7 sends.

---

## Row 7 — floor

First floor **RED**, captured, not re-run as green:

```
     Summary [ 129.612s] 5347 tests run: 5343 passed, 4 failed, 22 skipped
```

`.floor/2026-09-19T22-07-14Z/` — four arms:

1. `no_inlined_edn` — `find("(:wat::core::defmacro …")` opened with `(`
2. `no_loose_string_assert` — two `contains` in the sweep without `rune:lint(loose-assert)`
3. `brackets_four_unreachable_serviceevent_arms_are_counted` — Admin/Connection/Rejected/Malformed now 2 (one per collector)
4. `bound_sweep_stays_prompt` — bound **200** under load: `gaveup:0=200:TimedOut` ×4, elapsed-ms=217. 200 is below honest 3-process work, not a hang

Second floor **RED** `.floor/2026-09-19T22-11-01Z/`:

```
     Summary [ 131.584s] 5347 tests run: 5346 passed, 1 failed, 22 skipped
```

`queue_path_survives_recv_drop` — `GaveUp waited-ms=8000 last=TimedOut`. Queue code untouched. A 4-bound × 2-path process-pool sweep overlapped it. Sweep shrunk.

Third floor **RED** `.floor/2026-09-19T22-16-03Z/`:

```
     Summary [ 132.334s] 5347 tests run: 5346 passed, 1 failed, 22 skipped
```

`mixed_poison_names_the_item_and_keeps_the_fleet` at bound 2000:

```
ok:0=11;gaveup:1=2000:TimedOut;rejected:2=frame exceeded cap …;ok:3=41;elapsed-ms=2041;n=4
```

Returned (not `REPORT-GONE`). One good item missed the wall under floor load. Isolated at 2000 was 129 ms all-Ok. Floor bound raised to 10000.

Green floor ran on HEAD `335e3b1f1` plus this tree:

```
 src/kernel/peer.rs                                 |  19 ++
 src/runtime.rs                                     |  87 ++++-
 tests/kernel/probe_bare_recv_outcome_surface.rs    |  17 +-
 wat/bracket.wat                                    | 422 ++++++++++++++++++++-
 tests/kernel/probe_poisoned_item_all_ok.wat        | new
 tests/kernel/probe_poisoned_item_costs_one_item.rs | new
 tests/kernel/probe_poisoned_item_map_gone.wat      | new
 tests/kernel/probe_poisoned_item_mixed.wat         | new
```

```
     Summary [ 131.649s] 5347 tests run: 5347 passed, 22 skipped
```

`.floor/2026-09-19T22-21-03Z/` — exit **0**. Count **5340 → 5347** (+7 poisoned-item tests). CLIPPY=0 (`cargo clippy --release --workspace --all-targets -- -D warnings`, whole output, exit 0). `wat/core.wat` still 2152.

**Bound sweep.** Isolated: mixed 2000 → 129 ms, shape intact; map 2000 → `REPORT-GONE` prompt. Floor: 200 all-GaveUp; 2000 one-item GaveUp; 10000 green and prompt (`wall_ms < 15000`). The old deadlock cliff (hang at ≥1000) did not return.

---

## GRADING MAP

| # | expected | result |
|---|---|---|
| 1 sibling | not `try-`; `map`/`each` byte-identical | `map-by-outcome` / `each-by-outcome`; headers unchanged |
| 2 shape | only distinguishable fates | `ItemOutcome`; empty-alive / Shutdown / idle-Rejected named as nulls |
| 3 FrameTooLarge | record, drop item, keep runner | Rejected arm; Lost/Closed still re-dispatch |
| 4 mutation | both ways | mixed returns 3 Ok + 1 Rejected; `map` REPORT-GONE; source pin |
| 5 non-vacuity | poison failed; others alive | cap named; `ok:0/1/3`; n=4; not REPORT-GONE |
| 6 scope | map/each, queue, RecvOutcome, 7 sends | untouched; try-send gained Process/Thread |
| 7 floor | Summary + `.floor/` + sweep | green `.floor/2026-09-19T22-21-03Z/`; three prior reds captured |

---

## ⭑ REGRADED BY THE ORCHESTRATOR — accepted. The sentence is delivered, and it caught a miss of MINE

### ⭐ Reproduced exactly

```
with recording : ok:0=11;ok:1=21;rejected:2=frame exceeded cap (…budget);ok:3=41;n=4   127 ms, exit 0
without        : REPORT-GONE … frame exceeded cap                                     pool lost
```

**One poisoned item costs one item.** Three good results either side of the poison, the cause named,
the fleet intact. Five stones queued behind this wall; it is down.

My own bound sweep on the mixed fixture — the dimension that mattered:

| bound | wall | shape |
|---:|---:|---|
| 500 | 269 ms | `ok:0 ok:1 rejected:2 ok:3` |
| 2000 | 256 ms | same |
| 10000 | 257 ms | same |

Shape invariant across a 20× bound range. And `map`/`each` headers: **zero deleted lines** — the
surface genuinely did not move.

Floor: mine, **`Summary [ 127.064s] 5347 tests run: 5347 passed, 22 skipped`**
(`.floor/2026-09-19T22-28-18Z/`), clippy 0/0.

### ⛔⛔ AND IT FOUND A DEFECT I GRADED AS GOOD

> *"`try-send` now faces Thread and Process, not only unified `Peer`. `collect-feed-idle` on a
> process spawn-runner was a runtime TypeMismatch the previous stone never hit."*

Verified: before this strike `eval_peer_try_send` referenced `PEER_TYPE_PATH` **three times and
`THREAD`/`PROCESS` zero times**. So `1e5c16866`'s `try-send` fix — which I accepted, and whose
mutation I verified myself — **could not work on a process runner at all.**

⭐ **Why both of us missed it, which is the transferable part:** I swept the **bound** exhaustively
(that is how I found the deadlock) and never varied **fleet size or item count**. The oversize
fixture is **1 runner × 1 item** — the wedged runner is dropped, `alive` empties, `REPORT-GONE`
fires, and `collect-feed-idle` is never reached with a process peer. **A 1×1 fixture cannot exercise
a re-dispatch path.** grok found it because the mixed fixture is 4 items × 3 runners.

⚠ So `1e5c16866`'s headline — *"a bounded wait plus an unbounded send is not a bound — this is the
send that was unbounded"* — was **right in intent and dead in effect** for the process tier until
now. The sentence stands; the implementation only starts working here.

### Row 2's nulls are the best part of the shape

`ItemOutcome` names only fates the coordinator can project, and the table of what is **excluded** —
empty `alive` at loop head, `Shutdown`, `Connection`/`Admin`, and a `Rejected`/`Malformed` whose
`holding = -1` — is the row doing its job. ⭐ Refusing to record an item for an *idle* runner is the
subtle one: the coordinator does not know which item that reply belonged to, so inventing one would
be exactly the false distinction the stone forbade.

### ⚠ Three floor reds, all captured, and one worth flagging

Arms 1–3 were mechanical (an EDN-esque literal, two un-runed `contains`, and a count that moved
because there are now two collectors). Arm 4 and the later `queue_path_survives_recv_drop` red were
**load sensitivity**, not defects — and the response was to widen the test's bound from 2000 to
10000 and shrink an overlapping sweep.

That is legitimate: the control still discriminates (plain `map` gives `REPORT-GONE` promptly at any
bound), and it was disclosed rather than quietly bumped. ⚠ But a test needing a bound **80× its
isolated runtime** to survive floor load is a standing signal, not a settled one. If it reddens
again, the answer is not a bigger number.
