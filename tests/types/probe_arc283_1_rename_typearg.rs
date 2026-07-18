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

use wat::freeze::call_beside;
use wat::runtime::Value;

#[test]
fn rename_reaches_type_arguments() {
    let out = call_beside(file!(), ":user::run").expect("rename-keyword-prefix must eval");
    let s = match out {
        Value::String(ref s) => s.to_string(),
        other => panic!("expected migrated source String; got {other:?}"),
    };
    assert_eq!(s, include_str!("probe_arc283_1_rename_typearg__renamed.wat"));
}
