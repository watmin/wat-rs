# An oversized reply is not a death

**The one measured DoS that survives the fault-domain partition.** Excursus `001-sns-sqs`.

- `../the-bracket-peer-path-faces-chaos/SCORE.md` — where it was measured
- `../giving-up-does-not-give-up/FINDING.md` — the partition: shared memory / not. This stone is
  about the **IPC** side, where a worker really is another fault domain.

## The sentence

> **A worker that sends one frame too big must not kill its coordinator.**

Measured, reproduced twice:

```
bracket collect-loop: REPORT-GONE last runner 0 crashed holding item 0:
  frame exceeded cap (message larger than the receiver's max-message-bytes budget)
```

## ⛔ "Add caps" was the wrong diagnosis. The cap exists and fires.

`ProcessOpts.max-message-bytes` defaults to `DEFAULT-MAX-MESSAGE-BYTES` = 524288 (`spawn.wat:87`),
and the oversize probe trips it. The transport is doing its job. **The bug is the disposition:**

```rust
src/kernel/spawn.rs:282
    crate::comms::RecvError::FrameTooLarge => PeerDeath::Lost(output_err.to_string()),
```

⭐ **This is the SAME defect shape as the decode collapse, one variant over.** `one-selectable-set-
primitive` fixed *decode failure → `Lost`*; this is *frame-too-large → `Lost`*. Both classify **a
peer that is alive and well** as **dead**. And the unified engine already holds the correct answer
for this exact condition — `Rejected`, documented as *"a 400-class CLIENT error, NOT a 500-class
internal crash"*.

⚠ And it is a **trust** statement, not just a classification one: a service treats its clients as
untrusted and answers `Rejected`; a bracket decodes through `decode_trusted_wire` and dies. Once
the fleet is remote, the trusted premise is false.

## ⭐ FrameTooLarge is the cleanest REPORT-FINAL in the taxonomy — and that is the problem

The governing taxonomy could not separate `Malformed`'s two facts (wire damage vs sender garbage),
which is why that arm became RETRY with the wall-clock bound as its stop. **`FrameTooLarge` has no
such ambiguity:** the value *is* too big. It is a property of the payload, not the transmission, so
retrying or re-dispatching reproduces it exactly. It is REPORT-FINAL by construction.

⛔ **But REPORT-FINAL at a bracket means raise, because `(map …)` returns `(Vector :- [O])` and has
no slot for a per-item failure.** So the honest disposition and the survivable one disagree, and
this stone must not pretend otherwise.

## The work

### 1. Reclassify, at the substrate

`FrameTooLarge` stops meaning death on the spawn-process branch. The peer is alive: it sent one
frame that was refused. ⚠ **Check every consumer of that classification before changing it** —
`classify_peer_error` is not bracket-only, and a service or script that today sees `Lost` will see
something else.

### 2. Give bracket a disposition that keeps the coordinator alive

⛔ **The coordinator must not die.** That is the DoS and it is the whole point. Options, in order of
honesty:

| | |
|---|---|
| **(a) RETRY, bounded** | re-dispatch; deterministic, so it reproduces until `collect-deadline-ms`. Survivable, wasteful, consistent with the `Malformed` arm |
| **(b) drop the item, continue, report at the end** | ⭐ correct — but needs a per-item slot the surface does not have |
| **(c) raise** | today's behaviour; the DoS |

**Land (a) now** so a worker cannot kill a coordinator, and **name (b) as the builder's call**,
because it changes `map`'s return type and therefore every caller.

### 3. ⛔ THE SURFACE QUESTION, SURFACED NOT SMUGGLED

This is the **fourth** stone to hit `(Vector :- [O])` having no room for a per-item failure —
after the queue path, the `Malformed` arm, and `collect-gave-up!`'s *"per-item causes are not
carried — the surface has no slot"*. ⚠ **Do not widen `map`'s return type in this stone.** Report
that the wall has now been hit four times and let the builder rule.

## Scope wall

⛔ Not here: the thread tier (**not a fault domain** — see the corrected FINDING); suppression;
the lineage-`Admin` delay; queue knobs; and **do not** widen `map`/`each`.

## The four questions

- **Obvious** — ✅ a peer that is alive must not be reported dead; it is the second instance of a
  defect we already fixed once.
- **Simple** — ⚠ the reclassification is one arm; its **blast radius is not** — `classify_peer_error`
  has other consumers. Row 1 is the risk.
- **Honest** — ✅ it states that the correct disposition (b) is blocked by the surface, and lands
  the survivable one instead of pretending they are the same.
- **Good UX** — ✅ a worker's bad frame costs an item, not the run.
