# DESIGN STONE — a shutdown broadcast is a WAKE, never a preemption (CONFIRMED RED)

> ## ✅✅ CONFIRMED 2026-08-05 BY THE RED GATE — AND THE INTERVENING "CORRECTION" WAS ITSELF WRONG
>
> **The diagnosis below is RIGHT.** `tests/comms/probe_arc278_a_wake_is_not_a_preemption.rs` is RED,
> deterministically, both tests, in **0.006s**:
>
> ```
> [gate] cascade fired for real: broadcast fd 8 is READY (worker wrote it)
> [gate] -> recv with BOTH arms ready
> [gate] <- recv returned Err(Shutdown)          ← the frame was in the pipe
> ```
>
> A frame already delivered, the substrate cascade genuinely fired, and `broadcast wins ties`
> discards the frame and reports a stop. The drain test dies on the FIRST of three frames — none are
> handed back. In wat this surfaces as `RecvOutcome::Stopped`, which is precisely what
> `wat-tests/test.wat:290`'s arm calls *"stopped before the child sent its value — the child was
> ALIVE."* **The child did send.**
>
> **It is DETERMINISTIC, not a race.** `broadcast wins ties` is an unconditional branch: given both
> arms ready it discards the data every time. What is rare is REACHING the tie — a stop must land
> while a frame is unconsumed. That is why it reads as a flake and why ~15 floor re-runs never found
> it, but the rule itself never varies. It also needs NO load: two sends and a wake byte, 6ms.
>
> ### ⛔ WHY THIS FILE BRIEFLY CLAIMED THE OPPOSITE — the failure to not repeat
>
> An earlier banner here said the gate had REFUTED this diagnosis. That banner was wrong, and the
> reason is the whole lesson: **the first gate fired the wrong trigger.** It called
> `runtime::trigger_shutdown()`, which drops the crossbeam Sender (the THREAD-tier sever) and
> **never writes the broadcast byte** — the broadcast is written solely by the shutdown worker
> (`runtime.rs:477`). So `got_broadcast` was false, there was no tie at all, and the gate "passed"
> while testing nothing. I read that pass as a verdict and wrote it into this document as a
> refutation.
>
> **A non-vacuity guard did not save it.** The first gate asserted the broadcast fd was armed
> (`>= 0`) — and that assertion PASSED, because the fd existed. It guarded the APPARATUS, not the
> CONDITION. The fd being open says nothing about whether the cascade fired. The working gate waits
> on the wire — `poll()` the broadcast fd until it is genuinely readable — before recv'ing.
>
> The correct trigger is production's: **write the wake pipe → the worker wakes → the worker writes
> the broadcast.** A gate that drives a mechanism differently from the way production drives it
> measures the gate.
>
> ### Kept for the record: three claims that were asserted, then withdrawn, then re-proven
>
> The analysis below stands. These lines are the intervening errors, kept visible rather than
> deleted, because a document that quietly flips twice teaches nothing:
>
> | claim I made mid-session | status now |
> |---|---|
> | "a delivered frame loses the tie" | **TRUE — proven RED.** The earlier "it PASSES, data wins" was the broken gate: no tie existed. |
> | "the stop discards in-flight work" | **TRUE — proven RED.** The earlier "all three drained" was the same broken gate. |
> | "the broadcast is STICKY so deferring costs nothing" | **STILL UNVERIFIED.** Do not lean on it; the fix does not need it. |
>
> **The lost-wakeup theory that stood here is WITHDRAWN.** It was built entirely on the broken
> gate's 30s hang, and that hang was the gate firing the thread-tier sever while a process-tier
> receiver waited on a broadcast nobody had written. There is no evidence of a lost wakeup.

---

# THE CONFIRMED DIAGNOSIS — a shutdown broadcast is a WAKE, never a preemption

> **Status: DRAWN 2026-08-05.** Task #79, root-caused. The builder: *"wat is strongly lock step —
> find the misstep."* Found. It is a tie-break, it is four lines at the primary site, and it is
> stated in the code as a deliberate choice.

## The misstep

`comms::process::wait_for_data_or_cascade` polls two arms — the data fd and the shutdown broadcast
fd — and collects both readiness flags correctly. Then:

```rust
// Broadcast wins ties — substrate is going down; honest reporting
if got_broadcast {
    Ok(PollOutcome::Shutdown)      // ← got_data was collected, and is DISCARDED
} else if got_data {
    Ok(PollOutcome::DataReady)
}
```

