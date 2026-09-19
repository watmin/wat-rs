# Every worker races its own timer

> ⛔⛔ **REDIRECTED 2026-09-19 TO ROUTE (c) — A BOUNDED `select`. Builder: "(c) has been reasoned."**
> Rows 1–3 below described routes (a) and (b); **(b) is now superseded.** Read this banner first.
>
> ## Route (c): `select` gains a deadline
>
> ⭐ **`select` is the only wait in this system with no bounded form.** `recv` has
> `recv-by-deadline`; `call` has `call-by-deadline`. `select`, `poll` and `accept` have nothing.
> That asymmetry is the argument, and it holds independently of brackets — it fixes the gap for
> **all 11 `select` callers**.
>
> **The mechanism already exists at both tiers.** Thread: crossbeam's `select_timeout` is the
> direct analogue of the `.select()` already called (`comms/thread.rs:444`). Process: a **timerfd
> in the ring** — which is already how `recv-by-deadline` gets its timeout there
> (`comms/process.rs:2225`, an `itimerspec`). And `eval_peer_recv_by_deadline` already covers both
> spawn tiers, so this is the N-peer analogue of a 1-peer wait that works today.
>
> ## ⛔ N DEADLINES, ONE TIMER — and the builder's correction that got this right
>
> An earlier orchestrator line said "one deadline". **Wrong, and the distinction is the design.**
> Builder, with the worked case: a pool of 3, five tasks, a 20 s budget —
>
> > *"w-1 task-1 in 4 seconds, w-1 timer reaped, w-1 gets task 4, w-1 gets a FRESH timer for task 4
> > … every time a worker is assigned a task, they are given a fresh deadline."*
>
> The deadline is per **(worker, task) assignment** and is refreshed on every new assignment. What
> route (c) shares is the **wakeup**, not the deadline: one bounded wait whose timeout is
> `min(remaining)`, then a scan. ⭐ The builder reached the same implementation independently —
> *"we could probably optimize to exactly 1 timer with some map who knows when each will expire"* —
> so the semantics are per-assignment and the timer count is an optimisation, not a compromise.
>
> ## ⭐ THE RST HAZARD, AND IT IS ALREADY HALF-SOLVED
>
> Builder: *"we also need a 'RST wake' or something to kill the timedout worker."* The hazard is
> real and sharper than killing: a timed-out worker is **still running**, and its late reply
> arrives for a task another worker has already completed. Unguarded, the `Done` arm's `conj`
> would enter the same item **twice**.
>
> ⛔ It is guarded — the in-flight `already` fold discards a reply whose item is in `pairs-acc` and
> marks that runner **idle** rather than killing it. **Keep that; it is the answer to the RST
> question and it survives this redirect.** Reclaiming a healthy worker beats killing one, and it
> is the same fact `collect-requeue`'s `in-pairs` guard encodes on the other side.
>
> ## What survives, stated honestly
>
> | built under (b) | under (c) |
> |---|---|
> | the `already` duplicate guard on `Done` | ⭐ **keep** — essential under any scheme |
> | `PoolReply::Tick` | ❌ drop — a timeout is the bounded select returning, not a message |
> | the `PoolReply` enum itself | ⚠ **its justification went with the Tick.** With only `Done` it is a tuple with extra steps. Do **not** keep it merely because it is built; re-argue it or revert it |
>
> ## The work, under (c)
>
> 1. **`select-by-deadline`** — a separate verb, following `recv` / `recv-by-deadline`'s precedent
>    rather than an optional parameter. The unbounded `select` stays; all 11 callers unaffected.
> 2. **Per-assignment deadlines in `collect-loop`** — a vector beside `holding`, stamped at
>    dispatch and cleared on `Done`. The wait's timeout is `min(remaining)`.
> 3. **On expiry** — re-queue via the existing `collect-requeue` (its guards intact), mark the
>    runner idle, leave it in `alive`.
> 4. ⛔ **Keep the `already` guard**, and say in the SCORE how a late reply from a timed-out worker
>    is discarded.
>
> ⚠ Rows 4–7 of EXPECTATIONS still apply: control by mutation (remove the bound → the test hangs,
> and a timeout is a fail), non-vacuity, "at least N" margins, and a floor with its tree stated.

**The builder's design, specified.** Excursus `001-sns-sqs`. Read first:

- `../the-bracket-peer-path-faces-chaos/SCORE.md` — the measured DoS this answers: a fleet-wide
  stall hangs the coordinator for ever, because the bound reaches only a pool of one.
- `wat-scripts/scratch-pad/probe-selectables-homogeneity.wat` — the proof that a timer and a real
  peer can share one selectable set. **This stone's mechanism already works**; the question is
  reach.

## The sentence, and it is the builder's

