//! DISCONFIRMING PROBE (kwargs-start trap perception) — does the 260.1b companion-macro
//! kwargs sugar compose onto a `/`-named fn (a defservice method like `worker/start`)?
//!
//! A defservice emits its lifecycle fns as `:<fqdn>/start` / `:<fqdn>/resume` — names with a
//! `/`. The 260.1b kwargs branch mints `:<name>::Kwargs` by appending `::Kwargs` to the fn name,
//! so `:t::worker/start` → `:t::worker/start::Kwargs`. Since `/` is the accessor separator, that
//! name could collide / fail to mint. This probe isolates exactly that: a `& [argspec]` defn whose
//! name carries a `/`, called with inline `:k v` kwargs (in AND out of order).
//!
//! Wat source lives in the co-located fixture: probe_kwargs_slash_name.wat
//! (slurped via startup_beside(file!())).
//!
//! Run: cargo test --release -p wat --test probe_kwargs_slash_name

use wat::freeze::{eval_in_frozen, startup_beside};
use wat::runtime::{Environment, Value};

fn eval_to_i64(world: &wat::freeze::FrozenWorld, expr: &str) -> Value {
    let ast = wat::parse_one!(expr).expect("parse");
    eval_in_frozen(&ast, world, &Environment::new())
        .map(|tv| tv.value_owned())
        .unwrap_or_else(|e| panic!("{expr} raised: {e:?}"))
}

#[test]
fn slash_named_kwargs_fn_lowers_through_companion_macro() {
    let world = startup_beside(file!())
        .expect("startup should succeed if the companion-macro path composes onto a /-named fn");
    for f in ["(:t::via-kv)", "(:t::via-kv-reorder)", "(:t::via-map)"] {
        let got = eval_to_i64(&world, f);
        assert!(
            matches!(got, Value::i64(42)),
            "expected 42 from {f} — :k v / {{map}} on a /-named kwargs fn; got {got:?}"
        );
    }
}
