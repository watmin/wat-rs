# DESIGN-STONE — a stop is not a death, and not a clean close

> **Status: RULED 2026-08-04, unbuilt.** Board: **#73**. Ordered after **#72**, before **#71**.
>
> The builder, on being shown the gap and then watching the orchestrator shelve it for want of a
> caller: *"i continue to fail to see how this is acceptable — you are actively expressing our
> language is dishonest and then casually walk over it."*

## The lie

```
:wat::kernel::RecvOutcome<O>  =  Message [msg <- O] | Closed [] | Lost [cause <- LociDiedError]
:wat::kernel::SendOutcome     =  Sent []            | Closed [] | Lost [cause <- LociDiedError]
```

Neither has a `Stopped`. So when the substrate is told to stop, a blocked `recv` or `send` must
report one of two things, and **both are false**:

| reported | what it means | the truth |
|---|---|---|
| `Closed` | your peer hung up | the peer is **alive** and the channel is **open** |
| `Lost[cause]` | your peer **died** | nothing died — the carrier is literally named `LociDiedError` |

`src/kernel/peer.rs:118` says it in its own doc comment, in the same file that fixed half of this:

> `Err(Shutdown)` → `Err(PeerRecvError::Shutdown)` — *a stop was requested; **the peer is ALIVE and
> the channel is open**.*

## Rust carries the distinction four layers deep. Wat throws it away.

```
comms::RecvError::Shutdown                  (comms/mod.rs — built expressly to distinguish it)
comms::process::PollOutcome::Shutdown       (process.rs:738, :926)
kernel::spawn::PeerRecvError::Shutdown      (peer.rs:145 — the THREAD tier passes it through)
channel::RecvOutcome::Shutdown              (transfer.rs:29)
        |
        v
  :wat::kernel::RecvOutcome<O>              ← nowhere to put it. It becomes a lie.
```

**And one tier does not even get that far.** `classify_peer_error` (`src/kernel/spawn.rs:258`):

```rust
_ => match err.recv() {
    Ok(reason) => PeerDeath::Lost(reason),
    Err(_)     => PeerDeath::Closed,
}
```

`PeerDeath` (`:197`) is `Lost(String) | Closed`. `RecvError::Shutdown` falls into that `_` and comes
out as a clean EOF. **This is the identical wildcard `peer.rs:145` already fixed on the thread
tier** — whose own comment names the months-long `sigterm` flake the old wildcard caused. One tier
walled, one skipped.

## The shape is not a decision — it is settled four times over

`Stopped` is already `EnumVariant::Unit` at `types.rs:1110`, `:1152`, `:1194`, `:1335`. It carries
nothing, and it should not: a stop has no cause to report. The substrate was asked to stop. That is
the whole fact.

**And the in-family mirror is exact.** `ReadlnOutcome` (`types.rs:1143`) is `Datum [v] | Eof |
Stopped` — shipped at 170 closure #2 (#24).

```
ReadlnOutcome     Datum [v]      Eof        Stopped
RecvOutcome       Message [msg]  Closed     ————     <- no twin
SendOutcome       Sent           Closed     ————     <- no twin
```

`readln` got its `Stopped`. `recv` and `send` did not. Run the mirror on any pair and the missing
row is the finding — that is the instrument, and it is the third time this arc it has paid.

## The four questions

| | |
|---|---|
| **Obvious?** | **NO** — a caller told `Closed` has no way to learn the peer is alive. |
| **Simple?** | **NO** — one carrier is doing two jobs, and `LociDiedError` is doing a third by carrying a non-death. |
| **Honest?** | **NO.** This is the arc's own law (R55 `REVOLVTIONE NVLLA LARVA`, R57 `IGNORANTIAM DELEMVS`) violated in the type registry. |
| **Good UX?** | **NO** — the user cannot write the match that would tell them the truth. |

## ⛔ THIS IS NOT GATED ON A CONSUMER

The orchestrator found this, named it, and shelved it as *"a real honesty gap with no current
consumer."* That applies a demand test to a lie. **The no-hidden-failures law has no demand clause**
— a mask is not permitted to remain because nobody has tripped over it yet, and *nobody has tripped
over it* is exactly what a mask produces. Recorded here because it was the failure, not a
hypothetical.

## The strike

### 1 — `PeerDeath` gains `Shutdown` (prerequisite; small; mirrors an existing fix)

`classify_peer_error` stops flattening `RecvError::Shutdown`. Without this the process tier cannot
*produce* the fact, so nothing downstream can carry it. This is the `peer.rs:145` fix ported.

