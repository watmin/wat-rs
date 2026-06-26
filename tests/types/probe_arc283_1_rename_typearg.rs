//! Arc 283.1 — disconfirming probe: rename-keyword-prefix can't reach TYPE ARGUMENTS (RED at HEAD).
//!
//! `rename-keyword-prefix` (arc 269) renames keyword LEAVES whose name *starts-with* the prefix. But a
//! symbol used as a TYPE ARGUMENT — `:wat::core::Vector<t::Old>` — is one keyword starting with
//! `:wat::core::Vector`, with the renamed name embedded (colon-stripped) inside the `<…>`. Start-anchored
//! matching misses it. This is the gap arc 283's dogfood surfaced (a `Vector<…SourceFile>` survivor that
//! broke the stdlib). Renaming a TYPE is the common case — the tool must reach into type-args.
//!
//! At HEAD: renaming `:t::Old` → `:t::New` over a source with `:wat::core::Vector<t::Old>` leaves the
//! type-arg as `Vector<t::Old>` → RED. GREEN when 283.1 makes the rename boundary-aware over the whole
//! keyword name (start-anchored + embedded), without corrupting `:t::OldExtra`.
//!
//! Run: cargo test --release -p wat --test probe_arc283_1_rename_typearg -- --include-ignored

use std::sync::Arc;
use wat::freeze::{eval_in_frozen, startup_from_source};
use wat::load::InMemoryLoader;
use wat::runtime::{Environment, Value};

// Rename :t::Old → :t::New over a source exercising: a TYPE-ARG (Vector<t::Old>), a start-anchored
// return type (:t::Old), an accessor (:t::Old/make), and a boundary DECOY (:t::OldExtra — must NOT
// rename). The fixture is the `src` arg; rename-keyword-prefix returns the migrated source.
const PROGRAM: &str = r#"
(:wat::core::defn :user::run [] -> :wat::core::String
  (:wat::fix::rename-keyword-prefix ":t::Old" ":t::New"
    "(:wat::core::defn :u::f [xs <- :wat::core::Vector<t::Old> y <- :t::OldExtra] -> :t::Old (:t::Old/make xs))"))
(:wat::core::defn :user::main [] -> :wat::core::nil nil)
"#;

#[test]
fn rename_reaches_type_arguments() {
    let world = startup_from_source(PROGRAM, None, Arc::new(InMemoryLoader::new())).expect("startup");
    let ast = wat::parse_one!("(:user::run)").expect("parse");
    let out = eval_in_frozen(&ast, &world, &Environment::new())
        .expect("rename-keyword-prefix must eval")
        .value_owned();
    let s = match out {
        Value::String(ref s) => s.to_string(),
        other => panic!("expected migrated source String; got {other:?}"),
    };
    // THE GAP: the type-arg must be renamed.
    assert!(s.contains("Vector<t::New>"), "type-arg must rename to Vector<t::New>; got: {s}");
    // Regression: start-anchored cases still rename (return type + accessor).
    assert!(s.contains(" -> :t::New "), "return type must rename to :t::New; got: {s}");
    assert!(s.contains(":t::New/make"), "accessor must rename to :t::New/make; got: {s}");
    // BAR-RAISE: the boundary decoy must survive untouched (no SourceFileExtra-style corruption).
    assert!(s.contains(":t::OldExtra"), ":t::OldExtra must NOT be renamed (boundary guard); got: {s}");
    // And the old type-arg name must be gone entirely.
    assert!(!s.contains("Vector<t::Old>"), "no Vector<t::Old> may survive; got: {s}");
}
