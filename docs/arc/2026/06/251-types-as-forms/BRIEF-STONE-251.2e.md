# BRIEF — Stone 251.2e — LIFT the Value cluster → `src/value/value.rs` (the foundational, last lift)

## The work

Lift the foundational `Value` enum + its payloads + the Clause cluster + all its impls out of flat
`src/runtime.rs` into `src/value/value.rs`. This is the LAST lift of the value/ home (the vigilia
ward follows separately). PURE STRUCTURAL MOVE — no behavior change. Baseline **923 / 0 / 1**,
identical after. Uniform re-export (Value is used ×2156 internally — definitely re-exported).

## Read in order (the Value cluster, runtime.rs ~383–1410, one contiguous region)

- `pub enum Value` (383) — its variants reference MANY external types (see imports below).
- Clause cluster: `pub struct Clause` (710), `pub struct ClauseSet` (739), `pub struct ClauseAttempt`
  (754), `pub enum ClauseFailureReason` (767).
- `fn sequence_eq` (781) + `fn hash_sequence` (808) — private free fns serving Value's Eq/Hash; move
  with Value, stay private (no re-export).
- `impl PartialEq for Value` (844), `impl Eq for Value` (948), `impl std::hash::Hash for Value` (978).
- payloads: `pub struct StructValue` (1175), `pub struct EnumValue` (1194), `pub enum SpawnOutcome`
  (1210), `pub enum ProgramHandleInner` (1255).
- `impl Value` (1260 → its closing brace, ~1410) — contains `type_name` (1261) + `declared_type_name`
  (1342): **clojure-ination TRANSFORMS targets** (return keyword-encoded type strings) — add a
  `// TRANSFORMS — clojure-ination (keyword type-name strings)` comment, do NOT change them.
- **Also grep `impl .* for Value` + `impl Value`** to catch any other Value impls (Display/HolonRep/
  From/etc.) and move them ALL.

## Implementation sketch

1. `src/value/value.rs` — move the whole cluster. Wide import list (Value's variants); add what the
   compiler names, expected set:
   - `use crate::value::{Function, Environment};` (now in value — Value variants/closures reference them)
   - `use crate::types::TypeExpr;` (Clause carries TypeExpr)
   - holon: `use holon::{HolonAST, Vector, OnlineSubspace, Reckoner, Engram, EngramLibrary};`
   - `use crate::typed_channel::{SenderInner, ReceiverInner};`, `use crate::fork::ChildHandleInner;`,
     `use crate::rust_deps::{RustOpaqueInner, ThreadOwnedCell};`, `use crate::hologram::Hologram;`,
     `use crate::io::{WatReader, WatWriter};`, `use crate::ast::WatAST;`, `use crate::span::Span;`,
     `crossbeam_channel`, `chrono`, `uuid::Uuid`, std collections/sync/hash as needed.
2. `src/value/mod.rs` — add `pub mod value;` + `pub use value::{Value, StructValue, EnumValue,
   SpawnOutcome, ProgramHandleInner, Clause, ClauseSet, ClauseAttempt, ClauseFailureReason};`
3. `src/lib.rs` — move `Value, StructValue` (and any of these in `pub use runtime::{…}`) to `pub use value::{…}`.
4. `src/runtime.rs` — delete the cluster; add `pub use crate::value::{Value, StructValue, EnumValue,
   SpawnOutcome, ProgramHandleInner, Clause, ClauseSet, ClauseAttempt, ClauseFailureReason};` (uniform
   re-export; zero-churn for ×2156 internal + 75 external consumers; do NOT repoint).
5. **Flip the now-resolvable transitional imports in the value/ submodules** (Value etc. are home now):
   - `src/value/observe.rs:8` `use crate::runtime::Value;` → `use crate::value::Value;`
   - `src/value/signal.rs:8` `use crate::runtime::{ClauseAttempt, ClauseFailureReason, Value};` →
     `use crate::value::{ClauseAttempt, ClauseFailureReason, Value};`
   - `src/value/symbol_table.rs:10` `use crate::runtime::{EnumValue, Value};` → `use crate::value::{EnumValue, Value};`
6. `cargo build --release` → fix each path the compiler names. Then `cargo test --release --lib -p wat`.

## STOP triggers (rejection)

- STOP-1: any body/signature/behavior change beyond `use` paths, the uniform re-export, the submodule
  import flips, and the `// TRANSFORMS` comment → STOP, report.
- STOP-2: if an `impl Value` method has runtime-eval-loop coupling that can't move cleanly → STOP, report
  (the method may belong in the eval home, not value/ — surface it, don't force).
- STOP-3: a borrow/visibility/cycle tangle not resolved by imports → STOP, report (intra-crate cycles fine).
- STOP-4: `cargo test --release --lib -p wat` ≠ **923 / 0 / 1** → STOP, report the delta.

## Done =

value.rs in src/value/; the Value cluster GONE from runtime.rs (defs); all 3 value/ submodule
transitional `crate::runtime` imports flipped to `crate::value`; uniform `pub use` re-export;
`cargo build` clean; `cargo test --release --lib -p wat` = 923/0/1; clippy clean in src/value/.
Do NOT commit — leave dirty. Report: before/after count + files touched + confirm the 3 submodule flips.
