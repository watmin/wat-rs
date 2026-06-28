//! Arc 272 6c.2 DE-RISK — does a plain user `Record::def` round-trip over the process-fork IPC wire?
//!
//! This isolates the exact seam the 6c.2 (D1) crashed/orphaned strike tripped on: a RECORD carried
//! across `send'`/`recv'` after a `spawn-program' (process)` fork. D1 makes the address capability a
//! `SocketAddressWire` record; its whole foundation is "a user record survives the process IPC wire."
//! 234.7a proved records round-trip via `value_to_edn`/`edn_to_value` IN-PROCESS; this proves it ACROSS
//! a fork (the new axis). GREEN ⇒ D1's foundation is sound, the orphan's failure was implementation.
//! RED ⇒ records do not cross the fork wire (a deeper plumbing gap), and D1 is blocked until that's fixed.
//!
//! Forks (`spawn-program' (process)`) → its own [[test]] binary.
//! Run: cargo test --release -p wat --test probe_arc272_6c2_record_ipc_derisk

use wat::freeze::{eval_in_frozen, startup_beside};
use wat::runtime::{Environment, Value};

#[test]
fn plain_record_round_trips_over_process_ipc() {
    let world = startup_beside(file!())
        .expect("startup should succeed (de-risk: record over process IPC)");
    let ast = wat::parse_one!("(:user::compute)").expect("parse");
    let got = eval_in_frozen(&ast, &world, &Environment::new())
        .map(|tv| tv.value_owned())
        .unwrap_or_else(|e| panic!("compute raised: {e:?}"));
    assert!(
        matches!(got, Value::i64(42)),
        "expected 42: the child minted a :user::Pt{{7,35}}, sent it over the self-peer, the parent \
         recv'd it across the fork and read x+y. If this is RED, a plain record does NOT round-trip \
         over the process IPC wire — D1 is blocked on that, not on the capability codec. got {got:?}"
    );
}
