//! FOUNDATIONAL PROBE (kwargs-start de-risk) — does the 260.1b companion-macro path compose when the
//! kwargs `defn` is itself EMITTED by another macro (the defservice shape)?
//!
//! defservice is a macro that emits its lifecycle fns. Flipping `start`/`resume` to `& [argspec]` means
//! defservice's `(do …)` now contains a kwargs `defn` — which must expand and emit ITS OWN companion
//! `defmacro`, which must then hoist (macros-emit-macros). 260.1b proved the hoist for a *top-level*
//! `defn`; this probe proves it one level deeper: a wrapper macro emits a `/`-named kwargs `defn`, then
//! a kwargs call resolves through the hoisted companion. If GREEN, the kwargs-`start` strike is safe.
//!
//! Wat source lives in the co-located fixture: probe_kwargs_emitted_by_macro.wat
//! (slurped via startup_beside(file!())).
//!
//! Run: cargo test --release -p wat --test probe_kwargs_emitted_by_macro

use wat::freeze::call_beside;
use wat::runtime::Value;

// just-eval (rubric): each call form is a zero-arg entry fn in the co-located fixture, driven via
// call_beside — no inline wat driver expression.

#[test]
fn kwargs_defn_emitted_by_a_macro_lowers_through_its_hoisted_companion() {
    // CONTROL: plain wrapper-emitted defn resolves?
    let control = call_beside(file!(), ":t::via-plain")
        .expect("startup should succeed if a macro-emitted kwargs defn's companion macro hoists");
    assert!(
        matches!(control, Value::i64(42)),
        "CONTROL: a plain (non-kwargs) defn emitted by a wrapper macro should resolve"
    );
    for f in [":t::via-kv", ":t::via-kv-reorder", ":t::via-map"] {
        let got = call_beside(file!(), f)
            .unwrap_or_else(|e| panic!("({f}) raised: {e:?}"));
        assert!(
            matches!(got, Value::i64(42)),
            "expected 42 from ({f}) — kwargs on a macro-emitted /-named fn; got {got:?}"
        );
    }
}
