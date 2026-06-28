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

use wat::freeze::{eval_in_frozen, startup_beside};
use wat::runtime::{Environment, Value};

#[test]
fn rename_reaches_type_arguments() {
    let world = startup_beside(file!()).expect("startup");
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
