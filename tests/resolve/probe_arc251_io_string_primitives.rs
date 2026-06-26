//! Strike (examinare disconfirming probe) — the two basic primitives wat lacks for
//! self-driving the corpus migration: directory enumeration + substring.
//!
//! Named by an intueri cast, grounded in the existing families + Clojure faithfulness:
//!   `:wat::io::list-dir` (path) -> Vector<String> of FULL paths (read-file maps straight over it).
//!   `:wat::core::string::subs` (s start end) -> String — Clojure's `subs`, start-incl/end-excl, char-indexed.
//!
//! C01 subs prefix      : (subs "hello world" 0 5)  -> "hello"
//! C02 subs suffix      : (subs "hello world" 6 11) -> "world"
//! C03 subs empty range : (subs "abc" 1 1)          -> ""
//! C04 list-dir lists   : (list-dir "wat") contains the known full path "wat/fix.wat"
//! C05 list-dir paths   : every entry of (list-dir "wat") is a FULL path (starts with "wat/")
//!
//! RED at HEAD: neither `:wat::io::list-dir` nor `:wat::core::string::subs` exists (UnknownFunction).
//!
//! Run: `cargo test --release --test probe_arc251_io_string_primitives`

use std::sync::Arc;
use wat::freeze::{eval_in_frozen, startup_from_source};
use wat::load::InMemoryLoader;
use wat::runtime::{Environment, Value};

fn eval(body: &str, ret_ty: &str) -> Result<Value, String> {
    let src = format!(
        "(:wat::core::defn :user::compute [] -> {ret_ty} {body})\n\
         (:wat::core::defn :user::main [] -> :wat::core::nil nil)",
    );
    let world = startup_from_source(&src, None, Arc::new(InMemoryLoader::new()))
        .map_err(|e| format!("startup/check: {e:?}"))?;
    let ast = wat::parse_one!("(:user::compute)").expect("parse");
    eval_in_frozen(&ast, &world, &Environment::new())
        .map(|tv| tv.value_owned())
        .map_err(|e| format!("eval: {e:?}"))
}

fn subs(body: &str) -> Result<String, String> {
    match eval(body, ":wat::core::String")? {
        Value::String(s) => Ok((*s).clone()),
        other => Err(format!("non-string: {other:?}")),
    }
}

fn list_dir(body: &str) -> Result<Vec<String>, String> {
    match eval(body, ":wat::core::Vector<wat::core::String>")? {
        Value::Vec(v) => v
            .iter()
            .map(|e| match e {
                Value::String(s) => Ok((**s).clone()),
                other => Err(format!("non-string entry: {other:?}")),
            })
            .collect(),
        other => Err(format!("non-vector: {other:?}")),
    }
}

#[test]
fn contract_01_subs_prefix() {
    assert_eq!(subs("(:wat::core::string::subs \"hello world\" 0 5)"), Ok("hello".into()));
}

#[test]
fn contract_02_subs_suffix() {
    assert_eq!(subs("(:wat::core::string::subs \"hello world\" 6 11)"), Ok("world".into()));
}

#[test]
fn contract_03_subs_empty_range() {
    assert_eq!(subs("(:wat::core::string::subs \"abc\" 1 1)"), Ok("".into()));
}

#[test]
fn contract_04_list_dir_lists_known_file() {
    let entries = list_dir("(:wat::io::list-dir \"wat\")").expect("list-dir");
    assert!(
        entries.iter().any(|e| e == "wat/fix.wat"),
        "list-dir \"wat\" must contain the full path \"wat/fix.wat\"; got {entries:?}"
    );
}

#[test]
fn contract_05_list_dir_returns_full_paths() {
    let entries = list_dir("(:wat::io::list-dir \"wat\")").expect("list-dir");
    assert!(!entries.is_empty(), "list-dir must return entries");
    assert!(
        entries.iter().all(|e| e.starts_with("wat/")),
        "every entry must be a FULL path under the listed dir; got {entries:?}"
    );
}
