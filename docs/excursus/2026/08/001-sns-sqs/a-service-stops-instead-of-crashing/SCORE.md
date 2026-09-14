# SCORE — a service stops instead of crashing

**SCORED against the REFUTE brief** (`eddd8025c`). Executor: grok, 2026-09-13, branch `sns-sqs`. Did not commit.

A prior revision of this stone (Faulted-on-the-seam) was in flight when the refute landed. It was discarded. This SCORE is route C as written.

```
     Summary [ 536.288s] 5246 tests run: 5244 passed (9 slow), 2 failed, 22 skipped
```

`.floor/2026-09-13T23-46-04Z/` — `scripts/floor.sh` exit **100**, **ARM.txt kept**.
Count **5245 → 5246** (+1 negative-control unit test). Not a shrink.
Clippy: **CLIPPY=0**. `cargo nextest run --release --no-run` → **NORUN=0**.
Happy: `distinct=8000;dup=0`. `--check` sqs / circuit / sns-fanout: **0**.

---

## ⭑ THE HEADLINE — a wat raise is a graceful nil, and that is not the probe's headline

Acceptance probe (`probe-a-handler-raise-kills-the-service.wat`):

```
a-control=Ok
a-boom   =Lost
b-dial=connect-REFUSED
```

Row 2 holds. Row 3 holds (`Lost`, never `Ok`). **Row 1 does not.** Ending the serve recursion (on-fault returns `nil`, seam is in tail position of `serve -> nil`) **exits the process**. An innocent second client cannot connect. That is the route the refute demanded, measured.

The BRIEF still says the strike "makes the innocent client's call succeed." Those two sentences cannot both be true of one on-fault fn that only receives `(state, cause)` and returns `nil`. Recurring would need `self` / `l` / `selectables` / `next-id`, which the sketch does not pass.

---

## Floor red — owner sees Closed, the mute

```
FAIL recv_outcome_wall_panic_thread_admin_carries
  actual:   #probe.Outcome/Closed []
  expected: #probe.Outcome/Lost [true]

FAIL recv_outcome_wall_panic_process_admin_carries
  actual:   #probe.Outcome/Closed []
  expected: #probe.Outcome/Lost [true]
```

`.floor/2026-09-13T23-46-04Z/ARM.txt` kept. **Not re-run.**

The client-side panic tests **pass** (broadcast `PeerCrashed` → `Lost`, reason-free). The admin-side tests fail because a graceful `nil` is a **clean exit**: the owner handle sees `Closed`, not `Lost` carrying `BOOM-CRASH-SENTINEL-9173`. That is the mute the recv-outcome wall killed for clients, now on the owner. rterr-admin still passes (Diagnostic path still unwinds).

Did not patch those goldens. The mismatch is the contract.

---

## WHAT LANDED (route C)

`serve-dispatch-op` is arity **4**: `(clients body state on-fault-fn)`. `args[0]`/`args[1]` unchanged so passthrough inference on the body stays. `src/check.rs:12400` and `src/intrinsic/kernel/serve.rs` both take 4.

On `AssertionPayload`: broadcast as today, `apply_function(on-fault, [pre-op state, cause])`, return that `nil`. On any other panic: `resume_unwind` (STOP-4).

On-fault is a **top-level** `defn` (`:svc::on-fault`), emitted beside `hibernate-project-def` (STOP-6). It calls `(~hibernate-project-name state)` then `nil`. Bound after `:913` so the bijection goldens do not move.

STOP-1 re-derived: `(:wat::core::defn ~serve-name ~serve-params -> :wat::core::nil ~serve-body)` at `service.wat:3472` / `:3912`. Seam return is nil-typed.

---

## Mid-strike note (not the score of a discarded route)

A wrap-the-handler-body variant of `Outcome::Faulted` was measured green on the acceptance probe (`b-after=Ok`) before the refute was ingested. It was discarded as instructed. The refute's impossibility ("the seam cannot return an Outcome") holds **at the original wrap site** (Message arm, tail of serve). It would not have held if the wrap moved to the Outcome scrutinee. That is not this stone.

---

## GRADING MAP

| # | expected | result |
|---|---|---|
| 1 | ⭑ innocent client served | **NO** — `b-dial=connect-REFUSED`. Named above. |
| 2 | a-control=Ok | **yes** |
| 3 | a-boom is a failure, never Ok | **Lost** |
| 4 | substrate panic still crashes | unit test `serve_dispatch_op_resumes_a_non_assertion_panic`; rterr-admin still Lost |
| 4b | projection ran | `hibernate-project` is called in `:svc::on-fault` |
| 5 | one Outcome match | not applicable (no new variant) |
| 6 | tests compile | NORUN=0 |
| 7 | floor | **RED**, 5246, ARM kept |
| 8 | clippy | 0 |
| 9 | happy | `distinct=8000;dup=0` |
| 10 | floor-time vs 528.052 | 536.288 s, **+8.2 s** |
| 11 | scope | a raise in an op handler is a graceful **nil**, not "cannot crash". `:init`/`:hibernate`/`:stop`/serve-loop still fatal. rterr still fatal. |
| 12 | a real redial-failed arm | **not reached**. No injector fires those arms. Absence is D3-a's finding. |

---

## What this hands the orchestrator

Route C does what the refute asked: the seam returns `nil`, projection runs, clients still see `Lost`. It does **not** do what the probe's headline asked. The owner-side mute (`Closed`) is the same fact seen from the lineage. Either the headline is wrong, or the on-fault fn has to keep serving, which this arity cannot do.