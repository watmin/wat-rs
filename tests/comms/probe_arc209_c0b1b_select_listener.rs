//! Arc 209 C0b.1b / C0b.2e-i-c — `poll'` is the service multiplexer + the `ServiceEvent<I,O>` sum.
//!
//! THE GATE (this probe IS the hand-rolled thread service proof): a service `poll'`s over
//! THREE inputs — the **self-peer** (owner link), the **listener**, the **clients** — and
//! `match`es the returned `ServiceEvent`:
//!   - `:Shutdown`          → owner dropped the handle (RAII drain disconnected the self-peer);
//!                            exit the loop (DEADLOCK-FREE TERMINATION — structural, no Stop op)
//!   - `:Connection [peer]` → `poll'` accepted the dialing client; conj it (GROW)
//!   - `:Message [idx msg]` → handle the op on `clients[idx]`, reply, recur (SERVE)
//!   - `:Closed [idx]`      → `remove-at` (graceful SHRINK)
//!   - `:Lost [idx cause]`  → `remove-at` (abnormal SHRINK; remote tier; `cause` is a Failure)
//! Two clients dynamically connect, each round-trips a protected scalar (n*2); then the owner
//! simply DROPS the service handle at scope-exit → `:Shutdown` → the service terminates and the
//! join completes. No cooperative stop — dropping the handle IS the shutdown. (If this hangs,
//! `poll'` isn't watching the self-peer — the deadlock this stone annihilates.)
//!
//!
//! Run SERIALLY (spawns threads):
//!   `cargo test --release -p wat --test comms probe_arc209_c0b1b_select_listener -- --test-threads=1`

use wat::freeze::{eval_in_frozen, startup_beside};
use wat::runtime::{Environment, Value};

#[test]
fn select_grows_over_listener_serves_and_shrinks() {
    let world = startup_beside(file!())
        .expect("startup should succeed");
    let ast = wat::parse_one!("(:user::compute)").expect("parse");
    let got = eval_in_frozen(&ast, &world, &Environment::new())
        .map(|tv| tv.value_owned())
        .unwrap_or_else(|e| panic!("compute raised: {e:?}"));
    assert!(
        matches!(got, Value::i64(24)),
        "expected r1+r2 = 10+14 = 24 (two dynamically-connected clients each round-tripped n*2); got {got:?}"
    );
}
