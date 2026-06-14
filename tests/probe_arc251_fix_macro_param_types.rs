//! Arc 251 — the first real fix-wat RULE riding the fix-text engine: `fix-macro-param-types`.
//!
//! The macro-param-type bug: a defmacro param/return is annotated with a type the engine discards
//! (`:AST<T>` / `:wat::holon::HolonAST`), when the only honest type is `:wat::WatAST` (a macro arg
//! is always a form). This rule rewrites, COMMENT-FAITHFULLY (riding fix-text-apply's span-splice):
//!   - each fixed param's type    → `:wat::WatAST`
//!   - the rest param's type      → `:wat::core::Vector<wat::WatAST>`
//!   - the return type            → `:wat::WatAST`
//! and touches DEFMACRO forms ONLY — a sibling `defn`/`fn` type annotation is a real type and must
//! survive untouched.
//!
//! `(:wat::fix::fix-macro-param-types src) -> migrated-src` — read-string to locate, splice the
//! ORIGINAL text (reuses fix-text-apply + fix-text-offset-of), so comments survive byte-identical.
//!
//! RED at HEAD: `:wat::fix::fix-macro-param-types` does not exist.
//!
//! Run: cargo test --release -p wat --test probe_arc251_fix_macro_param_types

use std::sync::Arc;
use wat::freeze::{eval_in_frozen, startup_from_source};
use wat::load::InMemoryLoader;
use wat::runtime::{Environment, Value};

// Fixture: a comment, a defmacro with a fixed + rest param + holon return, and a defn whose
// real types must NOT be touched.
const PROGRAM: &str = r##"
(:wat::core::defn :user::run [] -> :wat::core::String
  (:wat::fix::fix-macro-param-types ";; keep me byte-identical\n(:wat::core::defmacro :user::m [a <- :wat::holon::HolonAST & rest <- :AST<wat::holon::Holons>] -> :wat::holon::HolonAST a)\n(:wat::core::defn :user::f [x <- :wat::core::i64] -> :wat::core::i64 x)"))

(:wat::core::defn :user::main [] -> :wat::core::nil nil)
"##;

#[test]
fn fix_macro_param_types_rewrites_defmacro_only_comment_faithful() {
    let world = startup_from_source(PROGRAM, None, Arc::new(InMemoryLoader::new()))
        .expect("startup should succeed (fix-macro-param-types rule)");
    let ast = wat::parse_one!("(:user::run)").expect("parse");
    let out = match eval_in_frozen(&ast, &world, &Environment::new()).map(|tv| tv.value_owned()) {
        Ok(Value::String(s)) => (*s).clone(),
        other => panic!("expected migrated source String; got {other:?}"),
    };
    // comment survives byte-identical
    assert!(out.contains(";; keep me byte-identical"), "comment must survive; got:\n{out}");
    // defmacro param/rest/return types rewritten to the honest AST types
    assert!(out.contains("a <- :wat::WatAST"), "fixed param type → :wat::WatAST; got:\n{out}");
    assert!(out.contains("rest <- :wat::core::Vector<wat::WatAST>"), "rest param type → Vector<wat::WatAST>; got:\n{out}");
    assert!(out.contains("-> :wat::WatAST a)"), "return type → :wat::WatAST; got:\n{out}");
    assert!(!out.contains(":wat::holon::HolonAST") && !out.contains(":AST<"), "no holon/AST<> macro types left; got:\n{out}");
    // the sibling defn's REAL types are untouched (defmacro-scoped rule)
    assert!(out.contains("x <- :wat::core::i64") && out.contains("-> :wat::core::i64 x)"),
        "the defn's real types must NOT be touched; got:\n{out}");
}
