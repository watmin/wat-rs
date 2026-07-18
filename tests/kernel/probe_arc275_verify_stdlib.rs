//! Arc 275 Stone 275.1 probe — run `(:wat::deporder::verify-stdlib)` on the
//! real baked stdlib and print violations + dependency edges to stdout.
//!
//! Run: `cargo test --release --test probe_arc275_verify_stdlib -- --nocapture`

use wat::freeze::call_beside;
use wat::runtime::Value;

#[test]
fn probe_stdlib_sources_count() {
    // Verify the intrinsic returns a vector with the expected number of files.
    let val = call_beside(file!(), ":user::compute-sources-count").expect("eval should succeed");
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
    let val = call_beside(file!(), ":user::compute-violation-count").expect("eval should succeed");
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
    let val = call_beside(file!(), ":user::compute-violations-detail").expect("eval should succeed");
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
