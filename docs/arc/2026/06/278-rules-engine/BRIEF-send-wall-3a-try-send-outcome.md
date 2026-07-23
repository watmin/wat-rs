# BRIEF — send' wall Phase 3a: `try-send'` gets its own `TrySendOutcome` (unblock the do-gate)

> **Tier:** sonnet shadowdancer. **Arc:** 278 send'-wall Phase 3 (see `DESIGN-send-outcome-wall.md`). The
> Phase-3 must-use do-gate is implemented in the tree (uncommitted) but the floor is RED for ONE grounded
> reason: `try-send'` reuses the checker's `infer_send_prime` (→ typed `SendOutcome`, now must-use), but its
> eval was never converted (still returns `nil`) and `service.wat:1167` never faces it. This strike gives
> `try-send'` its own honest outcome type and faces the one site — greening the do-gate.

## Why + the four-questions verdict (already reasoned — build A2)

`try-send'` is NON-BLOCKING, so it has an outcome `send'` cannot: **would-block** (a client not draining its
side; `service.wat:1163` deadlock guard). The four-questions ruled: add `WouldBlock` to `SendOutcome` FAILS
(re-breaks all 183 `send'` matches; a variant `send'` never returns — fails Obvious/Simple/Honest); map to
`Lost` FAILS Honest ("alive but not draining" is not "gone"). **Winner: `try-send'` gets its own
`:wat::kernel::TrySendOutcome{Sent, WouldBlock, Closed, Lost}`** — the type says exactly what the verb returns.

## The build

**1. Register `:wat::kernel::TrySendOutcome`** (`src/types.rs`, next to `SendOutcome`), **PURE**
(non-parametric, pure data — same reasoning as `SendOutcome`):
```
:wat::kernel::TrySendOutcome  (Purity::Pure)
  :Sent      []
  :WouldBlock[]                         ;; channel full / receiver not draining (crossbeam Full) — try-send' ONLY
  :Closed    []                         ;; peer already cleanly closed (cell None)
  :Lost      [cause <- :wat::kernel::Failure]  ;; receiver dropped mid-send (crossbeam Disconnected)
```

**2. Enrich the peer try-send to distinguish Full vs Disconnected** (`src/kernel/peer.rs:303` `try_send`,
`:315` `try_send_wire`). They return bare `bool` today — collapsing crossbeam's `TrySendError::Full` vs
`::Disconnected`. Change them to surface the distinction (e.g. return a small enum
`TrySendResult::{Sent, Full, Disconnected}` or `Result<(), TrySendErr>`). Crossbeam's `try_send` already
returns `Err(TrySendError::Full(_))` / `Err(TrySendError::Disconnected(_))` — thread the distinction up.
**STOP-1 if `try_send`'s callers beyond the eval make this cascade widely** — report the caller set; a
`bool`→enum change should be small (grep `try_send(` / `try_send_wire(`).

**3. Convert `eval_peer_try_send_prime`** (`src/runtime.rs:26017`) to RETURN `TrySendOutcome`:
- cell `None` (already closed) → `Closed`
- `try_send`/`try_send_wire` → `Sent` (ok) / `WouldBlock` (Full) / `Lost[message-only-failure("try-send': peer disconnected")]` (Disconnected)
No more `Ok(Value::Unit)`.

**4. Give `try-send'` its own checker infer** — stop reusing `infer_send_prime` at `src/check.rs:5112`.
Add `infer_try_send_prime` (mirror `infer_send_prime` but return `:wat::kernel::TrySendOutcome`).

**5. Add `TrySendOutcome` to the must-use set** — `MUST_USE_TYPES` in `src/check.rs` (so an unfaced
`try-send'` is also a compile error, same as `send'`).

**6. Face `wat/service.wat:1167`** — the `ServiceEvent::Rejected` arm's `try-send'`. All outcomes → the same
action (evict + keep serving — the reply is best-effort, the client is evicted regardless):
```clojure
(:wat::core::do
  (:wat::core::match (:wat::kernel::try-send' (:wat::core::nth selectables idx) (~reply-failed-kw cause))
    (:wat::kernel::TrySendOutcome::Sent       nil)
    (:wat::kernel::TrySendOutcome::WouldBlock nil)   ;; client not draining — evict anyway (it learns via EPIPE)
    (:wat::kernel::TrySendOutcome::Closed     nil)
    ((:wat::kernel::TrySendOutcome::Lost _c)  nil))
  (~serve-name self l (:wat::std::list::remove-at selectables idx) state))
```

## STOP triggers

- **STOP-0:** after this, the floor is NOT 0-failed — report which tests + why (a real swallow elsewhere, or
  a `try_send` enrichment ripple). Do NOT mass-edit.
- **STOP-1:** the `peer.try_send` bool→enum change cascades to many callers — report the caller set; don't
  refactor them all blindly.
- **STOP-2:** do NOT touch the `let [_ …]` gate or the 19-file let-sweep — that's Strike 3b. Scope is
  `try-send'` + `service.wat:1167` + the do-gate's must-use set.

## Verify (weigh by your own re-run)

1. `cargo build --release` compiles.
2. `./target/release/wat --check wat/service.wat` clean.
3. **Whole floor: `cargo nextest run --release`** — READ the Summary yourself, in the FOREGROUND of your turn
   (do NOT background it and end your turn). Target: **0 failed** (the do-gate now green — `send'` AND
   `try-send'` both walled, the `service.wat:1167` swallow faced). Report the Summary.
4. Confirm the Phase-3 RED probe (`probe_arc278_send_outcome_must_use_wall`) still passes.

## Deliverable

`TrySendOutcome` + the peer enrichment + the eval + `infer_try_send_prime` + must-use + the faced site.
Report: (1) the type + the four changes' final form; (2) the peer `try_send` caller set (STOP-1 check);
(3) the floor Summary (0 failed) read by you; (4) `git diff --stat`. Do NOT commit (wall lands after 3b).

## Blast radius

`src/types.rs`, `src/kernel/peer.rs` (try_send/try_send_wire + their callers if trivial), `src/runtime.rs`
(the try-send' eval), `src/check.rs` (infer_try_send_prime + MUST_USE_TYPES), `wat/service.wat:1167`. NO
let-gate, NO 19-file sweep (3b). Scratch logs → `/tmp/claude-scout/`.
