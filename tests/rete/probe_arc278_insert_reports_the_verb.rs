//! Arc 278 conferre L2-2 — `insert` reports the verb the user wrote.
//!
//! `require_record_fact` takes `op` so the same check serves `insert` and
//! `insert-all`. The 3+-arity `insert` entry used to feed it a hardcoded
//! `insert-all`. Both arms assert the structured `:op` field, not a substring
//! of the rendered message.
//!
//! Run: cargo test --release -p wat --test rete probe_arc278_insert_reports_the_verb

use wat::freeze::call_beside_value;
use wat::runtime::RuntimeErrorKind;

fn type_mismatch_op(entry: &str) -> String {
    let err = match call_beside_value(file!(), entry) {
        Ok(v) => panic!(
            "{entry} returned {v:?} — expected a TypeMismatch on a non-Record fact"
        ),
        Err(e) => e,
    };
    match err.kind() {
        RuntimeErrorKind::TypeMismatch { op, .. } => op.clone(),
        other => panic!("{entry} raised {other:?}, not TypeMismatch"),
    }
}

#[test]
fn insert_reports_insert() {
    assert_eq!(
        type_mismatch_op(":user::insert-non-record").as_str(),
        ":wat::rete::insert"
    );
}

#[test]
fn insert_all_reports_insert_all() {
    assert_eq!(
        type_mismatch_op(":user::insert-all-non-record").as_str(),
        ":wat::rete::insert-all"
    );
}
