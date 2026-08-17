//! 293.W.2e — `address-wire?` on a thread Address is false; on a process Address is true.
//!
//! RED at HEAD: startup fails (`UnknownFunction` `:wat::kernel::address-wire?`).
//! GREEN after the stone: `(:probe::compute)` = `[false true]`.
//!
//! cargo nextest run --release -E 'test(address_wire_is_false_on_thread_true_on_process)'

use wat::check::error::{CheckErrorKind, CheckErrors};
use wat::freeze::{call_beside_value, startup_from_file, StartupError};
use wat::runtime::Value;

#[test]
fn address_wire_is_false_on_thread_true_on_process() {
    let got = call_beside_value(file!(), ":probe::compute")
        .unwrap_or_else(|e| panic!("probe::compute must eval; got {e:?}"));
    match got {
        Value::Vec(ref v) => {
            let bools: Vec<bool> = v
                .iter()
                .map(|tv| match tv {
                    Value::bool(b) => *b,
                    other => panic!("expected bool elements, got {other:?}"),
                })
                .collect();
            assert_eq!(
                bools,
                vec![false, true],
                "thread address is shared memory (false); process address is a wire (true)"
            );
        }
        other => panic!("expected Vector<bool>, got {other:?}"),
    }
}

/// Negative control: `address-wire?` on an i64 is a TypeMismatch naming `Address<S,R>`.
///
/// cargo nextest run --release -E 'test(address_wire)'
#[test]
fn address_wire_non_address_is_type_mismatch() {
    let err = startup_from_file("tests/comms/probe_arc293_W2e_address_wire.wat.bad")
        .expect_err("address-wire? on i64 must fail check");
    let StartupError::Check(CheckErrors(errs)) = &err else {
        panic!("expected a type-check error, got {err:?}");
    };
    wat::assert_check_error_present!(errs,
        CheckErrorKind::TypeMismatch { expected, callee, .. }
            if callee == ":wat::kernel::address-wire?"
            && expected.contains("Address<S,R>"));
}
