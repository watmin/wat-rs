# Every worker races its own timer

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
