# BRIEF — Stone 243.7b — split control-flow signals out of `RuntimeError`

Refines `DESIGN-STONE-243.7b.md` at strike time. Names are LOCKED (intueri cast): `EvalSignal` (the trio) + `EvalBreak { Diagnostic(RuntimeError), Signal(EvalSignal) }`. **This is the channel split only — NO Pattern A shape change to the diagnostics (that is 243.7c).**

## What to do

`RuntimeError` (`src/runtime.rs:2089`) currently smuggles 3 eval-loop control SIGNALS through the `Err` channel alongside ~30 genuine diagnostics. Extract the signals into their own type so `RuntimeError` becomes diagnostic-only:

1. Mint `EvalSignal` (the 3 signal variants, fields verbatim), `EvalBreak { Diagnostic(RuntimeError), Signal(EvalSignal) }`, and `impl From<RuntimeError> for EvalBreak` (→ `Diagnostic`).
2. Move the signals' Display + EDN arms onto `EvalSignal`; **delete** the 3 signal variants from `RuntimeError`.
3. Cascade (substrate-as-teacher): the eval subgraph that constructs/propagates/catches signals returns `Result<_, EvalBreak>`; **leaf verbs stay `Result<_, RuntimeError>`** and lift at `?` via `From`. Let cargo name every site; iterate to green.

This composes pieces that all already exist (verified) — it is a type-channel reshape, **behavior-identical** (TCO/try/option produce the same results). No new `Value` variant; no holon-rs; no semantics change.

## Read in order (the rooms — pre-walked)

1. `src/runtime.rs:2089`–`2374` — `enum RuntimeError`. The 3 signal variants to move: `TryPropagate(Box<Value>)` @ 2206, `OptionPropagate` @ 2219, `TailCall { func: Arc<Function>, args: Vec<Value>, call_span: Span }` @ 2238. Everything else stays.
2. `src/runtime.rs:2388`–`2596` — `impl Display for RuntimeError` + `impl Error`. The 3 signal arms @ 2488/2492/2496 move to `impl Display for EvalSignal` (verbatim messages — the "interpreter bug if this escaped" prose is TRUE of a signal).
3. `src/runtime_error_edn.rs:126`–`205` + `:349`–`361` — the EDN serialization. The 3 signal arms (serialization ~185–205 + name arms ~359–361) move to an `EvalSignal` serializer; remove from the `RuntimeError` serializer.
4. **Construction sites (5, runtime.rs):** `TailCall` @ 4695; `TryPropagate` @ 14358, 14361; `OptionPropagate` @ 14408, 14411. → `Err(EvalBreak::Signal(EvalSignal::…))`.
5. **Catch boundaries (2, runtime.rs):** `apply_function` (fn @ 21743) trampoline @ 21844 (TailCall→recurse) / 21857 (TryPropagate) / 21860 (OptionPropagate); the propagation handler @ 25036 / 25039 / 25064 / 25065. → match `EvalBreak::Signal(EvalSignal::…)`; the non-signal arm is `EvalBreak::Diagnostic(re)`.
6. `src/check.rs:8371, 8488, 14487` — doc-comment prose only (`RuntimeError::TryPropagate`/`OptionPropagate` mentioned) → `EvalSignal::…`. No code.
7. `tests/probe_arc243_stone7b_signal_split.rs` — the committed FM 2-bis probe; the target type shape you produce (it goes red→green).

## Implementation sketch (fill the path; do not invent the shape)

```rust
// runtime.rs — near the RuntimeError enum
/// Eval-loop control signals — NOT diagnostics. Raised and caught at function
/// boundaries (the TCO trampoline; the ?/option propagation handler). If one
/// reaches user code, that is an interpreter bug (see the Display messages).
pub enum EvalSignal {
    TailCall { func: Arc<Function>, args: Vec<Value>, call_span: Span },
    TryPropagate(Box<Value>),
    OptionPropagate,
}

/// The eval loop's Err type: an evaluation breaks either with a located
/// diagnostic (user-directed) or a control signal (evaluator-directed).
pub enum EvalBreak {
    Diagnostic(RuntimeError),
    Signal(EvalSignal),
}

impl From<RuntimeError> for EvalBreak {
    fn from(e: RuntimeError) -> Self { EvalBreak::Diagnostic(e) }
}

// catch boundary (apply_function trampoline), shape:
//   Err(EvalBreak::Signal(EvalSignal::TailCall { func, args, call_span })) => { /* recurse */ }
//   Err(EvalBreak::Signal(EvalSignal::TryPropagate(v)))                    => { /* as before */ }
//   Err(EvalBreak::Signal(EvalSignal::OptionPropagate))                    => { /* as before */ }
//   Err(EvalBreak::Diagnostic(re))                                         => return Err(re.into()) // or propagate
```

## The contract rule (pinned — the one surface decision)

