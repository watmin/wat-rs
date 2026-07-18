//! Arc 170 Stone C2 — `:wat::kernel::ProcessPeer<I, O>` IPC round-trip,
//! substrate-composition proof.
//!
//! **What this file IS:** evidence that a `ProcessPeer<I, O>` can be
//! built out of the substrate primitives that already ship — `spawn-process`,
//! `Process/stdin` / `Process/stdout`, `Sender/from-pipe` / `Receiver/from-pipe`,
//! and the auto-generated `ProcessPeer/new` constructor — and that the
//! resulting peer routes typed values through real OS pipes to a real
//! subprocess via `Process/println` + `Process/readln`.
//!
//! **What this file is NOT:** the user-facing IPC pattern. User code does
//! NOT compose `spawn-process` + `Process/stdin` + `Sender/from-pipe` +
//! `ProcessPeer/new` + `Process/drain-and-join` by hand. The user-facing
//! surface is Stone D's `(:wat::kernel::run-processes ...)` bracket
//! macro, which expands to exactly this composition (plus tuple
//! aggregation across N peers). Stone C2's test exists to prove the
//! substrate composes correctly — Stone D consumes that proof.
//!
//! `Process/drain-and-join` is public per Stone B's design (it is the
//! canonical wait verb after `*_join-result` was hidden), but public
//! availability does NOT promote it to the user-facing IPC surface; the
//! bracket macro is still the path user code travels.
//!
//! **T1 — type mint.** `ProcessPeer<i64, String>` and the mirror
//!     `ProcessPeer<String, i64>` both type-check as function parameter
//!     types. Verifies the parametric type registration is well-formed.
//!
//! **T2 — real-spawn round-trip.** Spawns a subprocess (the *server*)
//!     whose `:user::main` does one `readln -> String` + one `println`.
//!     The test process (the *client*) builds a `ProcessPeer<String,
//!     String>` by composing `Receiver/from-pipe (Process/stdout server)`
//!     and `Sender/from-pipe (Process/stdin server)` through
//!     `ProcessPeer/new`, then exercises `Process/println peer "hello"`
//!     + `Process/readln peer` + `Process/drain-and-join server` from
//!     embedded wat source. Reply must equal `"hello"`. Substrate
//!     primitives compose with zero new verbs / types / structs.
//!
//! **T3 — asymmetry assertion.** TypeEnv contains `:wat::kernel::ProcessPeer`
//!     (client-side) but NOT `:wat::kernel::ProcessPeer/Server`. Server
//!     uses ambient stdio per design; the asymmetry is honest at the
//!     substrate-primitive level.
//!
//! Variable naming (T2): **client** = the test process running
//! spawn-process; **server** = the spawned subprocess servicing the
//! echo request. Not child/parent (OS-tree) — the role framing is the
//! conversation, not the process lineage.

use wat::freeze::{call_beside, startup_bare, startup_beside};
use wat::runtime::Value;

// ─── T1. type mint — both ProcessPeer<i64,String> and ProcessPeer<String,i64>
//      type-check as function parameter types ──────────────────────────

#[test]
fn process_peer_type_mints_in_both_parametric_orientations() {
    // Two helper fns, one per orientation. Each takes a ProcessPeer
    // parameter and returns nil. We never CALL them — the mint test is
    // purely that the parametric type resolves at freeze time. Mirror
    // of the Stone C1 ThreadPeer mint test (asymmetry vs symmetry
    // matters at the runtime surface, not at type-registration time).
    let world = startup_beside(file!()).expect("startup");
    assert!(
        world
            .symbols()
            .get(":my::client-reads-i64-writes-string")
            .is_some(),
        "ProcessPeer<i64,String> fn must be present after freeze"
    );
    assert!(
        world
            .symbols()
            .get(":my::client-reads-string-writes-i64")
            .is_some(),
        "ProcessPeer<String,i64> fn must be present after freeze"
    );
}

// ─── T2. real-spawn round-trip — substrate-composition proof ──────────

#[test]
fn process_peer_round_trips_string_via_real_subprocess() {
    // just-eval (rubric): the server spawn + ProcessPeer composition + println/readln
    // round-trip all live in the co-located fixture's `:my::round-trip-hello` (same
    // forms this Rust driver used to build dynamically — server side: a single
    // `:user::main` that reads one line via ambient readln and echoes it back via
    // println; client side: Receiver/from-pipe + Sender/from-pipe + ProcessPeer,
    // verbose by design per `feedback_verbose_is_honest`, surfacing what the
    // run-processes bracket macro hides).
    //
    // Hermetic time-bound: if eval ever blocks indefinitely on a wat-level deadlock,
    // the test harness's per-test timeout will kill us. On the clean-shutdown
    // failure path, Process/readln surfaces Err(chain) via the match-on-Err arm,
    // which calls assertion-failed! → RuntimeError.
    let reply = call_beside(file!(), ":my::round-trip-hello")
        .unwrap_or_else(|e| panic!("ProcessPeer round-trip failed: {}", e));
    match reply {
        Value::String(s) => assert_eq!(
            s.as_str(),
            "hello",
            "server should echo client's 'hello'; got {:?}",
            s
        ),
        other => panic!(
            "expected Value::String(\"hello\") from Process/readln (via match Ok arm); got {:?}",
            other
        ),
    }
}

// ─── T3. asymmetry assertion — TypeEnv has ProcessPeer (client) but
//      no ProcessPeer/Server (server uses ambient stdio) ──────────────

#[test]
fn process_peer_is_client_side_only_no_server_variant_emitted() {
    // Empty world; consult the global TypeEnv via FrozenWorld::types().
    // ProcessPeer must be registered; ProcessPeer/Server must NOT be —
    // the asymmetry is the design (the OS process has exactly one
    // stdin/stdout, so the server side has no peer struct; it uses
    // ambient `(:wat::kernel::readln)` / `(:wat::kernel::println)`).
    // The symmetric ThreadPeer is checked for contrast.
    let world = startup_bare().expect("startup");
    assert!(
        world.types().contains(":wat::kernel::ProcessPeer"), // rune:lint(loose-assert) — STOP-1: TypeEnv::contains() is an exact HashMap key lookup, not a string substring check
        ":wat::kernel::ProcessPeer (client-side) must be registered"
    );
    assert!(
        !world.types().contains(":wat::kernel::ProcessPeer/Server"), // rune:lint(loose-assert) — STOP-1: TypeEnv::contains() is an exact HashMap key lookup (targeted absence), not a string substring check
        ":wat::kernel::ProcessPeer/Server MUST NOT exist — server uses ambient stdio"
    );
    assert!(
        world.types().contains(":wat::kernel::ThreadPeer"), // rune:lint(loose-assert) — STOP-1: TypeEnv::contains() is an exact HashMap key lookup, not a string substring check
        ":wat::kernel::ThreadPeer (the symmetric Thread-side type) is present"
    );
}
