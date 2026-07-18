//! Reach-stumble: `:wat::core::nth` — the positional, TOTAL accessor.
//!
//! `Vector/get` is the associative, nil-safe form: `Vec<T> × i64 -> Option<T>`
//! (None on out-of-range, never raises). `nth` is the Clojure positional idiom:
//! `Vec<T> × i64 -> T` — "there IS an i-th element; give it or fail" — raising on
//! out-of-range. NOT an alias; the opposite contract at the edge. `nth` is sugar
//! over `Option/expect (Vector/get v i)` with that total promise.
//!
//! RED at HEAD: `:wat::core::nth` does not exist.

use wat::freeze::call_beside;
use wat::runtime::Value;

// just-eval (rubric): each `:t::…` entry is a zero-arg fn in the co-located
// `.wat` fixture, driven via `call_beside` — no inline wat driver.

/// `(nth [10 20 30] 1)` → 20 — the positional element, returned as T (not Option).
#[test]
fn nth_returns_the_positional_element() {
    match call_beside(file!(), ":t::nth-returns-positional").expect("eval") {
        Value::i64(n) => assert_eq!(n, 20, "nth returns the i-th element directly as T"),
        other => panic!("expected i64; got {other:?}"),
    }
}

/// `nth` out-of-range RAISES (unlike `get`, which returns None) — the total contract.
#[test]
#[should_panic] // the raise is a structured AssertionFailure payload (not a String),
                // so match any panic — same as the assert-true probe.
fn nth_raises_on_out_of_range() {
    call_beside(file!(), ":t::nth-out-of-range").expect("eval");
}