**When a frame is already in the pipe AND a stop has been requested, the frame is thrown away.**

It is not a slip. It is asserted four times in the same file: *"N+1 arms; broadcast wins ties"*
(`:26`), *"Broadcast wins ties (the process is going down…"* (`:59`), *"substrate-shutdown takes
precedence over pending data"* (`:1026`), and again in the multi-peer select (`:1692`).

## Why it is wrong — derived from the lockstep contract, not from taste

`ZERO-MUTEX.md` states the invariant: **"the 'lock' is the loop body itself; the 'release' is the
ack send."**

A frame sitting in the pipe means **the sender already completed its release.** It sent, it
unblocked, it moved on — and in lockstep it will never re-send, because from its side the
transaction is *done*. A consumer that discards that frame breaks a protocol the counterparty has
already finished. That is not "honest reporting" of a shutdown; it is a lost transfer reported as a
stop.

**And the asymmetry that makes the fix free: the broadcast is STICKY, the frame is not.** The
worker writes a wake byte (`POLLIN`) and then drops the write-end (`POLLHUP`); `POLL_ADD` is
re-armed on every call, so the broadcast stays readable forever after it fires. Deferring it by one
call costs nothing — the very next poll sees it again. A discarded frame, by contrast, is gone for
that consumer.

> **Data-wins is strictly safer than broadcast-wins.** It cannot hang (the broadcast fires again
> immediately), and it cannot lose (the frame is delivered). Broadcast-wins can only lose.

The file already contradicts itself on this: fifty lines above the tie-break, the broadcast arm's
own comment says *"broadcast means **WAKE** (POLLIN, a written byte) as well as SEVER"* — and then
the tie-break treats the wake as a sever. Arc 170 Phase 1 ruled exactly this distinction
(`b9f19ea5`: "the broadcast means WAKE, not SEVER"); the ruling never reached this line.

## ★ The correct form is already in-tree — twice. This is a consistency fix, not an invention.

| site | today | verdict |
|---|---|---|
| `kernel/peer.rs` **thread** `Peer::recv` | reads `output` FIRST; consults the crash channel **only on EOF** | ✅ **the exemplar** |
| `comms/process.rs:389` **Sender** | *"Writable wins ties"* — progress beats broadcast | ✅ already correct |
| `comms/process.rs` `wait_for_data_or_cascade` (recv) | broadcast wins | ❌ **invert** |
| `comms/process.rs:1692` multi-peer select | broadcast wins | ❌ **invert** |
| `comms/thread.rs` `Select::select` | *"Shutdown arm takes priority"* | ❌ **invert** |

The thread-tier `Peer::recv` doc explains why its ordering is load-bearing, and it is the same bug
we are fixing one tier over — a reader *"parked on a healthy peer during a stop was told its peer
had closed… it is what a months-long `sigterm` flake was made of."* **The process tier never got
that fix.** Its comment claims it *"mirrors `ProcessPeerBundle::recv`"*; the mirror is not a mirror.

### And `Select::select`'s inversion is already convicted, in writing

