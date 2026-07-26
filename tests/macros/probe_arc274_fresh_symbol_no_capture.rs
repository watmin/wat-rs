//! Arc 274.1 — `(:wat::core::fresh-symbol <base>)` gives a computing (program-body) macro a
//! capture-proof binder. Mirrors `probe_macro_hygiene_capture.rs` (the quasiquote-path capture test)
//! but on the PROGRAM-BODY path, where sets-of-scopes is NOT auto-applied (expand.rs:332) — so a plain
//! `(symbol-node "t")` binder WOULD capture; `(fresh-symbol "t")` must NOT.
//!
//! A program-body macro binds `(fresh-symbol "t")` to 100 and adds the caller's unquoted arg. The caller
//! passes its OWN `t` = 5:
//!   `(:wat::core::let [t 5] (:test::add-via-fresh t))`
//! expands to `(let [t{fresh-scope} 100] (i64::+ t{fresh-scope} t{user-scope}))`.
//!   - HYGIENIC → the macro's `t` (fresh unique scope, 100) is distinct from the user's `t` (5) → 105.
//!   - CAPTURED → the user's `t` resolves to the macro's inner binding (100) → 200.
//!
//! Wat source lives in the co-located fixture: probe_arc274_fresh_symbol_no_capture.wat
//! (slurped via startup_beside(file!())).
//!
//! Run: cargo test --release -p wat --test probe_arc274_fresh_symbol_no_capture -- --include-ignored

use wat::freeze::call_beside_value;
use wat::runtime::Value;

// just-eval (rubric): the probe is a zero-arg entry fn in the co-located fixture, driven via
// call_beside_value — no inline wat driver expression.
#[test]
fn fresh_symbol_binder_does_not_capture_caller() {
    let got = call_beside_value(file!(), ":user::compute")
        .unwrap_or_else(|e| panic!("compute raised: {e:?}"));
    assert!(
        matches!(got, Value::i64(105)),
        "expected 105 (HYGIENIC: macro's fresh `t`=100 distinct from caller's `t`=5); \
         200 would mean CAPTURE (caller's t bound to the macro's t); got {got:?}"
    );
}
