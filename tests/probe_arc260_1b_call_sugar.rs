//! Arc 260.1b — the CALL-SUGAR side: a `& [argspec]` kwargs fn is callable with inline `:k v` and
//! `{map}` (the companion-macro path; NOT the abandoned check/eval plain-fn reorder).
//!
//! `defn` with a kwargs section emits a COMPANION MACRO under the fn's name + the positional impl
//! under `<name>$impl` (the `$` = apparatus-minted-internal sigil — Clojure-faithful: `clojure.core$map`).
//! The macro scoops the trailing `:k v` pairs / `{map}` literal → reorders by the `::Kwargs` field order
//! → `(<name>$impl pos-args (::Kwargs …))`. The explicit-record form (260.1a) stays valid (passthrough).
//!
//! All four call forms must yield 444 (443 + (tls?1:0)):
//!   - inline `:k v` in order            (connect "h" :port 443 :tls true)
//!   - inline `:k v` OUT OF ORDER        (connect "h" :tls true :port 443)   ← only a real reorder gets this
//!   - literal `{map}`                   (connect "h" {:port 443 :tls true})
//!   - explicit record (escape hatch)    (connect "h" (connect::Kwargs 443 true))
//!
//! RED at HEAD: `connect` is a plain fn taking (host, ::Kwargs); `:k v` reads as 4 positional args and
//! `{map}` as a map (not a ::Kwargs) → arity/type error. Only the explicit-record form works today.
//! GREEN once 260.1b emits the companion macro.
//!
//! Run: cargo test --release -p wat --test probe_arc260_1b_call_sugar

use std::sync::Arc;
use wat::freeze::{eval_in_frozen, startup_from_source};
use wat::load::InMemoryLoader;
use wat::runtime::{Environment, Value};

const KWARGS_SUGAR: &str = r#"
(:wat::core::defn :user::connect
  [host <- :wat::core::String
   & [port <- :wat::core::i64  tls <- :wat::core::bool]]
  -> :wat::core::i64
  (:wat::core::i64::+ port (:wat::core::if tls -> :wat::core::i64 1 0)))

;; inline :k v, in order
(:wat::core::defn :user::via-kv [] -> :wat::core::i64
  (:user::connect "h" :port 443 :tls true))

;; inline :k v, OUT OF ORDER — only a true reorder-by-field yields 444
(:wat::core::defn :user::via-kv-reorder [] -> :wat::core::i64
  (:user::connect "h" :tls true :port 443))

;; literal {map}
(:wat::core::defn :user::via-map [] -> :wat::core::i64
  (:user::connect "h" {:port 443 :tls true}))

;; explicit record (the escape hatch — 260.1a; must still work)
(:wat::core::defn :user::via-record [] -> :wat::core::i64
  (:user::connect "h" (:user::connect::Kwargs 443 true)))

(:wat::core::defn :user::main [] -> :wat::core::nil nil)
"#;

fn eval_to_i64(world: &wat::freeze::FrozenWorld, expr: &str) -> Value {
    let ast = wat::parse_one!(expr).expect("parse");
    eval_in_frozen(&ast, world, &Environment::new())
        .map(|tv| tv.value_owned())
        .unwrap_or_else(|e| panic!("{expr} raised: {e:?}"))
}

#[test]
fn kwargs_call_sugar_kv_map_and_record_all_agree() {
    let world = startup_from_source(KWARGS_SUGAR, None, Arc::new(InMemoryLoader::new()))
        .expect("startup should succeed once 260.1b emits the companion macro");
    for f in ["(:user::via-kv)", "(:user::via-kv-reorder)", "(:user::via-map)", "(:user::via-record)"] {
        let got = eval_to_i64(&world, f);
        assert!(
            matches!(got, Value::i64(444)),
            "expected 444 from {f}: :k v / {{map}} / record all lower to (connect$impl \"h\" \
             (connect::Kwargs 443 true)) → 443 + 1; got {got:?}"
        );
    }
}
