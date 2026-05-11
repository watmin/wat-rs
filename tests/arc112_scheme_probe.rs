//! Arc 112 slice 1 — phantom type params on Process<I,O> survive
//! instantiation and unify against a user-annotated binding.
//!
//! Original probe used spawn-program (retired arc 170 slice 2) with the
//! 4-arg `:user::main` (also retired arc 170 slice 1e). Migrated (arc
//! 170 slice 1f-ζ) to use `spawn-process` — the direct successor — with
//! a canonical `(:user::main -> :wat::core::nil)` entry point.
//!
//! The probe annotates the return of `(:wat::kernel::spawn-process ...)`
//! as `:wat::kernel::Process<wat::core::i64,wat::core::i64>`. If the
//! substrate's `instantiate` / `unify` chain handles `Process<I,O>`
//! correctly, the source freezes; if Process degrades to
//! `Path(":Process")` anywhere along the way, the freeze fails with
//! a type mismatch — exactly the error that named the slice-1 dragon.

use std::sync::Arc;
use wat::freeze::startup_from_source;
use wat::load::InMemoryLoader;

#[test]
fn arc112_probe_spawn_program_parametric_return() {
    // The probe defines a `:user::process`-contract worker fn and a
    // launcher function whose declared return type is
    // `Process<i64,i64>`. spawn-process must instantiate the Process
    // type with the concrete I/O types from the worker fn's Receiver/Sender
    // params and unify against the annotated return — that is the
    // phantom-type-param probe.
    let src = r##"
        (:wat::core::defn :my::worker
          [rx <- :wat::kernel::Receiver<wat::core::i64>
           tx <- :wat::kernel::Sender<wat::core::i64>]
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
