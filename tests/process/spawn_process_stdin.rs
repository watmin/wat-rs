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
use wat::ast::WatAST;
use wat::freeze::startup_bare;
use wat::runtime::{eval, Environment, Value};

fn process_stdin_writer(process: &Value) -> Arc<dyn wat::io::WatWriter> {
    match process {
        Value::Aggregate(s) if s.holder == wat::Holder::Struct && s.class == "wat::kernel::Process" => match &s.fields[0] {
            Value::io__IOWriter(w) => w.clone(),
            other => panic!("expected IOWriter at fields[0]; got {:?}", other),
        },
        other => panic!("expected Process Struct; got {:?}", other),
    }
}

fn process_stdout_reader(process: &Value) -> Arc<dyn wat::io::WatReader> {
    match process {
        Value::Aggregate(s) if s.holder == wat::Holder::Struct && s.class == "wat::kernel::Process" => match &s.fields[1] {
            Value::io__IOReader(r) => r.clone(),
            other => panic!("expected IOReader at fields[1]; got {:?}", other),
        },
        other => panic!("expected Process Struct; got {:?}", other),
    }
}

fn process_handle(process: &Value) -> Arc<wat::runtime::ProgramHandleInner> {
    match process {
        Value::Aggregate(s) if s.holder == wat::Holder::Struct && s.class == "wat::kernel::Process" => match &s.fields[3] {
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
    // Arc 170 slice 6 — parent world is incidental (no user defns needed);
    // child's :user::main is constructed below via parse + AST, not loaded from WAT.
    let world = startup_bare().expect("startup should succeed");
    // Build the child program as a single :user::main define whose body
    // is the read-plus-print logic. Use parse to construct the body —
    // simpler than manual AST surgery and matches the source-form path.
    let child_program_src = r#"
        (:wat::core::defn :user::main [] -> :wat::core::nil
          (:wat::core::let
                      [n    (:wat::kernel::readln -> :wat::core::i64)
                       _out (:wat::kernel::println (:wat::core::i64::+ n 1))]
                      nil))
    "#;
    let child_forms = wat::parser::parse_all_with_file(child_program_src, "<probe>")
        .expect("child program parse");
    // Wrap them in (:wat::core::forms <form>...) for the spawn-process call.
    let mut forms_items = vec![WatAST::Keyword(":wat::core::forms".into(), wat::rust_caller_span!())];
    forms_items.extend(child_forms);
    let forms_call = WatAST::List(forms_items, wat::rust_caller_span!());
    let call = WatAST::List(
        vec![
            WatAST::Keyword(":wat::kernel::spawn-process".into(), wat::rust_caller_span!()),
            forms_call,
        ],
        wat::rust_caller_span!(),
    );
    let env = Environment::new();
    let process = eval(&call, &env, world.symbols()).expect("spawn-process succeeds").value_owned();
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
