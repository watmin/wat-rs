//! PROBE — one authority that can answer `is this a type?` for every type keyword.
//!
//! `type-of` asks `TypeEnv::get` (structure). Leaves have membership and no
//! TypeDef, so it raises "unknown type"; Doctrine 1 also refuses scalars as
//! values before the query runs. Stone Q is the membership sibling.
//!
//! Rows (exact stdout, one fact per line):
//!   primitive `:wat::core::i64`     true
//!   builtin   `:wat::core::Vector`  true
//!   user type `:usr::Shape`         true
//!   nonexistent `:usr::TotallyMadeUp` false  ← the row the stone exists for

use std::path::PathBuf;
use std::process::{Command, Stdio};

fn run(fixture: &str) -> (i32, String) {
    let path: PathBuf = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/reflection")
        .join(fixture);
    let out = Command::new(env!("CARGO_BIN_EXE_wat"))
        .arg(&path)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .expect("spawn wat");
    (
        out.status.code().unwrap_or(-1),
        String::from_utf8_lossy(&out.stdout).trim().to_owned(),
    )
}

#[test]
fn is_type_answers_primitive_builtin_user_and_false_for_a_typo() {
    let (code, out) = run("probe_arc296_is_type.wat");
    assert_eq!(code, 0, "is-type? fixture must run; got:\n{out}");
    // EDN bools — not quoted strings. A `contains` check would pass on "true\nfalse\ntrue\ntrue".
    assert_eq!(out, "true\ntrue\ntrue\nfalse");
}
