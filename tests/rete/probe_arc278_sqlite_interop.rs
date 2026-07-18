//! Arc 278 stone S1 gate — the functional proof that the baked `:wat::sqlite'` RAW interop
//! (`src/rust_deps/sqlite.rs` + `wat/sqlite.wat`, a fresh thread-owned errors-as-values rusqlite
//! binding) actually round-trips open -> execute-ddl -> execute -> select against a REAL sqlite
//! backend, and that a genuine sqlite fault (a duplicate PK) and a genuine driver fault (a bad
//! open path) both come back as `:wat::sqlite'::Error` VALUES — never a panic.
//!
//! Run: `cargo nextest run --release -E 'test(sqlite_interop)'`

use wat::freeze::call_beside;

#[test]
fn sqlite_interop() {
    let result = call_beside(file!(), ":user::sqlite_interop");
    assert!(
        result.is_ok(),
        "sqlite_interop deftest' must pass (real sqlite open/execute-ddl/execute/select \
         round-trip + Constraint/Fatal fault classification); got Err: {result:?}"
    );
}
