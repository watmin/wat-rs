//! Strike (examinare disconfirming probe) — the two basic primitives wat lacks for
//! self-driving the corpus migration: directory enumeration + substring.
//!
//! RED at HEAD: neither `:wat::io::list-dir` nor `:wat::core::string::subs` exists (UnknownFunction).
//!
//! Run: `cargo test --release --test probe_arc251_io_string_primitives`

use wat::freeze::{eval_in_frozen, startup_beside};
use wat::runtime::{Environment, Value};

fn eval_string(world: &wat::freeze::FrozenWorld, call: &str) -> Result<String, String> {
    let ast = wat::parse_one!(call).expect("parse");
    match eval_in_frozen(&ast, world, &Environment::new())
        .map(|tv| tv.value_owned())
        .map_err(|e| format!("eval: {e:?}"))?
    {
        Value::String(s) => Ok((*s).clone()),
        other => Err(format!("non-string: {other:?}")),
    }
}

fn eval_vec_strings(world: &wat::freeze::FrozenWorld, call: &str) -> Result<Vec<String>, String> {
    let ast = wat::parse_one!(call).expect("parse");
    match eval_in_frozen(&ast, world, &Environment::new())
        .map(|tv| tv.value_owned())
        .map_err(|e| format!("eval: {e:?}"))?
    {
        Value::Vec(v) => v.iter().map(|e| match e {
            Value::String(s) => Ok((**s).clone()),
            other => Err(format!("non-string entry: {other:?}")),
        }).collect(),
        other => Err(format!("non-vector: {other:?}")),
    }
}

#[test]
fn contract_01_subs_prefix() {
    let world = startup_beside(file!()).expect("startup");
    assert_eq!(eval_string(&world, "(:user::c01)"), Ok("hello".into()));
}

#[test]
fn contract_02_subs_suffix() {
    let world = startup_beside(file!()).expect("startup");
    assert_eq!(eval_string(&world, "(:user::c02)"), Ok("world".into()));
}

#[test]
fn contract_03_subs_empty_range() {
    let world = startup_beside(file!()).expect("startup");
    assert_eq!(eval_string(&world, "(:user::c03)"), Ok("".into()));
}

#[test]
fn contract_04_list_dir_lists_known_file() {
    let world = startup_beside(file!()).expect("startup");
    let entries = eval_vec_strings(&world, "(:user::c04)").expect("list-dir");
    assert!(
        entries.iter().any(|e| e == "wat/fix.wat"),
        "list-dir \"wat\" must contain the full path \"wat/fix.wat\"; got {entries:?}"
    );
}

#[test]
fn contract_05_list_dir_returns_full_paths() {
    let world = startup_beside(file!()).expect("startup");
    let entries = eval_vec_strings(&world, "(:user::c04)").expect("list-dir");
    assert!(!entries.is_empty(), "list-dir must return entries");
    assert!(
        entries.iter().all(|e| e.starts_with("wat/")),
        "every entry must be a FULL path under the listed dir; got {entries:?}"
    );
}
