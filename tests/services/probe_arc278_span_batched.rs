//! arc 278 item (b) — the batched writer.
//!
//! An over-cap buffer drains across cap-fitting submissions. Partial progress is
//! exact (no duplicate, no loss). A single over-cap item is RequestTooLarge.

use wat::freeze::startup_beside;
use wat::runtime::{apply_function, Value};

fn run(name: &str) -> Value {
    let world = startup_beside(file!()).unwrap_or_else(|e| panic!("startup failed: {e:?}"));
    let func = world
        .symbols()
        .get(name)
        .unwrap_or_else(|| panic!("{name} not registered"))
        .clone();
    apply_function(func, vec![], world.symbols(), wat::rust_caller_span!())
        .unwrap_or_else(|e| panic!("{name} raised: {e:?}"))
}

#[test]
fn overcap_buffer_drains() {
    let got = run(":user::overcap-drains");
    assert!(
        matches!(got, Value::i64(1)),
        "an over-cap buffer against a working sink must land every log across chunked writes; got {got:?}"
    );
}

#[test]
fn partial_progress_is_exact() {
    let got = run(":user::partial-exact");
    assert!(
        matches!(got, Value::i64(1)),
        "after chunk-1 success and chunk-2 failure a later drain must land exactly the suffix (got {got:?}: >n is a duplicate, <n is a loss)"
    );
}

#[test]
fn one_item_over_cap_is_request_too_large() {
    let got = run(":user::one-item-rtl");
    assert!(
        matches!(got, Value::i64(1)),
        "a single item whose encoding exceeds the cap must be FlushResponse::RequestTooLarge, not a hang; got {got:?}"
    );
}

#[test]
fn exact_cap_chunk_is_sent() {
    let got = run(":user::exact-cap-sent");
    assert!(
        matches!(got, Value::i64(1)),
        "a 1-item request sized exactly to the cap must be sent (cut at >, not >=); got {got:?} (-5 = could not hit exact cap)"
    );
}

#[test]
fn undercap_is_one_write() {
    let got = run(":user::undercap-one-write");
    assert!(
        matches!(got, Value::i64(1)),
        "a small buffer must be exactly one write (a second write was scripted to fail); got {got:?}"
    );
}
