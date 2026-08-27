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

use wat::freeze::call_beside_value;
use wat::runtime::{RuntimeError, RuntimeErrorKind, Value, ValueSnapshot};

// just-eval (rubric): each `:user::cNN` zero-arg fn lives in the co-located fixture;
// drive it via `call_beside_value` and inspect the returned typed Value.
//
// arc 296 Stone M: `call_beside_value` already returns `Result<Value, RuntimeError>` — not a
// `StartupError` chain — so the real (never-flattened) error type here is `RuntimeError`
// itself; the "wrong Value shape" arm is minted as the same `RuntimeErrorKind::TypeMismatch`
// the runtime itself raises for this shape (see `src/assertion.rs::eval_opt_string`).
fn eval_bool(fn_name: &str) -> Result<bool, RuntimeError> {
    let full = format!(":user::{}", fn_name);
    match call_beside_value(file!(), &full)? {
        Value::bool(b) => Ok(b),
        other => Err(RuntimeError::new(
            wat::rust_caller_span!(),
            RuntimeErrorKind::TypeMismatch {
                op: full,
                expected: "bool",
                got: Box::new(ValueSnapshot::of(&other)),
            },
        )),
    }
}

fn eval_string(fn_name: &str) -> Result<String, RuntimeError> {
    let full = format!(":user::{}", fn_name);
    match call_beside_value(file!(), &full)? {
        Value::String(s) => Ok((*s).clone()),
        other => Err(RuntimeError::new(
            wat::rust_caller_span!(),
            RuntimeErrorKind::TypeMismatch {
                op: full,
                expected: "String",
                got: Box::new(ValueSnapshot::of(&other)),
            },
        )),
    }
}

#[test]
fn contract_01_annotated_if_recognized() {
    assert!(
        eval_bool("c01").expect("eval_bool"),
        "an annotated if (head :wat::core::if, child[2] = sym \"->\") is recognized"
    );
}

#[test]
fn contract_02_bare_if_not_annotated() {
    assert!(
        !eval_bool("c02").expect("eval_bool"),
        "a bare if is NOT annotated (child[2] is the then-branch, not the \"->\" symbol)"
    );
}

#[test]
fn contract_03_option_expect_not_annotated_if() {
    assert!(
        !eval_bool("c03").expect("eval_bool"),
        "Option/expect's `-> :T` must NOT be mistaken for an if annotation"
    );
}

#[test]
fn contract_04_fix_source_strips_if_annotation() {
    assert!(
        !eval_bool("c04a").expect("eval_bool"),
        "fix-source strips the annotation (result no longer recognized as annotated)"
    );
    assert!(
        eval_bool("c04b").expect("eval_bool"),
        "after strip, child[2] is the then-branch (int 1), proving -> :T was removed"
    );
}

#[test]
fn contract_05_fix_source_recurses() {
    let out = eval_string("c05").expect("c05: fix-source + write-forms of nested if");
    assert_eq!(
        out,
        include_str!("probe_arc258_stone3_fix_source__contract-05-nested-do-if.wat"),
        "fix-source must recurse into (do …) and strip the inner if's annotation"
    );
}

#[test]
fn contract_06_fix_source_preserves_option_expect() {
    let out = eval_string("c06").expect("c06: fix-source + write-forms of Option/expect");
    assert_eq!(
        out,
        include_str!("probe_arc258_stone3_fix_source__contract-06-preserves-option-expect.wat"),
        "fix-source must preserve Option/expect's -> :T annotation through the walk"
    );
}

#[test]
fn contract_07_end_to_end_clean_source() {
    let out = eval_string("c07-str").expect("c07-str: fix-source + write-forms of annotated if");
    assert_eq!(
        out,
        include_str!("probe_arc258_stone3_fix_source__contract-07-end-to-end-clean.wat"),
        "fix-source + write-forms: cleaned if must carry no -> and preserve head :wat::core::if"
    );
    assert!(
        eval_bool("c07-bool").expect("eval_bool"),
        "the cleaned form's head node is still the :wat::core::if keyword (verbatim token)"
    );
}

#[test]
fn contract_08_maturity_quasiquote_in_defn() {
    // FLAGGED maturity probe: can a PLAIN defn (not a macro) quasiquote a form?
    let result = call_beside_value(file!(), ":user::compute-c08");
    match result {
        Ok(Value::bool(true)) => { /* quasiquote-in-defn WORKS — no gap */ }
        other => panic!(
            "MATURITY FLAG: quasiquote inside a plain defn did not yield a List node — \
             functions may not be able to quasiquote (macros-only). Outcome: {other:?}"
        ),
    }
}
