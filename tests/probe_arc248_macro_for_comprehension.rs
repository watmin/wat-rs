//! FM-2-bis probe for Arc 248 — generative-macro comprehension (`for` in templates).
//!
//! The missing capability: a BOUNDED template comprehension. `defmacro` today is
//! quasiquote-only — `,@rest` *splices* a list but cannot *map* (instantiate a
//! sub-template per element). This arc adds `(:wat::core::for [x xs] template)`:
//! at expansion time, iterate the finite list, bind `x` hygienically, walk the
//! template per element, splice the results.
//!
//! ROW STATUS:
//!   - REGRESSION (GREEN at HEAD + after): plain `,@items` splice still works.
//!   - MINT (RED at HEAD; `for` is unrecognized so `~x` is an unbound unquote;
//!     `#[ignore]`'d): un-ignored after the `for` comprehension is minted.
//!
//! Run: cargo test --release --test probe_arc248_macro_for_comprehension

use std::sync::Arc;
use wat::freeze::{eval_in_frozen, startup_from_source};
use wat::load::InMemoryLoader;
use wat::runtime::{Environment, Value};

fn with_nil_main(src: &str) -> String {
    format!(
        "{}\n(:wat::core::defn :user::main [] -> :wat::core::nil nil)",
        src
    )
}

fn run(src: &str) -> Result<Value, String> {
    let src = with_nil_main(src);
    let world = startup_from_source(&src, None, Arc::new(InMemoryLoader::new()))
        .map_err(|e| format!("startup: {:?}", e))?;
    let ast = wat::parse_one!("(:my::compute)").map_err(|e| format!("parse: {:?}", e))?;
    let env = Environment::new();
    eval_in_frozen(&ast, &world, &env)
        .map(|tv| tv.value_owned())
        .map_err(|e| format!("eval: {:?}", e))
}

// ═══════════════════════════════════════════════════════════════════════════
// REGRESSION — plain `,@rest` splice. GREEN at HEAD and after.
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn regression_variadic_splice() {
    let src = r#"
      (:wat::core::defmacro :my::vec-of
        [& items <- :AST<wat::holon::Holons>]
        -> :AST<wat::holon::HolonAST>
        `(:wat::core::Vector :wat::core::i64 ~@items))
      (:wat::core::defn :my::compute [] -> :wat::core::i64
        (:wat::core::match (:wat::core::first (:my::vec-of 10 20 30)) -> :wat::core::i64
          ((:wat::core::Some n) n)
          (:wat::core::None -1)))
    "#;
    assert_eq!(run(src).unwrap(), Value::i64(10));
}

// ═══════════════════════════════════════════════════════════════════════════
// MINT — `for` iterates + yields elements (≡ `,@items`). The core iteration.
// RED at HEAD (`for` unrecognized → `~x` unbound) → `#[ignore]`.
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn mint_for_yields_elements() {
    let src = r#"
      (:wat::core::defmacro :my::vof
        [& items <- :AST<wat::holon::Holons>]
        -> :AST<wat::holon::HolonAST>
        `(:wat::core::Vector :wat::core::i64 ~@(:wat::core::for [x items] ~x)))
      (:wat::core::defn :my::compute [] -> :wat::core::i64
        (:wat::core::match (:wat::core::first (:my::vof 10 20 30)) -> :wat::core::i64
          ((:wat::core::Some n) n)
          (:wat::core::None -1)))
    "#;
    assert_eq!(run(src).unwrap(), Value::i64(10));
}

// ═══════════════════════════════════════════════════════════════════════════
// MINT — `for` instantiates a TEMPLATE per element (the generative power).
// `(for [x items] (+ x 1))` over 10 20 30 → 11 21 31; first → 11.
// RED at HEAD → `#[ignore]`.
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn mint_for_transforms_per_element() {
    let src = r#"
      (:wat::core::defmacro :my::inc-vof
        [& items <- :AST<wat::holon::Holons>]
        -> :AST<wat::holon::HolonAST>
        `(:wat::core::Vector :wat::core::i64 ~@(:wat::core::for [x items] (:wat::core::i64::+ ~x 1))))
      (:wat::core::defn :my::compute [] -> :wat::core::i64
        (:wat::core::match (:wat::core::first (:my::inc-vof 10 20 30)) -> :wat::core::i64
          ((:wat::core::Some n) n)
          (:wat::core::None -1)))
    "#;
    assert_eq!(run(src).unwrap(), Value::i64(11));
}
