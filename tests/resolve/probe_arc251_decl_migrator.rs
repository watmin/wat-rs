//! Forward-proof probe — arc 251 Stone 251.5 / Slice 4.1: the declaration migrator.
//!
//! Verifies `fix-form` (`:migrate::fix-form`) closes the two gaps left by
//! `fix.wat`'s `fix-source` in DECLARATION and DEFN forms.
//!
//! The migrator code was baked verbatim into the co-located fixture
//! `probe_arc251_decl_migrator.wat` at migration time (wat-migrate/fix-decl.wat
//! is non-blessed and retires with the hard-cut; cannot be auto-loaded).
//!
//! Cases:
//!   C01 (gap A)  — typealias bare type-slot → `keyword/to-type-form`, not symbol.
//!   C02 (gap B)  — defn with `<T>` name → PLAIN symbol (`<T>` dropped).
//!   C03          — generic decl name + parametric target.
//!   C04          — preservation: user-type targets already correct under fix-source.
//!   C05          — newtype: bare type-slot handled; name-fix on plain name.
//!   C06          — typeunion: core member vector uses type form.
//!   C07          — typeunion: user-type member vector preserved.
//!   C08          — defenum: variant tags stay keywords, field types converted.
//!
//! Run: `cargo test --release --test probe_arc251_decl_migrator`

use wat::freeze::call_beside;
use wat::runtime::Value;

// just-eval (rubric): each `:user::cNN` zero-arg fn lives in the co-located fixture;
// drive it via `call_beside` and inspect the returned typed String.
fn eval_string(fn_name: &str) -> Result<String, String> {
    let full = format!(":user::{}", fn_name);
    match call_beside(file!(), &full).map_err(|e| format!("eval {fn_name}: {e:?}"))? {
        Value::String(s) => Ok((*s).clone()),
        other => Err(format!("non-string from {fn_name}: {other:?}")),
    }
}

fn normalize(s: String) -> String {
    s.split_whitespace().collect::<Vec<_>>().join(" ")
}

#[test]
fn c01_typealias_bare_type_slot_uses_type_form() {
    let got = normalize(eval_string("c01").expect("C01 fix-form"));
    assert_eq!(
        got,
        include_str!("probe_arc251_decl_migrator__c01-typealias-type-slot.wat"),
        "C01 (gap A): typealias core-scalar type-slot must render as wat.type/i64"
    );
}

#[test]
fn c02_defn_generic_name_drops_type_params() {
    let got = normalize(eval_string("c02").expect("C02 fix-form"));
    assert_eq!(
        got,
        include_str!("probe_arc251_decl_migrator__c02-defn-drop-type-params.wat"),
        "C02 (gap B, LOAD-BEARING): defn name with <T> must be a plain symbol, <T> dropped"
    );
}

#[test]
fn c03_generic_decl_name_and_parametric_target() {
    let got = normalize(eval_string("c03").expect("C03 fix-form"));
    assert_eq!(
        got,
        include_str!("probe_arc251_decl_migrator__c03-generic-decl-parametric.wat"),
        "C03: generic typealias name stripped + parametric target converted"
    );
}

#[test]
fn c04_user_type_target_preserved() {
    let got = normalize(eval_string("c04").expect("C04 fix-form"));
    assert_eq!(
        got,
        include_str!("probe_arc251_decl_migrator__c04-user-type-preserved.wat"),
        "C04 (preservation): user-type target must preserve namespace"
    );
}

#[test]
fn c05_newtype_type_slot_handled() {
    let got = normalize(eval_string("c05").expect("C05 fix-form"));
    assert_eq!(
        got,
        include_str!("probe_arc251_decl_migrator__c05-newtype-type-slot.wat"),
        "C05 (newtype): bare type-slot in newtype must use keyword/to-type-form"
    );
}

#[test]
fn c06_typeunion_core_member_vector_uses_type_form() {
    let got = normalize(eval_string("c06").expect("C06 fix-form"));
    assert_eq!(
        got,
        include_str!("probe_arc251_decl_migrator__c06-typeunion-core-members.wat"),
        "C06: typeunion core-scalar members must render as wat.type/ in the member vector"
    );
}

#[test]
fn c07_typeunion_user_member_vector_preserved() {
    let got = normalize(eval_string("c07").expect("C07 fix-form"));
    assert_eq!(
        got,
        include_str!("probe_arc251_decl_migrator__c07-typeunion-user-members.wat"),
        "C07: typeunion user-type members must preserve their namespace"
    );
}

#[test]
fn c08_defenum_variant_tags_stay_fields_converted() {
    let got = normalize(eval_string("c08").expect("C08 fix-form"));
    assert_eq!(
        got,
        include_str!("probe_arc251_decl_migrator__c08-defenum-variant-tags.wat"),
        "C08 (defenum): variant tags stay keywords; field types convert via the arrow rule"
    );
}
