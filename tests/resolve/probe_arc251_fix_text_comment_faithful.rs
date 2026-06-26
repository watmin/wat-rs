//! Arc 251 Strike-4.2 — wat's `rewrite-clj`: the comment-faithful span-edit codemod.
//!
//! The naive `read-string → fix-source → write-forms` round-trip DELETES every comment (a
//! Lisp reader drops trivia by design — confirmed: lexer.rs skips `;;`). The corpus carries
//! 2,000+ doc-lines, so that's disqualifying. The fix (DESIGN-STONE-251.5-4.2): a SPAN-EDIT
//! codemod — parse only to LOCATE edits (via `ast-span`), then splice the ORIGINAL text
//! right-to-left, so comments + formatting survive byte-identical.
//!
//! This probe gates the new text-level entry `:wat::fix::fix-text` (a runtime defn in fix.wat;
//! intueri may refine the name): `(fix-text src) -> migrated-src`, applying fix-source's rules
//! comment-faithfully. Fixture = a `;;` comment + an annotated-if (strip-if's target). Asserts
//! the comment survives byte-identical AND the redundant `-> :T` annotation is gone.
//!
//! RED at HEAD: `:wat::fix::fix-text` does not exist (only the node→node `fix-source` does).
//!
//! Run: cargo test --release -p wat --test probe_arc251_fix_text_comment_faithful

use std::sync::Arc;
use wat::freeze::{eval_in_frozen, startup_from_source};
use wat::load::InMemoryLoader;
use wat::runtime::{Environment, Value};

// A two-line source: a doc comment that MUST survive byte-identical, and an annotated-if whose
// `-> :wat::core::i64` strip-if removes. (`\n` is a wat string escape; the `r##"..."##` keeps
// the backslash literal so the wat lexer sees the newline.)
const PROGRAM: &str = r##"
(:wat::core::defn :user::probe [] -> :wat::core::String
  (:wat::fix::fix-text ";; this doc comment must survive byte-identical\n(:wat::core::if true -> :wat::core::i64 1 2)"))

(:wat::core::defn :user::main [] -> :wat::core::nil nil)
"##;

#[test]
fn fix_text_preserves_comments_and_strips_redundant_annotation() {
    let world = startup_from_source(PROGRAM, None, Arc::new(InMemoryLoader::new()))
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

// Richer fixture (3 comments + a blank line + an annotated-if) + idempotence: every comment and
// the blank line survive byte-identical, and a second pass over the migrated source is a no-op
// (faithful forms yield zero edits) — the design's full gate, probe-sized.
const RICHER: &str = r##"
(:wat::core::defn :user::once [] -> :wat::core::String
  (:wat::fix::fix-text ";; alpha comment\n;; beta comment\n\n(:wat::core::if true -> :wat::core::i64 1 2)\n;; gamma trailing"))

(:wat::core::defn :user::twice [] -> :wat::core::String
  (:wat::fix::fix-text (:user::once)))

(:wat::core::defn :user::main [] -> :wat::core::nil nil)
"##;

#[test]
fn fix_text_is_comment_faithful_on_many_comments_and_idempotent() {
    let world = startup_from_source(RICHER, None, Arc::new(InMemoryLoader::new()))
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
