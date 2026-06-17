//! Arc 277.1b — disconfirming probe: the nested-if-=-ladder rule has no AUTO-FIX yet (RED at HEAD).
//!
//! 277.1 shipped the ladder rule REPORT-ONLY (Finding.fix = ""). The arc-281 keystone (`ast-end-span`)
//! now makes a structural node's extent readable, so the rule can carry a real fix that REWRITES the
//! whole `(if (= x "a") true (if … false))` ladder into `(:wat::core::contains? (:wat::core::HashSet
//! :wat::type::Infer "a" "b" "c") x)`. `:wat::lint::lint-fix-file` lints a SourceFile + applies its
//! findings' fixes (via fix.wat's fix-text-apply), returning the fixed source.
//!
//! At HEAD `lint-fix-file` is undefined → RED. GREEN when 277.1b ships the fix + the applier.
//!
//! Run: cargo test --release -p wat --test probe_arc277_1b_ladder_autofix -- --include-ignored

use std::sync::Arc;
use wat::freeze::{eval_in_frozen, startup_from_source};
use wat::load::InMemoryLoader;
use wat::runtime::{Environment, Value};

// A SourceFile whose body is a 3-deep nested-if-=-ladder over one var `x`. lint-fix-file must rewrite
// the ladder form into a (contains? (HashSet …) x) call — so the fixed source contains "contains?"
// and no longer contains the nested "(:wat::core::if (:wat::core::= x".
const LINT_FIX: &str = r#"
(:wat::lint::lint-fix-file
  (:wat::deporder::SourceFile "t.wat"
    "(:wat::core::defn :t::f [x <- :wat::core::String] -> :wat::core::bool (:wat::core::if (:wat::core::= x \"a\") true (:wat::core::if (:wat::core::= x \"b\") true (:wat::core::if (:wat::core::= x \"c\") true false))))"))
"#;

#[test]
#[ignore = "arc 277.1b — RED until the ladder auto-fix + applier ship; un-ignore on green"]
fn ladder_autofix_rewrites_to_contains() {
    let world = startup_from_source("(:wat::core::defn :user::main [] -> :wat::core::nil nil)", None,
        Arc::new(InMemoryLoader::new()))
        .expect("startup");
    let ast = wat::parse_one!(LINT_FIX).expect("parse the lint-fix-file call");
    let got = eval_in_frozen(&ast, &world, &Environment::new())
        .unwrap_or_else(|e| panic!("lint-fix-file raised (undefined at HEAD): {e:?}"))
        .value_owned();
    let fixed = match got {
        Value::String(ref s) => s.to_string(),
        other => panic!("lint-fix-file must return the fixed source String; got {other:?}"),
    };
    assert!(
        fixed.contains("contains?") && fixed.contains("HashSet"),
        "the ladder must be rewritten to a (contains? (HashSet …) x) call; got: {fixed}"
    );
    assert!(
        !fixed.contains("(:wat::core::if (:wat::core::= x"),
        "the nested-if-=-ladder must be GONE after the fix; got: {fixed}"
    );
}
