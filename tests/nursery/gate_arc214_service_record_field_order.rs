//! Arc 214 Stone 8.2w — service record field-order gate (circumspicere F2 + F3).
//!
//! ## H1 — THE FIELD-ORDER GATE (the audit's best find)
//!
//! The peer's positional extraction (peer.rs field[0]; freeze.rs stdin reply_of
//! field[1]) and the wat defstruct order are ONE contract; a reorder must be a
//! red build, never a silent mis-route.
//!
//! Asserts:
//!   - field[0].name == "thread-id" for all three *Service::Req structs
//!   - field[1].name == "line" for StdOutService::Req, StdErrService::Req
//!   - field[0].name == "thread-id" AND field[1].name == "line" for StdInService::Rep
//!
//! ## H2 — guard arms (circumspicere F3)
//!
//! Spawn a peer with a real handle; send a malformed Req (not a Struct, then a
//! Struct with a String in field[0]); confirm the loop survives the `continue`
//! and a subsequent valid Req still round-trips.
//!
//! Run: `cargo test --release --test nursery gate_arc214_service_record_field_order`

use std::sync::Arc;

use wat::freeze::startup_from_source;
use wat::io::{PipeReader, PipeWriter};
use wat::load::InMemoryLoader;
use wat::runtime::Value;
use wat::services::{next_thread_id, spawn_service_peer, ServiceMsg, ThreadIO, install_thread_io};
use wat::types::TypeDef;

fn freeze_skeleton() -> wat::freeze::FrozenWorld {
    let src = r#"
        (:wat::core::defn :user::main [] -> :wat::core::nil nil)
    "#;
    startup_from_source(src, None, Arc::new(InMemoryLoader::new()))
        .expect("skeleton freeze succeeds")
}

// ─── H1: field-order gate ─────────────────────────────────────────────────────

/// StdInService::Req field[0] must be "thread-id" (no line field).
#[test]
fn h1_stdin_req_field_order() {
    let world = freeze_skeleton();
    let types = world.types();

    let def = types
        .get(":wat::kernel::services::StdInService::Req")
        .expect("StdInService::Req is registered in the type registry");
    let fields = match def {
        TypeDef::Struct(s) => &s.fields,
        other => panic!("StdInService::Req must be a Struct TypeDef; got {:?}", other),
    };
    assert_eq!(
        fields.len(), 1,
        "StdInService::Req must have exactly 1 field (thread-id); got {:?}",
        fields.iter().map(|(n, _)| n.as_str()).collect::<Vec<_>>()
    );
    assert_eq!(
        fields[0].0, "thread-id",
        "StdInService::Req field[0] must be 'thread-id' — \
         the peer's positional extraction (peer.rs field[0]) and this defstruct are ONE contract; \
         a reorder must be a red build, never a silent mis-route"
    );
}

/// StdInService::Rep field[0] must be "thread-id" and field[1] must be "line"
/// (freeze.rs stdin reply_of reads field[1]).
#[test]
fn h1_stdin_rep_field_order() {
    let world = freeze_skeleton();
    let types = world.types();

    let def = types
        .get(":wat::kernel::services::StdInService::Rep")
        .expect("StdInService::Rep is registered in the type registry");
    let fields = match def {
        TypeDef::Struct(s) => &s.fields,
        other => panic!("StdInService::Rep must be a Struct TypeDef; got {:?}", other),
    };
    assert!(
        fields.len() >= 2,
        "StdInService::Rep must have at least 2 fields; got {:?}",
        fields.iter().map(|(n, _)| n.as_str()).collect::<Vec<_>>()
    );
    assert_eq!(
        fields[0].0, "thread-id",
        "StdInService::Rep field[0] must be 'thread-id'"
    );
    assert_eq!(
        fields[1].0, "line",
        "StdInService::Rep field[1] must be 'line' — \
         freeze.rs stdin reply_of reads sv.fields[1] as the String; \
         a reorder is a silent mis-route"
    );
}

