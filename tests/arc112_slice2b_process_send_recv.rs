//! Arc 112 slice 2b — typed-channel send + recv schemes wire
//! through the type-checker at the PROCESS boundary. The verbs:
//!
//!   :wat::kernel::send
//!     :wat::kernel::Sender<O> :O -> :Result<:wat::core::nil, :wat::kernel::SendError>
//!   :wat::kernel::recv
//!     :wat::kernel::Receiver<I>    -> :Result<:Option<I>, :wat::kernel::RecvError>
//!
//! History:
//! - Arc 112 original: `fork-program-ast` + 3-arg main shape.
//! - Arc 170 slice 1f-ζ: migrated to `spawn-process` with 2-arg worker fn
//!   `[rx <- Receiver<I> tx <- Sender<O>]` — typed channels passed
//!   into the child as fn params.
//! - Arc 170 Stone C: retired the 2-arg shape. Child fn contract is now
//!   `[] -> :wat::core::nil`; the child reads via `readln` and writes via
//!   `println`. The typed-channel claim (send/recv at the PROCESS boundary)
//!   is preserved in Stone C's shape: the PARENT wraps `Process/stdin` with
//!   `:wat::kernel::Sender/from-pipe` and `Process/stdout` with
//!   `:wat::kernel::Receiver/from-pipe`, then `send`/`recv` operate on
//!   those wrapper values over OS pipes (EDN-encoded).
//!
//! Probe asserts that a wat program using the Stone-C-shape freezes:
//!   - `:wat::kernel::Sender/from-pipe` wraps `Process/stdin` (IOWriter)
//!   - `:wat::kernel::Receiver/from-pipe` wraps `Process/stdout` (IOReader)
//!   - `:wat::kernel::send` is called on the Sender wrapper (parent-side)
//!   - `:wat::kernel::recv` is called on the Receiver wrapper (parent-side)
//!   - The child uses `readln` + `println`; type-checks under Stone C contract.
//!
//! This is freeze-only (type-check + register); does NOT run the program.
//! Stone C's NEW probe `tests/probe_spawn_process_stdio.rs` exercises the
//! same path at runtime; this probe verifies the TYPE-CHECKER path.

use std::sync::Arc;
use wat::freeze::startup_from_source;
use wat::load::InMemoryLoader;

#[test]
fn arc112_slice2b_schemes_wire_through_typechecker() {
    let src = r##"
        ;; Child: Stone C contract — 0-arity, readln + println.
        (:wat::core::defn :my::echo-worker
          [] -> :wat::core::nil
          (:wat::core::let
            [n (:wat::kernel::readln -> :wat::core::i64)
             _ (:wat::kernel::println (:wat::core::i64::+'2 n 1))]
            :wat::core::nil))

        ;; Parent: spawn-process + wrap pipes + send/recv via Stone C wrappers.
        ;; The CLAIM under verification: send/recv verbs type-check correctly
        ;; over Sender/from-pipe and Receiver/from-pipe wrappers at the
        ;; process boundary.
        ;;
        ;; Arc 170 slice 6: spawn-process accepts a wat PROGRAM
        ;; (`Vec<WatAST>`) — the program here is a one-form program: the
        ;; child's `:user::main` define whose body invokes the worker fn.
        (:wat::core::define (:user::main -> :wat::core::nil)
          (:wat::core::let
            [proc (:wat::kernel::spawn-process
                    (:wat::core::forms
                      (:wat::core::define (:user::main -> :wat::core::nil)
                        (:my::echo-worker))))
             tx   (:wat::kernel::Sender/from-pipe   (:wat::kernel::Process/stdin  proc))
             rx   (:wat::kernel::Receiver/from-pipe (:wat::kernel::Process/stdout proc))
             ;; send: use Result/expect (non-silent per arc 110).
             _sent (:wat::core::Result/expect -> :wat::core::nil
                     (:wat::kernel::send tx 41)
                     "send failed")
             ;; recv returns Result<Option<I>, RecvError>; match all three
             ;; states per arc 110 grammar rule.
             recv-result (:wat::kernel::recv rx)
             _val (:wat::core::match recv-result -> :wat::core::i64
                    ((:wat::core::Ok (:wat::core::Some v)) v)
                    ((:wat::core::Ok :wat::core::None)    0)
                    ((:wat::core::Err _)                  0))]
            :wat::core::nil))
    "##;
    let result = startup_from_source(src, None, Arc::new(InMemoryLoader::new()));
    if let Err(e) = result {
        panic!("arc112 slice 2b probe failed to freeze: {e}");
    }
}
