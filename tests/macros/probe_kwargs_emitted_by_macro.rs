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

use wat::freeze::{eval_in_frozen, startup_beside};
use wat::runtime::{Environment, Value};

fn eval_to_i64(world: &wat::freeze::FrozenWorld, expr: &str) -> Value {
    let ast = wat::parse_one!(expr).expect("parse");
    eval_in_frozen(&ast, world, &Environment::new())
        .map(|tv| tv.value_owned())
        .unwrap_or_else(|e| panic!("{expr} raised: {e:?}"))
}

#[test]
fn kwargs_defn_emitted_by_a_macro_lowers_through_its_hoisted_companion() {
    let world = startup_beside(file!())
        .expect("startup should succeed if a macro-emitted kwargs defn's companion macro hoists");
    // CONTROL: plain wrapper-emitted defn resolves?
    assert!(
        matches!(eval_to_i64(&world, "(:t::via-plain)"), Value::i64(42)),
        "CONTROL: a plain (non-kwargs) defn emitted by a wrapper macro should resolve"
    );
    for f in ["(:t::via-kv)", "(:t::via-kv-reorder)", "(:t::via-map)"] {
        let got = eval_to_i64(&world, f);
        assert!(
            matches!(got, Value::i64(42)),
            "expected 42 from {f} — kwargs on a macro-emitted /-named fn; got {got:?}"
        );
    }
}
