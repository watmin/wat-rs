//! Arc 112 slice 1 — phantom type params on Process<I,O> survive
//! instantiation and unify against a user-annotated binding.
//!
//! Original probe used spawn-program (retired arc 170 slice 2) with the
//! 4-arg `:user::main` (also retired arc 170 slice 1e). Migrated (arc
//! 170 slice 1f-ζ) to use `spawn-process` with a canonical
//! `(:user::main -> :wat::core::nil)` entry point.
//!
//! Stone C migration: spawn-process child fn contract is now `[] -> nil`.
//! The phantom type params I/O on `Process<I,O>` are now inferred from
//! the caller's annotated return type (not from the fn's Receiver/Sender
//! params). The probe annotates the return of `(:wat::kernel::spawn-process ...)`
//! as `:wat::kernel::Process<wat::core::i64,wat::core::i64>`. If the
//! substrate's `instantiate` / `unify` chain handles `Process<I,O>`
//! correctly with context-driven inference, the source freezes — that
//! is the phantom-type-param probe under Stone C.

use std::sync::Arc;
use wat::freeze::startup_from_source;
use wat::load::InMemoryLoader;

#[test]
fn arc112_probe_spawn_program_parametric_return() {
    // Stone C: worker fn is [] -> nil. The Process<i64,i64> type params
    // unify against the launcher's declared return type annotation alone.
    let src = r##"
        (:wat::core::defn :my::worker
          []
          -> :wat::core::nil
          :wat::core::nil)

        (:wat::core::define
          (:my::launch -> :wat::kernel::Process<wat::core::i64,wat::core::i64>)
          (:wat::kernel::spawn-process :my::worker))

        (:wat::core::define (:user::main -> :wat::core::nil)
          :wat::core::nil)
    "##;
    let result = startup_from_source(src, None, Arc::new(InMemoryLoader::new()));
    if let Err(e) = result {
        panic!("arc112 probe failed to freeze: {e}");
    }
}
