# BRIEF — Stone 251.2b — LIFT signal + observe → `src/value/` (+ render_value; + 251.2a cleanup)

## The work (one paragraph)

Lift the runtime's **signal types** (EvalSignal/EvalBreak/RuntimeError/RuntimeErrorKind) into
`src/value/signal.rs` and the **value-observation types** (TrackedValue/ValueSnapshot/Provenance)
+ the `render_value` display engine into `src/value/observe.rs`. Pure structural move — NO behavior
change. Baseline: **923 passed / 0 failed / 1 ignored**, must be identical after. Also: drop two
dead `pub` re-exports left by 251.2a. This is the first lift where the **re-export principle** is
load-bearing (RuntimeError/ValueSnapshot are pervasive).

## Read in order (the rooms)

1. `src/runtime.rs` **observe block ~1880–2030** — `pub struct Provenance`, `pub struct TrackedValue`
   (1893) + `impl TrackedValue` (1898) + `impl From<Value> for TrackedValue` (1922), `pub struct
   ValueSnapshot` (1937) + `impl ValueSnapshot` (1943) + `impl Display for ValueSnapshot` (1990).
   (Confirm Provenance's exact span by grep; it sits in this block.)
2. `src/runtime.rs` **signal block 2031–2597** — `pub enum EvalSignal` (2031) + Display (2055);
   `pub enum EvalBreak` (2080) + `impl From<RuntimeError> for EvalBreak` (2090) + Display (2096);
   `pub struct RuntimeError` (2114); `pub enum RuntimeErrorKind` (2131) + `impl RuntimeErrorKind`
   (2374) + Display (2583); `impl Display for RuntimeError` (2590); `impl Error for RuntimeError` (2597).
3. `src/runtime.rs:18565` — `fn render_value(v: &Value, depth: usize) -> String` (~80–100 lines).
   Verified: NO eval/eval_inner/apply_function back-references → moves cleanly into observe.rs.
   `ValueSnapshot::of()` calls it.
4. The 251.2a dead re-exports to clean: `grep -n 'pub use crate::value::EncodingCtx' src/runtime.rs`
   and `grep -n 'pub use crate::value::{FrameInfo' src/runtime.rs` — drop the `pub` from BOTH
   (runtime.rs uses them internally; nothing consumes the old path; they must be plain `use`).

## The implementation sketch

1. `src/value/observe.rs` — move Provenance + TrackedValue + ValueSnapshot (+ their impls) + `render_value`.
   `use crate::value::Value`? NO — `Value` is still in runtime.rs this stone → `use crate::runtime::Value;`
   (transitional; becomes crate::value at 251.2e). Also `use crate::span::Span;`.
2. `src/value/signal.rs` — move EvalSignal/EvalBreak/RuntimeError/RuntimeErrorKind (+ impls).
   Imports: `use crate::value::observe::ValueSnapshot;` (RuntimeErrorKind variants carry
   `Box<ValueSnapshot>`); `use crate::runtime::Function;` (EvalSignal::TailCall carries `Arc<Function>`
   — Function still in runtime.rs until 251.2c, transitional); `use crate::value::Value;`→`use crate::runtime::Value;`
   (transitional); `use crate::span::Span;`; `crate::hash::HashError` if a variant carries it.
3. `src/value/mod.rs` — add `pub mod observe;` `pub mod signal;` and re-export the public surface:
   `pub use signal::{EvalSignal, EvalBreak, RuntimeError, RuntimeErrorKind};`
   `pub use observe::{TrackedValue, ValueSnapshot, Provenance};`
4. `src/lib.rs` — move RuntimeError/RuntimeErrorKind (and any of these in the `pub use runtime::{…}`
   list) to `pub use value::{…}` (preserve the external `wat::RuntimeError` API).
5. **The re-export principle — apply per type** (this is the load-bearing judgment):
   - runtime.rs uses these types HEAVILY internally → it needs `use crate::value::{…}` regardless.
   - For each moved type, count old-path consumer sites: `grep -rn 'runtime::<Type>' src/ tests/`.
     **If > ~15 sites → keep a `pub use crate::value::<Type>;` re-export in runtime.rs (zero-churn,
     leave consumers untouched). If ≤ ~15 → repoint those import lines to `crate::value::<Type>`
     (no re-export — the home owns it).**
   - Known counts: **ValueSnapshot = 74 → RE-EXPORT** (`pub use crate::value::ValueSnapshot;`).
     **RuntimeError = 7 → REPOINT** the 7 import sites. **TrackedValue = 4 → REPOINT.**
     Count EvalSignal / EvalBreak / RuntimeErrorKind / Provenance yourself and apply the rule.
6. Drop the `pub` from the two 251.2a re-export lines (step 4 in Read-in-order) → plain `use`.
7. `cargo build --release` → fix every path the compiler names (substrate-as-teacher). Then
   `cargo test --release --lib -p wat`.

## Blast radius

`src/value/` (+2 files: signal.rs, observe.rs; mod.rs grows) + `src/runtime.rs` (delete 3 segments;
add internal `use`s + the ValueSnapshot re-export; drop 2 dead pubs) + `src/lib.rs` (re-export moves)
+ the repointed import sites for the ≤15-count types. NO logic edits, NO signature changes.

## STOP triggers (rejection criteria — ship nothing, report)

- STOP-1: any function body / signature / behavior change beyond `use` paths, the re-export/repoint
  per the principle, and the `pub`→`use` drops → STOP, report (the boundary is wrong).
- STOP-2: if `render_value` turns out to call back into the eval loop (eval_inner/apply_function/
  dispatch) when you read its full body → STOP, report (it'd need a different resolution than a
  clean move).
- STOP-3: a borrow/visibility/cycle tangle not resolved by the imports above → STOP, report the exact
  tangle. (Intra-crate import cycles are fine in Rust — signal.rs using crate::runtime::Function while
  runtime.rs uses crate::value::RuntimeError is NOT a cycle problem.)
- STOP-4: `cargo test --release --lib -p wat` ≠ **923 / 0 / 1** → STOP, report the delta.

## Done =

signal.rs + observe.rs exist in src/value/; the three segments + render_value GONE from runtime.rs;
the 2 dead pubs dropped; re-export principle applied per type (ValueSnapshot re-exported, RuntimeError/
TrackedValue repointed, the rest by count); `cargo build` clean; `cargo test --release --lib -p wat`
= 923/0/1; clippy clean in src/value/. Do NOT commit — leave the tree dirty. Report: before/after
count, files touched, and for EACH moved type whether you re-exported or repointed (+ the count that decided it).
