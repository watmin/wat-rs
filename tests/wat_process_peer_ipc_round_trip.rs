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

use std::sync::Arc;

use wat::ast::WatAST;
use wat::freeze::startup_from_source;
use wat::load::InMemoryLoader;
use wat::runtime::{eval, Environment, Value};
use wat::span::Span;

// ─── helpers ───────────────────────────────────────────────────────────

fn freeze_ok(src: &str) -> wat::freeze::FrozenWorld {
    match startup_from_source(src, None, Arc::new(InMemoryLoader::new())) {
        Ok(w) => w,
        Err(e) => panic!("freeze should succeed; got: {}", e),
    }
}

/// Arc 170 slice 6 helper — wrap a child-program source string as a
/// `(:wat::kernel::spawn-process (:wat::core::forms <forms>...))` call
/// AST. Mirrors `tests/wat_arc170_program_contracts.rs::build_spawn_process_call`
/// and `tests/probe_spawn_process_stdio.rs`. Kept local to this file so
/// the test reads top-to-bottom without external indirection.
fn build_spawn_process_call(child_program_src: &str) -> WatAST {
    let child_forms = wat::parser::parse_all_with_file(child_program_src, "<spawn-process-program>")
        .expect("child program parse");
    let mut forms_items = vec![WatAST::Keyword(":wat::core::forms".into(), Span::unknown())];
    forms_items.extend(child_forms);
    let forms_call = WatAST::List(forms_items, Span::unknown());
    WatAST::List(
        vec![
            WatAST::Keyword(":wat::kernel::spawn-process".into(), Span::unknown()),
            forms_call,
        ],
        Span::unknown(),
    )
}

/// Drain the server's stderr to a String — diagnostic helper used when
/// the round-trip fails (subprocess panic or unexpected EOF). Mirrors
/// `tests/wat_arc170_program_contracts.rs:308-323`.
fn drain_server_stderr(server: &Value) -> String {
    match server {
        Value::Struct(s) if s.type_name == ":wat::kernel::Process" => match &s.fields[2] {
            Value::io__IOReader(rdr) => {
                let mut all = String::new();
                while let Ok(Some(line)) = rdr.read_line(Span::unknown()) {
                    all.push_str(&line);
                }
                all
            }
            _ => "<server.stderr not IOReader>".to_string(),
        },
        _ => "<server not Process Struct>".to_string(),
    }
}

// ─── T1. type mint — both ProcessPeer<i64,String> and ProcessPeer<String,i64>
//      type-check as function parameter types ──────────────────────────

#[test]
fn process_peer_type_mints_in_both_parametric_orientations() {
    // Two helper fns, one per orientation. Each takes a ProcessPeer
    // parameter and returns nil. We never CALL them — the mint test is
    // purely that the parametric type resolves at freeze time. Mirror
    // of the Stone C1 ThreadPeer mint test (asymmetry vs symmetry
    // matters at the runtime surface, not at type-registration time).
    let src = r#"
        (:wat::core::defn :my::client-reads-i64-writes-string
          [_peer <- :wat::kernel::ProcessPeer<wat::core::i64,wat::core::String>]
          -> :wat::core::nil
          :wat::core::nil)

        (:wat::core::defn :my::client-reads-string-writes-i64
          [_peer <- :wat::kernel::ProcessPeer<wat::core::String,wat::core::i64>]
          -> :wat::core::nil
          :wat::core::nil)
    "#;
    let world = freeze_ok(src);
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
    // Empty parent world (no helper defines at freeze time). The
    // subprocess program is self-contained per the arc 170 slice 6
    // substrate contract.
    let world = freeze_ok("");

    // Server side: a single `:user::main` that reads one line via
    // ambient `(:wat::kernel::readln -> :wat::core::String)` and echoes
    // it back via `(:wat::kernel::println line)`. The substrate wires
    // fd 0/1/2 to the OS pipes for the spawned subprocess; the
    // ambient verbs route through those.
    let server_program_src = r#"
        (:wat::core::define (:user::main -> :wat::core::nil)
          (:wat::core::let
            [line (:wat::kernel::readln -> :wat::core::String)
             _    (:wat::kernel::println line)]
            :wat::core::nil))
    "#;
    let spawn_call = build_spawn_process_call(server_program_src);
    let server = eval(&spawn_call, &Environment::new(), world.symbols())
        .expect("spawn-process should hand back a Process Struct");

    // Bind the server Process Value into the eval environment. The
    // rest of the round-trip lives in one embedded wat source — the
    // composition pattern this test exists to prove.
    //
    // Construction is verbose by design (per
    // `feedback_verbose_is_honest`): the three-step build (Receiver
    // over stdout, Sender over stdin, ProcessPeer/new over both)
    // surfaces what the bracket macro will hide. The substrate has
    // ZERO constructor verbs minted to compress this — that is the
    // point.
    let env = Environment::new()
        .child()
        .bind("server", server.clone())
        .build();
    // Arc 208 slice 1 — Process/println + Process/readln now return Result.
    // Wrapped with Result/expect to preserve panic-on-transport-error semantics.
    // reply is unwrapped :String (not Result<:String, ...>) so the Rust-side
    // match below stays unchanged. Walker requires Process/readln to appear in
    // Result/expect value-position; same for Process/println.
    let round_trip = wat::parse_one!(
        r#"
        (:wat::core::let
          [rx       (:wat::kernel::Receiver/from-pipe (:wat::kernel::Process/stdout server))
           tx       (:wat::kernel::Sender/from-pipe   (:wat::kernel::Process/stdin  server))
           peer     (:wat::kernel::ProcessPeer/new rx tx)
           _written (:wat::core::Result/expect -> :wat::core::nil
                       (:wat::kernel::Process/println peer "hello")
                       "Process/println failed: subprocess died")
           reply    (:wat::core::Result/expect -> :wat::core::String
                       (:wat::kernel::Process/readln peer)
                       "Process/readln failed: subprocess died")
           _drained (:wat::kernel::Process/drain-and-join server)]
          reply)
        "#
    )
    .expect("round-trip let form parses");

    // Hermetic time-bound: if eval ever blocks indefinitely on a wat-
    // level deadlock, the test harness's per-test timeout will kill us.
    // On the clean-shutdown failure path, Process/readln now surfaces
    // Err(chain) rather than RuntimeError::ChannelDisconnected — the
    // Result/expect above converts Err to a panic. The server stderr is
    // surfaced for diagnostic via the Err(e) arm below.
    let reply = match eval(&round_trip, &env, world.symbols()) {
        Ok(v) => v,
        Err(e) => {
            let stderr_text = drain_server_stderr(&server);
            panic!(
                "ProcessPeer round-trip failed: {}\nserver stderr:\n{}",
                e, stderr_text
            );
        }
    };
    match reply {
        Value::String(s) => assert_eq!(
            s.as_str(),
            "hello",
            "server should echo client's 'hello'; got {:?}",
            s
        ),
        other => panic!(
            "expected Value::String(\"hello\") from Process/readln (unwrapped via Result/expect); got {:?}",
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
    let world = freeze_ok("");
    assert!(
        world.types().contains(":wat::kernel::ProcessPeer"),
        ":wat::kernel::ProcessPeer (client-side) must be registered"
    );
    assert!(
        !world.types().contains(":wat::kernel::ProcessPeer/Server"),
        ":wat::kernel::ProcessPeer/Server MUST NOT exist — server uses ambient stdio"
    );
    assert!(
        world.types().contains(":wat::kernel::ThreadPeer"),
        ":wat::kernel::ThreadPeer (the symmetric Thread-side type) is present"
    );
}
