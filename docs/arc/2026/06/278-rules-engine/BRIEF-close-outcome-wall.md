# BRIEF — the `close'` OUTCOME WALL (peer-lifecycle Strike 2)

> **The work in one paragraph.** `close'` (`:wat::kernel::close'`, a `#[restricted_to(":wat::kernel::")]`
> kernel intrinsic — user-facing close' was retired arc-259 S2d, teardown is RAII Drop) currently RAISES
> on its *handleable* failures (thread-join-panic, process-signaled, process-wait-fail, process-stopped).
> Per the builder's peer-lifecycle LAW (2026-07-23) — *"we deliver an enum for code to handle exceptions
> with; raise is uncatchable on purpose, a thing that must never happen"* — every handleable failure
> becomes a matchable `CloseOutcome` variant; only the must-never-happen raises (double-close, close'-on-a-
> timer, arity/type) stay raises. This is the send-side/recv-side wall pattern (R53/R57) applied to close',
> RIGHT-SIZED: **0 wat call sites → NO corpus sweep**; the strike is the eval→`CloseOutcome` conversion +
> the type registration + the must-use gate + a positive Rust probe. Shape ruled (four-questions B, prior
> self + concurred): loci-agnostic, the exit code rides in an `Option`.

## Read in order (the rooms)
1. `src/types.rs:1210-1225` — the `SendOutcome` `register_builtin(TypeDef::Enum(EnumDef{…}))` — **the exact
   registration pattern to mirror** (Pure, non-parametric).
2. `src/runtime.rs:26491-26625` — `eval_peer_close_prime` — **the eval to convert** (the disposition table below).
3. `src/check.rs:12152-12180` — `infer_close_prime` (INTRINSIC) — currently returns nil/i64; make it return
   `CloseOutcome` (a `TypeExpr::Path(":wat::kernel::CloseOutcome")`).
4. `src/check.rs` — `const MUST_USE_TYPES` (the `[":wat::kernel::SendOutcome", ":wat::kernel::TrySendOutcome"]`
   array) — **add `":wat::kernel::CloseOutcome"`** (non-parametric → this list, NOT `MUST_USE_PARAMETRIC_HEADS`).
5. `tests/kernel/peer_select_prime_process.rs` (+ `tests/comms/probe_arc214_stone46aii_peer_verbs.rs`) — how
   close' is positively exercised today (Rust, spawns a peer + close's it) — **model the positive probe on these.**
6. `tests/kernel/probe_arc259_s2d_internal_only_close.wat.bad` — the NEGATIVE restriction fixture — LEAVE IT
   (it proves a `:user::` caller is a check error; still true).

## The type (shape B — RULED, do not re-fork)
```rust
// src/types.rs — mirror SendOutcome (:1210); Pure, non-parametric.
env.register_builtin(TypeDef::Enum(EnumDef {
    name: ":wat::kernel::CloseOutcome".into(),
    type_params: vec![],
    purity: Purity::Pure,          // no live resource — the peer is CONSUMED; carries only i64 + a pure Failure record
    variants: vec![
        // clean close — None = thread (no exit code), Some(code) = process exit status (loci-agnostic, R32)
        EnumVariant::Tagged { name: "Closed".into(), fields: vec![(
            "exit".into(),
            TypeExpr::Parametric { head: "wat::core::Option".into(),
                                   args: vec![TypeExpr::Path(":wat::core::i64".into())] },
        )] },
        EnumVariant::Tagged { name: "Signaled".into(), fields: vec![(
            "signal".into(), TypeExpr::Path(":wat::core::i64".into()),
        )] },
        EnumVariant::Tagged { name: "Failed".into(), fields: vec![(
            "cause".into(), TypeExpr::Path(":wat::kernel::Failure".into()),
        )] },
    ],
}));
```

## The eval disposition (`eval_peer_close_prime`, runtime.rs:26491-26625) — return a Value, not a raise
Each arm currently returns `Ok(Value::…)` or `Err(EvalBreak::from(RuntimeError{…MalformedForm…}))`. Convert to
return the CloseOutcome enum VALUE (construct the variant as a `Value` — see how `eval_peer_send_prime`
constructs `SendOutcome::Sent`/`Lost` at `runtime.rs:25823+` for the exact enum-value construction idiom).

