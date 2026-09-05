# SCORE — no client call can hang

**NOT FLOOR-GREEN.** Executor: grok, 2026-09-05. Tree safe, uncommitted.
`TimedOut` is a fifth `RecvOutcome` arm; generated client methods (Path A `send-recv-form`
and Path B `send_recv_ast`) bound the receive via `call-by-deadline`; recorded migration
`wat-scripts/fixes/add-timedout-arm.wat`. Default deadline **10 000 ms**.

Row 1 holds. The last captured floor is not 5215/5215.

## ROW 1 — a silent peer no longer hangs a caller

Before (HEAD, surface `:dp::Silent/wait` against Hold-shaped never-replies, process locus):

```
timeout -s KILL 15 → Killed, AFTER=137, ~15 s
```

Earlier in this stone the same hang was still running at 157 s.

After (same probe, five-arm match including `TimedOut`):

```
"TIMED-OUT"
elapsed_s=10.683
EXIT=0
```

STOP-1 did **not** fire as "unavailable". `Env/peer-kind` at `:user::main` is
`PeerKind/process`. The floor then proved the **wrong** fact: that kind is the *caller's*,
and `select` refuses a mixed-tier set. First captured floor:

```
Summary [ 366.871s] 5215 tests run: 4979 passed (1 slow), 236 failed, 22 skipped
.floor/2026-09-05T12-37-02Z/
```

Dominant arm: **348× mixed-tier** plus `no program env installed on this thread` at
`call-by-deadline`'s `(:wat::program::env)` (`service.wat:3150`). DESIGN-a-client-has-a-deadline
already named this: *tier is the raced peer's, not the caller's*.

Fix (not a locus special-case): `call-by-deadline` takes timer kind from
`(:wat::kernel::peer-wire? peer)` → `PeerKind::process` / `PeerKind::thread`. Circuit's four
sites still race process peers from process workers. Silent probe then:

```
"TIMED-OUT" elapsed_s=10.708 EXIT=0
```

thread-tier generated method `span_surface_freezes_and_every_declared_op_replies` 0.35 s ok.

## THE MIGRATION

Census (`wat --grep`, occurrences not lines), before apply:

| rule | occurrences | files |
|---|---|---|
| `add-timedout-arm` | **526** | **263** |
| `catchall-untouched` (control) | **95** | **28** |

DESIGN's 643 Message arms / 282 files includes constructors and catch-alls; the finder is the
arm *set* of a match. After apply: **needs=0**, catch-all still **95**.

Idempotency: re-apply, `diff` of the two `git diff`s is empty (**0 bytes**). STOP-2 did not fire.

Inserted arm spelling: DESIGN wrote `((:RecvOutcome::TimedOut) body)` (CallOutcome's
tagged-empty). `TimedOut` is `EnumVariant::Unit` like `Closed`/`Stopped`. The tagged form
type-checks as "not a tagged variant" and does not count toward exhaustiveness. The recorded
codemod now inserts and untags to the **unit** spelling
`(:wat::kernel::RecvOutcome::TimedOut body)`. First apply used tagged; bootstrap untag of that
unique token (now `:user::untag-timedout` in the same file) because the rebuilt binary could
not boot with tagged arms in frozen stdlib.

## THE ROWS

| # | row | result |
|---|---|---|
| 1 | ★★ silent peer returns TimedOut ~10 s | ✅ hang-before KILL 15 s; after `"TIMED-OUT"` **10.683 s** EXIT=0 |
| 2 | ★★ migration complete / floor type-checks | ⚠ last captured floor **5208 passed, 7 failed**, 22 skipped — not 5215. Compiler *is* the census for `.wat`; two remaining reds are Lost/Closed collapsed through `CallOutcome::PeerGone` |
| 3 | the floor | ⚠ see floors below. No deadline-raise reds naming a surface (STOP-3 did not fire as "too short") |
| 4 | codemod idempotent | ✅ 0 bytes |
| 5 | census | ✅ 526 touched / 95 catch-all; 263 files |
| 6 | longer deadlines | ✅ none named. MCP reds were **non-exhaustive TimedOut** in `<mcp>` jsonl, not elapsed-time |
| 7 | chaos | not re-run after the last captured floor |
| 8 | rate-0 | ✅ circuit ×1 `total=8000; distinct=8000; dup=0; seen-recorded=8000` |
| 9 | timings | report only: publish **49346** (before 47784–49856) |

