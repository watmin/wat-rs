//! Arc 112 slice 2b — typed-channel send + recv schemes wire
//! through the type-checker. The verbs:
//!
//!   :wat::kernel::send
//!     :wat::kernel::Sender<O> :O -> :Result<:wat::core::nil, :wat::kernel::SendError>
//!   :wat::kernel::recv
//!     :wat::kernel::Receiver<I>    -> :Result<:Option<I>, :wat::kernel::RecvError>
//!
//! Migrated from arc 112 original (arc 170 slice 1f-ζ): retired
//! `fork-program-ast` + 3-arg main replaced with `spawn-process` +
//! canonical `[] -> :wat::core::nil` entry point. The worker fn uses
//! `recv` + `send` (typed-channel surface); the probe asserts that a
//! wat program using these within the worker-fn shape freezes.
//!
//! Probe asserts that a wat program freezes successfully:
//!   - `:wat::kernel::recv` used in a let binding inside the worker fn.
//!   - `:wat::kernel::send` used as a Result/expect pattern.
//!   - The `spawn-process` form produces a `Process<I,O>` value at
//!     the launcher's declared return type.

use std::sync::Arc;
use wat::freeze::startup_from_source;
use wat::load::InMemoryLoader;

#[test]
fn arc112_slice2b_schemes_wire_through_typechecker() {
    let src = r##"
        (:wat::core::defn :my::echo-worker
          [rx <- :wat::kernel::Receiver<wat::core::i64>
           tx <- :wat::kernel::Sender<wat::core::i64>]
          -> :wat::core::nil
          (:wat::core::let
            [recv-result
              (:wat::kernel::recv rx)
             ;; process-recv returns Result<Option<I>, RecvError>;
             ;; match all three states per arc 110 grammar rule.
             val
              (:wat::core::match recv-result -> :wat::core::i64
                ((:wat::core::Ok (:wat::core::Some v)) v)
                ((:wat::core::Ok :wat::core::None)    0)
                ((:wat::core::Err _)                  0))
             ;; process-send: use Result/expect (non-silent).
             _sent
              (:wat::core::Result/expect -> :wat::core::nil
                (:wat::kernel::send tx val)
                "send failed")]
            :wat::core::nil))

        (:wat::core::define
          (:my::launch -> :wat::kernel::Process<wat::core::i64,wat::core::i64>)
          (:wat::kernel::spawn-process :my::echo-worker))

        (:wat::core::define (:user::main -> :wat::core::nil)
          :wat::core::nil)
    "##;
    let result = startup_from_source(src, None, Arc::new(InMemoryLoader::new()));
    if let Err(e) = result {
        panic!("arc112 slice 2b probe failed to freeze: {e}");
    }
}
