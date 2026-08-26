//! Strike 2 (examinare disconfirming probe) — `keyword/to-type-form`: the type-converter.
//!
//! RED at HEAD: the verb `:wat::core::keyword/to-type-form` does not exist (UnknownFunction).
//!
//! Run: `cargo test --release --test probe_arc251_keyword_to_type_form`

use wat::freeze::{call_beside_value, startup_beside};
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
fn contract_01_scalar() {
    assert_eq!(
        eval_string(":user::c01a"),
        Ok(include_str!("probe_arc251_keyword_to_type_form__contract-01a-scalar-i64.wat").into())
    );
    assert_eq!(
        eval_string(":user::c01b"),
        Ok(include_str!("probe_arc251_keyword_to_type_form__contract-01b-scalar-user.wat").into())
    );
}

// Arc 109 ③ — angle brackets are ILLEGAL for types now, so contracts 02/03/04/05/08 (below)
// no longer have a legal INPUT to feed `keyword/to-type-form`: the fixture built a keyword-
// NODE whose text embedded `Head<args>` (e.g. `":wat::core::Vector<wat::core::i64>"`) and
// proved the converter re-spells it in Clojure mode. There is no OTHER keyword-string
// spelling for a parametric type any more (the reference FORM `(Head :- [args])` only parses
// from a structural `WatAST::List`, never from a keyword's flat text) — so the conversion
// these contracts exercised is not merely untested, it is UNREACHABLE.
//
// STONE-the-last-mint — the refusal now fires ONE DOOR EARLIER than it used to. It used to be
// `keyword/to-type-form`'s own type-parser (`src/types.rs`, "angle-bracket parametric types
// are illegal") that caught these, because `keyword-node` itself was unwalled and happily
// built the angle-bearing keyword first. `keyword-node` is walled now (`angle_type_head_in_name`,
// `src/runtime.rs`/`src/edn/render.rs`), so the fixture's own `keyword-node` call refuses BEFORE
// `keyword/to-type-form` is ever reached — the mechanism moved from "the type parser rejects a
// parsed angle string" to "the minting primitive refuses to build the angle-bearing NAME at
// all". Each assertion below checks for THAT mechanism: the `keyword-node` head, and the
// minted-name wall's own reason text — not the (now unreachable) type-parser wording.
#[test]
fn contract_02_parametric() {
    let err = eval_string(":user::c02").expect_err("angle-bracket parametric keyword must be REFUSED");
    assert!( // rune:lint(loose-assert) — targeted substring: asserting the keyword-node minting wall fired, not the whole located error's structure
        err.contains(":wat::core::keyword-node") && err.contains("angle-bracket type parameters are illegal in a name"),
        "expected the keyword-node minting wall's reason; got: {err}"
    );
}

#[test]
fn contract_03_nested_parametric() {
    let err = eval_string(":user::c03").expect_err("angle-bracket parametric keyword must be REFUSED");
    assert!( // rune:lint(loose-assert) — targeted substring: asserting the keyword-node minting wall fired, not the whole located error's structure
        err.contains(":wat::core::keyword-node") && err.contains("angle-bracket type parameters are illegal in a name"),
        "expected the keyword-node minting wall's reason; got: {err}"
    );
}

#[test]
fn contract_04_type_var_stays_bare() {
    let err = eval_string(":user::c04").expect_err("angle-bracket parametric keyword must be REFUSED");
    assert!( // rune:lint(loose-assert) — targeted substring: asserting the keyword-node minting wall fired, not the whole located error's structure
        err.contains(":wat::core::keyword-node") && err.contains("angle-bracket type parameters are illegal in a name"),
        "expected the keyword-node minting wall's reason; got: {err}"
    );
}

#[test]
fn contract_05_multi_arg() {
    let err = eval_string(":user::c05").expect_err("angle-bracket parametric keyword must be REFUSED");
    assert!( // rune:lint(loose-assert) — targeted substring: asserting the keyword-node minting wall fired, not the whole located error's structure
        err.contains(":wat::core::keyword-node") && err.contains("angle-bracket type parameters are illegal in a name"),
        "expected the keyword-node minting wall's reason; got: {err}"
    );
}

#[test]
fn contract_06_tuple() {
    assert_eq!(
        eval_string(":user::c06"),
        Ok(include_str!("probe_arc251_keyword_to_type_form__contract-06-tuple.wat").into())
    );
}

#[test]
fn contract_07_empty_tuple_is_not_nil() {
    assert_eq!(
        eval_string(":user::c07"),
        Ok(include_str!("probe_arc251_keyword_to_type_form__contract-07-empty-tuple.wat").into())
    );
}

#[test]
fn contract_08_nested_tuple() {
    // Arc 109 ③ / STONE-the-last-mint — the fixture's `:(wat::core::Vector<T>,wat::core::i64)`
    // embeds an angle-bracket parametric INSIDE the native tuple spelling; same refusal
    // mechanism as contracts 02/03/04/05 above (see the block comment there) — the
    // `keyword-node` call refuses before `keyword/to-type-form` ever sees the string.
    let err = eval_string(":user::c08").expect_err("angle-bracket parametric keyword must be REFUSED");
    assert!( // rune:lint(loose-assert) — targeted substring: asserting the keyword-node minting wall fired, not the whole located error's structure
        err.contains(":wat::core::keyword-node") && err.contains("angle-bracket type parameters are illegal in a name"),
        "expected the keyword-node minting wall's reason; got: {err}"
    );
}

#[test]
fn contract_09_tuple_form_round_trips_as_a_type() {
    // The c09-f defn in the fixture uses `(wat.type/Tuple …)` as a param type.
    // Startup succeeding proves the parser handles it as a tuple type.
    let r = startup_beside(file!());
    assert!(r.is_ok(), "(wat.type/Tuple wat.type/i64 wat.type/String) must parse as a type; got {r:?}");
}
