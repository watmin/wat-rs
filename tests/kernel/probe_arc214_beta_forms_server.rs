//! Arc 214 — `1b-ii-β` FM-2-bis probe: a `:process` peer spawns a wat PROGRAM
//! (forms) that runs as a `readln`/`println` server, driven by the parent client
//! via `send'`/`recv'`.
//!
//! # The model under test (builder, 2026-06-08)
//!
//! In a fork the spawned program is just a normal wat program: it reads fd 0 with
//! `(:wat::kernel::readln -> :T)`, writes fd 1 with `(:wat::kernel::println v)`,
//! and panics to fd 2 with `(:wat::kernel::eprintln …)`. From its perspective it
//! is the same as any other wat program — it operates as a "server." Its "client"
//! is the PARENT, who drives it with `(send' peer v)` / `(recv' peer)` directly on
//! the peer. β makes `spawn-program' :process` spawn such a program (forms), not a
//! Rust apply-loop over a fn.
//!
//! # Why this is RED at HEAD (the gap)
//!
//! At HEAD `spawn-program' :process` takes a FN (the apply-loop): the type-checker
//! `infer_spawn_program_prime` projects `Process'<I,O>` from a fn `[I]->O` and
//! REJECTS a non-fn arg, and the child runs a Rust apply-loop, not a wat program.
//! So spawning a `(:wat::core::forms (defn :user::main …))` program here fails at
//! the type-checker (args[2] is not a fn) — RED on exactly the surface+runtime gap
//! β closes (forms-as-server + `:process` infers `Process'<Value,Value>`, γ-1).
//!
//! The server body is the PROVEN arc112_slice2b worker (echo+1: reads i64, writes
//! n+1) — known-good under `spawn-process` — so the only thing untested-at-HEAD is
//! `spawn-program' :process` running forms as a server driven by `send'`/`recv'`.
//!
//! # Containment
//!
//! Forks a `:process` child — run under setsid + timeout, single-threaded:
//!   setsid timeout 180 cargo test --release --test kernel \
//!     probe_arc214_beta_forms_server -- --ignored --test-threads=1

use wat::freeze::call_beside;

/// FM-2-bis (β): a forms-program spawned `:process` runs as a `readln`/`println`
/// server, driven by `send'`/`recv'`; the client sends 41, the server echoes 41+1,
/// the client receives 42.
///
/// `#[ignore]` — process-tier probe; run under setsid + timeout, `--test-threads=1`.
#[test]
#[ignore = "process-tier FM-2-bis probe (arc 214 1b-ii-β): run via setsid timeout 180 cargo test --release --test kernel probe_arc214_beta_forms_server -- --ignored --test-threads=1"]
fn beta_forms_server_round_trip_via_send_recv_prime() {
    // Parent (client): spawn the forms-server, send' 41, recv' the echo+1.
    // Server (the spawned program): the proven arc112_slice2b worker —
    //   read one i64, write n+1 — wrapped as the program's :user::main.
    let result = call_beside(file!(), ":user::compute")
        .expect("call_beside must succeed: β forms-server round-trip (RED at HEAD = type error here)");

    match result {
        wat::runtime::Value::i64(n) => assert_eq!(
            n, 42,
            "forms-server echo+1 must return 42 for input 41; got {}",
            n
        ),
        other => panic!("expected i64(42) from the forms-server via recv'; got {:?}", other),
    }
}
