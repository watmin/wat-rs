//! Arc 260.1a — the DECLARE side: a `defn` kwargs section `& {fields}` mints a typed record and the
//! fn takes/destructures it. The foundation (no call-site sugar yet — the explicit-record call form).
//!
//! `(:user::connect [host <- :String & [port <- :i64 tls <- :bool]] …)` → `defn` mints
//! `:user::connect::Kwargs` (the rs-1 mint). The kwargs section is an ARGSPEC nested once (same
//! binder-triple syntax as the main params, reusing the existing parse; a nested `&` inside it is
//! disallowed — flat, one level). `defn` reshapes the fn so its last param is that record, and
//! destructures the fields into the body scope (clojure `& {:keys}` binds `port`/`tls`). The body uses
//! `port` + `tls` by name. The call constructs the record explicitly and passes it.
//!
//! RED at HEAD: `& [argspec]` in a param vector is unparseable (today `& name <- :T` expects a binder
//! NAME, not a `[]` vector) → startup fails. GREEN once 260.1a mints + reshapes + destructures.
//! The inline `:k v` call sugar is 260.1b (separate; the headline probe probe_arc260_keyword_args stays
//! RED until then).
//!
//! Run: cargo test --release -p wat --test probe_arc260_decl_kwargs_minted_record -- --include-ignored

use std::sync::Arc;
use wat::freeze::{eval_in_frozen, startup_from_source};
use wat::load::InMemoryLoader;
use wat::runtime::{Environment, Value};

// `& {port tls}` mints :user::connect::Kwargs; the body uses port + tls by name (destructured);
// the call constructs the record explicitly (no sugar) and passes it. 443 + (tls?1:0) = 444.
const DECL_KWARGS: &str = r#"
(:wat::core::defn :user::connect
  [host <- :wat::core::String
   & [port <- :wat::core::i64  tls <- :wat::core::bool]]
  -> :wat::core::i64
  (:wat::core::i64::+ port (:wat::core::if tls -> :wat::core::i64 1 0)))

(:wat::core::defn :user::compute [] -> :wat::core::i64
  (:user::connect "example.com" (:user::connect::Kwargs 443 true)))

(:wat::core::defn :user::main [] -> :wat::core::nil nil)
"#;

#[test]
#[ignore = "arc 260.1a RED until defn mints :<name>::Kwargs from & {fields} + reshapes/destructures. \
            Today & {…} is unparseable. UN-IGNORE when 260.1a lands."]
fn decl_kwargs_mints_record_and_destructures() {
    let world = startup_from_source(DECL_KWARGS, None, Arc::new(InMemoryLoader::new()))
        .expect("startup should succeed once defn mints the kwargs record + destructures");
    let ast = wat::parse_one!("(:user::compute)").expect("parse");
    let got = eval_in_frozen(&ast, &world, &Environment::new())
        .map(|tv| tv.value_owned())
        .unwrap_or_else(|e| panic!("compute raised: {e:?}"));
    assert!(
        matches!(got, Value::i64(444)),
        "expected 444: & {{port tls}} minted :user::connect::Kwargs; the explicit-record call passed \
         (Kwargs 443 true); the body read port + tls by name (destructured) → 443 + 1; got {got:?}"
    );
}