/// StdOutService::Req field[0]=="thread-id", field[1]=="line".
#[test]
fn h1_stdout_req_field_order() {
    let world = freeze_skeleton();
    let types = world.types();

    let def = types
        .get(":wat::kernel::services::StdOutService::Req")
        .expect("StdOutService::Req is registered in the type registry");
    let fields = match def {
        TypeDef::Struct(s) => &s.fields,
        other => panic!("StdOutService::Req must be a Struct TypeDef; got {:?}", other),
    };
    assert!(
        fields.len() >= 2,
        "StdOutService::Req must have at least 2 fields; got {:?}",
        fields.iter().map(|(n, _)| n.as_str()).collect::<Vec<_>>()
    );
    assert_eq!(
        fields[0].0, "thread-id",
        "StdOutService::Req field[0] must be 'thread-id' — \
         peer.rs extracts thread-id from field[0]; a reorder is a silent mis-route"
    );
    assert_eq!(
        fields[1].0, "line",
        "StdOutService::Req field[1] must be 'line'"
    );
}

/// StdErrService::Req field[0]=="thread-id", field[1]=="line".
#[test]
fn h1_stderr_req_field_order() {
    let world = freeze_skeleton();
    let types = world.types();

    let def = types
        .get(":wat::kernel::services::StdErrService::Req")
        .expect("StdErrService::Req is registered in the type registry");
    let fields = match def {
        TypeDef::Struct(s) => &s.fields,
        other => panic!("StdErrService::Req must be a Struct TypeDef; got {:?}", other),
    };
    assert!(
        fields.len() >= 2,
        "StdErrService::Req must have at least 2 fields; got {:?}",
        fields.iter().map(|(n, _)| n.as_str()).collect::<Vec<_>>()
    );
    assert_eq!(
        fields[0].0, "thread-id",
        "StdErrService::Req field[0] must be 'thread-id' — \
         peer.rs extracts thread-id from field[0]; a reorder is a silent mis-route"
    );
    assert_eq!(
        fields[1].0, "line",
        "StdErrService::Req field[1] must be 'line'"
    );
}

// ─── H2: guard arms — loop survives malformed Req ─────────────────────────────

/// Spawn a real stdout peer; send a non-Struct Req (i64 99) — the loop logs and
/// continues; then send Register + a valid Req and assert the ack arrives
/// (loop survived the malformed input; no hang).
#[test]
fn h2_guard_arm_non_struct_req_loop_survives() {
    let world = freeze_skeleton();

    // ── stdout pipe + peer ──────────────────────────────────────────────
    let (stdout_pipe_r, stdout_pipe_w) =
        wat::fork::make_pipe(":test::h2-stdout").expect("pipe for stdout");
    let stdout_writer = Value::io__IOWriter(Arc::new(PipeWriter::from_owned_fd(stdout_pipe_w)));
    let _stdout_reader = PipeReader::from_owned_fd(stdout_pipe_r);

    let stdout_handle = world
        .symbols()
        .get(":wat::kernel::services::StdOutService/handle")
        .expect("/handle in baked stdlib")
        .clone();
    let peer = spawn_service_peer("stdout", stdout_handle, stdout_writer, world.symbols().clone(), |_| Ok(()));
    let input_tx = peer.input_tx.clone();

    // ── Register this test thread ────────────────────────────────────────
    let tid = next_thread_id();
    let (reply_tx, reply_rx) = wat::comms::thread::pair::<Result<(), String>>();
    input_tx
        .send(ServiceMsg::Register(tid, reply_tx))
        .expect("Register sent");

    install_thread_io(ThreadIO {
        stdout_reply_rx: reply_rx,
        stderr_reply_rx: {
            let (_, rx) = wat::comms::thread::pair::<Result<(), String>>();
            rx
        },
        thread_id: tid,
        stdin_reply_rx: {
            let (_, rx) = wat::comms::thread::pair::<Result<String, String>>();
            rx
        },
    });

    // ── Malformed Req #1: not a Struct — the loop should log + continue ──
    input_tx
        .send(ServiceMsg::Req(Value::i64(99)))
        .expect("malformed Req sent");

    // No wait needed: both Reqs ride the same FIFO input channel — the loop
    // processes them in order, so the valid Req's reply arriving IS the
    // proof the malformed one was handled (continue) first. Time is I/O.

    // ── Valid Req: should still round-trip ───────────────────────────────
    let valid_req = Value::Struct(Arc::new(wat::runtime::StructValue {
        type_name: ":wat::kernel::services::StdOutService::Req".into(),
        fields: vec![
            Value::i64(tid),
            Value::String(Arc::new("hello from h2".into())),
        ],
    }));
    input_tx
        .send(ServiceMsg::Req(valid_req))
        .expect("valid Req sent");

    // Block for ack — if this hangs, the loop died from the malformed Req (FAIL).
    let io_cell = wat::services::uninstall_thread_io().expect("ThreadIO was installed");
    let ack = io_cell.stdout_reply_rx.recv();
    assert!(
        matches!(ack, Ok(Ok(()))),
        "valid Req after malformed Req must ack Ok(()) — the loop must survive the continue; got {:?}",
        ack
    );

    // Teardown.
    drop(input_tx);
    drop(peer.input_tx);
    let _ = peer.thread.join();
}

