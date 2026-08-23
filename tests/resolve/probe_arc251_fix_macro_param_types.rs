//! Arc 251 — the first real fix-wat RULE riding the fix-text engine: `fix-macro-param-types`.
//!
//! `(:wat::fix::fix-macro-param-types src) -> migrated-src` — read-string to locate, splice the
//! ORIGINAL text (reuses fix-text-apply + fix-text-offset-of), so comments survive byte-identical.
//!
//! RED at HEAD: `:wat::fix::fix-macro-param-types` does not exist.
//!
//! Run: cargo test --release -p wat --test probe_arc251_fix_macro_param_types

use wat::freeze::call_beside_value;
use wat::runtime::Value;

#[test]
fn fix_macro_param_types_rewrites_defmacro_only_comment_faithful() {
    // just-eval (rubric): the fix-macro-param-types call lives in the co-located fixture's
    // `:user::run`, driven via `call_beside_value`.
    //
    // Arc 109 wave 2 class 3 (a) — the fixture's rest-param annotation used to read
    // `:AST<wat::holon::Holons>`; that angle text is now refused at the lexer, before
    // `fix-macro-param-types`'s own `read-string` call ever runs. See the fixture's
    // comment for why swapping in a plain keyword there is a faithful (a) migration,
    // not a weakened one: the rewrite rule discards the old type's content entirely.
    let out = match call_beside_value(file!(), ":user::run") {
        Ok(Value::String(s)) => (*s).clone(),
        other => panic!("expected migrated source String; got {other:?}"),
    };
    assert_eq!(
        out,
        ";; keep me byte-identical\n\
(:wat::core::defmacro :user::m [a <- :wat::WatAST & rest <- (:wat::core::Vector :- [:wat::WatAST])] -> :wat::WatAST a)\n\
(:wat::core::defn :user::f [x <- :wat::core::i64] -> :wat::core::i64 x)",
        "fix-macro-param-types golden mismatch; comment must survive byte-identical, \
         defmacro params rewritten to :wat::WatAST, defn untouched"
    );
}
