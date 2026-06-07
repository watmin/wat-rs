//! Test home for `src/types/` — the type system surface: structs, enums,
//! newtypes, typealiases, tuples, parametric types, restriction/destructure.
//!
//! Behavior-named (not arc-numbered): each file says what type feature it
//! exercises; arc lineage lives in git + the arc docs, not the filename.
//!
//! One `[[test]] name="types"` binary (Cargo.toml). The module list is GENERATED
//! by build.rs into OUT_DIR — drop a `.rs` here and it is compiled automatically;
//! no manual step, no drift gate.
//!
//! Run: `cargo test --release -p wat --test types`
include!(concat!(env!("OUT_DIR"), "/types_mods.rs"));
