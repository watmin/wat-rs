//! Arc 301 stone 2b — the delete differential (mem vs sqlite).
//!
//! NO production code. Runs the same delete sequence (ensure-schema with one
//! GSI → put 3 rows that project into it → delete the middle by (pk,sk) →
//! scan + scan-index → delete the same key again) against mem-store' and
//! sqlite-store' (`:memory:`). The .wat returns the shared summary IFF they
//! match, else a `DIFFERENTIAL-MISMATCH` sentinel carrying both payloads.
//!
//! A disagreement is a successful stone (STOP-1: do not edit either backend).
//! An empty GSI would make this vacuous — the fixture declares `by-v` and
//! drives `scan-index` after the delete.
//!
//! Run: `cargo nextest run --release -E 'test(delete_differential)'`

use wat::freeze::startup_beside;
use wat::runtime::{apply_function, Value};

/// If both backends implement stone 2's STOP-2 claim, this is what they
/// agree on: delete `b` (which projected `v2`) leaves base `{a,c}` and GSI
/// `{v1,v3}`; a second delete of `b` is `:Success` and a no-op.
const AGREED_SUMMARY: &str =
    "d1=Success;base=2:a,c;gsi=2:v1,v3;d2=Success;base2=2:a,c;gsi2=2:v1,v3";

#[test]
fn delete_differential_mem_and_sqlite_agree() {
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
        Err(e) => panic!("delete differential raised: {e:?}"),
    };
    assert_eq!(
        stored, AGREED_SUMMARY,
        "delete differential: expected STOP-2 summary (base {{a,c}}, GSI {{v1,v3}}, \
         duplicate-ack Success). A DIFFERENTIAL-MISMATCH prefix means mem and sqlite \
         disagreed — that disagreement IS the deliverable, do not edit either backend. got: {stored}"
    );
}
