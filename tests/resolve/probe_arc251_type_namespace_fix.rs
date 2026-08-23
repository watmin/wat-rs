//! Strike (examinare probe) — 4.0: the faithful type-NAMESPACE fix (intueri-named).
//!
//! RED at HEAD: C03/C04/C05/C06 fail (mis-rendering).
//!
//! Run: `cargo test --release --test probe_arc251_type_namespace_fix`

use wat::freeze::call_beside_value;
use wat::runtime::Value;

// just-eval (rubric): each `:user::cNN` zero-arg fn lives in the co-located fixture;
// drive it via `call_beside_value` and inspect the returned typed Value.
fn eval_string(fn_name: &str) -> Result<String, String> {
    match call_beside_value(file!(), fn_name).map_err(|e| format!("eval: {e:?}"))? {
        Value::String(s) => Ok((*s).clone()),
        other => Err(format!("non-string: {other:?}")),
    }
}

#[test]
fn c01_core_fqdn_scalar_stays_wat_type() {
    assert_eq!(
        eval_string(":user::c01a"),
        Ok(include_str!("probe_arc251_type_namespace_fix__c01a-core-fqdn-i64.wat").into())
    );
    assert_eq!(
        eval_string(":user::c01b"),
        Ok(include_str!("probe_arc251_type_namespace_fix__c01b-core-fqdn-string.wat").into())
    );
}

#[test]
fn c02_core_parametric_stays_wat_type() {
    // Arc 109 ③ — angle brackets are ILLEGAL for types now; the fixture's C02 keyword-node
    // embeds `Vector<i64>` — there is no other keyword-string spelling for a parametric
    // type any more, so the refusal is the coverage. STONE-the-last-mint — `keyword-node`
    // itself is walled now (`angle_type_head_in_name`), the SAME mechanism
    // `probe_arc251_keyword_to_type_form.rs`'s contracts 02-05/08 hit, so this refuses one
    // door earlier than it used to (`keyword-node`, before `keyword/to-type-form` ever runs).
    let err = eval_string(":user::c02").expect_err("angle-bracket parametric keyword must be REFUSED");
    assert!( // rune:lint(loose-assert) — targeted substring: asserting the keyword-node minting wall fired, not the whole located error's structure
        err.contains(":wat::core::keyword-node") && err.contains("angle-bracket type parameters are illegal in a name"),
        "expected the keyword-node minting wall's reason; got: {err}"
    );
}

#[test]
fn c03_bare_legacy_primitive_renders_core() {
    assert_eq!(
        eval_string(":user::c03a"),
        Ok(include_str!("probe_arc251_type_namespace_fix__c03a-legacy-i64.wat").into())
    );
    assert_eq!(
        eval_string(":user::c03b"),
        Ok(include_str!("probe_arc251_type_namespace_fix__c03b-legacy-string.wat").into())
    );
    assert_eq!(
        eval_string(":user::c03c"),
        Ok(include_str!("probe_arc251_type_namespace_fix__c03c-legacy-bool.wat").into())
    );
}

#[test]
fn c04_user_type_preserves_namespace() {
    assert_eq!(
        eval_string(":user::c04"),
        Ok(include_str!("probe_arc251_type_namespace_fix__c04-user-type-namespace.wat").into())
    );
}

#[test]
fn c05_distinct_user_types_do_not_collide() {
    let a = eval_string(":user::c05a");
    let b = eval_string(":user::c05b");
    assert!(a.is_ok() && b.is_ok(), "both must render: {a:?} {b:?}");
    assert_ne!(a, b, "distinct types must NOT render to the same faithful name (collision)");
}

#[test]
fn c06_user_type_two_segment_preserves_namespace() {
    assert_eq!(
        eval_string(":user::c06"),
        Ok(include_str!("probe_arc251_type_namespace_fix__c06-user-type-two-segment.wat").into())
    );
}

#[test]
fn c07_type_var_stays_bare() {
    assert_eq!(
        eval_string(":user::c07a"),
        Ok(include_str!("probe_arc251_type_namespace_fix__c07a-type-var-t.wat").into())
    );
    assert_eq!(
        eval_string(":user::c07b"),
        Ok(include_str!("probe_arc251_type_namespace_fix__c07b-type-var-k.wat").into())
    );
}

#[test]
fn c08_bare_head_parametric_errors_cleanly() {
    assert!(
        call_beside_value(file!(), ":user::c08a").is_err(),
        "bare parametric head must error cleanly, not panic"
    );
    assert!(
        call_beside_value(file!(), ":user::c08b").is_err(),
        "higher-kinded head must error cleanly, not panic"
    );
}

#[test]
fn c09_trailing_colons_path_errors_cleanly() {
    assert!(
        call_beside_value(file!(), ":user::c09").is_err(),
        "trailing-`::` path must error cleanly, not panic"
    );
}
