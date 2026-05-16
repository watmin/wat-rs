//! Arc 112 slice 1 — phantom type params on Process<I,O> survive
//! instantiation and unify against a user-annotated binding.
//!
//! Original probe used spawn-program (retired arc 170 slice 2) with the
//! 4-arg `:user::main` (also retired arc 170 slice 1e). Migrated (arc
//! 170 slice 1f-ζ) to use `spawn-process` with a canonical
//! `(:user::main -> :wat::core::nil)` entry point.
//!
//! Arc 170 slice 6 — spawn-process now accepts a wat PROGRAM
//! (`Vec<WatAST>`) instead of a fn. The phantom type params I/O on
//! `Process<I,O>` continue to unify from the caller's annotated return
//! type alone; the substrate registers `type_params: vec!["I", "O"]`
//! on `:wat::kernel::spawn-process` with the program shape's
//! `Vec<WatAST>` parameter and a `Process<I,O>` return.

use std::sync::Arc;
use wat::freeze::startup_from_source;
use wat::load::InMemoryLoader;

#[test]
fn arc112_probe_spawn_program_parametric_return() {
    // Arc 170 slice 6: substrate accepts a wat program. The launch site
    // constructs the program inline via `:wat::core::forms` + an inner
    // `:user::main` define whose body invokes the worker. The
    // Process<i64,i64> type params unify against the launcher's declared
    // return type annotation alone (program shape carries no fn
    // signature).
    let src = r##"
        (:wat::core::defn :my::worker
          []
          -> :wat::core::nil
          :wat::core::nil)

        (:wat::core::define
          (:my::launch -> :wat::kernel::Process<wat::core::i64,wat::core::i64>)
          (:wat::kernel::spawn-process
            (:wat::core::forms
              (:wat::core::define (:user::main -> :wat::core::nil)
                (:my::worker)))))

        (:wat::core::define (:user::main -> :wat::core::nil)
          :wat::core::nil)
    "##;
    let result = startup_from_source(src, None, Arc::new(InMemoryLoader::new()));
    if let Err(e) = result {
        panic!("arc112 probe failed to freeze: {e}");
    }
}