### 2 — `RecvOutcome` and `SendOutcome` each gain `Stopped` — ONE pass, both

Builder-ruled. The send side has the identical hole (`Sent | Closed | Lost`), one codemod is cheaper
than two, and **a half-fixed pair is precisely how this arc got here** — the send side went unwalled
for months after recv was done (R57). Do not split them.

`RecvOutcome` stays `Impure` (its `O` may be a live resource); `SendOutcome` stays `Pure`. The new
variant changes neither.

### 3 — The corpus: ~420 arms, and the checker enumerates them for us

**MEASURED 2026-08-04:** 234 files match over `RecvOutcome`; occurrences `Closed` 418 · `Lost` 465 ·
`Message` 430. Full-enum-match is doctrine — there is **no `_` wildcard** to absorb a new variant —
so every site goes non-exhaustive the moment the variant lands.

**That is the method, not the problem.** R52 `QVOD LEX ACCENDIT` / the 24r symbol-deletion crusade:
arm the law, let the checker scream, the screams are the coordinates. No grep, no hand-built caller
map.

## ★★ THE HARD PART — what each arm DOES, and why a shared body is forbidden

A codemod can insert an arm. It cannot decide behaviour, and **`Stopped` must not simply do what
`Closed` does**, or the variant is `[[feedback_a_match_with_identical_arms_is_a_discard]]` at 420×
scale — the exact defect that deadlocked a rider yesterday, minted fresh and industrialised.

Builder-ruled: **classify by role, default per bucket, hand-check the boundaries.**

| bucket | `Closed` means | `Stopped` must mean | why they differ |
|---|---|---|---|
| **serve loops** | a client left → **keep serving** | the world is ending → **return** | this is where the variant earns itself; sharing a body here is a live bug |
| **client call sites** (generated methods, brackets, stdlib callers) | the service is gone → surface it | terminal for this call → surface it, **named as a stop** | today these get `Lost[Stopped]` and report a death |
| **drain loops** (`recv-all`) | clean end → `Ok(collected)` | **incomplete** drain → not `Ok` | a drain cut short by a stop did not finish, and must not claim it did |
| **test assertions** | assert `Closed` | assert `Stopped` | the specimen is the point |

**Where a bucket genuinely wants the same body as `Closed`, the site must say so in a comment,
naming the precondition.** That is the standing rule from yesterday's deadlock: a uniform match is
legal, an *unexplained* uniform match is a discard, and the comment is what lets the next person see
when the reason expires.

## The execution shape — map-reduce, one golden exemplar per bucket

Per `[[feedback_map_reduce_crusade_golden_exemplar]]` and `[[feedback_prove_one_exemplar_then_arm_riders]]`:
the orchestrator proves **one exemplar per bucket by its own hand**, then fans edit-only riders
against it, then runs the single reduce. Four buckets, four exemplars. Riders do not run the floor;
the orchestrator weighs centrally once (FM 18).

## STOPs

- **⛔ No `_` wildcard arm on an enum scrutinee.** Doctrine (`109/NOTE-full-enum-match-mandatory-no-wildcard-arm.md`). The checker rule is unbuilt, so nothing will stop a rider taking it. Taking it is a rejected strike.
- **⛔ `Stopped` carries no cause.** Four unit-variant precedents. A cause would be inventing a reason for "you asked me to stop."
- **⛔ No hand-edits to `.wat`.** wat-fix codemod, dry-run on a `/tmp` copy + `diff`, idempotent, committed as the recorded migration.
- **⛔ Do not let a codemod author the arm BODY.** It inserts nothing; the checker enumerates, riders fill per bucket. A default body sprayed over 420 sites is 420 unexamined decisions wearing the shape of work.
- **⛔ Do not gate any part of this on finding a caller who needs it.** See above; that framing shelved it once.
- **⛔ Do not re-cast the name.** `Stopped` is established at four sites and was already ruled once (170 closure #3, `Shutdown` → `Stopped`, #25).

## What it unblocks — a consequence, not the justification

A wat test can assert *"a stop woke my blocked recv"* without reaching below the peer surface into
Rust. The two shutdown-cascade tests deleted in `85b789ac` existed **only** because that assertion
was inexpressible in wat; they poked `ReceiverInner` and `typed_recv` directly and asserted an
internal variant no wat program can observe. With `Stopped` at the surface, that coverage is
expressible in the spawn tooling like everything else.

It is worth saying which way the causality runs: the missing variant is why those tests were written
in Rust, why they reached for a process-wide signal, and why they resisted migration for a day. The
variant is the root; the tests were the stem.
