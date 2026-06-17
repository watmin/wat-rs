//! Arc 279.1 — FOUNDATION probe (the worked reference the format rewrite copies).
//!
//! Proves the macro-eval engine can run the exact mechanics the `{{`/`}}` tokenizer needs:
//!   1. `string::subs` at EXPAND TIME (newly added to is_pure_total) — char-walk via
//!      `(map (fn [i] (subs s i (+ i 1))) (range 0 (length s)))`.
//!   2. A `Tuple` heterogeneous ACCUMULATOR threaded through `foldl`, accessed via first/second.
//!   3. Per-char `=` compares + `if` driving state, at expand time.
//!
//! The probe macro `:user::strip-braces` walks a string literal char-by-char, drops every `{`,
//! keeps the rest, and counts the dropped braces — returning a Tuple-driven result emitted as a
//! literal. If this expands + runs GREEN, the real tokenizer's foundation is proven.
//!
//! Run: cargo test --release -p wat --test probe_arc279b_subs_tuple_macro_eval

use std::sync::Arc;
use wat::freeze::{eval_in_frozen, startup_from_source};
use wat::load::InMemoryLoader;
use wat::runtime::{Environment, Value};

// A macro that, at expand time, walks the chars of a string literal carrying a Tuple(kept, n-open)
// accumulator: appends each non-`{` char to `kept`, increments `n-open` on each `{`. Emits the
// string literal `"<kept>|<n-open>"`.
const PROGRAM: &str = r#"
(:wat::core::defmacro :user::strip-braces
  [s <- :wat::WatAST]
  -> :wat::WatAST
  (:wat::core::let
    [str   (:wat::core::ast-name s)
     len   (:wat::core::string::length str)
     chars (:wat::core::map
             (:wat::core::fn [i <- :wat::core::i64] -> :wat::core::String
               (:wat::core::string::subs str i (:wat::core::i64::+ i 1)))
             (:wat::core::range 0 len))
     final (:wat::core::foldl
             (:wat::core::fn [acc <- :wat::core::Tuple
                              c   <- :wat::core::String]
               -> :wat::core::Tuple
               (:wat::core::let
                 [kept   (:wat::core::first acc)
                  nopen  (:wat::core::second acc)]
                 (:wat::core::if
                   (:wat::core::= c "{")
                   -> :wat::core::Tuple
                   (:wat::core::Tuple kept (:wat::core::i64::+ nopen 1))
                   (:wat::core::Tuple (:wat::core::string::concat kept c) nopen))))
             (:wat::core::Tuple "" 0)
             chars)
     kept   (:wat::core::first final)
     nopen  (:wat::core::second final)
     out    (:wat::core::string::concat kept
              (:wat::core::string::concat "|" (:wat::core::i64::to-string nopen)))]
    (:wat::core::Option/expect -> :wat::WatAST
      (:wat::core::first
        (:wat::core::ast->children
          (:wat::core::read-string
            (:wat::core::string::concat "\"" (:wat::core::string::concat out "\"")))))
      "strip-braces: node")))

(:wat::core::defn :user::probe [] -> :wat::core::String (:user::strip-braces "a{b{c"))
(:wat::core::defn :user::main [] -> :wat::core::nil nil)
"#;

#[test]
fn subs_tuple_char_walk_runs_at_macro_eval() {
    let world = startup_from_source(PROGRAM, None, Arc::new(InMemoryLoader::new()))
        .expect("strip-braces macro (subs + Tuple foldl) must expand cleanly at compile time");
    let ast = wat::parse_one!("(:user::probe)").expect("parse the defn call");
    let got = eval_in_frozen(&ast, &world, &Environment::new())
        .expect("probe must eval")
        .value_owned();
    let s = match got {
        Value::String(ref s) => s.to_string(),
        other => panic!("expected String; got {other:?}"),
    };
    // "a{b{c" → kept "abc", dropped 2 braces → "abc|2".
    assert_eq!(s, "abc|2", "subs char-walk + Tuple foldl at expand time; got {s:?}");
}
