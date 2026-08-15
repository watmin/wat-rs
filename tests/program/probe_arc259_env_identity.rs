//! Arc 259 — the program-env's first IDENTITY fields (the escape hatch grows).
//!
//! `:wat::program::Env` is wat's structured **escape hatch for system
//! interrogation**: kernel-stamped, user-unforgeable `wat.*` fields a pure wat
//! program reads as ordinary data instead of reaching for raw syscalls. It carried
//! only the two timing fields; this stone adds the first IDENTITY fields — the
//! companions to timing:
//!   `process-id`   : i64 — `std::process::id()`
//!   `os-thread-id` : i64 — the OS thread (Linux `gettid`)
//! Both kernel-stamped at the post-bootstrap / pre-`:user::main` seam; read via
//! the record accessors. (`peer-kind`, the typed `PeerKind` enum, is the next
//! stone.)
//!
//! RED at HEAD: `Env` is a 2-arg record → the 4-arg constructor is an arity error,
//! and the seam-installed env carries no `process-id`.
//!
//! Wat source lives in the co-located sibling fixture `probe_arc259_env_identity.wat`,
//! slurped via `startup_beside(file!())`.
//!
//! Run: `cargo test --release --test program probe_arc259_env_identity`

use wat::freeze::{call_beside_value, invoke_user_main, startup_beside};
use wat::runtime::Value;

/// The record carries `process-id` + `os-thread-id` as readable i64 fields
/// (RED via arity at HEAD: a 4-arg `Env` constructor is an arity error).
#[test]
fn env_record_carries_process_and_thread_id() {
    let got = call_beside_value(file!(), ":probe::c01-compute").expect("eval");
    assert_eq!(
        got,
        Value::i64(12345),
        "Env carries process-id as its 3rd field (os-thread-id is the 4th)"
    );
}

/// The SEAM stamps the REAL process-id and os-thread-id: `invoke_user_main` installs
/// an env with both fields; the fixture's :user::main reads them for effect — accessor
/// errors if the seam did not stamp them. RED at HEAD because the fields do not exist.
#[test]
fn seam_installs_env_with_process_id() {
    let world = startup_beside(file!()).expect("startup");
    let result = invoke_user_main(&world, vec![]);
    assert!(
        result.is_ok(),
        "seam must stamp process-id and os-thread-id into the env; main read failed: {:?}",
        result.err()
    );
}
