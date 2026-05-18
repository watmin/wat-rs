//! Diagnostic probe — counter actor process-tier subprocess startup.
//!
//! Mirrors the wat-tests/counter-actor-proof-process.wat subprocess program
//! and surfaces the subprocess stderr so we can see exactly what fails.
//!
//! The subprocess forms contain:
//!   1. enum declarations (counter::Request, counter::Response)
//!   2. defn :counter/dispatch using ambient readln/println
//!   3. define :user::main calling dispatch
//!
//! This probe exists to surface the actual startup error (if any) from
//! the subprocess. Delete after the process-tier deftest passes.

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

fn build_spawn_process_call(child_program_src: &str) -> WatAST {
    let child_forms =
        wat::parser::parse_all_with_file(child_program_src, "<spawn-process-program>")
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

fn drain_stderr(process: &Value) -> String {
    match process {
        Value::Struct(s) if s.type_name == ":wat::kernel::Process" => match &s.fields[2] {
            Value::io__IOReader(rdr) => {
                let mut all = String::new();
                while let Ok(Some(line)) = rdr.read_line(Span::unknown()) {
                    all.push_str(&line);
                    all.push('\n');
                }
                all
            }
            _ => "<stderr field not IOReader>".into(),
        },
        _ => "<not a Process Struct>".into(),
    }
}

/// Probe 1 — minimal subprocess with just enum declarations and a user::main.
///
/// This verifies that enum declarations + user::main can be in subprocess forms.
#[test]
fn probe_counter_subprocess_minimal() {
    let server_program_src = r#"
        (:wat::core::enum :counter::Request
          (Get)
          (Increment (n :wat::core::i64))
          (Reset)
          (Shutdown))
        (:wat::core::enum :counter::Response
          (Value (v :wat::core::i64))
          (Ok    (v :wat::core::i64))
          (Final (v :wat::core::i64)))
        (:wat::core::define (:user::main -> :wat::core::nil)
          :wat::core::nil)
    "#;
    let world = freeze_ok("");
    let spawn_call = build_spawn_process_call(server_program_src);
    let process = eval(&spawn_call, &Environment::new(), world.symbols())
        .expect("spawn-process should succeed");

    // Close stdin by dropping (no stdin write; subprocess exits cleanly)
    let exit_code = join_process(&process);
    let stderr = drain_stderr(&process);
    println!("exit_code: {}", exit_code);
    println!("stderr: {}", stderr);
    assert_eq!(exit_code, 0, "subprocess should exit cleanly; stderr: {}", stderr);
}

/// Probe 2 — subprocess with enum declarations + defn using readln.
///
/// This is closer to the actual counter pattern — defn :counter/dispatch
/// uses readln. The subprocess does NOT call dispatch (user::main is nil).
#[test]
fn probe_counter_subprocess_with_defn() {
    let server_program_src = r#"
        (:wat::core::enum :counter::Request
          (Get)
          (Increment (n :wat::core::i64))
          (Reset)
          (Shutdown))
        (:wat::core::enum :counter::Response
          (Value (v :wat::core::i64))
          (Ok    (v :wat::core::i64))
          (Final (v :wat::core::i64)))
        (:wat::core::defn :counter/dispatch
          [state <- :wat::core::i64]
          -> :wat::core::nil
          (:wat::core::match (:wat::kernel::readln -> :counter::Request)
            -> :wat::core::nil
            ((:counter::Request::Get)
               (:wat::core::do
                 (:wat::kernel::println (:counter::Response::Value state))
                 (:counter/dispatch state)))
            ((:counter::Request::Increment n)
               (:wat::core::let [new-n (:wat::core::i64::+'2 state n)]
                 (:wat::kernel::println (:counter::Response::Ok new-n))
                 (:counter/dispatch new-n)))
            ((:counter::Request::Reset)
               (:wat::core::do
                 (:wat::kernel::println (:counter::Response::Ok 0))
                 (:counter/dispatch 0)))
            ((:counter::Request::Shutdown)
               (:wat::kernel::println (:counter::Response::Final state)))))
        (:wat::core::define (:user::main -> :wat::core::nil)
          :wat::core::nil)
    "#;
    let world = freeze_ok("");
    let spawn_call = build_spawn_process_call(server_program_src);
    let process = eval(&spawn_call, &Environment::new(), world.symbols())
        .expect("spawn-process should succeed");

    let exit_code = join_process(&process);
    let stderr = drain_stderr(&process);
    println!("exit_code: {}", exit_code);
    println!("stderr: {}", stderr);
    assert_eq!(exit_code, 0, "subprocess should exit cleanly; stderr: {}", stderr);
}

/// Probe 3 — full counter subprocess IPC round-trip using ProcessPeer.
///
/// Uses the same approach as wat_process_peer_ipc_round_trip.rs T2 — builds
/// the ProcessPeer in embedded wat code and exercises it from there.
/// This is the actual test pattern from counter-actor-proof-process.wat.
#[test]
fn probe_counter_subprocess_full_process_peer() {
    let server_program_src = r#"
        (:wat::core::enum :counter::Request
          (Get)
          (Increment (n :wat::core::i64))
          (Reset)
          (Shutdown))
        (:wat::core::enum :counter::Response
          (Value (v :wat::core::i64))
          (Ok    (v :wat::core::i64))
          (Final (v :wat::core::i64)))
        (:wat::core::defn :counter/dispatch
          [state <- :wat::core::i64]
          -> :wat::core::nil
          (:wat::core::match (:wat::kernel::readln -> :counter::Request)
            -> :wat::core::nil
            ((:counter::Request::Get)
               (:wat::core::do
                 (:wat::kernel::println (:counter::Response::Value state))
                 (:counter/dispatch state)))
            ((:counter::Request::Increment n)
               (:wat::core::let [new-n (:wat::core::i64::+'2 state n)]
                 (:wat::kernel::println (:counter::Response::Ok new-n))
                 (:counter/dispatch new-n)))
            ((:counter::Request::Reset)
               (:wat::core::do
                 (:wat::kernel::println (:counter::Response::Ok 0))
                 (:counter/dispatch 0)))
            ((:counter::Request::Shutdown)
               (:wat::kernel::println (:counter::Response::Final state)))))
        (:wat::core::define (:user::main -> :wat::core::nil)
          (:counter/dispatch 10))
    "#;

    // Parent world needs to know the counter::Response enum type
    // so Process/readln can deserialize the responses.
    // Also needs counter::Request to construct the request variants.
    let parent_src = r#"
        (:wat::core::enum :counter::Request
          (Get)
          (Increment (n :wat::core::i64))
          (Reset)
          (Shutdown))
        (:wat::core::enum :counter::Response
          (Value (v :wat::core::i64))
          (Ok    (v :wat::core::i64))
          (Final (v :wat::core::i64)))
    "#;
    let world = freeze_ok(parent_src);
    let spawn_call = build_spawn_process_call(server_program_src);
    let process = eval(&spawn_call, &Environment::new(), world.symbols())
        .expect("spawn-process should succeed");

    // Bind process and exercise via embedded wat code
    let env = Environment::new().child().bind("proc", process.clone()).build();

    // Arc 208 slice 2 — Process/println + Process/readln are matched honestly.
    // resp is the unwrapped counter::Response (from the Ok arm).
    let client_code = wat::parse_one!(
        r#"
        (:wat::core::let
          [rx    (:wat::kernel::Receiver/from-pipe (:wat::kernel::Process/stdout proc))
           tx    (:wat::kernel::Sender/from-pipe   (:wat::kernel::Process/stdin  proc))
           peer! (:wat::kernel::ProcessPeer/new rx tx)]
          (:wat::core::match (:wat::kernel::Process/println peer! (:counter::Request::Increment 5))
            -> :counter::Response
            ((:wat::core::Ok _)
              (:wat::core::match (:wat::kernel::Process/readln peer!)
                -> :counter::Response
                ((:wat::core::Ok resp)
                  (:wat::core::match (:wat::kernel::Process/println peer! (:counter::Request::Shutdown))
                    -> :counter::Response
                    ((:wat::core::Ok _)
                      (:wat::core::match (:wat::kernel::Process/readln peer!)
                        -> :counter::Response
                        ((:wat::core::Ok _resp2)
                          (:wat::core::let [_joined (:wat::kernel::Process/drain-and-join proc)]
                            resp))
                        ((:wat::core::Err _chain)
                          (:wat::kernel::assertion-failed! "Process/readln (Shutdown resp) failed" :wat::core::None :wat::core::None))))
                    ((:wat::core::Err _chain)
                      (:wat::kernel::assertion-failed! "Process/println (Shutdown) failed" :wat::core::None :wat::core::None))))
                ((:wat::core::Err _chain)
                  (:wat::kernel::assertion-failed! "Process/readln (Increment resp) failed" :wat::core::None :wat::core::None))))
            ((:wat::core::Err _chain)
              (:wat::kernel::assertion-failed! "Process/println (Increment) failed" :wat::core::None :wat::core::None))))
        "#
    )
    .expect("client code parses");

    let result = match eval(&client_code, &env, world.symbols()) {
        Ok(v) => v,
        Err(e) => {
            let stderr = drain_stderr(&process);
            panic!("client eval failed: {}\nprocess stderr:\n{}", e, stderr);
        }
    };
    println!("result: {:?}", result);
    // The response to Increment 5 starting at 10 should be Ok(15)
    match &result {
        Value::Enum(ev) => {
            println!("variant: {}, fields: {:?}", ev.variant_name, ev.fields);
            assert!(
                ev.variant_name == "Ok" || ev.variant_name == "Value",
                "expected Ok or Value variant; got {}",
                ev.variant_name
            );
        }
        other => panic!("expected Enum from Process/readln (via match Ok arm); got {:?}", other),
    }
}

fn join_process(process: &Value) -> i64 {
    match process {
        Value::Struct(s) if s.type_name == ":wat::kernel::Process" => match &s.fields[3] {
            Value::wat__kernel__ProgramHandle(h) => match h.as_ref() {
                wat::runtime::ProgramHandleInner::Forked(child) => child.wait_or_cached(),
                _ => -999,
            },
            _ => -998,
        },
        _ => -997,
    }
}
