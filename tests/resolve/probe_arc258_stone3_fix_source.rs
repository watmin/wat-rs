//! FM 2-bis probe — arc 258 Stone 258.3: `fix-source`'s if-annotation strip rule.
//!
//! C01: annotated-if? is TRUE on an annotated if.
//! C02: annotated-if? is FALSE on a bare if.
//! C03: annotated-if? is FALSE on `Option/expect -> :T …` (the guard).
//! C04: fix-source STRIPS — result not annotated, child[2] is then-branch (int).
//! C05: fix-source RECURSES — annotated if inside `(do …)` is stripped.
//! C06: fix-source PRESERVES `Option/expect -> :T` under recursion.
//! C07: end-to-end via write-forms — cleaned source carries no `->`, head still :if.
//! C08: quasiquote inside a plain `defn` (maturity probe).
//!
//! Run: `cargo test --release --test probe_arc258_stone3_fix_source`

use wat::freeze::{eval_in_frozen, startup_beside};
use wat::runtime::{Environment, Value};

fn eval_bool(fn_name: &str) -> Result<bool, String> {
    let world = startup_beside(file!()).map_err(|e| format!("startup: {e:?}"))?;
    let call = format!("(:user::{})", fn_name);
    let ast = wat::parse_one!(&call).expect("parse");
    match eval_in_frozen(&ast, &world, &Environment::new())
        .map(|tv| tv.value_owned())
        .map_err(|e| format!("eval: {e:?}"))?
    {
        Value::bool(b) => Ok(b),
        other => Err(format!("non-bool: {other:?}")),
    }
}

fn eval_string(fn_name: &str) -> Result<String, String> {
    let world = startup_beside(file!()).map_err(|e| format!("startup: {e:?}"))?;
    let call = format!("(:user::{})", fn_name);
    let ast = wat::parse_one!(&call).expect("parse");
    match eval_in_frozen(&ast, &world, &Environment::new())
        .map(|tv| tv.value_owned())
        .map_err(|e| format!("eval: {e:?}"))?
    {
        Value::String(s) => Ok((*s).clone()),
        other => Err(format!("non-string: {other:?}")),
    }
}

#[test]
fn contract_01_annotated_if_recognized() {
    assert_eq!(
        eval_bool("c01"),
        Ok(true),
        "an annotated if (head :wat::core::if, child[2] = sym \"->\") is recognized"
    );
}

#[test]
fn contract_02_bare_if_not_annotated() {
    assert_eq!(
        eval_bool("c02"),
        Ok(false),
        "a bare if is NOT annotated (child[2] is the then-branch, not the \"->\" symbol)"
    );
}

#[test]
fn contract_03_option_expect_not_annotated_if() {
    assert_eq!(
        eval_bool("c03"),
        Ok(false),
        "Option/expect's `-> :T` must NOT be mistaken for an if annotation"
    );
}

#[test]
fn contract_04_fix_source_strips_if_annotation() {
    assert_eq!(
        eval_bool("c04a"),
        Ok(false),
        "fix-source strips the annotation (result no longer recognized as annotated)"
    );
    assert_eq!(
        eval_bool("c04b"),
        Ok(true),
        "after strip, child[2] is the then-branch (int 1), proving -> :T was removed"
    );
}

#[test]
fn contract_05_fix_source_recurses() {
    let out = eval_string("c05").expect("c05: fix-source + write-forms of nested if");
    assert!(
        !out.contains("->"),
        "fix-source recurses into (do …) and strips the inner if's annotation; got: {out}"
    );
}

#[test]
fn contract_06_fix_source_preserves_option_expect() {
    let out = eval_string("c06").expect("c06: fix-source + write-forms of Option/expect");
    assert!(
        out.contains("->"),
        "Option/expect's `-> :T` must be preserved through the walk; got: {out}"
    );
}

#[test]
fn contract_07_end_to_end_clean_source() {
    let out = eval_string("c07-str").expect("c07-str: fix-source + write-forms of annotated if");
    assert!(!out.contains("->"), "cleaned if carries no `->`; got: {out}");
    assert_eq!(
        eval_bool("c07-bool"),
        Ok(true),
        "the cleaned form's head node is still the :wat::core::if keyword (verbatim token)"
    );
}

#[test]
fn contract_08_maturity_quasiquote_in_defn() {
    // FLAGGED maturity probe: can a PLAIN defn (not a macro) quasiquote a form?
    let world = startup_beside(file!()).expect("startup: fixture must load for C08");
    let ast = wat::parse_one!("(:user::compute-c08)").expect("parse");
    let result = eval_in_frozen(&ast, &world, &Environment::new())
        .map(|tv| tv.value_owned());
    match result {
        Ok(Value::bool(true)) => { /* quasiquote-in-defn WORKS — no gap */ }
        other => panic!(
            "MATURITY FLAG: quasiquote inside a plain defn did not yield a List node — \
             functions may not be able to quasiquote (macros-only). Outcome: {other:?}"
        ),
    }
}
