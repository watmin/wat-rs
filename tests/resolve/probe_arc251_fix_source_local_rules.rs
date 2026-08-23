//! Strike 3 (examinare disconfirming probe) — fix-source's position-aware LOCAL rules.
//!
//! RED at HEAD: fix.wat only does {strip-if, head-rule} — no arrows, no post-arrow/structural types.
//!
//! Run: `cargo test --release --test probe_arc251_fix_source_local_rules`

use wat::freeze::call_beside_value;
use wat::runtime::Value;

// just-eval (rubric): each `:user::cNN` zero-arg fn lives in the co-located fixture;
// drive it via `call_beside_value` and inspect the returned typed String.
fn eval_string(fn_name: &str) -> Result<String, String> {
    match call_beside_value(file!(), fn_name).map_err(|e| format!("eval: {e:?}"))? {
        Value::String(s) => Ok((*s).clone()),
        other => Err(format!("non-string: {other:?}")),
    }
}

#[test]
fn contract_01_arrow_in_binder() {
    assert_eq!(
        eval_string(":user::c01"),
        Ok(include_str!("probe_arc251_fix_source_local_rules__contract-01-arrow-in-binder.wat").into())
    );
}

#[test]
fn contract_02_post_arrow_scalar_type() {
    assert_eq!(
        eval_string(":user::c02"),
        Ok(include_str!("probe_arc251_fix_source_local_rules__contract-02-post-arrow-scalar.wat").into())
    );
}

#[test]
fn contract_03_structural_parametric_type() {
    // Arc 109 ③ — angle brackets are ILLEGAL for types now; the fixture's C03 source embeds
    // `Vector<i64>`, which `wat/fix.wat`'s converter renders through the SAME walled
    // `keyword/to-type-form` `probe_arc251_keyword_to_type_form.rs`'s contracts 02-05/08 hit —
    // there is no other keyword-string spelling for a parametric type to migrate FROM any
    // more, so the refusal itself is the coverage.
    let err = eval_string(":user::c03").expect_err("angle-bracket parametric type must be REFUSED");
    assert!( // rune:lint(loose-assert) — targeted substring: asserting the angle-bracket wall fired, not the whole located TypeError's structure
        err.contains("angle-bracket parametric types are illegal"),
        "expected the angle-bracket wall's reason; got: {err}"
    );
}

#[test]
fn contract_04_head_still_inverts() {
    assert_eq!(
        eval_string(":user::c04"),
        Ok(include_str!("probe_arc251_fix_source_local_rules__contract-04-head-inverts.wat").into())
    );
}

#[test]
fn contract_05_full_fn_literal() {
    assert_eq!(
        eval_string(":user::c05"),
        Ok(include_str!("probe_arc251_fix_source_local_rules__contract-05-full-fn-literal.wat").into()),
        "head inverts, binder + return arrows -> :-, both types -> wat.type/, in one pass"
    );
}

#[test]
fn contract_06_less_than_operator_is_not_a_type() {
    assert_eq!(
        eval_string(":user::c06a"),
        Ok(include_str!("probe_arc251_fix_source_local_rules__contract-06a-less-than.wat").into())
    );
    assert_eq!(
        eval_string(":user::c06b"),
        Ok(include_str!("probe_arc251_fix_source_local_rules__contract-06b-less-equal.wat").into())
    );
}

#[test]
fn contract_07_greater_than_operator() {
    assert_eq!(
        eval_string(":user::c07"),
        Ok(include_str!("probe_arc251_fix_source_local_rules__contract-07-greater-than.wat").into())
    );
}
