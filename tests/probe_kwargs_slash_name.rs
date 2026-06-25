//! DISCONFIRMING PROBE (kwargs-start trap perception) — does the 260.1b companion-macro
//! kwargs sugar compose onto a `/`-named fn (a defservice method like `worker/start`)?
//!
//! A defservice emits its lifecycle fns as `:<fqdn>/start` / `:<fqdn>/resume` — names with a
//! `/`. The 260.1b kwargs branch mints `:<name>::Kwargs` by appending `::Kwargs` to the fn name,
//! so `:t::worker/start` → `:t::worker/start::Kwargs`. Since `/` is the accessor separator, that
//! name could collide / fail to mint. This probe isolates exactly that: a `& [argspec]` defn whose
//! name carries a `/`, called with inline `:k v` kwargs (in AND out of order).
//!
//! Run: cargo test --release -p wat --test probe_kwargs_slash_name

use std::sync::Arc;
use wat::freeze::{eval_in_frozen, startup_from_source};
use wat::load::InMemoryLoader;
use wat::runtime::{Environment, Value};

// A `/`-named kwargs fn (mirrors a defservice `worker/start` shape) + wrapper fns that invoke the
// companion macro at startup (macro expansion time), exactly as the 260.1b probe does.
const SLASH_KWARGS: &str = r#"
(:wat::core::defn :t::worker/start
  [& [count <- :wat::core::i64  step <- :wat::core::i64]]
  -> :wat::core::i64
  (:wat::core::i64::+ count step))

;; inline :k v, in order
(:wat::core::defn :t::via-kv [] -> :wat::core::i64
  (:t::worker/start :count 40 :step 2))

;; inline :k v, OUT OF ORDER — only a true reorder-by-field yields 42
(:wat::core::defn :t::via-kv-reorder [] -> :wat::core::i64
  (:t::worker/start :step 2 :count 40))

;; literal {map}
(:wat::core::defn :t::via-map [] -> :wat::core::i64
  (:t::worker/start {:count 40 :step 2}))

(:wat::core::defn :t::main [] -> :wat::core::nil nil)
"#;

fn eval_to_i64(world: &wat::freeze::FrozenWorld, expr: &str) -> Value {
    let ast = wat::parse_one!(expr).expect("parse");
    eval_in_frozen(&ast, world, &Environment::new())
        .map(|tv| tv.value_owned())
        .unwrap_or_else(|e| panic!("{expr} raised: {e:?}"))
}

#[test]
fn slash_named_kwargs_fn_lowers_through_companion_macro() {
    let world = startup_from_source(SLASH_KWARGS, None, Arc::new(InMemoryLoader::new()))
        .expect("startup should succeed if the companion-macro path composes onto a /-named fn");
    for f in ["(:t::via-kv)", "(:t::via-kv-reorder)", "(:t::via-map)"] {
        let got = eval_to_i64(&world, f);
        assert!(
            matches!(got, Value::i64(42)),
            "expected 42 from {f} — :k v / {{map}} on a /-named kwargs fn; got {got:?}"
        );
    }
}
