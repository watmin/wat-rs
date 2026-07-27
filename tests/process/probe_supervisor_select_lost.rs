//! Stone 259 — `select'` over a crashed process child must return
//! `ServiceEvent::Lost{idx, cause}`, not raise or fold to `:Closed`.
//!
//! **RED at HEAD**: `select'` raises on process peer EOF
//! ("peer closed / child exited") instead of returning a `ServiceEvent`.
//!
//! **GREEN after the strike**: `select'` reads the crash channel on output-EOF,
//! builds `ServiceEvent::Lost{idx, cause}` from the reason, and returns it.
//!
//! The probe spawns a process child whose `:user::main` immediately panics
//! via `(:wat::core::Option/expect -> :wat::core::nil :wat::core::None "boom")`,
//! then calls `(select' (Vector child))` from embedded wat and asserts the
//! returned `ServiceEvent` is `:Lost{idx=0, cause}` whose cause message contains
//! "boom".
//!
//! Modeled on `tests/wat_process_peer_ipc_round_trip.rs`.
//!
// rune:lint(no-inlined-wat) — the crashing-child forms + the `select'` driver expression must
// bind a Rust-spawned Process VALUE (`child`) into an Environment before eval; a co-located
// static .wat fixture cannot express that runtime binding. The exact golden below also embeds
// the child program's own file-tag ("<spawn-process-program>") + source line/col from the
// inline CRASHING_CHILD_SRC string — moving the program into a real .wat fixture would change
// those location fields and silently corrupt the byte-exact assertion. Genuine dynamic-driver
// need (docs/CONVENTIONS.md § Test idioms, escape hatch).

use wat::ast::WatAST;
use wat::freeze::startup_bare;
use wat::runtime::{eval, Environment, Value};

/// Build `(:wat::kernel::spawn-program (:wat::spawn::process) (:wat::core::forms <forms>...))`
fn build_spawn_process_call(child_program_src: &str) -> WatAST {
    let child_forms =
        wat::parser::parse_all_with_file(child_program_src, "<spawn-process-program>")
            .expect("child program parse");
    let mut forms_items = vec![WatAST::Keyword(":wat::core::forms".into(), wat::rust_caller_span!())];
    forms_items.extend(child_forms);
    let forms_call = WatAST::List(forms_items, wat::rust_caller_span!());
    WatAST::List(
        vec![
            WatAST::Keyword(":wat::kernel::spawn-program".into(), wat::rust_caller_span!()),
            WatAST::List(
                vec![WatAST::Keyword(":wat::spawn::process".into(), wat::rust_caller_span!())],
                wat::rust_caller_span!(),
            ),
            forms_call,
        ],
        wat::rust_caller_span!(),
    )
}

/// The process child immediately panics with "boom" via `Option/expect` on `None`.
const CRASHING_CHILD_SRC: &str = r#"
    (:wat::core::defn :user::main [] -> :wat::core::nil
      (:wat::core::Option/expect :wat::core::None "boom"))
"#;

/// `select'` over `[child]` where child immediately crashes.
/// At HEAD this raises; after the strike it returns `ServiceEvent::Lost{idx=0}`.
#[ignore = "296-recapture-pending: golden asserts pre-stone-B rust-debug face; unlock: 296 recapture (.edn data-equality flip)"]
#[test]
fn select_prime_yields_lost_when_process_child_crashes() {
    // Empty parent world — substrate stdlib only.
    let world = startup_bare().expect("freeze should succeed");

    // Spawn the crashing child.
    let spawn_call = build_spawn_process_call(CRASHING_CHILD_SRC);
    let child = eval(&spawn_call, &Environment::new(), world.symbols())
        .expect("spawn-program' should succeed")
        .value_owned();

    // Bind child into the env.
    let env = Environment::new()
        .child()
        .bind("child", wat::rust_caller_span!(), child.into())
        .build();

    // Eval: (select' (Vector :wat::kernel::Process<wat::core::nil,wat::core::nil> child))
    //
    // Note: we use a plain vector literal [child] via embedded wat.
    let select_call = wat::parse_one!(
        r#"
        (:wat::kernel::select (:wat::core::Vector :wat::kernel::Process<wat::core::nil,wat::core::nil> child))
        "#
    )
    .expect("parse select' call");

    // Give the child a moment to crash (it panics immediately on startup, but
    // the OS process spawn + EDN framing path means we just block on select'
    // which should return quickly once the child dies).
    let result = eval(&select_call, &env, world.symbols());

    match result {
        Ok(tv) => {
            let event = tv.value_owned();
            match &event {
                Value::Enum(ev) => {
                    assert_eq!(
                        ev.type_path, ":wat::spawn::ServiceEvent",
                        "select' must return ServiceEvent; got type_path {:?}",
                        ev.type_path
                    );
                    assert_eq!(
                        ev.variant_name, "Lost",
                        "crashed child must yield ServiceEvent::Lost; got variant {:?}",
                        ev.variant_name
                    );
                    // fields[0] = idx (i64), fields[1] = cause (Failure struct)
                    assert!(ev.fields.len() >= 2, "Lost must have idx + cause fields");
                    assert_eq!(
                        ev.fields[0],
                        Value::i64(0),
                        "single-peer select': idx must be 0; got {:?}",
                        ev.fields[0]
                    );
                    // Cause must be a Failure struct whose message field contains "boom".
                    match &ev.fields[1] {
                        Value::Aggregate(s) => {
                            assert_eq!(
                                s.class, "wat::kernel::Failure",
                                "cause must be Failure struct; got {:?}",
                                s.class
                            );
                            // Arc 278 the string-wrap annihilation — Failure.fields[0] is
                            // the mandatory `error` (Fault); its fields[0] is the message String.
                            match s.fields.first() {
                                Some(Value::Aggregate(err)) => match err.fields.first() {
                                    Some(Value::String(msg)) => {
                                        wat::assert_edn_eq!(
                                            msg.as_str().to_string(),
                                            include_str!("probe_supervisor_select_lost__process_panics.edn"),
                                            "Failure.error.message must match the process crash sentinel golden"
                                        );
                                    }
                                    other => panic!(
                                        "Failure.error.message must be String; got {:?}",
                                        other
                                    ),
                                },
                                other => panic!(
                                    "Failure.error (field 0) must be an Aggregate; got {:?}",
                                    other
                                ),
                            }
                        }
                        other => panic!("Lost.cause must be Failure struct; got {:?}", other),
                    }
                }
                other => panic!(
                    "select' must return ServiceEvent enum; got {:?}",
                    other
                ),
            }
        }
        Err(e) => {
            // At HEAD this is the expected failure (select' raises on EOF).
            // This is RED — the test should panic here at HEAD.
            panic!(
                "select' raised instead of returning ServiceEvent::Lost (RED at HEAD — implement the strike): {}",
                e
            );
        }
    }
}
