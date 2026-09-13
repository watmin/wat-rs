//! the-dial-declares-its-peer — third `:peers` bijection check.
//!
//! A `connect` inside `:impls` of an `Address<S>`-typed field requires `S ∈ :peers`.
//! Holding an Address without dialing it is not this check; `:init` dials are not
//! this check. Pair of probes: the FINDING's variant A is now a compile-time
//! `macro-error`; the same shape with `:peers` + an `:ephemeral` peer still runs.

use wat::freeze::{startup_from_file, StartupError};
use wat::macros::{MacroError, MacroErrorKind};
use wat::runtime::{apply_function, Value};

#[test]
fn impls_connect_of_undeclared_address_field_is_refused_at_compile_time() {
    let r = startup_from_file("tests/services/probe_dial_declares_its_peer_refusal.wat");
    wat::assert_startup_error!(r,
        StartupError::Macro(MacroError {
            kind: MacroErrorKind::ProgramBodyEvalFailed { macro_name, cause },
            ..
        }) if macro_name == ":wat::service::defservice"
            && matches!(
                &cause.kind,
                MacroErrorKind::MalformedTemplate { reason }
                    if reason == "probe::front: :impls dials Peer<probe::Mid::Op,…::Reply> but \
                        surface :probe::Mid is not declared in :peers — add :peers [… \
                        :probe::Mid …] (the explicit s2s dependency DAG)"
            )
    );
}

#[test]
fn declared_handler_redial_of_address_field_still_compiles_and_runs() {
    let world = startup_from_file("tests/services/probe_dial_declares_its_peer_redial.wat")
        .expect("declared redial ( :peers + :ephemeral peer + impls connect of Address field ) must compile");
    let func = world
        .symbols()
        .get(":user::compute")
        .unwrap_or_else(|| panic!(":user::compute not registered"))
        .clone();
    let got = apply_function(func, vec![], world.symbols(), wat::rust_caller_span!())
        .unwrap_or_else(|e| panic!("declared handler redial raised: {e:?}"));
    assert!(
        matches!(got, Value::String(ref s) if s.as_str() == "pong:hi"),
        "expected handler redial of a declared :peers surface to round-trip; got {got:?}"
    );
}
