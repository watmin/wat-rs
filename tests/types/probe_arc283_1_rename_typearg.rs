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
    assert_eq!(s, "(:wat::core::defn :u::f [xs <- :wat::core::Vector<t::New> y <- :t::OldExtra] -> :t::New (:t::New/make xs))");
}
