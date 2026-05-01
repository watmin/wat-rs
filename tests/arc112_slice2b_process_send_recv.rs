//! Arc 112 slice 2b — process-send + process-recv schemes wire
//! through the type-checker. Both verbs:
//!
//!   :wat::kernel::process-send
//!     :Process<I,O> :I -> :Result<:(), :wat::kernel::ProcessDiedError>
//!   :wat::kernel::process-recv
//!     :Process<I,O>    -> :Result<:Option<O>, :wat::kernel::ProcessDiedError>
//!
//! Probe asserts that a wat program freezes successfully:
//!   - process-send used in a let* binding (allowed — Result<()> doesn't
//!     gate on disconnection).
//!   - process-recv used as a match-scrutinee (required by arc 110 +
//!     arc 112 slice 3 grammar rule — silent disconnect is a compile
//!     error; receiver must match all three states).

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
            -> :wat::core::unit)
          (:wat::core::let*
            (((proc :wat::kernel::Program<wat::core::i64,wat::core::i64>)
              (:wat::kernel::fork-program-ast
                (:wat::core::Vector :wat::WatAST)))
             ;; process-send: must be matched (slice 3 grammar rule).
             ;; Use Result/expect to panic on disconnect; same shape
             ;; sandbox.wat / hermetic.wat use for write paths.
             ((_sent :wat::core::unit)
              (:wat::core::Result/expect -> :wat::core::unit
                (:wat::kernel::process-send proc 42)
                "send to forked program failed")))
            ;; process-recv: matched as scrutinee — three-state shape.
            (:wat::core::match (:wat::kernel::process-recv proc) -> :wat::core::unit
              ((:wat::core::Ok (:wat::core::Some _v))    ())
              ((:wat::core::Ok :wat::core::None)        ())
              ((:wat::core::Err _died)       ()))))
    "##;
    let result = startup_from_source(src, None, Arc::new(InMemoryLoader::new()));
    if let Err(e) = result {
        panic!("arc112 slice 2b probe failed to freeze: {e}");
    }
}
