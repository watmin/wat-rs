//! Arc 301 stone 2c — re-put of an existing `(pk, sk)` is a replace, not an
//! append. Promoted from `docs/arc/2026/08/301-sns-sqs/PROBE-reput-divergence.wat`
//! (standalone `:user::main` dropped; this harness drives `:user::compute`).
//!
//! Put `(q#1, a)` projecting `v1`, then the same key projecting `v9`. Both
//! backends must agree on `base=1:a;gsi=1:v9` (PutItem). Pre-fix mem produced
//! `base=2:a,a;gsi=2:v1,v9`.
//!
//! Run: `cargo nextest run --release -E 'test(reput_differential)'`

use wat::freeze::startup_beside;
use wat::runtime::{apply_function, Value};

const AGREED_SUMMARY: &str = "base=1:a;gsi=1:v9";

#[test]
fn reput_differential_mem_and_sqlite_agree() {
    let world = startup_beside(file!())
        .expect("startup should succeed (mem-store' + sqlite-store' baked)");
    let func = world
        .symbols()
        .get(":user::compute")
        .unwrap_or_else(|| panic!(":user::compute not registered"))
        .clone();
    let stored = match apply_function(func, vec![], world.symbols(), wat::rust_caller_span!()) {
        Ok(Value::String(s)) => (*s).clone(),
        Ok(other) => panic!(":user::compute returned non-String: {other:?}"),
        Err(e) => panic!("reput differential raised: {e:?}"),
    };
    assert_eq!(
        stored, AGREED_SUMMARY,
        "re-put of (q#1,a) must replace: one base row, GSI moved to v9. \
         A DIFFERENTIAL-MISMATCH prefix means mem still appends. got: {stored}"
    );
}
