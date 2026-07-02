//! Arc 251 — the first real fix-wat RULE riding the fix-text engine: `fix-macro-param-types`.
//!
//! `(:wat::fix::fix-macro-param-types src) -> migrated-src` — read-string to locate, splice the
//! ORIGINAL text (reuses fix-text-apply + fix-text-offset-of), so comments survive byte-identical.
//!
//! RED at HEAD: `:wat::fix::fix-macro-param-types` does not exist.
//!
//! Run: cargo test --release -p wat --test probe_arc251_fix_macro_param_types

use wat::freeze::{eval_in_frozen, startup_beside};
use wat::runtime::{Environment, Value};

#[test]
fn fix_macro_param_types_rewrites_defmacro_only_comment_faithful() {
    let world = startup_beside(file!())
        .expect("startup should succeed (fix-macro-param-types rule)");
    let ast = wat::parse_one!("(:user::run)").expect("parse");
    let out = match eval_in_frozen(&ast, &world, &Environment::new()).map(|tv| tv.value_owned()) {
        Ok(Value::String(s)) => (*s).clone(),
        other => panic!("expected migrated source String; got {other:?}"),
    };
    assert_eq!(
        out,
        ";; keep me byte-identical\n\
(:wat::core::defmacro :user::m [a <- :wat::WatAST & rest <- :wat::core::Vector<wat::WatAST>] -> :wat::WatAST a)\n\
(:wat::core::defn :user::f [x <- :wat::core::i64] -> :wat::core::i64 x)",
        "fix-macro-param-types golden mismatch; comment must survive byte-identical, \
         defmacro params rewritten to :wat::WatAST, defn untouched"
    );
}
