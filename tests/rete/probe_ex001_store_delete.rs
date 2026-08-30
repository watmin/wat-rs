//! Arc 301 stone 2 — Store gains `delete`. Promoted from
//! `docs/excursus/2026/08/001-sns-sqs/PROBE-store-has-no-delete.wat` (byte-identical copy;
//! the arc-dir probe is the gate and is not edited).
//!
//! Puts 3 rows under one pk, deletes the middle one by `(pk, sk)`, asserts
//! the subsequent scan count is 2. mem-store only (sqlite twin is stone 2b).
//!
//! Run: `cargo nextest run --release -E 'test(store_delete)'`

use wat::freeze::call_beside;

#[test]
fn store_delete_removes_exactly_the_named_row() {
    call_beside(file!(), ":user::delete-removes-exactly-the-named-row").expect_passed(
        "delete-removes-exactly-the-named-row: put 3, delete middle by (pk,sk), scan count 3 → 2",
    );
}
