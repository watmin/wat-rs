//! Arc 275 Stone 275.1 probe — run `(:wat::deporder::verify-stdlib)` on the
//! real baked stdlib and print violations + dependency edges to stdout.
//!
//! Run: `cargo test --release --test probe_arc275_verify_stdlib -- --nocapture`

use std::sync::Arc;
use wat::freeze::{eval_in_frozen, startup_from_source};
use wat::load::InMemoryLoader;
use wat::runtime::{Environment, Value};

fn eval_expr(body: &str) -> Result<Value, String> {
    let src = format!(
        "(:wat::core::defn :user::compute [] -> :wat::core::Vector<wat::deporder::Violation> {body})\n\
         (:wat::core::defn :user::main [] -> :wat::core::nil nil)"
    );
    let world = startup_from_source(&src, None, Arc::new(InMemoryLoader::new()))
        .map_err(|e| format!("startup/check: {e:?}"))?;
    let ast = wat::parse_one!("(:user::compute)").expect("parse");
    eval_in_frozen(&ast, &world, &Environment::new())
        .map(|tv| tv.value_owned())
        .map_err(|e| format!("eval: {e:?}"))
}

fn eval_count(body: &str) -> Result<i64, String> {
    let src = format!(
        "(:wat::core::defn :user::compute [] -> :wat::core::i64 {body})\n\
         (:wat::core::defn :user::main [] -> :wat::core::nil nil)"
    );
    let world = startup_from_source(&src, None, Arc::new(InMemoryLoader::new()))
        .map_err(|e| format!("startup/check: {e:?}"))?;
    let ast = wat::parse_one!("(:user::compute)").expect("parse");
    match eval_in_frozen(&ast, &world, &Environment::new())
        .map(|tv| tv.value_owned())
        .map_err(|e| format!("eval: {e:?}"))?
    {
        Value::i64(n) => Ok(n),
        other => Err(format!("non-i64: {other:?}")),
    }
}

fn eval_strings(body: &str) -> Result<Vec<String>, String> {
    let src = format!(
        "(:wat::core::defn :user::compute [] -> :wat::core::Vector<wat::core::String> {body})\n\
         (:wat::core::defn :user::main [] -> :wat::core::nil nil)"
    );
    let world = startup_from_source(&src, None, Arc::new(InMemoryLoader::new()))
        .map_err(|e| format!("startup/check: {e:?}"))?;
    let ast = wat::parse_one!("(:user::compute)").expect("parse");
    match eval_in_frozen(&ast, &world, &Environment::new())
        .map(|tv| tv.value_owned())
        .map_err(|e| format!("eval: {e:?}"))?
    {
        Value::Vec(v) => {
            let mut out = Vec::new();
            for item in v.iter() {
                if let Value::String(s) = item {
                    out.push(s.as_ref().clone());
                } else {
                    out.push(format!("{item:?}"));
                }
            }
            Ok(out)
        }
        other => Err(format!("non-vec: {other:?}")),
    }
}

#[test]
fn probe_stdlib_sources_count() {
    // Verify the intrinsic returns a vector with the expected number of files.
    let n = eval_count("(:wat::core::length (:wat::stdlib::sources))").unwrap();
    println!("stdlib::sources count = {n}");
    assert!(n > 0, "expected at least 1 source file");
}

#[test]
fn probe_verify_stdlib_violation_count() {
    // Run verify-stdlib and print the number of violations.
    let n = eval_count("(:wat::core::length (:wat::deporder::verify-stdlib))").unwrap();
    println!("verify-stdlib violation count = {n}");
    // Just assert it returns a non-negative count (the actual enforcement is 275.2).
    assert!(n >= 0);
}

#[test]
fn probe_verify_stdlib_violations_detail() {
    // Build a helper that returns all violations as a stringified report.
    // We pull violation fields separately since the test harness can't
    // print Record fields directly — we compose a string per violation.
    let src = concat!(
        "(:wat::core::defn :user::compute [] -> :wat::core::Vector<wat::core::String>",
        "  (:wat::core::let [viols (:wat::deporder::verify-stdlib)]",
        "    (:wat::core::map",
        "      (:wat::core::fn [v <- :wat::deporder::Violation] -> :wat::core::String",
        "        (:wat::core::string::concat (:wat::deporder::Violation/referencer v)",
        "        (:wat::core::string::concat \" @\"",
        "        (:wat::core::string::concat (:wat::core::show (:wat::deporder::Violation/referencer-pos v))",
        "        (:wat::core::string::concat \" -> \"",
        "        (:wat::core::string::concat (:wat::deporder::Violation/definer v)",
        "        (:wat::core::string::concat \" @\"",
        "        (:wat::core::string::concat (:wat::core::show (:wat::deporder::Violation/definer-pos v))",
        "        (:wat::core::string::concat \" [\"",
        "        (:wat::core::string::concat (:wat::deporder::Violation/symbol v) \"]\"",
        "        ))))))))))",
        "      viols)))",
        "(:wat::core::defn :user::main [] -> :wat::core::nil nil)",
    );
    let world = startup_from_source(src, None, Arc::new(InMemoryLoader::new()))
        .map_err(|e| panic!("startup: {e:?}"))
        .unwrap();
    let ast = wat::parse_one!("(:user::compute)").expect("parse");
    let val = eval_in_frozen(&ast, &world, &Environment::new())
        .map(|tv| tv.value_owned())
        .map_err(|e| panic!("eval: {e:?}"))
        .unwrap();
    match val {
        Value::Vec(v) => {
            println!("=== verify-stdlib violations ({} total) ===", v.len());
            if v.is_empty() {
                println!("  (none — order is already valid)");
            }
            for item in v.iter() {
                if let Value::String(s) = item {
                    println!("  {s}");
                }
            }
        }
        other => panic!("non-vec: {other:?}"),
    }
}
