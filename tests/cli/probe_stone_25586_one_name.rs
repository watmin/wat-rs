//! Stone 255.86 amend 4 — the replacement values, pinned as data.
//!
//! `:probe::hold` is `:wat::test::assert-eq` of each Equatable result against the
//! value written as a value. PersistentMap is not a member of Equatable, so
//! `:probe::pm-got` and `:probe::pm-want` are the three map results compared here
//! by `Value`'s `PartialEq`.

use wat::freeze::{call_beside_value, startup_from_file};
use wat::Value;

#[test]
fn one_name_replacements_return_the_pinned_values() {
    call_beside_value(file!(), ":probe::hold").expect("pins");
    let got = call_beside_value(file!(), ":probe::pm-got").expect("pm got");
    let want = call_beside_value(file!(), ":probe::pm-want").expect("pm want");
    assert_eq!(got, want);
    let Value::Vec(xs) = &got else {
        panic!(":probe::pm-got returned {got:?}");
    };
    assert_eq!(xs.len(), 3, "assoc, dissoc, dissoc-miss");
}

#[test]
fn formattable_and_reject_queried_in_the_type_registry() {
    let form = startup_from_file("tests/types/probe_diagnostic_defprotocol_dispatch_p1.wat")
        .expect("p1 loads");
    let formattable = form.types().contains(":myapp::Formattable");
    let reject_load = startup_from_file("wat-tests/holon/Reject.wat");
    let reject = reject_load
        .as_ref()
        .ok()
        .map(|w| w.types().contains(":wat-tests::holon::Reject"));
    let err = reject_load
        .as_ref()
        .err()
        .map(|e| format!("{e:?}"))
        .unwrap_or_default();
    assert!(
        !formattable && reject == Some(false),
        "formattable={formattable} reject={reject:?} startup={err}"
    );
}