> *"how would it block when there's a timer per runner? if there's a pool of 10 workers then they
> get 10 timers paired with them? every worker is racing its own dedicated timer?"*

**Yes. And the orchestrator was wrong to say `select` would block** — a stalled runner's paired
timer firing IS a select event, so the set is never quiet. The obstacle was never the logic.

## What already works, measured

`probe-selectables-homogeneity.wat` put a timer and a reply-ing client in **one** `poll`
`selectables` vector and got both delivered as `ServiceEvent::Message`. Two facts make it type:

- **I-side is free.** A timer never sends, so `after` infers `I = :wat::core::Never`, the named
  bottom. `Never <: T` for every `T`, so a timer assigns into any peer vector's element type.
- ⛔ **O-side is NOT free.** *"the timer must deliver the SAME `Op` the client speaks"* — in the
  probe, a shared enum with a `:Tick` variant beside the client's `:Ping`.

## ⛔ So the blocker is the UP WIRE, and it is an ASYMMETRY

| direction | type | can a timer live here? |
|---|---|---|
| coordinator → runner | `:wat::bracket::PoolMsg :- [D I]` — an **enum** (`:Setup`, `:Work`) | n/a |
| runner → coordinator | a bare `(:wat::core::Tuple :- [:wat::core::i64 O])` | ⛔ **no — a tuple has no variant to be a `:Tick`** |

**The down wire is already an enum; the up wire is a bare tuple.** That asymmetry is the whole
obstacle, and closing it is an argument on its own terms — not a concession to make timers fit.

Measured: **32 sites** in `wat/bracket.wat` mention that tuple shape.

## The work

### 1. The up wire gains variants

`(Tuple i64 O)` → an enum, symmetric with `PoolMsg`: a `:Done [pair]` carrying today's payload and
a `:Tick [runner-idx]` a timer can deliver. ⛔ **Name it for what it is** (a reply on the pool wire),
not for the timer that prompted it.

⚠ 32 sites. This is a `.wat` structural rewrite across one file — read `CLAUDE.md` on the
self-hosted codemod before hand-editing, and **count occurrences, not lines**.

### 2. ⭐ A timer per runner, in the set

`select` over 2N entries: N runners and N timers, each timer armed when its runner is handed work
and carrying that runner's index. A `:Tick` for runner *k* means *k* missed its budget → re-queue
`holding[k]` through the existing `collect-requeue` and hand it to a survivor.

⭐ **This is the visibility timeout, with the coordinator as its own broker** — which is exactly why
it works: the builder's earlier framing (*"almost like a visibility timeout thing"*) is the same
mechanism the queue already uses, minus the third party.

### 3. ⛔ The reach problem, and its two routes — DECIDE, do not assume

The homogeneity proof used `poll` over **unified** `Peer`s. Bracket's runners are spawn-tier
`:wat::kernel::Thread` / `::Process` opaques, and `eval_peer_select_values` dispatches on the
**first** peer's `type_path`, so a unified-Peer timer cannot join that branch today.

| route | cost |
|---|---|
| **(a)** make bracket's runners unified `Peer`s | loses the **crash channel** — which is how `Lost`/death detection works today. ⛔ A stall bound bought by breaking death detection is not a win |
| **(b)** teach the spawn-tier select to accept a timer entry | reaches back into `one-selectable-set-primitive`'s un-merged half |

⛔ **Read the crash-channel dependency before choosing.** Route (a) looks smaller and may cost the
one fault we can already induce. Report the choice and the evidence.

### 4. The control, by mutation

A pool of N where **every** runner stalls must produce `collect-gave-up!` (or per-item re-queue)
rather than hang. ⛔ Remove the timers → the test **hangs**, which a timeout marks as a fail.
⚠ Non-vacuity: prove the stall fired, and assert the bound as **"at least N"**, never "exactly N".

## Scope wall

⛔ Not here: the peer-path **hardening** (a cap on the runner wire, a 400-class disposition for an
oversized reply) — its own stone, and already evidenced. Suppression. The thread tier. The
lineage-`Admin` delay. `wat/queue.wat`'s knobs.

⚠ And **not** the queue path: it already solves fleet-stall via visibility expiry, and
`probe_queue_visibility.rs` proves the mechanism. That the *bracket* queue path is untested for
stall is a separate, cheaper stone.

## The four questions

- **Obvious** — ✅ it is the builder's own design, and the mechanism is already proven for one tier.
- **Simple** — ⚠ **no.** A 32-site wire change plus a select-reach decision. The up-wire enum is
  justified independently (the down wire is already one), which is what keeps this from being
  machinery bolted on for a timer.
- **Honest** — ✅ row 3 forbids assuming the cheap route, and names what route (a) would cost.
- **Good UX** — ✅ `map`/`each` unchanged; a stalled fleet loses items to survivors instead of
  hanging the caller.
