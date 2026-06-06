# BRIEF — Stone 251.2d — LIFT SymbolTable → `src/value/symbol_table.rs`

## The work

Lift the `SymbolTable` god-struct (struct + Debug impl + `impl SymbolTable`) out of flat
`src/runtime.rs` into `src/value/symbol_table.rs`. PURE STRUCTURAL MOVE — no behavior change.
Baseline **923 / 0 / 1**, identical after. Uniform re-export (same as 251.2c — no per-type judgment).

## Read in order

1. `src/runtime.rs` SymbolTable block: `pub struct SymbolTable` (1428), `impl std::fmt::Debug for
   SymbolTable` (1544), `impl SymbolTable` (1564) — move the WHOLE contiguous region (struct through
   the closing brace of `impl SymbolTable`; find its end). Its capability-carrier set/get methods are
   stable (no eval-loop coupling expected — if a method DOES call into the eval loop, STOP-2).
2. The keyword-keyed maps (`unit_variants`, `defined_values`, `binding_metadata`) are clojure-ination
   TRANSFORMS targets (keyword-encoded keys) — add a `// TRANSFORMS — clojure-ination (keyword-keyed)`
   comment near them; do NOT change them.

## Implementation sketch

1. `src/value/symbol_table.rs` — move struct + both impls. Imports (add what the compiler names):
   - `use crate::value::{EncodingCtx, Function};` (now in value/),
   - `use crate::runtime::EnumValue;` (TRANSITIONAL — EnumValue is Value-payload, moves at 251.2e),
   - `use crate::load::SourceLoader;`, `use crate::sigma::{SigmaFn, DefaultCoincidentSigma,
     DefaultPresenceSigma};`, `use crate::macros::MacroRegistry;`,
     `use crate::thread_io::{RuntimeServices, ThreadIO};`, `use crate::types::{TypeEnv, TypeExpr};`,
     `use crate::check::CheckEnv;` (if a field), `use crate::ast::WatAST;`, `use crate::span::Span;`,
     `std::collections::HashMap`, `std::sync::Arc`.
2. `src/value/mod.rs` — add `pub mod symbol_table;` + `pub use symbol_table::SymbolTable;`
3. `src/lib.rs` — move `SymbolTable` from `pub use runtime::{…}` to `pub use value::{…}`.
4. `src/runtime.rs` — delete the block; add `pub use crate::value::SymbolTable;` (uniform re-export;
   runtime uses it ×351 internally — zero-churn for the 17 external consumers; do NOT repoint).
5. `cargo build --release` → fix each path the compiler names. Then `cargo test --release --lib -p wat`.

## Blast radius

`src/value/` (+symbol_table.rs) + runtime.rs (delete block, add re-export) + lib.rs (re-export move).
With uniform re-export, NO consumer files should need changes. NO logic/signature edits.

## STOP triggers (rejection)

- STOP-1: any body/signature/behavior change beyond `use` paths, the uniform re-export, and the
  `// TRANSFORMS` comment → STOP, report.
- STOP-2: if a SymbolTable method calls into the eval loop / has runtime-internal coupling that can't
  move cleanly → STOP, report (the boundary is wrong).
- STOP-3: a borrow/visibility/cycle tangle → STOP, report (intra-crate cycles are fine — symbol_table.rs
  using crate::check::CheckEnv while check uses crate::value::SymbolTable is NOT a cycle problem).
- STOP-4: `cargo test --release --lib -p wat` ≠ **923 / 0 / 1** → STOP, report the delta.

## Done =

symbol_table.rs in src/value/; SymbolTable GONE from runtime.rs (def); uniform `pub use` re-export;
`cargo build` clean; `cargo test --release --lib -p wat` = 923/0/1; clippy clean in src/value/.
Do NOT commit — leave dirty. Report: before/after count + files touched.