**The compiler decides each function's `Err` type, not you:**
- Cargo demands `EvalBreak` at a site → it is **on the signal path**; flip its return type to `EvalBreak`.
- Cargo demands `RuntimeError` → it is a **leaf diagnostic**; leave it; the caller's `?` lifts via `From`.
- **NEVER hand-wrap a leaf in `EvalBreak` to silence the compiler. NEVER re-add a trio variant to `RuntimeError`.** The `From` impl is the only bridge.

## Cascade method-guidance

The return-type flip across the eval subgraph is mechanical and may be sizable (subset of runtime.rs's 432 `Result<_, RuntimeError>` sigs). If the flip exceeds ~50 sites, **an ephemeral Cargo tool that parses + rewrites the signatures is the preferred path** (build → use → **delete before the commit**; it never lands in the substrate) — the same move as 243.6a's `transform-checkerror`. Otherwise iterate by hand from the cargo error stream. The fail-count is the progress meter; watch it waterfall to 0.

## Discipline

- `src/runtime.rs` + `src/runtime_error_edn.rs` + `src/check.rs` (doc prose only) ONLY. No other files unless cargo names a genuine signal-path site there (surface it if so — leaves should NOT need it).
- **Behavior-preserving.** TCO/try/option semantics IDENTICAL. A moved/changed lib test = a behavior change to undo, not accept.
- No Pattern A shape change to RuntimeError (no outer struct, no `kind` enum — that's 243.7c). RuntimeError keeps its current flat variant shape, minus the 3 signals.
- Do NOT commit. Leave the tree dirty.

## STOP triggers (REJECTION, not defer)

1. A site needs `EvalBreak` but flipping its return type ripples to a caller that is plainly a leaf diagnostic with no signal involvement → STOP, name it (the subgraph boundary may be mis-drawn; do not hand-wrap leaves).
2. A trio Display/EDN message cannot move to `EvalSignal` verbatim (some field is RuntimeError-only) → STOP, name it (it shouldn't be — the fields move with the variant).
3. The TCO trampoline or try/option handler cannot be expressed over `EvalBreak::Signal(...)` without changing what it does → STOP (this is behavior change; the split must be a pure rehoming of the same match).
4. A lib test for TCO / `?`-propagation / option-propagation changes result → STOP and surface (parity is the kill criterion).

## FM 2-bis evidence

`tests/probe_arc243_stone7b_signal_split.rs` — committed; disconfirms at HEAD with a SINGLE `E0432` (*no `EvalSignal`/`EvalBreak` in runtime*); `RuntimeError` + the import path resolve clean (gap isolated). Post-stone: the 4 contracts compile + pass (`signal_enum_holds_the_trio`, `evalbreak_wraps_diagnostic_and_signal`, `from_runtimeerror_lifts_to_evalbreak`, `runtimeerror_is_diagnostic_only`).

## Verify (your own commands)

- `cargo test --release --test probe_arc243_stone7b_signal_split` → **4/0** pass.
- `cargo test --release --lib -p wat` → the baseline parity count (the orchestrator pins the exact number in EXPECTATIONS; TCO/try/option green).
- `cargo build --release --tests` → clean.
- `grep -nE "RuntimeError::(TailCall|TryPropagate|OptionPropagate)" src/` → **0** (the trio is gone from RuntimeError; only `EvalSignal::…` remains).
- `git status --porcelain` → exactly the authorized files (runtime.rs, runtime_error_edn.rs, check.rs, the probe) + NO scratch crate (the ephemeral tool, if used, is deleted).
- `cargo clippy --release -p wat 2>&1 | grep -c result_large_err` → **0**. `EvalBreak::Diagnostic(RuntimeError)` adds a wrapping layer over `RuntimeError` (whose large payloads 243.7a boxed). If `result_large_err` re-fires on `Result<_, EvalBreak>`, **box the diagnostic arm**: `EvalBreak::Diagnostic(Box<RuntimeError>)` (update the `From` impl + the `Diagnostic(re)` match/catch sites). Do NOT add an `#[allow]` — boxing is the fix (mirrors 243.7a). If it does not fire, leave the arm unboxed.

## SCORE

`SCORE-STONE-243.7b.md` (mirror `SCORE-STONE-243.6a.md` shape): the scope (channel split only; Pattern A is 243.7c); the cascade size (how many sigs flipped; ephemeral tool used or not + deleted); probe 4/0; lib parity vs baseline; the 5 construction + 2 catch boundaries rehomed; behavior-identical confirmation.

## Calibration

90–180 min Mode A. STOP at 360 min. Mint + move impls + flip subgraph + cascade-to-green; lib-parity is the gate. **No FM 2-bis composition risk beyond the probe** (behavior-preserving). **No vigilia REMARKABLE** (flat runtime.rs is wards-optional per `feedback_selective_lift_and_ward`; the `src/runtime/` home-carve is a future stone). Cite `SCORE-STONE-243.6a.md` (Rust-syntax ephemeral-tool cascade) for the cascade shape.
