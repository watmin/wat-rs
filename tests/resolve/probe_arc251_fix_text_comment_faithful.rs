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

use wat::freeze::call_beside;
use wat::runtime::Value;

// just-eval (rubric): each `:user::…` zero-arg fn lives in the co-located fixture;
// drive it via `call_beside` and inspect the returned typed String.
fn eval_string(fn_name: &str) -> String {
    match call_beside(file!(), fn_name) {
        Ok(Value::String(s)) => (*s).clone(),
        other => panic!("expected a migrated String from {fn_name}; got {other:?}"),
    }
}

#[test]
fn fix_text_preserves_comments_and_strips_redundant_annotation() {
    let out = eval_string(":user::probe");
    assert_eq!(
        out,
        include_str!("probe_arc251_fix_text_comment_faithful__probe-comment-faithful.wat"),
        "fix-text golden mismatch; comment must survive byte-identical, \
         -> :wat::core::i64 annotation must be stripped"
    );
}

#[test]
fn fix_text_is_comment_faithful_on_many_comments_and_idempotent() {
    let once = eval_string(":user::once");
    assert_eq!(
        once,
        include_str!("probe_arc251_fix_text_comment_faithful__once-many-comments-idempotent.wat"),
        "fix-text (many-comments) golden mismatch; all comments + blank line must survive, \
         -> :wat::core::i64 annotation must be stripped"
    );
    let twice = eval_string(":user::twice");
    assert_eq!(twice, once, "fix-text must be IDEMPOTENT — a second pass yields zero edits (byte-identical)");
}
