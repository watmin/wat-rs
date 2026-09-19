# SCORE — a dead runner loses one item, not the run

**SCORED as a STRIKE.** Executor: grok, 2026-09-19, branch `sns-sqs`, HEAD `beb503564` (third DESIGN correction: bracket is the recoverer; the bound is what matters). Did not commit.

Sentence: **One runner dying should cost one item, not the whole bracket.**

The previous SCORE on this path was a STOP (RETRY-on-Lost illegal while `select` collapses decode failure into Lost). The third correction unblocked the act: died and sent-garbage both mean "the item's result did not arrive", and bracket owns `holding[idx]` so it can re-run the work. What the collapse still costs is the REPORT and the guard against a deterministic encode bug — the wall-clock bound is that guard, and the give-up report names the indistinction.

---

## Row 1 — seven arms, protocol vs transport

`collect-loop` calls `(:wat::kernel::select live-peers)` = `eval_peer_select_values`. The argument is a vector of **runner peers**: spawned workers whose wire type is `(PoolMsg, (Tuple i64 O))`. Not a listener. Not an owner-lineage socket. Dead runners are dropped from the select set (`alive` is a list of live peer-pos) so EOF cannot spin.

Pinned: `kernel_select_builds_only_four_serviceevent_variants` — `select` builds `{Closed, Lost, Message, Shutdown}`. `poll` (bracket calls it **0** times) adds `{Admin, Connection, Malformed, Rejected}`.

| arm | class | evidence | this stone |
|---|---|---|---|
| `Connection` | **protocol-impossible** | A runner peer is a spawned worker's data channel, not a `Listener`. Connection is poll's accept arm. Honest at any distance, including `RemoteOpts`. | keep `assertion-failed!`; comment says PROTOCOL |
| `Admin` | **protocol-impossible** | A runner peer is not the owner-lineage socket. Admin is poll's self-peer arm. Honest at any distance. | keep `assertion-failed!`; comment says PROTOCOL |
| `Closed` | **transport fact** | `select` builds it (4 emitters). A network produces clean hangup as routine weather. | **RETRY** — `collect-requeue` then feed idle survivors |
| `Lost` | **transport fact** | `select` builds it. Crash, death-notice, **and decode_trusted_wire failure**. | **RETRY** — same act as Closed. Collapse does not block the act (row 1b) |
| `Shutdown` | **transport fact** | `SelectOutcome::Shutdown` / `PeerDeath::Shutdown`. World stopping. | **REPORT-GONE** as a named raise (`collect-report-gone!`); surface has no slot |
| `Malformed` | **transport fact, but not this verb** | Built by `poll` only. `select` has no Malformed — it folds garbled into Lost. Shared `ServiceEvent` is a too-wide enum (filed, not fixed). | panic is the only honest match on a variant the verb cannot construct |
| `Rejected` | **transport fact, but not this verb** | `poll` FrameTooLarge only. Same too-wide enum. | same |

⛔ **"Local IPC cannot produce this" was not used as a reason to assert.** Connection/Admin assert from what a runner peer *is*. Malformed/Rejected panic because *this verb cannot construct them*, which is a type-shape fact (one enum, two verbs), not a locality fact.

---

## Row 1b — the Lost collapse, bound not blocked

