//! Arc 269 vehicle — `:wat::fix::rename-keyword-prefix`: a reusable, comment-faithful
//! keyword-PREFIX rename rule on the fix-text engine.
//!
//! The spawn-coherence move (`:wat::kernel::{Bound,Spawned,ServiceEvent}` → `:wat::spawn::…`) and
//! the larger arc-269 kernel split both need a keyword/namespace rename codemod. fix-wat has the
//! comment-faithful span-edit engine (`fix-text` / `fix-text-apply`) + `fix-macro-param-types`, but
//! no rename rule. This is it — a PREFIX rename so a single call catches the base keyword AND its
//! accessor/variant forms (`:p::Bound`, `:p::Bound/listener`, `:p::ServiceEvent::Shutdown` all share
//! the `:p::Bound` / `:p::ServiceEvent` prefix).
//!
//! Contract: `(:wat::fix::rename-keyword-prefix old-prefix new-prefix src) -> migrated-src` — for
//! every keyword leaf whose name STARTS WITH `old-prefix`, splice the prefix → `new-prefix`
//! (the suffix, incl. `/accessor` and `::Variant`, is preserved); comments + formatting survive
//! byte-identical (rides `fix-text-apply`'s right-to-left span splice).
//!
//! RED at HEAD: `:wat::fix::rename-keyword-prefix` does not exist → UnknownCallee. GREEN once the
//! rule ships on the fix-text engine.
//!
//! Run: cargo test --release -p wat --test probe_arc269_rename_keyword_prefix

use std::sync::Arc;
use wat::freeze::{eval_in_frozen, startup_from_source};
use wat::load::InMemoryLoader;
use wat::runtime::{Environment, Value};

// A tiny wat source with the base keyword, an accessor form, a variant form, AND a comment that
// must survive byte-identical. The rule renames the `:my::old` prefix → `:my::new`.
const PROGRAM: &str = r#"
(:wat::core::defn :user::go [] -> :wat::core::String
  (:wat::fix::rename-keyword-prefix ":my::old::Bound" ":my::new::Bound"
    "(:wat::core::let
   ;; KEEP THIS COMMENT byte-identical
   [b (:my::old::Bound/listener x)
    s (:my::old::Bound/address b)]
   b)"))
(:wat::core::defn :user::main [] -> :wat::core::nil nil)
"#;

#[test]
fn rename_keyword_prefix_swaps_prefix_comment_faithful() {
    let world = startup_from_source(PROGRAM, None, Arc::new(InMemoryLoader::new()))
        .expect("startup should succeed (rename-keyword-prefix exists on the fix-text engine)");
    let ast = wat::parse_one!("(:user::go)").expect("parse");
    let got = match eval_in_frozen(&ast, &world, &Environment::new())
        .map(|tv| tv.value_owned())
        .unwrap_or_else(|e| panic!("go raised: {e:?}"))
    {
        Value::String(s) => (*s).clone(),
        other => panic!("expected String, got {other:?}"),
    };
    assert!(
        got.contains(":my::new::Bound/listener") && got.contains(":my::new::Bound/address"),
        "prefix should be swapped on both accessor forms; got:\n{got}"
    );
    assert!(
        !got.contains(":my::old::Bound"),
        "no old prefix should remain; got:\n{got}"
    );
    assert!(
        got.contains(";; KEEP THIS COMMENT byte-identical"),
        "the comment must survive byte-identical (comment-faithful); got:\n{got}"
    );
}
