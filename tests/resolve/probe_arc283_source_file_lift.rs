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

use wat::freeze::call_beside;
use wat::runtime::Value;

#[test]
fn source_file_is_at_the_neutral_home() {
    // just-eval (rubric): the File ctor + accessor call lives in the co-located fixture's
    // `:user::compute`, driven via `call_beside`.
    let got = call_beside(file!(), ":user::compute")
        .unwrap_or_else(|e| panic!(":wat::source::File undefined at HEAD: {e:?}"));
    match got {
        Value::String(ref s) => assert_eq!(s.as_str(), "t.wat", "File/path must read the path field"),
        other => panic!("File/path must return a String; got {other:?}"),
    }
}
