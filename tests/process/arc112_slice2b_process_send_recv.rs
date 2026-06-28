//! Arc 112 slice 2b — typed-channel send + recv schemes wire
//! through the type-checker at the PROCESS boundary. The verbs:
//!
//!   :wat::kernel::send
//!     :wat::kernel::Sender<O> :O -> :Result<:wat::core::nil, :wat::kernel::SendError>
//!   :wat::kernel::recv
//!     :wat::kernel::Receiver<I>    -> :Result<:Option<I>, :wat::kernel::RecvError>
//!
//! History:
//! - Arc 112 original: `spawn-process` (forms-based OS-fork path).
//! - Arc 170 slice 1f-ζ: `spawn-process` with 2-arg worker fn
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

use wat::freeze::startup_beside;

#[test]
fn arc112_slice2b_schemes_wire_through_typechecker() {
    // World loaded from co-located arc112_slice2b_process_send_recv.wat via startup_beside.
    // Verifies Stone C typed-channel scheme wires through the type-checker at the process boundary.
    let result = startup_beside(file!());
    if let Err(e) = result {
        panic!("arc112 slice 2b probe failed to freeze: {e}");
    }
}
