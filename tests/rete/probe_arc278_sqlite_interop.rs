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
    // Arc 278 the vacuous-gate wall — this gate USED to read `call_beside(..).is_ok()`,
    // which answered "did the fixture evaluate?" while claiming to answer "did it pass?".
    // Proven vacuous on 2026-07-25 by mutating the fixture's `(assert-eq n 1)` to `n 4242`
    // and watching this test stay green. `call_beside` now returns a DeftestOutcome with no
    // `is_ok()`; a fired assertion surfaces as its structured located Failure.
    call_beside(file!(), ":user::sqlite_interop").expect_passed(
        "sqlite_interop deftest must pass (real sqlite open/execute-ddl/execute/select \
         round-trip + Constraint/Fatal fault classification)",
    );
}
