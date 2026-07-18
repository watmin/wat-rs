//! Arc 259 — `wat.peer-kind`: the program-env's first TYPED-ENUM identity field.
//!
//! peer-kind answers "what KIND of peer am I" as a nominal `:wat::program::PeerKind`
//! enum, not a raw keyword:
//!   `:thread`  — shares the parent's address space (a thread peer)
//!   `:process` — owns its own address space (the root `:user::main`, or a forked
//!                `:process` peer)
//! The root main owns its address space → **`:process`** (builder, 2026-06-11).
//! Kernel-stamped at the post-bootstrap / pre-main seam; read via the accessor.
//!
//! RED at HEAD: `Env` is a 4-arg record → the 5-arg constructor is an arity error,
//! and the seam env carries no `wat.peer-kind`.
//!
//! Wat source lives in the co-located sibling fixture `probe_arc259_env_peer_kind.wat`,
//! slurped via `startup_beside(file!())`.
//!
//! Run: `cargo test --release --test program probe_arc259_env_peer_kind`

use wat::freeze::{call_beside, invoke_user_main, startup_beside};
use wat::runtime::Value;

/// The record carries `wat.peer-kind` as a `PeerKind` value (RED via arity at HEAD:
/// a 5-arg `Env` constructor is an arity error). `conforms?` proves the field holds
/// a genuine `:wat::program::PeerKind` (the proven nominal-membership idiom).
#[test]
fn env_record_carries_peer_kind() {
    let got = call_beside(file!(), ":probe::compute").expect("eval");
    assert_eq!(
        got,
        Value::bool(true),
        "Env carries wat.peer-kind (5th field) holding a :wat::program::PeerKind"
    );
}

/// The SEAM stamps `:process` for the root `:user::main` (it owns its address
/// space). RED at HEAD because the field does not exist → the accessor fails →
/// main errors → invoke returns Err. The `assert-eq<PeerKind>` in the fixture proves
/// the stamped value is EXACTLY `:process`, not just any PeerKind member.
#[test]
fn seam_stamps_process_for_root_main() {
    let world = startup_beside(file!()).expect("startup");
    assert!(
        invoke_user_main(&world, vec![]).is_ok(),
        "the seam must stamp wat.peer-kind = :process before :user::main; assert-eq failed"
    );
}
