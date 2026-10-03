//! Stone 255.86 amend 3 — the replacement values, pinned.
//!
//! The old names are retired, so these tests do not call them. Each string is
//! `label` plus `(:wat::core::str value)` of the replacement on the container
//! the old name accepted. HashMap and HashSet renders of more than one entry
//! follow hash order, so those pins are length and `get`/`contains?`.

use wat::freeze::{call_beside_value, startup_from_file};
use wat::Value;

const PINNED: &[&str] = &[
    "hm-len1",
    "hm-empty0true",
    "hm-empty1false",
    "hm-has-atrue",
    "hm-has-zfalse",
    "hm-get-a#wat.core/Option.Some {:value 1}",
    "hm-get-z#wat.core/Option.None {}",
    "hm-assoc-len2",
    "hm-assoc-a#wat.core/Option.Some {:value 1}",
    "hm-assoc-b#wat.core/Option.Some {:value 2}",
    "hm-dissoc{}",
    "hm-dissoc-miss{\"a\" 1}",
    "hm-keys[\"a\"]",
    "hm-values[1]",
    "pm-len1",
    "pm-empty0true",
    "pm-empty1false",
    "pm-has-atrue",
    "pm-has-zfalse",
    "pm-get-a#wat.core/Option.Some {:value 1}",
    "pm-get-z#wat.core/Option.None {}",
    "pm-assoc#wat.core/PersistentMap {\"a\" 1 \"b\" 2}",
    "pm-dissoc#wat.core/PersistentMap {}",
    "pm-dissoc-miss#wat.core/PersistentMap {\"a\" 1}",
    "pm-keys[\"a\"]",
    "pm-values[1]",
    "v-len2",
    "v-empty0true",
    "v-empty1false",
    "v-has-1true",
    "v-has-9false",
    "v-get-0#wat.core/Option.Some {:value 1}",
    "v-conj[1 2 3]",
    "v-concat[1 2]",
    "v-into-v[1 2]",
    "v-into-pv[1 2]",
    "pv-len2",
    "pv-empty0true",
    "pv-empty1false",
    "pv-has-1true",
    "pv-has-9false",
    "pv-get-0#wat.core/Option.Some {:value 1}",
    "pv-conj#wat.core/PersistentVector [1 2 3]",
    "pv-into-v#wat.core/PersistentVector [1 2]",
    "pv-into-pv#wat.core/PersistentVector [1 2]",
    "hs-len1",
    "hs-empty0true",
    "hs-empty1false",
    "hs-has-1true",
    "hs-has-9false",
    "hs-conj-len2",
    "hs-conj-1true",
    "hs-conj-2true",
    "ls-len1",
    "ls-empty0true",
    "ls-empty1false",
    "ls-has-2true",
    "ls-has-9false",
    "ls-get-0#wat.core/Option.Some {:value 2}",
    "ls-conj(1 2)",
    "list-ctor(1 2)",
    "char-ctor\\a",
    "to-hexff0010",
    "from-hex#wat.core/Option.Some {:value [255 0 16]}",
    "field-at9",
    "same-yestrue",
    "same-nofalse",
    "rec-assoc#probe/PinRec {:sk 9}"
];

#[test]
fn one_name_replacements_return_the_pinned_values() {
    let v = call_beside_value(file!(), ":probe::pins").expect("pins");
    let Value::Vec(xs) = v else {
        panic!(":probe::pins returned {v:?}");
    };
    let got: Vec<String> = xs
        .iter()
        .map(|x| match x {
            Value::String(s) => s.as_str().to_string(),
            other => panic!("pin element {other:?}"),
        })
        .collect();
    assert_eq!(got.len(), PINNED.len(), "pin count");
    for (i, (got, exp)) in got.iter().zip(PINNED.iter()).enumerate() {
        assert_eq!(got, exp, "pin {i}");
    }
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