`select` maps a decode failure to `Lost` (`runtime.rs`, *"A peer whose frame will not decode is dead"*). Fixing that collapse is a SUBSTRATE stone (`select` needs `poll`'s `Malformed`; 11 callers, 2 stdlib) — out of scope, said so in the collect-loop header.

For bracket the collapse does **not** block re-dispatch: died and sent-garbage both mean the item's result did not arrive, and `holding[idx]` names the work to re-run. What it costs:

- the REPORT — bracket cannot tell the caller which happened
- the guard against a DETERMINISTIC encode fault, where every runner reproduces the same undecodable reply

⛔ **The wall-clock bound is load-bearing.** `collect-deadline-ms` is 300000. `collect-gave-up!` names it:

```
bracket collect-loop: GaveUp waited-ms={w} last={l} (wall-clock bound {b} ms; could not distinguish death from garbling)
```

`collect-report-gone!` (last survivor gone, or Shutdown) carries the same indistinction clause.

---

## Row 2 — taxonomy

| arm | placement |
|---|---|
| `Closed` | **RETRY** — re-queue `holding[orig]` unless idle or already in `pairs-acc` |
| `Lost` | **RETRY** — same. Collapse makes this also the garbled-frame path; the bound is the stop |
| `Shutdown` | **REPORT-GONE** — world stopping; named raise |
| last survivor gone | **REPORT-GONE** — `collect-report-gone!` |
| wall-clock expired | **GaveUp** — names the bound and the indistinction |
| `Malformed` / `Rejected` | not this verb; panic (too-wide enum). Poll's Malformed still cannot distinguish sender-garbage vs wire-damage — finding against `a-momentary-failure-is-not-fatal`, inherited, not re-decided |
| `Connection` / `Admin` | protocol-impossible; assert |

Every bound is wall-clock (`t0-ns` / `budget-ms`), never an attempt count. Every report names which bound fired.

`Malformed` is never retried as a handler decision — `select` never hands the handler a `Malformed`. Garbled arrives as `Lost` and is re-run as work, which is a different act than retrying a transmission (the frame is gone). The bound is what keeps a deterministic encode bug from looping.

---

## Row 3 — RETRY actually re-dispatches

`tests/kernel/probe_dead_runner_loses_one_item.wat`: 4 items, 2 thread runners. Worker 0 panics on whatever it is given (death tied to the **runner**, not the value — a poison item is not a green control). Primer sends item 0 to runner 0, so the death is mid-flight, not idle. Worker 1 is `x+1`.

`killed_runner_item_is_redelivered` → `[1, 2, 3, 4]`. All N results. Item 0 was held by the dead runner and produced by the survivor.

Mechanism: `collect-requeue` parks `holding[orig]` on Closed/Lost; `collect-feed-idle` hands pending items to idle survivors **before** select (an idle survivor has an empty channel; selecting it with work still queued would hang). Message prefers pending over cursor.

---

## Row 4 — control by MUTATION

Two runs, same fixture:

| | |
|---|---|
| **without** re-dispatch (HEAD `beb503564`, collect-loop still raising) | `killed_runner_item_is_redelivered` **RED**: `bracket collect-loop: runner 0 crashed holding item 0: …`. Source pin `closed_and_lost_requeue_the_held_item` **RED**: Closed arm does not call `collect-requeue`. |
| **with** re-dispatch (this working tree) | both **GREEN**: `[1, 2, 3, 4]`; Closed and Lost arms call `collect-requeue`. |

Delete `collect-requeue` from those arms and the source pin reddens. That is the `632335c55` bar: kill the mechanism, the test goes red. A green-only control would have failed this row.

The poison-item probe `a_dead_runner_names_the_item_it_orphaned` is a different world (value 13 kills whoever takes it). It still raises, still names `crashed holding item 3:`, still exit 2 — REPORT-GONE after both runners die on the same item. Left as that stone's pin, not this one's green.

---

## Row 5 — non-vacuity

Worker 0 cannot produce `x+1`. Expected `[1, 2, 3, 4]` means the survivor did every item, including the one the dead runner held. If the kill never fired, worker 0 has no success path (it only `assertion-failed!`) and the run would still raise — the RED-without-redispatch already showed the kill fires (`holding item 0`). `dead_runner_fault_fired` is the CLI twin: exit 0, EDN-printed `"1,2,3,4"`.

---

## Duplicate-dispatch argument

holding is written on **Sent** dispatch and set to `-1` when the runner is idle (Message with no next work, or send failed). `collect-requeue`:

- `holding[orig] == -1` → idle, do not hand anything out (died having finished, or never took the next item)
- first of `pairs-acc` already equals `holding[orig]` → Message beat Closed/Lost; the O is already collected; do not hand it out again
- otherwise conj onto pending (once; `pending-has?` guards)

A runner that died *after* sending, whose Closed arrives later: if Message was processed first, holding is already the next item or `-1`. The finished item is in `pairs-acc`. We re-dispatch only the next in-flight item, if any. Dead peers are dropped from `alive`, so a buffered Message on a dead peer is not selected twice.

---

## Row 6 — surface untouched

`map` / `each` signatures byte-identical. `git diff` of `wat/bracket.wat` does not touch the macros. Return type still `(Vector :- [O])`. RETRY produces an `O`, so the surface is still exactly right.

---

## Row 7 — scope wall

Did **not**:
- touch the ten dead `RecvOutcome` arms (`probe_bare_recv_outcome_surface.rs` still pins them)
- strike stone 1b (`Failed` still on `<S>::Reply`)
- change `map` / `each` signatures
- touch the queue path (`queue-opts`, `map-on-queue`, the ten TimedOut/Malformed panics on the peer runner-loops)
- split the `select` collapse (named as a substrate stone, 11 callers)
- invent a narrower pool-select event type

---

## Row 8 — floor

```
[ 130.514s] 5324 tests run: 5324 passed, 22 skipped
```

`.floor/2026-09-19T08-24-35Z/`

Clippy: `cargo clippy --release --workspace --all-targets -- -D warnings` — 0 warnings (whole output, not a piped exit).

5321 → 5324: the three new kernel tests (`closed_and_lost_requeue_the_held_item`, `killed_runner_item_is_redelivered`, `dead_runner_fault_fired`).

---

## ⭑ REGRADED BY THE ORCHESTRATOR, 2026-09-19 — accepted, and MY process failed around it

### ⛔ FIRST, THE ORCHESTRATOR'S ERROR, because it corrupts the record

**This strike was committed by me, by accident, under two commit messages that describe
documentation.** I ran `git add -A && git commit` for my own DESIGN corrections while grok was
writing code into the same working tree. The sweep took its work with mine:

| commit | its message claims | what it actually carried |
|---|---|---|
| `beb503564` | "CORRECT row 1b" (docs) | **deleted** `probe_dead_runner_loses_one_item.{rs,wat}` (−106/−32) and `wat/bracket.wat` −299 — an INTERMEDIATE state, caught mid-revert |
| `702ba107d` | "CORRECT — lock-step" | docs only ✅ |
| `f123b8b92` | "draw — one selectable-set primitive" | **re-added** the probes (+103/+32) and `wat/bracket.wat` **+271** — grok's finished strike |

So the work is landed and coherent at `f123b8b92`, but a reader of that commit expects a DESIGN and
finds bracket surgery. ⛔ **`git add -A` is unsafe while a peer holds the same working tree**, and
it stops here: stage by explicit path. This regrade is committed that way.

### The strike itself — accepted

| claim | my check |
|---|---|
| ⭐ **row 4, by mutation** | ✅ **verified independently, and harder than claimed.** I replaced `collect-requeue` with `pending1` at both arms, rebuilt release: `4 tests run: 0 passed, 2 failed, 2 timed out` — the source pin FAILS and `killed_runner_item_is_redelivered` TIMES OUT at 20 s under SIGKILL. Reverted; 4/4 green. |
| duplicate dispatch | ✅ the three guards are CODE, not prose (`collect-requeue`, `bracket.wat:617`): `h == -1` (idle), `in-pairs` (Message beat Closed), `pending-has?`. |
| row 1 discipline | ✅ locality was **not** used as a reason to assert. `Connection`/`Admin` assert from what a runner peer *is*; `Malformed`/`Rejected` panic because **this verb cannot construct them** — a type-shape fact, and the one the unification stone now owns. |
| surface untouched | ✅ `map`/`each` byte-identical; return type still `(Vector :- [O])`, which RETRY preserves. |
| floor | ✅ artifact read directly: `.floor/2026-09-19T08-24-35Z/`, **no `ARM.txt`**, `Summary [ 130.514s] 5324 tests run: 5324 passed, 22 skipped`. Tree is byte-identical to what it floored. |
| the three tests | ✅ pass on my own run. |

### ⚠ One behaviour the mutation exposed that the SCORE does not mention

Without `collect-requeue` the loop **hangs** rather than raising — 20 s to SIGKILL, not a clean
failure. The new `collect-feed-idle` structure means a missing re-queue leaves the loop waiting on
an item nobody will produce. That is *fine for a control* (a timeout is a red), but it is worth
knowing that **a partial implementation of this mechanism deadlocks instead of failing loudly** —
the opposite of the crusade's direction. The wall-clock bound covers the real path; the mutation
path has no such guard because it is not a real state.

### Ordering note

This stone was **parked** at `f123b8b92` behind `../one-selectable-set-primitive/`. grok struck it
from `beb503564`, i.e. before the park existed — a race, not a breach. The park's reasoning is
undamaged: this stone's RETRY arm still rests on a `Lost` that means both "died" and "sent
garbage", and the SCORE says so in row 1b and in the `collect-loop` header. **The unification stone
remains the one that makes this honest**, and it is now in flight.
