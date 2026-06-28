//! Arc 251 Strike-4.2 — wat's `rewrite-clj`: the comment-faithful span-edit codemod.
//!
//! This probe gates the new text-level entry `:wat::fix::fix-text` (a runtime defn in fix.wat;
//! intueri may refine the name): `(fix-text src) -> migrated-src`, applying fix-source's rules
//! comment-faithfully. Fixture = a `;;` comment + an annotated-if (strip-if's target). Asserts
//! the comment survives byte-identical AND the redundant `-> :T` annotation is gone.
//!
//! RED at HEAD: `:wat::fix::fix-text` does not exist (only the node→node `fix-source` does).
//!
//! Run: cargo test --release -p wat --test probe_arc251_fix_text_comment_faithful

use wat::freeze::{eval_in_frozen, startup_beside};
use wat::runtime::{Environment, Value};

#[test]
fn fix_text_preserves_comments_and_strips_redundant_annotation() {
    let world = startup_beside(file!())
        .expect("startup should succeed (251.5-4.2: fix-text comment-faithful codemod)");
    let ast = wat::parse_one!("(:user::probe)").expect("parse");
    let out = match eval_in_frozen(&ast, &world, &Environment::new()).map(|tv| tv.value_owned()) {
        Ok(Value::String(s)) => (*s).clone(),
        other => panic!("expected fix-text to return a migrated source String; got {other:?}"),
    };
    assert!(
        out.contains(";; this doc comment must survive byte-identical"),
        "the comment must survive BYTE-IDENTICAL (span-edit, not AST-reprint); got:\n{out}"
    );
    assert!(
        !out.contains("-> :wat::core::i64"),
        "the redundant if-return annotation `-> :wat::core::i64` must be stripped; got:\n{out}"
    );
}

#[test]
fn fix_text_is_comment_faithful_on_many_comments_and_idempotent() {
    let world = startup_beside(file!())
        .expect("startup (richer fixture + idempotence)");
    let s = |expr: &str| -> String {
        let ast = wat::parse_one!(expr).expect("parse");
        match eval_in_frozen(&ast, &world, &Environment::new()).map(|tv| tv.value_owned()) {
            Ok(Value::String(s)) => (*s).clone(),
            other => panic!("expected a migrated String from {expr}; got {other:?}"),
        }
    };
    let once = s("(:user::once)");
    for c in ["alpha comment", "beta comment", "gamma trailing"] {
        assert!(once.contains(&format!(";; {c}")), "comment `;; {c}` must survive byte-identical; got:\n{once}");
    }
    assert!(once.contains("\n\n"), "the blank line between forms must survive; got:\n{once}");
    assert!(!once.contains("-> :wat::core::i64"), "the redundant annotation must be stripped; got:\n{once}");
    let twice = s("(:user::twice)");
    assert_eq!(twice, once, "fix-text must be IDEMPOTENT — a second pass yields zero edits (byte-identical)");
}
