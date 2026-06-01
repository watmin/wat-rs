# BRIEF — Stone 243.7c — `RuntimeError` → Pattern A (the shape retrofit)

Refines `DESIGN-STONE-243.7c.md` at strike time. Pattern A reshape of the now-signal-free `RuntimeError` (243.7b removed the control signals). **Flat in `src/runtime.rs` — NO home carve, NO vigilatum** (runtime.rs is wards-optional; the `src/runtime/` home is a future undertaking).

## What to do

`RuntimeError` (`src/runtime.rs:2170`, ~30 diagnostic variants) carries `span` per-variant (hand-discipline) — except the freeze pair which carry none. Reshape to Pattern A so the spanless shape is structurally unrepresentable:

1. `pub struct RuntimeError { pub span: Span, pub kind: RuntimeErrorKind }` + `pub enum RuntimeErrorKind`.
2. Single-span variants (~25): drop the `span` field → it moves to the outer struct.
3. Two multi-span variants: outer `span` = most-actionable; secondary = domain-named kind field (§contract below).
4. Freeze pair (`UserMainMissing`, `EvalVerificationFailed`): kind variants with NO span; construct with outer `Span::unknown()`.
5. Split Display; collapse the EDN serializer to `self.span`/`self.kind`; cascade the ~1186 sites to green.

This is the **same move as CheckError (243.6a)** at ~2.5× scale. Mirror `src/check/error.rs`. Behavior-preserving — location-discipline reshape only, no message/semantics change. No new `Value` variant; no holon-rs.

## Read in order (rooms — pre-walked)

1. `src/runtime.rs:2170`–`2400` — `enum RuntimeError`. The variant set; note the 2 multi-span (`SandboxScopeLeak` @ 2303, `PostconditionFailed` @ 2389) and the freeze pair (`UserMainMissing` @ 2211, `EvalVerificationFailed` @ 2216).
2. `src/runtime.rs:2406` — `span_prefix(&Span)` (already present; reuse for outer-span rendering + unknown elision).
3. `src/runtime.rs:2414`–`2610` — `impl Display for RuntimeError` (N-arm; split into Kind span-free + RuntimeError delegating).
4. `src/runtime_error_edn.rs` — the EDN serializer ("all 28 variants"); collapse to `self.span`/`self.kind`.
5. `src/check/error.rs` — **the shipped CheckError Pattern A; THIS IS YOUR TEMPLATE.** Mirror its `struct`/`Kind`/Display-split/elision shape.
6. `tests/probe_arc243_stone7c_runtimeerror_pattern_a.rs` — the committed FM 2-bis probe (goes red→green).

## Implementation sketch (fill the path; mirror src/check/error.rs)

```rust
pub struct RuntimeError { pub span: Span, pub kind: RuntimeErrorKind }

pub enum RuntimeErrorKind {
    UnboundSymbol(String),                                  // was (String, Span)
    DivisionByZero,                                         // was (Span) -> unit
    TypeMismatch { op: String, expected: &'static str, got: Box<ValueSnapshot> },  // span dropped
    // ... all ~30, span removed; payload fields preserved verbatim ...
    UserMainMissing,                                        // freeze pair, no span
    EvalVerificationFailed { err: crate::hash::HashError }, // freeze pair, no span
    SandboxScopeLeak { offending_name: String, outer_define_span: Span },  // secondary span kept
    PostconditionFailed { /* ... */ ensure_span: Span },                    // secondary span kept
}

impl fmt::Display for RuntimeErrorKind { /* span-free per-variant message */ }
impl fmt::Display for RuntimeError {
    fn fmt(&self, f) -> ... { write!(f, "{}{}", span_prefix(&self.span), self.kind) }  // elides unknown
}
```

## The error contract (pinned)

| Variant | spans | outer `span` = most-actionable | secondary → domain-named kind field |
|---|---|---|---|
| `SandboxScopeLeak` | 2 | `call_span` | `outer_define_span` |
| `PostconditionFailed` | 2 | `body_span` | `ensure_span` |

**Freeze pair:** `UserMainMissing` / `EvalVerificationFailed` → NO span on the kind; construct `RuntimeError { span: Span::unknown(), kind: ... }`. Honest because `span_prefix` elides unknown (no `<runtime>:0:0` leak). Do NOT invent a separate location type.

## Cascade — the weapon (MANDATORY)

~1186 `RuntimeError::` sites (913 in runtime.rs; rest in io/time/freeze/string_ops/thread_io/marshal/runtime_error_edn + crates/wat-telemetry-sqlite). **Build an ephemeral *Rust* Cargo tool** that parses + rewrites the construction sites (`RuntimeError::Variant { …, span }` → `RuntimeError { span, kind: RuntimeErrorKind::Variant { … } }`) — exactly like 243.6a's `transform-checkerror`. **Do NOT use Python or `sed`/shell — `python3` and shell mass-editors are sandbox-BLOCKED here and will waste a cycle.** Build → run → **DELETE the tool before you finish** (it must not land in the tree). Match-site destructuring + the residue iterate by hand from the cargo error stream. Fail-count is the progress meter.

## Discipline

- `src/runtime.rs` + `src/runtime_error_edn.rs` + the cascade fan-out (io/time/freeze/string_ops/thread_io/marshal/`crates/wat-telemetry-sqlite`) ONLY.
- **Behavior-preserving.** No message rewrites, no merging, no recovery. A moved/failed lib test = a behavior change to undo.
- **The 243.7b `EvalBreak::Diagnostic(RuntimeError)` wrap must stay intact** — it holds a RuntimeError by value; the struct reshape is transparent to it. Confirm `tests/probe_arc243_stone7b_signal_split.rs` still passes 4/0.
- NO `src/runtime/` home, NO vigilatum (flat file).
- Do NOT commit. Leave the tree dirty.

## STOP triggers (REJECTION)

1. A variant's span can't be cleanly classified single vs multi → STOP, name it (the §contract covers the 2 multi-span; everything else is single).
2. The Display/EDN split changes a rendered message → STOP (behavior change; the split must be a faithful rehoming).
3. A lib test changes result → STOP and surface (parity is the kill criterion).
4. `result_large_err` fires on the new struct → box the large kind payload (the hot ones are already `Box<ValueSnapshot>`); NO `#[allow]`.

## Verify (your own commands)

- `cargo test --release --test probe_arc243_stone7c_runtimeerror_pattern_a` → **4/0**.
- `cargo test --release --test probe_arc243_stone7b_signal_split` → **4/0** (EvalBreak wrap intact).
- `cargo test --release --lib -p wat` → **895/0/1** (parity).
- `cargo build --release --tests` → clean.
- `cargo clippy --release -p wat 2>&1 | grep -c result_large_err` → **0**.
- `git status --porcelain` → runtime.rs, runtime_error_edn.rs + the cascade fan-out files; **NO scratch crate** (the ephemeral Rust tool deleted).

## SCORE

`SCORE-STONE-243.7c.md` (mirror `SCORE-STONE-243.6a.md`): cascade size (sites reshaped; ephemeral Rust tool used + deleted); the 2 multi-span + 2 freeze-pair decisions; probe 4/0; 7b probe 4/0; lib parity; clippy result; behavior-identical confirmation.

## Calibration

120–240 min Mode A. STOP at 480 min. Largest cascade in the arc (~1186 sites). Ephemeral **Rust** Cargo tool mandatory. Cite `SCORE-STONE-243.6a.md` (the CheckError Pattern A — same move, 459 sites) for shape. No vigilia (flat file); lib parity + the two probes are the gate.
