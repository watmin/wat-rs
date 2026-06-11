//! Strike 3 (examinare disconfirming probe) — fix-source's position-aware LOCAL rules.
//!
//! fix.wat grows from {strip-if, head-rule} into the full grammar-FREE walk: at each child,
//!   - a `::`-keyword head/ref → faithful symbol           (keyword/to-symbol)
//!   - a bare `<-` / `->` arrow → `:-`                      (context-free: threading head is the
//!     KEYWORD `:wat::core::->`, the annotation is the bare SYMBOL `->` — different tokens)
//!   - a keyword right after an arrow → faithful type form  (keyword/to-type-form, post-arrow)
//!   - a structurally-type-shaped keyword (`<>`/`(`/`Fn`) → faithful type form  (anywhere)
//!   - else → recurse / leave.
//! Grammar-bearing forms (ctor type-args, declarations) are NOT this strike — Strike 4's
//! throwaway migrator dispatches them away before fix.wat sees them. So these contracts use
//! binders + fn-literals: no def-name or ctor-arg ambiguity.
//!
//! C01 arrow            : [x <- y]                                  -> [x :- y]
//! C02 post-arrow scalar: [x <- :wat::core::i64]                    -> [x :- wat.type/i64]
//! C03 structural param : [x <- :wat::core::Vector<wat::core::i64>] -> [x :- (wat.type/Vector wat.type/i64)]
//! C04 head (regression): (:wat::core::map f xs)                    -> (wat.core/map f xs)
//! C05 full fn-literal  : (:wat::core::fn [a <- :wat::core::i64] -> :wat::core::bool a)
//!                                       -> (wat.core/fn [a :- wat.type/i64] :- wat.type/bool a)
//!
//! RED at HEAD: fix.wat only does {strip-if, head-rule} — no arrows, no post-arrow/structural types.
//!
//! Run: `cargo test --release --test probe_arc251_fix_source_local_rules`

use std::sync::Arc;
use wat::freeze::{eval_in_frozen, startup_from_source};
use wat::load::InMemoryLoader;
use wat::runtime::{Environment, Value};

// fix-source loads as blessed stdlib (wat/fix.wat); available at :wat::fix::* after startup.
fn eval_string(body: &str) -> Result<String, String> {
    let src = format!(
        "(:wat::core::defn :user::topform [src <- :wat::core::String] -> :wat::WatAST \
            (:wat::core::Option/expect -> :wat::WatAST \
              (:wat::core::first (:wat::core::ast->children (:wat::core::read-string src))) \"topform\"))\n\
         (:wat::core::defn :user::compute [] -> :wat::core::String {body})\n\
         (:wat::core::defn :user::main [] -> :wat::core::nil nil)",
    );
    let world = startup_from_source(&src, None, Arc::new(InMemoryLoader::new()))
        .map_err(|e| format!("startup/check: {e:?}"))?;
    let ast = wat::parse_one!("(:user::compute)").expect("parse");
    match eval_in_frozen(&ast, &world, &Environment::new())
        .map(|tv| tv.value_owned())
        .map_err(|e| format!("eval: {e:?}"))?
    {
        Value::String(s) => Ok((*s).clone()),
        other => Err(format!("non-string: {other:?}")),
    }
}

fn embed(payload: &str) -> String {
    payload.replace('\\', "\\\\").replace('"', "\\\"")
}

fn fix(src: &str) -> Result<String, String> {
    eval_string(&format!(
        "(:wat::core::write-forms (:wat::fix::fix-source (:user::topform \"{}\")))",
        embed(src)
    ))
}

#[test]
fn contract_01_arrow_in_binder() {
    assert_eq!(fix("[x <- y]"), Ok("[x :- y]".into()));
}

#[test]
fn contract_02_post_arrow_scalar_type() {
    assert_eq!(fix("[x <- :wat::core::i64]"), Ok("[x :- wat.type/i64]".into()));
}

#[test]
fn contract_03_structural_parametric_type() {
    assert_eq!(
        fix("[x <- :wat::core::Vector<wat::core::i64>]"),
        Ok("[x :- (wat.type/Vector wat.type/i64)]".into())
    );
}

#[test]
fn contract_04_head_still_inverts() {
    assert_eq!(fix("(:wat::core::map f xs)"), Ok("(wat.core/map f xs)".into()));
}

#[test]
fn contract_05_full_fn_literal() {
    assert_eq!(
        fix("(:wat::core::fn [a <- :wat::core::i64] -> :wat::core::bool a)"),
        Ok("(wat.core/fn [a :- wat.type/i64] :- wat.type/bool a)".into()),
        "head inverts, binder + return arrows -> :-, both types -> wat.type/, in one pass"
    );
}
