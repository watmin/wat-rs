//! Arc 283 — disconfirming probe: the source-unit struct still lives in deporder (RED at HEAD).
//!
//! `SourceFile {path, source}` was born in `deporder` (arc 275) but is the universal input to any
//! source-processing tool (deporder, lint, the sweep, arc 282's Rust facts). It lifts to a neutral
//! home, named by intueri: **`:wat::source::File`** in **`wat/source.wat`** (loaded before deporder).
//! `deporder` keeps `Violation`/`SymDef`.
//!
//! At HEAD `:wat::source::File` is undefined → RED. GREEN when arc 283 lifts the record (and the
//! dogfooded `fix::rename-keyword-prefix` sweep retires every `:wat::deporder::SourceFile`).
//!
//! Run: cargo test --release -p wat --test probe_arc283_source_file_lift -- --include-ignored

use wat::freeze::{eval_in_frozen, startup_bare};
use wat::runtime::{Environment, Value};

#[test]
fn source_file_is_at_the_neutral_home() {
    let world = startup_bare().expect("startup");
    // Construct a :wat::source::File and read its path back through the accessor.
    let ast = wat::parse_one!(
        "(:wat::source::File/path (:wat::source::File \"t.wat\" \"(:t::f)\"))"
    ).expect("parse the File ctor + accessor");
    let got = eval_in_frozen(&ast, &world, &Environment::new())
        .unwrap_or_else(|e| panic!(":wat::source::File undefined at HEAD: {e:?}"))
        .value_owned();
    match got {
        Value::String(ref s) => assert_eq!(s.as_str(), "t.wat", "File/path must read the path field"),
        other => panic!("File/path must return a String; got {other:?}"),
    }
}
