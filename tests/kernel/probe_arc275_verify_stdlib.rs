//! Arc 275 Stone 275.1 probe — run `(:wat::deporder::verify-stdlib)` on the
//! real baked stdlib and print violations + dependency edges to stdout.
//!
//! Run: `cargo test --release --test probe_arc275_verify_stdlib -- --nocapture`

use wat::freeze::{eval_in_frozen, startup_beside};
use wat::runtime::{Environment, Value};

#[test]
fn probe_stdlib_sources_count() {
    // Verify the intrinsic returns a vector with the expected number of files.
    let world = startup_beside(file!()).expect("startup");
    let ast = wat::parse_one!("(:user::compute-sources-count)").expect("parse");
    let val = eval_in_frozen(&ast, &world, &Environment::new())
        .expect("eval should succeed")
        .value_owned();
    match val {
        Value::i64(n) => {
            println!("stdlib::sources count = {n}");
            assert!(n > 0, "expected at least 1 source file");
        }
        other => panic!("expected i64; got {other:?}"),
    }
}

#[test]
fn probe_verify_stdlib_violation_count() {
    // Run verify-stdlib and print the number of violations.
    let world = startup_beside(file!()).expect("startup");
    let ast = wat::parse_one!("(:user::compute-violation-count)").expect("parse");
    let val = eval_in_frozen(&ast, &world, &Environment::new())
        .expect("eval should succeed")
        .value_owned();
    match val {
        Value::i64(n) => {
            println!("verify-stdlib violation count = {n}");
            // Just assert it returns a non-negative count (the actual enforcement is 275.2).
            assert!(n >= 0);
        }
        other => panic!("expected i64; got {other:?}"),
    }
}

#[test]
fn probe_verify_stdlib_violations_detail() {
    // Build a helper that returns all violations as a stringified report.
    let world = startup_beside(file!()).expect("startup");
    let ast = wat::parse_one!("(:user::compute-violations-detail)").expect("parse");
    let val = eval_in_frozen(&ast, &world, &Environment::new())
        .expect("eval should succeed")
        .value_owned();
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
