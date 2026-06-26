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
//! RED at HEAD: `:wat::core::fresh-symbol` does not exist (grep-confirmed) → the macro fails to
//! expand → startup fails. GREEN once 274.1 ships the scope-stamped primitive.
//!
//! Run: cargo test --release -p wat --test probe_arc274_fresh_symbol_no_capture -- --include-ignored

use std::sync::Arc;
use wat::freeze::{eval_in_frozen, startup_from_source};
use wat::load::InMemoryLoader;
use wat::runtime::{Environment, Value};

// A PROGRAM-BODY macro: top-level `let` computes the temp via `fresh-symbol`, then a quasiquote tail
// uses it as a binder AND a reference (same value → same fresh scope → matches itself, never the user).
const MACRO: &str = r#"
(:wat::core::defmacro :test::add-via-fresh
  [x <- :wat::WatAST]
  -> :wat::WatAST
  (:wat::core::let
    [t (:wat::core::fresh-symbol "t")]
    `(:wat::core::let [~t 100] (:wat::core::i64::+ ~t ~x))))
"#;

#[test]
fn fresh_symbol_binder_does_not_capture_caller() {
    let src = format!(
        "{MACRO}\n\
         (:wat::core::defn :user::compute [] -> :wat::core::i64 \
           (:wat::core::let [t 5] (:test::add-via-fresh t)))\n\
         (:wat::core::defn :user::main [] -> :wat::core::nil nil)"
    );
    let world = startup_from_source(&src, None, Arc::new(InMemoryLoader::new()))
        .expect("startup should succeed once fresh-symbol exists");
    let ast = wat::parse_one!("(:user::compute)").expect("parse");
    let got = eval_in_frozen(&ast, &world, &Environment::new())
        .map(|tv| tv.value_owned())
        .unwrap_or_else(|e| panic!("compute raised: {e:?}"));
    assert!(
        matches!(got, Value::i64(105)),
        "expected 105 (HYGIENIC: macro's fresh `t`=100 distinct from caller's `t`=5); \
         200 would mean CAPTURE (caller's t bound to the macro's t); got {got:?}"
    );
}
