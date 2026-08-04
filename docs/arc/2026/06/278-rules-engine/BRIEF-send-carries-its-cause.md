# BRIEF — #70: `send` carries its cause

**Stone:** `DESIGN-STONE-send-carries-its-cause.md` — fully ruled.
**RED probe, committed and measured:** `probes/red-send-cause-is-not-matchable.wat`.

## The work, in one paragraph

`SendOutcome::Lost` and `TrySendOutcome::Lost` carry a flat `:wat::kernel::Failure` (a message
record). Their recv twin carries `:wat::kernel::LociDiedError` — a matchable enum that already
declares `Stopped` and `Disconnected`, the exact two states the send path currently collapses into
one hardcoded string. **Widen both carriers to `LociDiedError`, then replace every `Err(_)` in the
send path with a real match on the error**, mapping the shutdown-woke-a-blocked-write case to
`Stopped`, a genuine peer-loss to `Disconnected`, and carrying the real reason for anything else.

Builder's ruling, and it is the contract: *"the only acceptable opaque error is the '500' served to
clients, and the real cause is shipped to the admin handle."* There is no justification for a
discarded cause — only **redact outward, route the truth**. None of these sites is client-facing.

## Read in order

1. **`docs/arc/2026/06/278-rules-engine/probes/red-send-cause-is-not-matchable.wat`** — the RED
   probe. Measured today: `1 type-check error … expects LociDiedError; got Failure` at its send arm,
   while its **positive control** (the same variants against the recv outcome) type-checks clean.
   **This file is your gate.** It must go GREEN, and its control must stay green.
2. **`src/types.rs:1668`** — `SendOutcome::Lost`'s field, `TypeExpr::Path(":wat::kernel::Failure")`.
   **`src/types.rs:1705`** — `TrySendOutcome::Lost`'s, the same. These two lines are the carrier.
3. **`src/runtime.rs:23471`** `send_outcome_lost(reason: String)` and **`:23520`**
   `try_send_outcome_lost` — the constructors. They take a `String` today; they must take (or
   build) a `LociDiedError`.
4. **`src/runtime.rs:26108`** — a worked example of the shape you are replacing, and the smallest:

   ```rust
   Some(peer) => match peer.send(payload_val) {
       Ok(())  => send_outcome_sent(),
       Err(_)  => send_outcome_lost("send: peer disconnected".into()),   // <- the drop
   },
   ```
5. **`src/runtime.rs:26480`** — how **recv** already does it right
   (`PeerRecvError::Shutdown => recv_outcome_shutdown()` = `Lost[LociDiedError::Stopped]`).
   Mirror this; do not invent a second style.
6. **`src/io.rs:713`** — where `RuntimeErrorKind::WriteStopped` is produced. This is the named stop
   arc-170 closure #5 built, and the thing currently indistinguishable from a disconnect.

## Sketch

```rust
Err(e) => send_outcome_lost(match e {
    /* the shutdown broadcast woke a blocked write */ => loci_died_stopped(),
    /* the peer is genuinely gone                   */ => loci_died_disconnected(),
    other                                             => /* carry `other`'s real reason */,
}),
```

**Read each site's actual error type before mapping it** — the sketch names the shape, not the
variants you will find. `LociDiedError` already declares `Panic`, `RuntimeError`, `Disconnected`,
`Stopped`, `StartupError`, and more; pick the honest one per site.

## Grounded for you, so you do not have to re-derive it

- **The purity swap is legal.** `SendOutcome`/`TrySendOutcome` are declared `Purity::Pure`, and their
  Pure-ness rests on the carrier being pure (`types.rs:1652`). `LociDiedError` is
  `Purity::Pure` — *"a death report — Pure (crosses back to the owner as EDN data)."* Checked. No
  containment violation, and both enums stay Pure.
- **The wat-side cascade is likely near-zero.** All six live `SendOutcome::Lost` consumers bind the
  cause as `_c` and discard it (`wat/test.wat:392`, `wat/bracket.wat:47/85/139/411/537`) — `_c`
  binds anything, so a widened carrier should not disturb them. Verify; do not assume.

## Blast radius

`src/types.rs`, `src/runtime.rs`, and whatever the compiler names. **Do not** touch
`wat/service.wat`, the recv path, or the outcome walls' shapes.

## ⛔ STOPs — ship nothing, surface the gap

- **⛔ STOP-1 — DO NOT thread the error into the existing `String`.** Widening the carrier is the
  strike. A real reason inside a flat message still leaves the caller reading prose instead of
  matching a variant, which is the shape the substrate forbids. If the carrier cannot be widened,
  STOP and report why — do not deliver the string version as a partial win.
- **⛔ STOP-2 — DO NOT invent a variant.** `Stopped` and `Disconnected` already exist on
  `LociDiedError` and are already consumed by recv. If you believe a site needs a variant that does
  not exist, STOP and name the site.
- **⛔ STOP-3 — DO NOT annotate a drop.** There is no justification rune for a discarded cause; the
  ruling admits only carry-or-route. An `Err(_)` you cannot map is a STOP, not a comment.
- **⛔ STOP-4 — the population is UNMEASURED and you must not trust a grep for it.** `grep 'Err(_) *=>'`
  returns 5 and **cannot reach** the try-send pair (its arm is formatted across lines). At least 7
  are known; 89 bare `Err(_)` exist in `src/`. **Change the carrier first and let the compiler
  enumerate every site** — that list is the worklist. Report the number the compiler gave you, and
  say plainly that it is the compiler's and not a grep's.
- **⛔ STOP-5 — `channel/transfer.rs:76` (`Err(_) => SendOutcome::Disconnected`) is a DIFFERENT enum**
  — the Rust-internal `channel::SendOutcome`, not the wat-surface one. Report whether it is in scope;
  do not silently fold it in.

## ★ THE DELIBERATE BREAK — the row that matters

After it is green, make the shutdown case map to `Disconnected` instead of `Stopped` (a one-word
lie), rebuild, and run the RED probe's send arm. Confirm it can no longer tell the two apart —
i.e. that the probe is actually discriminating and not merely compiling. Restore byte-exact,
confirm green. **Report both, with the actual output.**

## Done means

- `probes/red-send-cause-is-not-matchable.wat` type-checks clean, control included.
- Every send-path `Err(_)` either maps to an honest variant or is a reported STOP.
- The break went RED and the restore went green — both reported with real output.
- `cargo nextest run --release` **Summary line verbatim**, against the floor `4343/4343/0/262`,
  with the count arithmetic explained.
- `cargo clippy --release --all-targets` clean.

You are a rider, not the orchestrator. **Ending your turn ENDS you** — nothing will wake you and no
notification is coming. Run every verification in the FOREGROUND and block on it: your turn ends
when the numbers are in your hands, not when the command is launched. Do not commit, do not push,
do not stash.