Circuit ×1 also printed `timeout=yes;discarded=yes;redial=Connected;retry-on=fresh`.
call-by-deadline's four circuit sites were **not** rewritten (STOP-4).

## FLOORS, CAPTURED, NOT RE-RUN

**1.** `.floor/2026-09-05T12-37-02Z/`

```
Summary [ 366.871s] 5215 tests run: 4979 passed (1 slow), 236 failed, 22 skipped
```

Arm: mixed-tier / missing program env (STOP-1, caller's kind).

**2.** `.floor/2026-09-05T12-47-21Z/` after peer-wire? fix:

```
Summary [ 361.020s] 5215 tests run: 5208 passed (2 slow), 7 failed, 22 skipped
```

| test | arm |
|---|---|
| `completeness_gate` | 4 new dispatched verbs: `CallOutcome::{Answered,DeadlineFired,PeerGone}`, `call-by-deadline` (stdlib now calls the helper). **Fix on disk:** `RULES` `":wat::service::" Disp::Impure` |
| `wat_mcp::a_counter_increments_across_turns` (~9.5 s) | non-exhaustive TimedOut in `<mcp>` jsonl (not a `.wat` file; codemod did not see it). **Fix on disk:** TimedOut arms added |
| `wat_mcp::a_thread_counter_increments_across_turns` (~9.7 s) | same |
| `unknown_trailing_option_is_rejected` | expected clause list lacked `:deadline-ms`. **Fix on disk:** assertion string updated. Bijection goldens at `:896/:913` **unmoved** (zero-line-shift on known-clauses) |
| `rst_peer_notify_baseline` | `select (process tier) EDN decode failed: invalid keyword: keyword begins with ::` at `call-by-deadline`'s select. Process death used to arrive as `RecvOutcome::Lost` via `recv`; select tries to EDN-decode the death line as Reply |
| `an_owner_drop_reaches_the_client_as_severed` | `left: "CLOSED:MUTE" right: "SEVERED"`. Generated methods map `CallOutcome::PeerGone` → `RecvOutcome::Closed`. Lost\|Closed is merged; Severed's cause has no home |
| `every_wat_scripts_file_loads` | rotting `probe-call-by-deadline-from-main.wat` `load-file!` of `circuit.wat` (`set-redef!` in a loaded file). **Deleted** |

Fixes after floor 2 are on disk and **not re-weighed**. Do not treat 5208/7 as stale-green.

## FINDINGS

1. **Generated methods cannot keep Lost's cause if they return through `CallOutcome::PeerGone`.**
   `recv-by-deadline` (RecvOutcome-returning twin) was written and withdrawn:
   `ServiceEvent::Lost` carries `Failure`, `RecvOutcome::Lost` wants `LociDiedError`.
2. **Process-tier select is not a drop-in for `recv` on death.** RST's panic surfaces as an EDN
   decode raise, not `Lost`.
3. **Path B (`src/runtime.rs` `send_recv_ast`) is the caller-facing generated method** for
   `:S/method`. Binding only `wat/service.wat` `send-recv-form` leaves surface calls hanging.
   Both paths are bound. Brief said `src/` is "one variant"; Path B is required for row 1.
4. **Unit vs tagged empty.** Closed/Stopped/TimedOut are Unit; extra parens is CallOutcome.

## NOT TOUCHED

Circuit's four `call-by-deadline` sites (still CallOutcome). Chaos cells. Default 10000
(STOP-3: do not raise it). RecvOutcome still four-arm at `kernel::recv` (TimedOut is only
produced by the generated-method deadline).

---

Tree uncommitted. Do not commit unless asked.

---

# ORCHESTRATOR GRADING — claude, 2026-09-05

**NOT STRUCK.** My own floor on the executor's tree, with its post-floor-2 fixes included:

```
Summary [ 356.911s] 5215 tests run: 5213 passed (3 slow), 2 failed, 22 skipped
.floor/2026-09-05T13-05-30Z/
```

Grok's on-disk fixes cleared 5 of their 7. **Two survive, and they are one root cause.**

| red | arm |
|---|---|
| `client_sees_peer_crashed_not_bare_disconnect` | an RST death arrives through `select` as an **EDN decode raise**, not `Lost` |
| `an_owner_drop_reaches_the_client_as_severed` | `left: "CLOSED:MUTE" right: "SEVERED"` — the severed cause flattened into `Closed` |

## ⛔ THE ROOT CAUSE IS A SUBSTRATE ASYMMETRY, AND I VERIFIED IT

`src/types.rs`:

```
:1831   ServiceEvent::Lost [cause <- Failure]          ← what select reports
:1939   RecvOutcome::Lost  [cause <- LociDiedError]    ← what recv reports
```

★ **`select` and `recv` report peer death with different cause types.** So a bounded receive
built on `select` cannot reconstruct a faithful `RecvOutcome::Lost` — the cause has no way
across. Grok wrote `recv-by-deadline` and **withdrew it for exactly this reason**; that was the
right call, and the withdrawal is the finding.

★★ **This is the wall under rung 3**, and it is bigger than this stone: the bounded-receive
primitive the whole deadline design rests on **cannot faithfully replace an unbounded receive.**
The circuit's four `call-by-deadline` sites got away with it only because they *choose* to treat
`PeerGone` uniformly. A generated method cannot choose — its callers already distinguish `Lost`
from `Closed` and consume the cause.

## ★ AND THIS IS MY OWN FINDING COMING TRUE

`SCORE-every-client-call-has-a-deadline`, my grading, two stones ago:

> *`CallOutcome`'s `PeerGone` merges `Lost` and `Closed`… **This is now stdlib.** Every future
> service client will copy this form, so the moment to tighten it is before there is a second
> copy… If a caller ever needs `Lost` apart from `Closed`, that is a fourth arm.*

I graded it *"not a defect today"* and deferred it. **Today is the day a caller needs them
apart, and it is two floor reds.** The deferral was wrong, and the cost of tightening it then
was four call sites; the cost now includes a stalled 270-file migration.

## WHAT GROK GOT RIGHT AND SHOULD NOT BE LOST

- **Row 1 holds.** Before: `timeout -s KILL 15 → Killed` (and 157 s earlier in the strike).
  After: `"TIMED-OUT" elapsed_s=10.683 EXIT=0`. **The hang is real and it is gone.**
- **STOP-1 fired in a form neither of us predicted.** `Env/peer-kind` *is* available at
  `:user::main` — but it is the **caller's** tier, and `select` refuses a mixed-tier set: 348
  failures. The fix — take the timer's tier from `(:wat::kernel::peer-wire? peer)`, the **raced
  peer's** — is a real design correction, not a locus special-case.
- **Path B.** My BRIEF said `src/` was "one variant". Wrong: the caller-facing generated method
  for `:S/method` is `send_recv_ast` in `src/runtime.rs`. Binding only `wat/service.wat` leaves
  surface calls hanging. **My blast-radius estimate was incomplete and grok found it.**
- **The migration is done and idempotent**: 526 arms across 263 files, 95 catch-alls correctly
  untouched, re-apply changes 0 bytes.
- **Unit vs tagged.** `TimedOut` is a Unit variant; the tagged spelling my DESIGN wrote
  type-checks as "not a tagged variant" and does not count toward exhaustiveness — so it would
  have silently failed to make matches exhaustive. Caught and corrected.

## ⚠ THE DECISION I AM NOT TAKING ALONE

**270 files of finished, idempotent migration sit uncommitted against a 2-red floor.** Campaign
doctrine is commit-on-green, and this is not green. But this is far more work than is comfortable
to leave dangling, and the reds are a *known, named, single* cause rather than a mystery.

That is the builder's call, and it is stated here rather than resolved by me.

## THE NEXT STONE

**Make a bounded receive that returns a faithful `RecvOutcome`.** Options, none free:
`select` learning to report a `LociDiedError`; `RecvOutcome::Lost` accepting a `Failure`; or an
honest conversion — and a conversion that *fabricates* a cause is the collapse this arc keeps
refusing. It is a `src/` stone, and it blocks rung 3.