/// Spawn a real stdout peer; send a Struct whose field[0] is a String (not i64)
/// — the loop logs and continues; then Register + valid Req and assert ack.
#[test]
fn h2_guard_arm_wrong_field0_type_loop_survives() {
    let world = freeze_skeleton();

    let (stdout_pipe_r, stdout_pipe_w) =
        wat::fork::make_pipe(":test::h2-stdout-wrong-field").expect("pipe for stdout");
    let stdout_writer = Value::io__IOWriter(Arc::new(PipeWriter::from_owned_fd(stdout_pipe_w)));
    let _stdout_reader = PipeReader::from_owned_fd(stdout_pipe_r);

    let stdout_handle = world
        .symbols()
        .get(":wat::kernel::services::StdOutService/handle")
        .expect("/handle in baked stdlib")
        .clone();
    let peer = spawn_service_peer("stdout", stdout_handle, stdout_writer, world.symbols().clone(), |_| Ok(()));
    let input_tx = peer.input_tx.clone();

    let tid = next_thread_id();
    let (reply_tx, reply_rx) = wat::comms::thread::pair::<Result<(), String>>();
    input_tx
        .send(ServiceMsg::Register(tid, reply_tx))
        .expect("Register sent");

    install_thread_io(ThreadIO {
        stdout_reply_rx: reply_rx,
        stderr_reply_rx: {
            let (_, rx) = wat::comms::thread::pair::<Result<(), String>>();
            rx
        },
        thread_id: tid,
        stdin_reply_rx: {
            let (_, rx) = wat::comms::thread::pair::<Result<String, String>>();
            rx
        },
    });

    // ── Malformed Req #2: Struct but field[0] is String, not i64 ────────
    let bad_req = Value::Struct(Arc::new(wat::runtime::StructValue {
        type_name: ":wat::kernel::services::StdOutService::Req".into(),
        fields: vec![
            Value::String(Arc::new("not-an-i64".into())), // field[0] must be i64
            Value::String(Arc::new("line".into())),
        ],
    }));
    input_tx.send(ServiceMsg::Req(bad_req)).expect("bad Req sent");

    // No wait: same-channel FIFO ordering serializes bad-then-valid (see above).

    // ── Valid Req after the bad one ──────────────────────────────────────
    let valid_req = Value::Struct(Arc::new(wat::runtime::StructValue {
        type_name: ":wat::kernel::services::StdOutService::Req".into(),
        fields: vec![
            Value::i64(tid),
            Value::String(Arc::new("hello after bad field".into())),
        ],
    }));
    input_tx.send(ServiceMsg::Req(valid_req)).expect("valid Req sent");

    let io_cell = wat::services::uninstall_thread_io().expect("ThreadIO was installed");
    let ack = io_cell.stdout_reply_rx.recv();
    assert!(
        matches!(ack, Ok(Ok(()))),
        "valid Req after wrong-field-type Req must ack Ok(()) — loop must survive; got {:?}",
        ack
    );

    drop(input_tx);
    drop(peer.input_tx);
    let _ = peer.thread.join();
}
