//! Arc 278 stone S1 gate — the functional proof that the baked `:wat::sqlite'` RAW interop
//! (`src/rust_deps/sqlite.rs` + `wat/sqlite.wat`, a fresh thread-owned errors-as-values rusqlite
//! binding) actually round-trips open -> execute-ddl -> execute -> select against a REAL sqlite
//! backend, and that a genuine sqlite fault (a duplicate PK) and a genuine driver fault (a bad
//! open path) both come back as `:wat::sqlite'::Error` VALUES — never a panic.
//!
//! Run: `cargo nextest run --release -E 'test(sqlite_interop)'`

use wat::freeze::{eval_in_frozen, startup_beside};
use wat::runtime::Environment;

#[test]
fn sqlite_interop() {
    let world = startup_beside(file!())
        .expect("startup should succeed (:wat::sqlite' must load from the baked stdlib)");
    let ast = wat::parse_one!("(:user::sqlite_interop)").expect("parse test-fn call");
    let result = eval_in_frozen(&ast, &world, &Environment::new());
    assert!(
        result.is_ok(),
        "sqlite_interop deftest' must pass (real sqlite open/execute-ddl/execute/select \
         round-trip + Constraint/Fatal fault classification); got Err: {result:?}"
    );
}
