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
    // Arc 109 ③ (a PRIOR stone) walled the TYPE PARSER, so C03 used to prove `Vector<i64>`
    // got refused by `keyword/to-type-form` with "angle-bracket parametric types are
    // illegal". Arc 109 wave 2 (THIS stone) walls the LEXER, one door earlier:
    // `:user::topform`'s `read-string` on `"[x <- :wat::core::Vector<wat::core::i64>]"`
    // now fails at the READER, before `fix-source`/`keyword/to-type-form` ever run — and
    // `:user::topform`'s `ReadOutcome::Malformed` arm calls `(:wat::core::Error/message
    // __cause)` on a `:wat::edn::ForeignRecord` with no `message` surface method, so the
    // read-string path crashes with an unrelated `UnknownFunction` before it can report
    // the lex refusal cleanly. That crash is a SEPARATE, already-documented defect
    // (DESIGN-STONE-annihilate-the-angle-bracket.md's sequencing section; identical to
    // `probe_arc251_decl_migrator.rs`'s `c03`) — out of this stone's boundary.
    //
    // Class 3 (b) — re-pointed as a refusal control on the mechanism that now actually
    // fires. Also (c): the parametric-target case `keyword/to-type-form` exists to parse
    // is unreachable input from here on — purge candidate for the sibling stone.
    let err = eval_string(":user::c03").expect_err("angle-bracket source must fail to read");
    assert!( // rune:lint(loose-assert) — targeted substring: the read-string crash's mechanism, not the whole located error's structure
        err.contains("ForeignRecord") && err.contains("message"),
        "expected the read-string/ForeignRecord crash (see comment above); got: {err}"
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
