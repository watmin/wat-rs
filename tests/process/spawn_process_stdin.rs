//! Arc 170 Stone C — spawn-process stdin probe (Row G).
//!
//! Verifies that a parent can write typed values to `Process/stdin` (IOWriter
//! at fields[0] of the Process struct) and the spawn-process child can read
//! them with `(:wat::kernel::readln -> :T)` through bootstrap services.
//!
//! Child fn contract: `[] -> :wat::core::nil` (Stone C).
//! Child reads one i64 via readln, adds 1, prints via println.
//! Parent sends 41 via Sender/from-pipe over Process/stdin.
//! Parent reads 42 via Receiver/from-pipe over Process/stdout.

use std::sync::Arc;
use wat::freeze::startup_beside;
use wat::runtime::{apply_function, Value};

fn process_stdin_writer(process: &Value) -> Arc<dyn wat::io::WatWriter> {
    match process {
        Value::Aggregate(s) if s.nature == wat::Nature::Struct && s.class == "wat::kernel::Process" => match &s.fields[0] {
            Value::io__IOWriter(w) => w.clone(),
            other => panic!("expected IOWriter at fields[0]; got {:?}", other),
        },
        other => panic!("expected Process Struct; got {:?}", other),
    }
}

fn process_stdout_reader(process: &Value) -> Arc<dyn wat::io::WatReader> {
    match process {
        Value::Aggregate(s) if s.nature == wat::Nature::Struct && s.class == "wat::kernel::Process" => match &s.fields[1] {
            Value::io__IOReader(r) => r.clone(),
            other => panic!("expected IOReader at fields[1]; got {:?}", other),
        },
        other => panic!("expected Process Struct; got {:?}", other),
    }
}

fn process_handle(process: &Value) -> Arc<wat::runtime::ProgramHandleInner> {
    match process {
        Value::Aggregate(s) if s.nature == wat::Nature::Struct && s.class == "wat::kernel::Process" => match &s.fields[3] {
            Value::wat__kernel__ProgramHandle(h) => h.clone(),
            other => panic!("expected ProgramHandle at fields[3]; got {:?}", other),
        },
        other => panic!("expected Process Struct; got {:?}", other),
    }
}

/// Row G — parent writes to `Process/stdin`, child reads via `readln`.
///
/// Parent sends i64(41) via Sender/from-pipe over Process/stdin (IOWriter).
/// Child reads via `(:wat::kernel::readln -> :wat::core::i64)`, adds 1,
/// prints 42 via `(:wat::kernel::println ...)`.
/// Parent reads 42 via Receiver/from-pipe over Process/stdout (IOReader).
#[test]
fn probe_spawn_process_stdin() {
    // World + child program loaded from co-located spawn_process_stdin.wat via startup_beside.
    let world = startup_beside(file!()).expect("startup should succeed");
    let launch = world
        .symbols()
        .get(":my::launch")
        .expect(":my::launch defined");
    let process = apply_function(launch.clone(), Vec::new(), world.symbols(), wat::rust_caller_span!())
        .expect("spawn-process succeeds");
    let types = world.symbols().types().map(|a| a.as_ref());

    // Parent sends 41 via Sender/from-pipe wrapping Process/stdin (IOWriter).
    let stdin_writer = process_stdin_writer(&process);
    let sender_val = wat::channel::sender_from_pipe(stdin_writer);
    let sender_inner = match &sender_val {
        Value::wat__kernel__Sender(inner) => inner.as_ref(),
        other => panic!("expected Sender Value; got {:?}", other),
    };
    let send_outcome = wat::channel::typed_send(
        sender_inner,
        Value::i64(41),
        types,
        wat::rust_caller_span!(),
    );
    assert!(
        matches!(send_outcome, wat::channel::SendOutcome::Ok),
        "expected send Ok; got {:?}",
        send_outcome
    );
    // Drop sender so child's readln sees EOF after the read (orderly shutdown).
    drop(sender_val);

    // Parent reads 42 via Receiver/from-pipe wrapping Process/stdout (IOReader).
    let stdout_reader = process_stdout_reader(&process);
    let receiver_val = wat::channel::receiver_from_pipe(stdout_reader);
    let receiver_inner = match &receiver_val {
        Value::wat__kernel__Receiver(inner) => inner.as_ref(),
        other => panic!("expected Receiver Value; got {:?}", other),
    };
    let recv_outcome = wat::channel::typed_recv(
        receiver_inner,
        types,
        wat::rust_caller_span!(),
    );
    let val = match recv_outcome {
        wat::channel::RecvOutcome::Value(v) => v,
        wat::channel::RecvOutcome::Disconnected => {
            let stderr_text = match &process {
                Value::Aggregate(s) => match &s.fields[2] {
                    Value::io__IOReader(rdr) => {
                        let mut all = String::new();
                        while let Ok(Some(line)) = rdr.read_line(wat::rust_caller_span!()) {
                            all.push_str(&line);
                            all.push('\n');
                        }
                        all
                    }
                    _ => "<not IOReader>".to_string(),
                },
                _ => "<not Struct>".to_string(),
            };
            panic!("recv: Disconnected before value flowed; child stderr:\n{}", stderr_text)
        }
        wat::channel::RecvOutcome::DecodeError(msg) => {
            panic!("recv: decode error: {}", msg)
        }
        wat::channel::RecvOutcome::Shutdown => {
            panic!("recv: unexpected process-wide shutdown during test")
        }
    };
    match val {
        Value::i64(n) => assert_eq!(n, 42, "expected 42 (41+1); got {}", n),
        other => panic!("expected i64 42; got {:?}", other),
    }

    // Wait for clean exit.
    use wat::runtime::ProgramHandleInner;
    let handle = process_handle(&process);
    let code = match handle.as_ref() {
        ProgramHandleInner::Forked(child) => child.wait_or_cached_exit(),
        other => panic!("expected Forked ProgramHandle; got {:?}", other),
    };
    assert_eq!(code, 0, "expected child exit 0; got {}", code);
}
