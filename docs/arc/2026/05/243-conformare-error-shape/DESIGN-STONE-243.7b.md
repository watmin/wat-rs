# DESIGN — Stone 243.7b — split control-flow signals out of `RuntimeError` (the channel split)

**Status:** DRAFT → names LOCKED (intueri cast 2026-06-01: `EvalSignal` + `EvalBreak{Diagnostic,Signal}`). FM 2-bis probe owed before STRIKE-READY. Child of arc 243 (conformare). **Split from the banked "RuntimeError → Pattern A" obligation by proactive slicing (2026-06-01):**
- **243.7b (this stone)** — extract the eval-loop control signals into their own channel so `RuntimeError` becomes diagnostic-only. **No shape change to the diagnostics.**
- **243.7c** — `RuntimeError` → Pattern A (`struct { location, kind }`) over the now-pure diagnostic set, the same known move as TypeError (243.3) / CheckError (243.6a).

The split is the stepping-stone test answering YES: the trio's presence is the *only reason* RuntimeError can't be Pattern A (a `TailCall` has no source location because it isn't an error). Remove the signals first and 243.7c operates on a settled, signal-free, all-diagnostic enum with zero special-casing. Each piece gets a clean "did it work": 7b = lib behavioral parity (TCO/try/option preserved); 7c = the Pattern-A probe.

## Why this stone

`RuntimeError` (`src/runtime.rs:2089`, ~33 variants) conflates two categories that the conformare doctrine exists to separate:

1. **~28 genuine diagnostics** — carry `span: Span`; surface to user code; "anything wat can toss must be location-aware."
2. **3 eval-loop control SIGNALS** — `TailCall`, `TryPropagate(Box<Value>)`, `OptionPropagate` — the runtime talking to itself through the `Err` channel. They are **not errors**: their own Display arms admit it (`runtime.rs:2498` — *"TCO: internal error — a tail-call signal escaped its enclosing apply_function… reaching the user with one unwound indicates an interpreter bug"*). They have no source location because there is no diagnostic to locate. (The codebase ALREADY calls these "control-flow signal" in 10+ prose sites — the name `EvalSignal` is the substrate's own word, finally given to the type.)

(A fourth category — the freeze-time pair `UserMainMissing` / `EvalVerificationFailed { err: HashError }` — is genuinely diagnostic but spanless-by-domain; it **stays in RuntimeError** and earns a non-source location in 243.7c. Out of scope here.)

You cannot put a mandatory location on `RuntimeError` (243.7c's Pattern A) while `TailCall`/`TryPropagate`/`OptionPropagate` live in it. **243.7b removes that blocker by relocating the signals to a channel that says what they are.** Failure-engineering at the ✅✅✅ rung: a control signal becomes structurally unable to masquerade as a located diagnostic.

## What it delivers

- `enum EvalSignal` holding the trio: `TailCall { func, args, call_span }`, `TryPropagate(Box<Value>)`, `OptionPropagate`. Fields preserved verbatim.
- `enum EvalBreak { Diagnostic(RuntimeError), Signal(EvalSignal) }` — the eval-loop's `Err` type. (`Diagnostic`, not `Error`: post-split `RuntimeError` IS a diagnostic; `Diagnostic`/`Signal` are the parallel user-directed/evaluator-directed faces.)
- `impl From<RuntimeError> for EvalBreak` (`fn from(e) -> Self { EvalBreak::Diagnostic(e) }`) — the `?`-boundary. A leaf call that produces a `RuntimeError` lifts to `EvalBreak::Diagnostic` for free; **leaf verbs never change signature.**
- `RuntimeError` loses exactly the 3 trio variants (and their 3 Display arms + 3 EDN arms move to `EvalSignal`). Every diagnostic variant is **untouched** (shape retrofit is 243.7c).
- The eval subgraph in `runtime.rs` that constructs/propagates/catches signals returns `Result<_, EvalBreak>`; the leaf verbs (io/time/string_ops/marshal/…) stay `Result<_, RuntimeError>`.
- **Behavior identical** — TCO trampoline, `?`-propagation, option-propagation produce the same results; this is a type-channel reshape, not a control-flow change.

## The algorithm

1. **Mint the channel.** In `runtime.rs`, define `EvalSignal` (the trio, fields verbatim) + `EvalBreak { Diagnostic(RuntimeError), Signal(EvalSignal) }` + `impl From<RuntimeError> for EvalBreak`.
2. **Move the signal-specific impls.** The 3 trio arms in `impl Display for RuntimeError` (`runtime.rs:2488/2492/2496`) → `impl Display for EvalSignal` (verbatim messages — the "interpreter bug" prose belongs on the signal, where it is true). The 3 trio arms in `src/runtime_error_edn.rs` (the `RuntimeError::{TryPropagate,OptionPropagate,TailCall}` serialization @ ~185–205 + the name arms @ ~359–361) → an `EvalSignal` serializer.
3. **Remove the trio from `RuntimeError`.** Delete the 3 variants from the enum (`runtime.rs:2206/2219/2238`). Cargo now reds every construction + match + Display + EDN site — the meter.
4. **Construction sites (5, all in `runtime.rs`):** `TailCall` @ 4695; `TryPropagate` @ 14358/14361; `OptionPropagate` @ 14408/14411 → `Err(EvalBreak::Signal(EvalSignal::Variant{…}))`.
5. **Catch boundaries (2, all in `runtime.rs`):** the `apply_function` trampoline @ 21844/21857/21860 + the propagation handler @ 25036/25039/25064/25065 → match `Err(EvalBreak::Signal(EvalSignal::…))` for the control arms; the non-signal `Err(EvalBreak::Diagnostic(re))` arm returns/propagates the diagnostic.
6. **Cascade (substrate-as-teacher):** the subgraph between construction and catch must carry `EvalBreak`. Flip the boundary types; let cargo name every fn on the propagation path; convert `Result<_, RuntimeError>` → `Result<_, EvalBreak>` for those, leaving leaves on `RuntimeError` (the `From`-at-`?` lifts them). **Fail-count is the progress meter; iterate to green.** If the mechanical flip exceeds ~50 sites, the weapon is an **ephemeral Cargo tool** (build → use → delete before commit) per `feedback_cascade_ephemeral_tool` — sanctioned in the BRIEF as method-guidance, never tool-reassurance (FM 16).

## The error contract (the one surface decision, pinned)

**A function's `Err` type is determined by the compiler, not by hand: it returns `EvalBreak` iff it transitively constructs or propagates a signal; otherwise it stays `RuntimeError` and converts at `?`.**

The rule sonnet follows when cargo reds a site:
- Cargo wants `EvalBreak` here → the site is **on the signal path**; flip its return type to `EvalBreak`.
- Cargo wants `RuntimeError` here → it is a **leaf diagnostic**; leave it; the caller's `?` lifts via `From`.
- **Never hand-wrap a leaf in `EvalBreak` to silence the compiler, and never re-add a trio variant to `RuntimeError`.** The `From` impl is the only bridge; the boundary falls wherever the signal actually flows.

This is the load-bearing decision: it makes the leaf/subgraph boundary *emergent from the call graph* rather than guessed — the same discipline that bounded the cascade off all 626 `Result<_, RuntimeError>` sigs to just the signal subgraph.

## Files touched

- `src/runtime.rs` — mint `EvalSignal`/`EvalBreak`/`From`; remove trio from `RuntimeError` + its 3 Display arms; 5 construction sites; 2 catch boundaries; the subgraph return-type cascade.
- `src/runtime_error_edn.rs` — move the 3 trio EDN arms (serialization + name) to an `EvalSignal` serializer; remove from the `RuntimeError` serializer.
- `src/check.rs` — doc-comment references only (8371, 8488, 14487 — `RuntimeError::TryPropagate`/`OptionPropagate` in prose) updated to `EvalSignal::…`. No code.
- `tests/probe_arc243_stone7b_signal_split.rs` — the FM 2-bis probe (flips fail-compile → pass).

## Out of scope (REJECTED, not deferred)

- `RuntimeError` → Pattern A `{ location, kind }` shape retrofit → **Stone 243.7c** (this stone leaves every diagnostic variant byte-identical).
- The freeze pair (`UserMainMissing`/`EvalVerificationFailed`) location-typing → **Stone 243.7c** (stays in RuntimeError, unchanged here).
- `src/runtime/` home carve + vigilia REMARKABLE → **future undertaking** (runtime.rs is 24k-line flat-untrusted; wards-optional per `feedback_selective_lift_and_ward`; not this chain).
- Any control-flow semantics change — TCO/try/option behavior is **identical**; this is a type-channel reshape only.
- `result_large_err` boxing → already shipped (243.7a).

## Naming (intueri cast — COMPLETE, verdict locked 2026-06-01)

Cast `EvalSignal` / `EvalBreak{Diagnostic,Signal}` (a real spawned cast; spell embedded by value). The cast disagreed with the working names and improved one I had not questioned:
- **`Control` → `EvalSignal`** — the codebase already uses "control-flow signal" in 10+ prose sites; `EvalSignal` gives the type the substrate's own word. Avoids `std::ops::ControlFlow` confusion + the `Signal::Signal` stutter that `Signal` alone would cause.
- **`EvalBreak`** — held (working name confirmed). `Exit` would lie about TCO (a loop-back, not a termination); `Outcome` implies the `Ok` arm.
- **`Error(RuntimeError)` → `Diagnostic(RuntimeError)`** — `EvalBreak::Error(RuntimeError)` is "error of error" (redundant); `Diagnostic` names the role honestly and makes `Diagnostic`/`Signal` semantically parallel (user-directed / evaluator-directed) — the variant pair now speaks the architecture.

## Probe contracts (`tests/probe_arc243_stone7b_signal_split.rs` — committed; must disconfirm at HEAD)

1. `signal_enum_holds_the_trio` — `EvalSignal::{TailCall,TryPropagate,OptionPropagate}` construct. Pre-stone: type unresolved (gap).
2. `evalbreak_wraps_diagnostic_and_signal` — `EvalBreak::Diagnostic(re)` + `EvalBreak::Signal(EvalSignal::OptionPropagate)` construct. Pre-stone: type unresolved (gap).
3. `from_runtimeerror_lifts_to_evalbreak` — `let b: EvalBreak = some_runtimeerror.into();` compiles. Pre-stone: no `From` impl (gap).
4. `runtimeerror_no_longer_holds_signals` — the trio variants no longer construct on `RuntimeError` (negative contract; verified by cargo, expressed as a doc/comment contract).

**The disconfirmation must isolate exactly the gap** (the new types don't exist yet) — everything around it compiles. Expected at HEAD: `E0433`/`E0412` unresolved-type on `EvalSignal`/`EvalBreak` + `E0599` no `into()` target. Post-stone: 3/0 pass.

**Behavioral parity is NOT in the probe** (a no-op-behavior change can't disconfirm). It is the SCORE's load-bearing row: the existing TCO / try-propagation / option-propagation lib tests must stay green. **Re-run them to establish the baseline BEFORE spawning** (FM 9).

## Trap-doors

| # | Risk | Detection | Resolution |
|---|---|---|---|
| **T1** | A fn on the signal path missed (stays `RuntimeError`) → can't carry `EvalBreak::Signal` | cargo type error at the catch boundary | flip it to `EvalBreak` — it is on the path (the contract rule) |
| **T2** | A leaf wrongly flipped to `EvalBreak` → cascade balloons past the real subgraph | review: leaf has no signal construction | revert to `RuntimeError`; let the caller's `?` lift via `From` |
| **T3** | `From<RuntimeError> for EvalBreak` collides with an existing blanket impl | cargo `E0119` | **EMPTY** — no `From<…> for RuntimeError` exists today (verified); low risk |
| **T4** | Trio EDN/Display arms orphaned or double-defined | cargo + the runtime_error_edn test | move all 3 arms to `EvalSignal`; remove from RuntimeError |
| **T5** | TCO trampoline (`21844`) loop semantics drift when its `Err` type becomes `EvalBreak` | cargo + the TCO lib tests (parity baseline) | the loop matches `EvalBreak::Signal(EvalSignal::TailCall{..})` to recurse, returns `EvalBreak::Diagnostic` / `EvalBreak::Signal(other)` |
| **T6** | `apply_function`'s signature change ripples to callers outside the eval core | cargo fail-count | follow the cascade; `apply_function`'s callers are within the eval subgraph by construction |

## Calibration

A careful split on the hottest file; cascade bounded by the signal subgraph (a subset of runtime.rs's 432 sigs, NOT all 626 — leaves stay `RuntimeError`).
- **Phase A (mint channel + move impls + flip subgraph + cascade to green):** 90–180 min Mode A. STOP at 360 min. Ephemeral Cargo tool if the mechanical return-type flip is ≥~50 sites.
- **Gate:** lib parity — `cargo test --release --lib -p wat` at the prior baseline (TCO/try/option green) + `cargo build --release --tests` clean. **No vigilia REMARKABLE** (flat runtime.rs is wards-optional; the home-carve is a future stone).
- Behavioral parity is the kill criterion: a moved/failed TCO/try/option test = a behavior change to undo, not accept.

## Cross-references

- Template: `DESIGN-STONE-243.6a.md` (this doc mirrors its shape) + `BRIEF-STONE-243.7a.md` (the prior RuntimeError stone — boxing) + `SCORE-STONE-243.7a.md`.
- `docs/CONFORMARE.md` (zero-exceptions; the diagnostic-vs-signal distinction this stone makes structural).
- `docs/SUBSTRATE-AS-TEACHER.md` (the cascade) + `feedback_cascade_ephemeral_tool` (the ephemeral tool weapon).
- `feedback_selective_lift_and_ward` (why runtime.rs stays flat — no vigilia this stone).
- arc 243 `DESIGN.md` (243.7… rolling-audit row; this stone + 243.7c refine it).
