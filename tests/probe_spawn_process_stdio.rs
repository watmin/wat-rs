//! Arc 170 Stone C — spawn-process stdio probe (Row F).
//!
//! Verifies that a spawn-process child can call `(:wat::kernel::println v)`
//! and the parent captures the output via `Process/stdout` (IOReader at
//! fields[1] of the Process struct).
//!
//! Child fn contract: `[] -> :wat::core::nil` (Stone C).
//! Child uses bootstrap services (fd 1 wired to stdout pipe).
//! Parent reads the printed line from the IOReader and verifies value.

use std::sync::Arc;
use wat::ast::WatAST;
use wat::freeze::startup_from_source;
use wat::load::InMemoryLoader;
use wat::runtime::{eval, Environment, Value};
use wat::span::Span;

fn freeze_ok(src: &str) -> wat::freeze::FrozenWorld {
    match startup_from_source(src, None, Arc::new(InMemoryLoader::new())) {
        Ok(w) => w,
        Err(e) => panic!("freeze should succeed; got: {}", e),
    }
}

fn process_stdout_reader(process: &Value) -> Arc<dyn wat::io::WatReader> {
    match process {
        Value::Struct(s) if s.type_name == ":wat::kernel::Process" => match &s.fields[1] {
            Value::io__IOReader(r) => r.clone(),
            other => panic!("expected IOReader at fields[1]; got {:?}", other),
        },
        other => panic!("expected Process Struct; got {:?}", other),
    }
}

fn process_handle(process: &Value) -> Arc<wat::runtime::ProgramHandleInner> {
    match process {
        Value::Struct(s) if s.type_name == ":wat::kernel::Process" => match &s.fields[3] {
            Value::wat__kernel__ProgramHandle(h) => h.clone(),
            other => panic!("expected ProgramHandle at fields[3]; got {:?}", other),
        },
        other => panic!("expected Process Struct; got {:?}", other),
    }
}

/// Row F — child calls `println`, parent reads from `Process/stdout`.
///
/// The child prints the i64 value 42. The parent wraps Process/stdout
/// (IOReader) with `Receiver/from-pipe` and reads the typed value back.
/// The received value must equal 42.
#[test]
fn probe_spawn_process_stdio() {
    // Child: Stone C contract — [] -> nil; uses println for output.
    let src = r#"
        (:wat::core::defn :my::print-42
          []
          -> :wat::core::nil
          (:wat::kernel::println 42))
    "#;
    let world = freeze_ok(src);
    let call = WatAST::List(
        vec![
            WatAST::Keyword(":wat::kernel::spawn-process".into(), Span::unknown()),
            WatAST::Keyword(":my::print-42".into(), Span::unknown()),
        ],
        Span::unknown(),
    );
    let env = Environment::new();
    let process = eval(&call, &env, world.symbols()).expect("spawn-process succeeds");
    let types = world.symbols().types().map(|a| a.as_ref());

    // Parent reads from Process/stdout via Receiver/from-pipe.
    let stdout_reader = process_stdout_reader(&process);
    let receiver_val = wat::typed_channel::receiver_from_pipe(stdout_reader);
    let receiver_inner = match &receiver_val {
        Value::wat__kernel__Receiver(inner) => inner.as_ref(),
        other => panic!("expected Receiver Value; got {:?}", other),
    };
    let recv_outcome = wat::typed_channel::typed_recv(
        receiver_inner,
        types,
        Span::unknown(),
    );
    let val = match recv_outcome {
        wat::typed_channel::RecvOutcome::Value(v) => v,
        wat::typed_channel::RecvOutcome::Disconnected => {
            // Drain stderr for diagnostic.
            let stderr_text = match &process {
                Value::Struct(s) => match &s.fields[2] {
                    Value::io__IOReader(rdr) => {
                        let mut all = String::new();
                        while let Ok(Some(line)) = rdr.read_line(Span::unknown()) {
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
        wat::typed_channel::RecvOutcome::DecodeError(msg) => {
            panic!("recv: decode error: {}", msg)
        }
    };
    match val {
        Value::i64(n) => assert_eq!(n, 42, "expected 42 from child println; got {}", n),
        other => panic!("expected i64 42; got {:?}", other),
    }

    // Wait for clean exit.
    use wat::runtime::ProgramHandleInner;
    let handle = process_handle(&process);
    let code = match handle.as_ref() {
        ProgramHandleInner::Forked(child) => child.wait_or_cached(),
        other => panic!("expected Forked ProgramHandle; got {:?}", other),
    };
    assert_eq!(code, 0, "expected child exit 0; got {}", code);
}
