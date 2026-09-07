//! PROBE — arc 296 stone H-3: Option and Result keep their TYPE PARAMS across the move to wat.
//!
//! ⚠ GREEN AT HEAD, ON PURPOSE. This is not a disconfirming probe; it is the regression guard for
//! the one failure mode H-3 can have that nothing else would notice.
//!
//! Today `:wat::core::Option` / `:wat::core::Result` are Rust literals in `types.rs` carrying
//! `type_params: ["T"]` / `["T","E"]`. H-3 moves the source of truth into `wat/core.wat` and
//! registers via `wat_enum_register_from!` — which, measured 2026-09-06, hardcodes
//! `type_params: ::std::vec::Vec::new()` (`crates/wat-source-derive/src/lib.rs:727`, and `:497` for
//! the record sibling). The `defenum` parser DETECTS the `:- [T …]` binder but only to shift the
//! payload offset; it discards the vector.
//!
//! So the move can land, compile, and leave both types NON-PARAMETRIC — and the file's own header
//! already warns this class fails SILENTLY: *"getting the second one wrong is SILENT: the payload
//! shifts by two when a binder is present, so a scan with a fixed offset reads the binder's
//! `[T …]` vector as the first field list."*
//!
//! No live parametric type goes through those macros today (measured: 14 registered targets, zero
//! with a `:-` binder), so H-3 is the FIRST consumer and this guard is the first thing that would
//! catch its absence.

use wat::types::{TypeDef, TypeEnv};

fn params_of(env: &TypeEnv, path: &str) -> Vec<String> {
    match env.get(path) {
        Some(TypeDef::Enum(e)) => e.type_params.clone(),
        other => panic!("{path} must be a registered builtin enum; got {other:?}"),
    }
}

#[test]
fn option_and_result_are_registered_parametric() {
    let env = TypeEnv::with_builtins();
    assert_eq!(
        params_of(&env, ":wat::core::Option"),
        vec!["T".to_string()],
        "Option lost its type parameter — a non-parametric Option type-checks `(Option :- [i64])` \
         as an arity error and silently widens every Some/None site"
    );
    assert_eq!(
        params_of(&env, ":wat::core::Result"),
        vec!["T".to_string(), "E".to_string()],
        "Result lost its type parameters, in declaration ORDER — [T, E], not [E, T]"
    );
}
