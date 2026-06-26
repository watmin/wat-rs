//! FOUNDATIONAL PROBE (kwargs-start de-risk) — does the 260.1b companion-macro path compose when the
//! kwargs `defn` is itself EMITTED by another macro (the defservice shape)?
//!
//! defservice is a macro that emits its lifecycle fns. Flipping `start`/`resume` to `& [argspec]` means
//! defservice's `(do …)` now contains a kwargs `defn` — which must expand and emit ITS OWN companion
//! `defmacro`, which must then hoist (macros-emit-macros). 260.1b proved the hoist for a *top-level*
//! `defn`; this probe proves it one level deeper: a wrapper macro emits a `/`-named kwargs `defn`, then
//! a kwargs call resolves through the hoisted companion. If GREEN, the kwargs-`start` strike is safe.
//!
//! Run: cargo test --release -p wat --test probe_kwargs_emitted_by_macro

use std::sync::Arc;
use wat::freeze::{eval_in_frozen, startup_from_source};
use wat::load::InMemoryLoader;
use wat::runtime::{Environment, Value};

const EMITTED_KWARGS: &str = r#"
;; a wrapper macro that EMITS a (do …) containing a /-named kwargs defn — the defservice shape.
(:wat::core::defmacro :t::make-adder [] -> :wat::WatAST
  `(:wat::core::do
     (:wat::core::defn :t::svc/add
       [& [a <- :wat::core::i64  b <- :wat::core::i64]]
       -> :wat::core::i64
       (:wat::core::i64::+ a b))))

;; CONTROL: a wrapper emitting a PLAIN (non-kwargs) defn — isolates whether wrapper-emitted defns
;; register AT ALL vs whether only the kwargs-companion hoist fails through the nesting.
(:wat::core::defmacro :t::make-plain [] -> :wat::WatAST
  `(:wat::core::do
     (:wat::core::defn :t::svc/plain [x <- :wat::core::i64] -> :wat::core::i64 x)))

;; expand the wrappers at top level → emits the defns (+ via the hoist, the kwargs companion macro)
(:t::make-adder)
(:t::make-plain)

;; CONTROL caller — a plain emitted defn must resolve (no kwargs involved)
(:wat::core::defn :t::via-plain [] -> :wat::core::i64
  (:t::svc/plain 42))

;; call the macro-emitted kwargs fn with inline :k v (in order + reordered) and {map}
(:wat::core::defn :t::via-kv [] -> :wat::core::i64
  (:t::svc/add :a 40 :b 2))
(:wat::core::defn :t::via-kv-reorder [] -> :wat::core::i64
  (:t::svc/add :b 2 :a 40))
(:wat::core::defn :t::via-map [] -> :wat::core::i64
  (:t::svc/add {:a 40 :b 2}))

(:wat::core::defn :t::main [] -> :wat::core::nil nil)
"#;

fn eval_to_i64(world: &wat::freeze::FrozenWorld, expr: &str) -> Value {
    let ast = wat::parse_one!(expr).expect("parse");
    eval_in_frozen(&ast, world, &Environment::new())
        .map(|tv| tv.value_owned())
        .unwrap_or_else(|e| panic!("{expr} raised: {e:?}"))
}

#[test]
fn kwargs_defn_emitted_by_a_macro_lowers_through_its_hoisted_companion() {
    let world = startup_from_source(EMITTED_KWARGS, None, Arc::new(InMemoryLoader::new()))
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
