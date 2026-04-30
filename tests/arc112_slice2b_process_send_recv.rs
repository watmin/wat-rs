//! Arc 112 slice 2b — process-send + process-recv schemes wire
//! through the type-checker. Both verbs:
//!
//!   :wat::kernel::process-send
//!     :Process<I,O> :I -> :Result<:(), :wat::kernel::ProcessDiedError>
//!   :wat::kernel::process-recv
//!     :Process<I,O>    -> :Result<:Option<O>, :wat::kernel::ProcessDiedError>
//!
//! Probe asserts that a wat program annotating these verbs against
//! a `Process<i64,i64>` freezes successfully (instantiate + unify
//! resolve `I` and `O` correctly; return type matches the binding).

use std::sync::Arc;
use wat::freeze::startup_from_source;
use wat::load::InMemoryLoader;

#[test]
fn arc112_slice2b_schemes_wire_through_typechecker() {
    let src = r##"
        (:wat::core::define
          (:user::main
            (stdin :wat::io::IOReader)
            (stdout :wat::io::IOWriter)
            (stderr :wat::io::IOWriter)
            -> :())
          (:wat::core::let*
            (((proc :wat::kernel::Process<i64,i64>)
              (:wat::kernel::fork-program-ast
                (:wat::core::vec :wat::WatAST)))
             ;; process-send: takes Process<i64,i64> + i64 → Result<(),ProcessDiedError>
             ((sent :Result<(),wat::kernel::ProcessDiedError>)
              (:wat::kernel::process-send proc 42))
             ;; process-recv: takes Process<i64,i64> → Result<Option<i64>,ProcessDiedError>
             ((rcv :Result<Option<i64>,wat::kernel::ProcessDiedError>)
              (:wat::kernel::process-recv proc)))
            ()))
    "##;
    let result = startup_from_source(src, None, Arc::new(InMemoryLoader::new()));
    if let Err(e) = result {
        panic!("arc112 slice 2b probe failed to freeze: {e}");
    }
}
