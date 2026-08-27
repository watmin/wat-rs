//! Arc 272 rs-1 — defservice State is a STRUCT (not a record); the :durable Record is the EDN soul.
//!
//! > SUPERSEDED by arc 291 4b-ii: the original premise ("`:state` mints a RECORD") INVERTS.
//! > After 4b-ii, `:state`/`:ephemeral` mints a STRUCT (`defstruct`, non-portable by kind);
//! > `:durable` mints the `::Record` (EDN soul, crosses the wire). The file is rewritten to
//! > assert the new invariant while keeping its still-valid rejection tests. — arc 291 4b-ii-a.
//!
//! NEW INVARIANT: defservice's `:durable [fields]` mints `:<fqdn>::Record` (EDN, record? = true
//! on holon variant); `:<fqdn>::State` is a `defstruct` (record? = FALSE on the struct).
//! The holon `:durable-parent` now parents the `::Record` (not the State struct), so
//! `(record? (State/durable s))` is TRUE while `(record? s)` is FALSE.
//!
//! KEPT VALID: bare scalar in `:durable` is still rejected; unknown clause still rejected.
//!
//! Run: cargo test --release -p wat --test probe_arc272_rs1_state_must_be_record

use wat::freeze::{call_beside_value, startup_from_file, StartupError};
use wat::macros::{MacroError, MacroErrorKind};
use wat::runtime::{apply_function, Value};

#[test]
fn durable_field_vector_mints_record_soul_round_trips() {
    // Base (default) — :durable [fields] mints ::Record (the soul); ::State is a defstruct.
    // Wat source lives in the co-located fixture: probe_arc272_rs1_state_must_be_record.wat
    let got = call_beside_value(file!(), ":user::compute")
        .unwrap_or_else(|e| panic!("compute raised: {e:?}"));
    assert!(
        matches!(got, Value::i64(5)),
        "expected 5: :durable [count] minted ::Record (soul); State is a struct; \
         stop returned Record{{count 5}}, extracted via Record/count; got {got:?}"
    );
}

#[test]
fn durable_parent_holon_parents_the_durable_record_not_the_struct() {
    // Holon variant. Wat source: probe_arc272_rs1_state_must_be_record_holon.wat
    let world = startup_from_file("tests/services/probe_arc272_rs1_state_must_be_record_holon.wat")
        .expect("startup should succeed (rs-1 inverted: :durable-parent parents the ::Record, not the State struct)");
    let func = world
        .symbols()
        .get(":user::compute")
        .expect("no :user::compute in probe_arc272_rs1_state_must_be_record_holon.wat")
        .clone();
    let got = apply_function(func, vec![], world.symbols(), wat::rust_caller_span!())
        .unwrap_or_else(|e| panic!("compute raised: {e:?}"));
    assert!(
        matches!(got, Value::bool(true)),
        "expected true: :durable-parent :wat::holon::Record parents the ::Record (soul); \
         (record? (State/durable s)) must be TRUE (holon record); got {got:?}"
    );
}

#[test]
fn bare_type_keyword_state_is_rejected() {
    // NEGATIVE: bare type keyword in :durable slot. Wat source: probe_arc272_rs1_state_must_be_record_type_keyword.wat
    let result = startup_from_file("tests/services/probe_arc272_rs1_state_must_be_record_type_keyword.wat");
    wat::assert_startup_error!(result,
        StartupError::Macro(MacroError {
            kind: MacroErrorKind::ProgramBodyEvalFailed { macro_name, cause },
            ..
        }) if macro_name == ":wat::service::defservice"
            && matches!(
                &cause.kind,
                MacroErrorKind::MalformedTemplate { reason }
                    if reason == "defservice: :durable takes a FIELD VECTOR [name <- :Type …] \
                        — a bare type keyword / scalar durable is unexpressible; the durable IS \
                        the soul: a set of named fields that crosses the wire and survives \
                        hibernation"
            )
    );
}

#[test]
fn unknown_trailing_option_is_rejected() {
    // NEGATIVE: bogus trailing option. Wat source: probe_arc272_rs1_state_must_be_record_unknown_option.wat
    let result = startup_from_file("tests/services/probe_arc272_rs1_state_must_be_record_unknown_option.wat");
    wat::assert_startup_error!(result,
        StartupError::Macro(MacroError {
            kind: MacroErrorKind::ProgramBodyEvalFailed { macro_name, cause },
            ..
        }) if macro_name == ":wat::service::defservice"
            && matches!(
                &cause.kind,
                MacroErrorKind::MalformedTemplate { reason }
                    if reason == "defservice: unknown clause :bogus-option — recognized \
                        clauses: :durable :ephemeral :ops :init :hibernate :stop \
                        :durable-parent :satisfies :impls :peers :max-frame-bytes"
            )
    );
}
