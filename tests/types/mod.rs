//! Test home for `src/types/` — the type system surface: structs, enums,
//! newtypes, typealiases, tuples, parametric types, restriction/destructure.
//!
//! Behavior-named (not arc-numbered): each file says what type feature it
//! exercises; arc lineage lives in git + the arc docs, not the filename.
//!
//! One `[[test]] name="types"` binary (Cargo.toml). The module list below is
//! GENERATED — drop a `.rs` here, run `scripts/gen-test-mods.sh`; the --check
//! gate (green-gate 1/4) fails loud if the list drifts, so no file is lost.
//!
//! Run: `cargo test --release -p wat --test types`

// BEGIN GENERATED MODS (scripts/gen-test-mods.sh) — do not hand-edit below
mod enums;
mod newtype;
mod parametric_enum;
mod struct_destructure;
mod struct_restricted;
mod structs;
mod tuple;
mod typealias;
mod typed_if_match;
mod uuid;
// END GENERATED MODS
