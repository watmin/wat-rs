//! Strike (examinare disconfirming probe) — the two basic primitives wat lacks for
//! self-driving the corpus migration: directory enumeration + substring.
//!
//! RED at HEAD: neither `:wat::io::list-dir` nor `:wat::core::string::subs` exists (UnknownFunction).
//!
//! Run: `cargo test --release --test probe_arc251_io_string_primitives`

use wat::freeze::call_beside_value;
use wat::runtime::{RuntimeError, RuntimeErrorKind, Value, ValueSnapshot};

// just-eval (rubric): each `:user::cNN` zero-arg fn lives in the co-located fixture;
// drive it via `call_beside_value` and inspect the returned typed Value.
//
// arc 296 Stone M: `call_beside_value` already returns `Result<Value, RuntimeError>` — not a
// `StartupError` chain — so the real (never-flattened) error type here is `RuntimeError`
// itself; the "wrong Value shape" arm is minted as the same `RuntimeErrorKind::TypeMismatch`
// the runtime itself raises for this shape (see `src/assertion.rs::eval_opt_string`).
fn eval_string(fn_name: &str) -> Result<String, RuntimeError> {
    match call_beside_value(file!(), fn_name)? {
        Value::String(s) => Ok((*s).clone()),
        other => Err(RuntimeError::new(
            wat::rust_caller_span!(),
            RuntimeErrorKind::TypeMismatch {
                op: fn_name.into(),
                expected: "String",
                got: Box::new(ValueSnapshot::of(&other)),
            },
        )),
    }
}

fn eval_vec_strings(fn_name: &str) -> Result<Vec<String>, RuntimeError> {
    match call_beside_value(file!(), fn_name)? {
        Value::Vec(v) => v.iter().map(|e| match e {
            Value::String(s) => Ok((**s).clone()),
            other => Err(RuntimeError::new(
                wat::rust_caller_span!(),
                RuntimeErrorKind::TypeMismatch {
                    op: fn_name.into(),
                    expected: "String",
                    got: Box::new(ValueSnapshot::of(other)),
                },
            )),
        }).collect(),
        other => Err(RuntimeError::new(
            wat::rust_caller_span!(),
            RuntimeErrorKind::TypeMismatch {
                op: fn_name.into(),
                expected: "Vector",
                got: Box::new(ValueSnapshot::of(&other)),
            },
        )),
    }
}

#[test]
fn contract_01_subs_prefix() {
    assert_eq!(eval_string(":user::c01").expect("eval_string"), "hello");
}

#[test]
fn contract_02_subs_suffix() {
    assert_eq!(eval_string(":user::c02").expect("eval_string"), "world");
}

#[test]
fn contract_03_subs_empty_range() {
    assert_eq!(eval_string(":user::c03").expect("eval_string"), "");
}

#[test]
fn contract_04_list_dir_lists_known_file() {
    let entries = eval_vec_strings(":user::c04").expect("list-dir");
    assert!(
        entries.iter().any(|e| e == "wat/fix.wat"),
        "list-dir \"wat\" must contain the full path \"wat/fix.wat\"; got {entries:?}"
    );
}

#[test]
fn contract_05_list_dir_returns_full_paths() {
    let entries = eval_vec_strings(":user::c04").expect("list-dir");
    assert!(!entries.is_empty(), "list-dir must return entries");
    // rune:lint(loose-assert) — property over a variable set: the `wat/` directory's contents
    // grow as .wat files are added to the repo; the set and its enumeration order vary; the
    // contract is the path-prefix property, not any fixed exact set.
    assert!(
        entries.iter().all(|e| e.starts_with("wat/")),
        "every entry must be a FULL path under the listed dir; got {entries:?}"
    );
}
