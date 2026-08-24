//! Arc 255 Stone D — `:wat::core::string::join` widens from `(Vector :- [T])` to the
//! `(Seqable :- [T])` surface (Vector · PersistentVector · List · Stream).
//!
//! Four rows, per BRIEF-STONE-D-join-widens-to-seqable.md:
//!   1. Vector unchanged (no-regression).
//!   2. Stream accepted — the gap; REFUSED at CHECK time before this stone.
//!   3. List accepted — proves the widening reached the whole Seqable set.
//!   4. Rendering survives the widening — a non-string element via the Stream path
//!      renders identically to the same elements via the Vector path (catches a widening
//!      that forgot `render_str_total`).

use wat::freeze::call_beside_value;
use wat::runtime::Value;

fn run_fn(fn_name: &str) -> Value {
    call_beside_value(file!(), fn_name).expect("eval should succeed")
}

fn assert_str(val: Value, expected: &str) {
    match val {
        Value::String(s) => assert_eq!(
            &*s, expected,
            "expected String({expected:?}); got String({s:?})"
        ),
        other => panic!("expected String({expected:?}); got {:?}", other),
    }
}

/// Row 1 — Vector, the fast path. Must stay green (it was green before Stone D).
#[test]
fn row1_vector_unchanged() {
    assert_str(run_fn(":probe::join-vector"), "1-2-3");
}

/// Row 2 — Stream, the gap. Before Stone D this was refused at CHECK time
/// (`:wat::core::string::join: parameter #2 expects (:wat::core::Vector :- [T]); got
/// (:wat::stream::Stream :- [...])`), so `call_beside_value` itself would have failed
/// with a startup/type-check error rather than reaching eval.
#[test]
fn row2_stream_accepted() {
    assert_str(run_fn(":probe::join-stream"), "2-3-4");
}

/// Row 3 — List. Proves the widening reached the whole `Seqable :- [T]` set, not just Stream.
#[test]
fn row3_list_accepted() {
    assert_str(run_fn(":probe::join-list"), "1-2-3");
}

/// Row 4 — rendering parity. A non-string element (bool, distinct from row 2's i64) joined
/// through the Stream path must render BYTE-IDENTICALLY to the same elements joined through
/// the Vector path. This is the row that catches a Stream arm that skipped
/// `render_str_total` (a naive Debug/to_string render would print `Bool(true)`, not `true`).
#[test]
fn row4_rendering_survives_the_widening() {
    let via_vector = run_fn(":probe::join-vector-bool");
    let via_stream = run_fn(":probe::join-stream-bool");
    assert_str(via_vector.clone(), "true,false,true");
    assert_str(via_stream.clone(), "true,false,true");
    match (via_vector, via_stream) {
        (Value::String(v), Value::String(s)) => {
            assert_eq!(&*v, &*s, "Stream-path render diverged from Vector-path render");
        }
        other => panic!("expected two Strings; got {:?}", other),
    }
}
