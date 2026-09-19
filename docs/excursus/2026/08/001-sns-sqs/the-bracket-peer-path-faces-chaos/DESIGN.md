# The bracket peer path faces chaos

**The reach gap.** Excursus `001-sns-sqs`. Read first:

- `../FINDING-we-cannot-induce-a-slow-peer.md` — the three-primitive inventory
- `../a-peer-that-is-merely-slow/SCORE.md` — the latency knob, landed `424498eb6`. It is
  **store-scoped**; this stone is about where the knobs can *point*.
- `../a-dead-runner-loses-one-item-not-the-run/SCORE.md` — the re-dispatch this must fault-test

## The sentence

> **A bracket runner must be able to die, stall, and answer badly — on purpose.**

Today it can only **die**.

## ⛔ MEASURE FIRST. The work-fn may already be an injector.

`:wat::bracket::runner-loop` (`wat/bracket.wat:32`) is:

```wat
(:wat::core::match (:wat::kernel::send self (work-fn item)) …)
```

⭐ **The work-fn is user-supplied and its result is always sent.** So a chaos work-fn already
controls the **value** and the **timing** of every reply, and can **die** (the dead-runner probe
does exactly this). What it cannot do is **not send** — the send is unconditional.

⛔ **So row 1 is a measurement, not a build.** Adding a knob for something a work-fn already
reaches is the waste this stone exists to avoid, and this excursus has twice spent a strike on
code that turned out to be unreachable or already covered.

| primitive | plausible route from a work-fn | verdict |
|---|---|---|
| **die** | panic | ✅ already used — `probe_dead_runner_loses_one_item` |
| **slow** | wait on `(:wat::kernel::after …)` before returning | **measure** |
| **oversize** | return an `O` above `DEFAULT-MAX-MESSAGE-BYTES` (524288, `spawn.wat:87`) | **measure** — and see the warning below |
| **suppress** | none — `send` is unconditional | needs a bracket-side knob |

## ⛔⛔ And a question about MY OWN commit, which this stone must answer

`c4026f99e` made `ServiceEvent::Malformed` re-dispatch instead of killing the run. **That arm may be
unreachable in practice.** `Malformed` comes from `decode_trusted_wire` failing — and a
**type-correct work-fn cannot produce an undecodable frame**: the value it returns is an `O`, which
encodes and decodes by construction. Genuine wire corruption is not injectable by anything in the
tree (the FINDING lists byte corruption as absent).

⚠ **Oversize is NOT the same fault.** Over `DEFAULT-MAX-MESSAGE-BYTES` on the process-spawn branch
is `RecvError::FrameTooLarge`, which that branch maps through `classify_peer_error` — **not** to
`Malformed`. Do not conflate them; report which `ServiceEvent` an oversized reply actually produces.

⛔ **If `Malformed` proves unreachable on the bracket peer path, say so.** My fix then stands as
correct-but-latent, guarded by a source pin, and that is the honest state — not a failure of the
fix, and not something to paper over by inventing a corruption injector to justify it.

## The work

### 1. The measurement — what a chaos work-fn already reaches

Three probes, one per row of the table. For each: which `ServiceEvent` arrives at `collect-loop`,
and what `collect-loop` does with it. ⛔ **Report the arm, not the outcome** — "the run survived" is
not the same fact as "it survived via re-dispatch".

### 2. ⭐ The bound that has never fired

`collect-deadline-ms` is **300000** (`bracket.wat`). The wall-clock bound the re-dispatch stone
called *"the only thing between a deterministic encode bug and an infinite re-dispatch loop"* has
**never been exercised** — nothing could make a runner slow. A slow work-fn is exactly the fault
that reaches it.

⚠ 300 s is unaffordable in a floor, so the deadline must be **injectable**. ⭐ The precedent is in
this same excursus: `the-handshake-deadline-is-injectable` did this for the startup handshake, via
an env-read intrinsic with one home for the value. **Copy that shape**; do not add a second
constant, and do not lower the production default.

### 3. Only then, the missing knob

Suppression, if rows 1–2 show it is genuinely unreachable. ⛔ **Build nothing that row 1 shows a
work-fn already does.**

### 4. Fault-test the re-dispatch, not just the arm

The dead-runner control kills a runner. Now make one **stall past the bound** and show
`collect-gave-up!` fires with its wall-clock reason — the report that has never been produced by a
real fault.

## Scope wall

⛔ The thread tier as a chaos target (*"a thread peer cannot have latency"* — its own question).
The lineage-`Admin` delay (the FINDING's actual hazard, still unreachable). Backpressure, a healing
partition, half-close, byte corruption. `wat/queue.wat`'s knobs stay where they are.

## The four questions

- **Obvious** — ✅ brackets target networked hosts and can currently only be faulted by killing a
  worker.
- **Simple** — ⚠ unknown until row 1. If the work-fn is the injector, this is three probes and a
  deadline knob. If not, it is surgery on the runner loop. **Row 1 decides, and it is cheap.**
- **Honest** — ✅ it asks whether my own `Malformed` fix is reachable and permits the answer "no".
- **Good UX** — ✅ no surface change; chaos arrives as an ordinary work-fn.
