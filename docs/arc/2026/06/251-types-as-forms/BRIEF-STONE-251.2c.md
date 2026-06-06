# BRIEF — Stone 251.2c — LIFT Function + Environment cluster → `src/value/environment.rs`

## The work (one paragraph)

Lift the `Function` struct and the `Environment` cluster (Environment, EnvBuilder, EnvCell,
BoundEntry) out of the flat `src/runtime.rs` into `src/value/environment.rs`. Co-locating them
(Function carries `closed_env: Option<Environment>`) collapses the transitional cross-refs.
PURE STRUCTURAL MOVE — no behavior change. Baseline **923 / 0 / 1**, identical after.

## Re-export approach (READ — changed from 251.2b; uniform, no per-type judgment)

**Re-export EVERY moved `pub` type uniformly** via a single `pub use crate::value::{…};` in
runtime.rs (zero-churn — leave ALL consumers untouched, do NOT repoint, do NOT count or judge
per type). runtime.rs uses these types heavily internally (Function ×79, Environment ×312) so it
needs the names regardless; `pub use` serves both. The 251.2e ward runs ONE purgare sweep that
converts every non-externally-consumed `pub use`→plain `use`. Do not try to minimize the re-export
surface here — uniformity now, purgare at the ward. (EnvCell is `struct EnvCell` private — it moves
with Environment, no re-export.)

## Read in order (the rooms)

1. `src/runtime.rs:1413–1465` — `pub struct Function` (1413) + `impl fmt::Debug for Function` (1450).
   Fields include `param_types`/`ret_type`/`rest_param_type: …TypeExpr…` + `type_params: Vec<String>`
   — these are scheme-related and clojure-ination TRANSFORMS targets: add a `// TRANSFORMS —
   clojure-ination (scheme/type fields)` comment on the struct, do NOT change them.
2. `src/runtime.rs:1466–1549` — `pub struct Environment` (1466), `pub struct BoundEntry` (1474),
   `struct EnvCell` (1479, private), `impl Environment` (1484), `impl Default for Environment` (1543).
3. `src/runtime.rs:1550–1613` — `pub struct EnvBuilder` (1550) + `impl EnvBuilder` (1555).
4. `src/value/signal.rs:8` — currently `use crate::runtime::{ClauseAttempt, ClauseFailureReason,
   Function, Value};`. After this move, `Function` is in value → change to
   `use crate::runtime::{ClauseAttempt, ClauseFailureReason, Value};` + `use crate::value::Function;`
   (ClauseAttempt/ClauseFailureReason/Value stay in runtime until 251.2e).

## Implementation sketch

1. `src/value/environment.rs` — move all of: Function (+Debug), Environment (+impl +Default),
   EnvBuilder (+impl), EnvCell (private), BoundEntry. Imports it needs:
   - `use crate::value::{TrackedValue, Provenance};` (Environment::lookup → TrackedValue; BoundEntry
     carries TrackedValue — both now in value/observe.rs, reachable via the value/ re-exports).
   - `use crate::types::TypeExpr;` (Function's type fields).
   - `use crate::ast::WatAST;` (Function body), `use crate::span::Span;`, `std::collections::HashMap`,
     `std::sync::Arc` as needed — add whatever the compiler asks for.
2. `src/value/mod.rs` — add `pub mod environment;` + `pub use environment::{Function, Environment,
   EnvBuilder, BoundEntry};` (NOT EnvCell — private).
3. `src/lib.rs` — move `Function, Environment, EnvBuilder` (and any of these in the `pub use
   runtime::{…}` list) to `pub use value::{…}`; leave the rest.
4. `src/runtime.rs` — delete the moved block (1413–1613); add `pub use crate::value::{Function,
   Environment, EnvBuilder, BoundEntry};` (uniform re-export — runtime's eval loop uses these heavily;
   zero-churn for all consumers).
5. `src/value/signal.rs` — fix the import (step 4 of Read-in-order).
6. `cargo build --release` → fix every path the compiler names. Then `cargo test --release --lib -p wat`.

## Blast radius

`src/value/` (+environment.rs; mod.rs grows) + `src/runtime.rs` (delete block, add re-export) +
`src/lib.rs` (re-export moves) + `src/value/signal.rs` (1 import fix). With uniform re-export,
NO other consumer files should need changes (zero-churn). NO logic/signature edits.

## STOP triggers (rejection)

- STOP-1: any body/signature/behavior change beyond `use` paths, the uniform re-export, the signal.rs
  import fix, and the `// TRANSFORMS` comment on Function → STOP, report.
- STOP-2: a borrow/visibility/cycle tangle not resolved by the imports above → STOP, report. (Intra-crate
  cycles are fine: environment.rs using crate::types::TypeExpr while runtime uses crate::value::Environment
  is NOT a cycle problem.)
- STOP-3: `cargo test --release --lib -p wat` ≠ **923 / 0 / 1** → STOP, report the delta.

## Done =

environment.rs exists in src/value/; the Function + Environment cluster GONE from runtime.rs (defs);
EnvCell private in environment.rs; signal.rs import fixed; uniform `pub use` re-export in runtime.rs;
`cargo build` clean; `cargo test --release --lib -p wat` = 923/0/1; clippy clean in src/value/.
Do NOT commit — leave the tree dirty. Report: before/after count, files touched.
