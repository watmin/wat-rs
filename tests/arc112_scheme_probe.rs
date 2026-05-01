//! Arc 112 slice 1 — phantom type params on Process<I,O> survive
//! instantiation and unify against a user-annotated binding.
//!
//! The probe ANNOTATES `sr` as `wat::core::Result<Process<i64,i64>, StartupError>`
//! and binds it to `(spawn-program "()" :None)`. If the substrate's
//! `instantiate` / `unify` chain handles `Process<I,O>` correctly, the
//! source freezes; if Process degrades to `Path(":Process")` anywhere
//! along the way, the freeze fails with `expects :Process<i64,i64>;
//! got :Process` — exactly the error that named the slice-1 dragon.

use std::sync::Arc;
use wat::freeze::startup_from_source;
use wat::load::InMemoryLoader;

#[test]
fn arc112_probe_spawn_program_parametric_return() {
    let src = r##"
        (:wat::core::define
          (:user::main
            (stdin :wat::io::IOReader)
            (stdout :wat::io::IOWriter)
            (stderr :wat::io::IOWriter)
            -> :wat::core::unit)
          (:wat::core::let*
            (((sr :wat::core::Result<wat::kernel::Process<wat::core::i64,wat::core::i64>,wat::kernel::StartupError>)
              (:wat::kernel::spawn-program "()" :None)))
            ()))
    "##;
    let result = startup_from_source(src, None, Arc::new(InMemoryLoader::new()));
    if let Err(e) = result {
        panic!("arc112 probe failed to freeze: {e}");
    }
}
