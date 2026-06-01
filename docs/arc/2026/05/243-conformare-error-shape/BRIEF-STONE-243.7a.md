# BRIEF — Stone 243.7a — Box `RuntimeError` large variants (`result_large_err`)

Refines `DESIGN-STONE-243.7a.md` at strike time. The DESIGN offered box-large-variants vs box-whole; the four-questions pick **box the large variant payloads in-place** (surgical; no `?`-chain impact → no probe needed; the box-whole option would have touched 569 signatures + every `?`).

## Scope decision (four-questions — record in SCORE)

`RuntimeError` (`src/runtime.rs:2089`) is a flat enum conflating ~28 diagnostic variants (carry `span`) + ~5 internal control-flow SIGNALS (`TryPropagate(Value)`, `OptionPropagate`, `TailCall`, `UserMainMissing`, `EvalVerificationFailed`) that never surface to user code and carry no location.

- **243.7a = the BOXING only.** The named obligation of the 10 `rune:excusare(OPEN-DEFERRAL → 243.7a)` runes: box the large variant payloads so `clippy::result_large_err` clears + the runes/stamp-drift close. Behavior-preserving; lib + clippy gate.
- **NOT Pattern A.** RuntimeError → Pattern A first requires resolving the error/signal conflation (signals don't fit span-at-outer-struct). That is a SEPARATE future rolling-audit stone — do NOT attempt it here. (runtime.rs is flat/wards-optional; no vigilia REMARKABLE this stone.)

## What to do

`clippy::result_large_err` fires because RuntimeError is large-by-value (clippy points at `TypeMismatch` @ runtime.rs:2093, driven by `ValueSnapshot`). Box the large payloads in-place:
- `NotCallable { got: ValueSnapshot, span }` → `got: Box<ValueSnapshot>`
- `TypeMismatch { got: ValueSnapshot, ... }` → `got: Box<ValueSnapshot>`
- `BadCondition { got: ValueSnapshot, span }` → `got: Box<ValueSnapshot>`
- `TryPropagate(Value)` → `TryPropagate(Box<Value>)`
- (+ any other variant clippy still flags after these — see iterate.)

**Iterate:** after boxing the above, re-run `cargo clippy --release -p wat`; if `result_large_err` still fires anywhere, box the next-largest variant clippy names, until the lint shows **0** occurrences.

**Cascade (substrate-as-teacher):** at construction sites, wrap the payload (`got: Box::new(ValueSnapshot::of(v))`, `TryPropagate(Box::new(v))`); at match/access sites, `Box<T>` Derefs to `T` so most field reads are transparent — where a match binds the field by value or needs the owned `T`, deref/clone as the compiler directs. Box the field, let cargo name every site, fix to green.

## Then close the runes + stamps

- Remove EVERY `#[allow(clippy::result_large_err)]` and the `rune:excusare(OPEN-DEFERRAL → 243.7a)` comment above each. Sites (grep `result_large_err` + `OPEN-DEFERRAL → 243.7a` to confirm the full set): `src/function/eval.rs` (30-31), `src/function/parse.rs` (186-187), `src/rust_deps/marshal.rs` (51-52, 359-360, 392-393), `src/rust_deps/custodia.rs` (54-55, 74-75, 89-90, 150-151), `src/runtime.rs` (13073-13074).
- Update the `vigilatum` stamps that note the drift: `src/function/mod.rs:1` (and `src/rust_deps/mod.rs` if it carries the same note) — remove the `(clippy clean-or-runed: result_large_err → excusare OPEN-DEFERRAL 243.7a)` qualifier so the stamp reads as the clean form (the home is now fully clippy-clean; no rune outstanding). Keep the ISO8601 date + the `vigilia N-spell L1+L2=0` core; only drop the result_large_err qualifier.

## Discipline
- `src/runtime.rs` (the enum + its construction/match sites across src/) + the rune/allow/stamp sites ONLY. **No Pattern A. No signal-split.** No new Value variant; no holon-rs. Behavior-preserving — boxing is transparent at runtime; identical behavior.
- Do NOT commit. Leave the tree dirty.

## STOP triggers (rejection, not defer)
1. A match needs the owned `T` (not `&T`) from a now-boxed field and the deref/clone isn't mechanical → STOP, name it.
2. `result_large_err` won't clear after boxing ValueSnapshot+Value (a different variant is huge in a way boxing is awkward) → box what clippy names; if genuinely awkward, STOP + surface — do NOT re-add an allow.
3. Removing a rune/allow re-triggers the lint at that site → the enum is still too big; box more. Never re-add the allow to "make it pass."

## Verify (your own commands)
- `cargo clippy --release -p wat 2>&1 | grep -c result_large_err` → **0**.
- `cargo test --release --lib -p wat` → **895 / 0 / 1** (behavior-preserving; a moved test = a behavior change to undo).
- `cargo build --release --tests` → clean.
- `grep -rn "OPEN-DEFERRAL → 243.7a" src/` → **0**; `grep -rn "result_large_err" src/` → **0** (every allow + rune gone).

## SCORE
`SCORE-STONE-243.7a.md`: the scope decision (boxing-only; RuntimeError→Pattern-A + signal-split BANKED), which variants boxed (+ any iterate-added), the cascade size, clippy result_large_err 0, lib parity, the 10 runes + the function/rust_deps stamp-drifts closed.

## Calibration
30–60 min Mode A. Boxing cascade + clippy-iterate + rune/stamp cleanup; lib + clippy are the gate. No FM 2-bis probe (behavior-preserving; box-in-place doesn't touch `?`).