`freeze.rs:1392-1396` documents the consequence: a service's serve loop blocks in `select'`, the
shutdown arm *"returns `Shutdown` regardless of which user receivers are pending — so a severed
service wakes and exits **WITHOUT ever draining** the `Admin::Stop` sitting in its queue, and the
ask then blocks forever… **That turns a race into a deterministic hang.**"

That is this same law, violated at the thread tier, already understood — and worked around by
*ordering* (`trigger_shutdown()` deferred until after the ask) rather than by fixing the precedence.

## ★★ THE LAW, to be stated once in `ZERO-MUTEX.md` — builder-ruled 2026-08-05

> *"winner or loser — the result must be graceful."*

> **A shutdown broadcast is a WAKE, never a preemption — and NEITHER OUTCOME MAY LOSE WORK.**
>
> When a wait can report either *"a transfer has completed"* or *"the substrate is stopping"*:
> 1. **the completed transfer wins the tie** — it is delivered;
> 2. **the stop path DRAINS before it stops** — a shutdown consumes what has already been delivered
>    rather than discarding it;
> 3. **anything that still cannot be delivered is a NAMED outcome carrying what was lost** — never
>    silence.
>
> The broadcast is sticky and fires again on the next call; a delivered frame does not.

**Why (3) is not aspirational, and why (2) is always affordable: WE HAVE NO TIMEOUT.** Arc 170's
graceful-stop stone pinned it deliberately — *"a wedged stop must hang VISIBLY, naming the
service"*, and the deadline *"belongs to the supervisor, which already has one"* (systemd
`TimeoutStopSec`, k8s `terminationGracePeriodSeconds`). **The only thing that could ever justify
abandoning an already-delivered frame is a deadline, and we chose not to have one.** So there is no
case in this substrate where discarding in-flight work is the honest move — which means the drain is
always available, and a silent drop is never excusable.

**And a dropped frame is a HIDDEN FAILURE.** Today `Shutdown` is returned and the frame simply
ceases to exist — nobody is told a transfer was lost. That is precisely the class R53/R55/R57
annihilated everywhere else: a failure with no matchable value, reported as something it is not.
The receiver side never got the treatment.

### The symmetry this restores

The stop protocol on the SEND side already works this way: `Admin::Stop` → the handler finishes its
current op → replies → `Status::Stopped`. **A service stops at its own safe point, mid-transaction
never.** The receive side should stop at *its* safe point too — which is "nothing left in the
pipe" — rather than wherever the broadcast happened to land.

## The fix

**Primary site — swap two branches:**

```rust
// Data wins ties. The broadcast is a WAKE — it exists so a blocked reader cannot hang
// forever — not a preemption. A frame already in the pipe means the SENDER ALREADY
// COMPLETED ITS RELEASE (ZERO-MUTEX: "the release is the ack send"); it has unblocked and
// will never re-send, so discarding the frame breaks a transaction the counterparty
// considers finished. Deferring the stop costs nothing: the broadcast is sticky (wake byte,
// then POLLHUP on drop) and POLL_ADD re-arms every call, so the next recv — with no data
// pending — returns Shutdown. Mirrors the thread tier (kernel/peer.rs::Peer::recv), which
// reads output first and consults the crash channel only on EOF, and the Sender fifty lines
// up, which already lets "writable" win ties.
if got_data {
    Ok(PollOutcome::DataReady)
} else if got_broadcast {
    Ok(PollOutcome::Shutdown)
}
```

Then the same inversion at `process.rs:1692` and `thread.rs`'s `Select::select`.

## ⛔ The RED gate — deterministic, and it must go red first

Construct the tie **on purpose** rather than waiting for load to produce it:

1. child writes one frame and exits → data pending **and** pipe at EOF
2. fire the substrate shutdown → broadcast readable
3. parent `recv`s → **both arms ready, deterministically**

**Today:** `Shutdown` (the frame is lost). **After:** the frame, then `Shutdown` on the next call.

`test_run_string_entry_path` is the natural end-to-end acceptance test — and its own arm text is
the tell that convicted this: *"stopped before the child sent its value — **the child was ALIVE**"*
and *"child closed before sending its value"*. Both messages are **lies** in this scenario: the
child did send, and the frame was in the pipe when the parent was told otherwise.

## STOPs

- **⛔ Do not "fix" this by raising the deftest 5000ms watchdog.** That threshold has already been
  raised twice (200→1000→5000ms) to silence this class as "parallel-load false positives." Its own
  comment says *"a test taking 5s is genuinely stuck and worth investigating."* This is the third
  occurrence; treating the stem again is the failure `extirpare` names.
- **⛔ Do not change `Sender`'s tie-break** (`process.rs:389`). "Writable wins ties" is already the
  correct shape for the send side.
- **⛔ Do not remove `freeze.rs`'s ask-before-sever ordering as part of this.** It may become
  unnecessary once `Select::select` stops preempting — but that is a CONSEQUENCE to verify with a
  run, not a claim to bundle in. Removing a workaround on the strength of a fix's theory is how the
  original hang comes back.
- **⛔ Weigh the whole floor, not the one test.** The inversion is on every process-tier recv, so a
  behaviour change here touches every spawn/IPC test.

## Honest bound

This is grounded to `file:line` and derived from the stated invariant, and the arm text of the
failing test matches the predicted symptom exactly. It is **not yet proven** to be the cause of the
intermittent failure — the RED gate above is what would prove it, and it must be built and seen to
fail before the fix lands. Until then this is a real defect with a matching signature, not a
confirmed root cause.