| site (line) | current | → CloseOutcome |
|---|---|---|
| thread success `Ok(Value::Unit)` (~26550) | value | `Closed[exit = None]` |
| process `Exited(code)` `Ok(Value::i64(code))` (~26587) | value | `Closed[exit = Some(code)]` |
| `Thread peer join failed` raise (~26542) | raise | `Failed[cause = message-only-failure("Thread peer join failed (thread panicked)")]` |
| `Process peer wait failed` raise (~26577) | raise | `Failed[cause = message-only-failure("Process peer wait failed: {io_err}")]` |
| `Signaled(sig)` raise (~26588) | raise | `Signaled[signal = sig as i64]` |
| `Stopped(sig)` raise (~26595) | raise | `Failed[cause = message-only-failure("Process peer stopped by signal {sig}")]` — a stopped-not-terminated child during teardown is an abnormal close, NOT a kill; `Signaled` means *terminated by a signal* (four-Q Honest). **Pin this in a comment.** |
| `peer already closed` raise (×2, ~26535/26570) | raise | **STAYS a raise** (double-close = must-never-happen bug) |
| `close' on a timer peer` raise (~26606) | raise | **STAYS a raise** (arc-292 L3 = must-never-happen bug) |
| arity mismatch (~26508) / type-mismatch non-peer (~26615) | raise | **STAYS a raise** (checker-prevented; defensive) |

Use the canonical `message-only-failure` constructor (the same one `send'`/`recv'` `Lost` use — grep
`message-only-failure` / how `eval_peer_send_prime` builds its `Lost` cause `Failure` record; do NOT hand-roll
a `struct-new` Failure — R57's Struct-Failure mask, `3c72ef9c`).

## The positive probe (RED-first: prove close' returns the VALUE, not a raise)
`close'` is `restricted_to :wat::kernel::` → a wat probe would need a kernel-namespace caller. The established
positive path is a **RUST** probe (see `peer_select_prime_process.rs`). Write
`tests/kernel/probe_arc278_close_outcome_wall.rs`:
- spawn a **process** peer that exits 0 → `close'` → assert `CloseOutcome::Closed[Some(0)]` (structural EDN
  assert on the returned Value — `assert_edn_eq!` / field-extraction, NEVER a loose `format!("{:?}").contains`).
- spawn a **thread** peer, clean → `close'` → assert `CloseOutcome::Closed[None]`.
- (if cheaply reachable) a process peer that a signal kills → `Signaled[signal]`; a worker that panics on
  join → `Failed[cause]`. If not cheaply reachable, assert them via a focused unit on the eval mapping and
  say so — do NOT fake a hard-to-reach path.
The probe must FAIL RED before the eval change (close' still raises / returns Unit-or-i64) and pass GREEN after.

## Blast radius
- `src/types.rs` (+1 enum), `src/runtime.rs` (`eval_peer_close_prime` only), `src/check.rs` (`infer_close_prime`
  return + `MUST_USE_TYPES` +1 entry), `tests/kernel/probe_arc278_close_outcome_wall.rs` (new).
- **NO `.wat` corpus sweep** (0 wat call sites — the must-use gate registration is a 0-site pre-arm for a
  future teardown caller; that is correct, not vacuous-dishonest).
- Do NOT touch the `.bad` restriction fixture. Do NOT touch `send'`/`recv'`/`poll'` walls (banked).

## STOP triggers
- **STOP-1:** if a handleable close' raise you're converting turns out to be reachable from a live **wat**
  caller (i.e. the "0 wat sites" grounding is wrong — re-grep `:wat::kernel::close'` across the WHOLE tree),
  STOP and surface it: the strike then needs a sweep and this brief is under-scoped.
- **STOP-2:** if constructing the `Option<i64>` enum-field VALUE at runtime (the `Closed[exit]` payload) has no
  existing idiom to copy (how does the codebase build a `Some(i64)`/`None` Value?), STOP and surface the gap
  rather than invent one.
- **STOP-3:** if adding `CloseOutcome` to `MUST_USE_TYPES` turns any *existing* test RED (a kernel-internal
  wat that DOES drop a close' outcome), STOP — that's a real swallow site the "0 sites" grounding missed; report it.

## Weigh (the orchestrator re-runs; do NOT trust the report)
- the RED probe: RED before, GREEN after.
- **the floor: `cargo nextest run --release`, read the Summary line** (never a piped exit). Expected: 4212/0
  + the new probe green (the prior poll' floor was 4212/0). Any new RED that isn't the probe = a swallow site
  the grounding missed → STOP-3.
- content-integrity: the diff is exactly the 4 files above; nothing else moved.

## Copy for shape
`DESIGN-send-outcome-wall.md` (the send' wall — Phase 1 foundation is the twin of this whole strike) +
`BRIEF-recv-must-use-sweep.md` (the recv' must-use registration).
