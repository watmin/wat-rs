//! Stone — `select'` over a process child that floods stdout with > 512 KiB
//! of un-terminated data must return `ServiceEvent::Lost` without deadlocking.
//!
//! ## The bug (unfixed HEAD)
//!
//! `eval_peer_select_prime`'s process arm (src/runtime.rs ~24755) handles
//! `SelectOutcome::Recv { result: Err(_) }` with a single `Err(_)` arm that
//! calls `classify_peer_death(err_rxs[index.0].recv())` regardless of whether
//! the error is `RecvError::FrameTooLarge` or `RecvError::Disconnected`.
//!
//! When the child floods its stdout pipe with an un-terminated frame larger
//! than `DEFAULT_MAX_FRAME_BYTES` (512 KiB):
//! 1. The parent's `process::Select::select()` reads bytes, accumulates past
//!    512 KiB, `take_buffered_frame` returns `Err(RecvError::FrameTooLarge)`.
//! 2. `SelectOutcome::Recv { result: Err(FrameTooLarge) }` arrives.
//! 3. The `Err(_)` arm calls `err_rxs[index.0].recv()` — BLOCKING.
//! 4. The child is ALIVE and blocked in `write_all` (pipe full; parent stopped
//!    draining). The err channel is EMPTY until the child exits.
//! 5. Parent blocks on `err.recv()` ↔ child blocks on `write_all` — DEADLOCK.
//! 6. Watchdog fires → `_exit(124)` → test runner reports FAIL.
//!
//! ## The fix
//!
//! Before the `Err(_)` path that calls `err_rxs[index.0].recv()`, check for
//! `RecvError::FrameTooLarge` distinctly. On FrameTooLarge, return
//! `ServiceEvent::Lost { idx, cause }` IMMEDIATELY without reading the err
//! channel (the peer is torn down via RAII drop when the guards drop).
//!
//! ## Why WAT-level (not Rust-level like probe_overcap_no_deadlock)?
//!
//! The `recv'` fix was tested at the Rust level to bypass `ThreadOwnedCell`.
//! The `select'` path goes through `eval_peer_select_prime` — that function IS
//! called from the eval thread, which IS the ThreadOwnedCell owner thread (the
//! same thread that called `spawn-program'`). So calling WAT `select'` via
//! `eval()` exercises the EXACT code path that deadlocks, without needing to
//! bypass ThreadOwnedCell.
//!
//! ## Flood strategy
//!
//! The child builds a 1 MiB string (2^20 = 1,048,576 bytes of 'x') via
//! `double-string` and calls `(:wat::kernel::println big-string)`. The `println`
//! wire format is a quoted EDN string: `"xxxx...xxx"\n`. Total frame size is
//! ~1,048,578 bytes — well above the 512 KiB cap. The child stays alive (blocked
//! in the kernel's `write(2)` because the pipe buffer is full) while the parent
//! accumulates bytes past the cap.
//!
//! Does NOT use `print-raw'` (being eliminated in a separate strike).

use std::sync::Arc;
use std::time::Duration;

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

/// Build `(:wat::kernel::spawn-program' (:wat::spawn::process) (:wat::core::forms <forms>...))`
fn build_spawn_process_call(child_program_src: &str) -> WatAST {
    let child_forms =
        wat::parser::parse_all_with_file(child_program_src, "<spawn-process-program>")
            .expect("child program parse");
    let mut forms_items = vec![WatAST::Keyword(":wat::core::forms".into(), Span::unknown())];
    forms_items.extend(child_forms);
    let forms_call = WatAST::List(forms_items, Span::unknown());
    WatAST::List(
        vec![
            WatAST::Keyword(":wat::kernel::spawn-program'".into(), Span::unknown()),
            WatAST::List(
                vec![WatAST::Keyword(":wat::spawn::process".into(), Span::unknown())],
                Span::unknown(),
            ),
            forms_call,
        ],
        Span::unknown(),
    )
}

/// Child builds a 1 MiB string via `double-string` and `println`s it.
/// `println` encodes to EDN (quoted string ~1,048,578 bytes) + newline.
/// Total frame > 512 KiB cap → parent hits FrameTooLarge while child stays alive.
const FLOOD_CHILD_SRC: &str = r#"
    (:wat::core::defn :user::double-string
        [s <- :wat::core::String n <- :wat::core::i64]
        -> :wat::core::String
      (:wat::core::if (:wat::core::= n 0) -> :wat::core::String
        s
        (:user::double-string (:wat::core::String/concat s s) (:wat::core::- n 1))))

    (:wat::core::defn :user::main [] -> :wat::core::nil
      (:wat::kernel::println
        (:user::double-string "x" 20)))
"#;

/// `select'` over `[child]` where child floods stdout with > 512 KiB.
///
/// At HEAD: `eval_peer_select_prime` receives `SelectOutcome::Recv { result:
/// Err(RecvError::FrameTooLarge) }` and falls into `Err(_) => classify_peer_death(
/// err_rxs[0].recv())` — DEADLOCK (child alive, err channel empty).
/// Watchdog fires → exit 124 → FAIL.
///
/// After fix: `FrameTooLarge` is handled before the `Err(_)` path; `select'`
/// returns `ServiceEvent::Lost { idx: 0, cause }` immediately → PASS.
#[test]
fn select_prime_flood_no_deadlock() {
    // Arm the watchdog: deadlock → _exit(124) → test FAIL.
    arm_watchdog(Duration::from_secs(10));

    let world = freeze_ok("");

    // Spawn the flooding child.
    let spawn_call = build_spawn_process_call(FLOOD_CHILD_SRC);
    let child = eval(&spawn_call, &Environment::new(), world.symbols())
        .expect("spawn-program' should succeed")
        .value_owned();

    // Bind child into the env.
    let env = Environment::new()
        .child()
        .bind("child", Span::unknown(), child.into())
        .build();

    // Eval: (select' (Vector :wat::kernel::Process'<:wat::core::nil,:wat::core::nil> child))
    let select_call = wat::parse_one!(
        r#"
        (:wat::kernel::select' (:wat::core::Vector :wat::kernel::Process'<:wat::core::nil,:wat::core::nil> child))
        "#
    )
    .expect("parse select' call");

    // This is the blocking call — deadlocks at HEAD, returns fast after fix.
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
                        "flood child must yield ServiceEvent::Lost (FrameTooLarge); got variant {:?}",
                        ev.variant_name
                    );
                    // fields[0] = idx (i64)
                    assert!(ev.fields.len() >= 1, "Lost must have idx field");
                    assert_eq!(
                        ev.fields[0],
                        Value::i64(0),
                        "single-peer select': idx must be 0; got {:?}",
                        ev.fields[0]
                    );
                    // fields[1] = cause (Failure struct) — cause message must mention the cap.
                    if ev.fields.len() >= 2 {
                        match &ev.fields[1] {
                            Value::Struct(s) => {
                                assert_eq!(
                                    s.type_name, ":wat::kernel::Failure",
                                    "cause must be Failure struct; got {:?}",
                                    s.type_name
                                );
                                match s.fields.first() {
                                    Some(Value::String(msg)) => {
                                        assert!(
                                            msg.contains("cap") || msg.contains("frame") || msg.contains("512"),
                                            "Failure.message must mention the cap; got {:?}",
                                            msg
                                        );
                                        eprintln!("[select_prime_flood_no_deadlock] Got Lost with cause: {:?}", msg);
                                    }
                                    other => panic!(
                                        "Failure.message (field 0) must be String; got {:?}",
                                        other
                                    ),
                                }
                            }
                            other => panic!(
                                "Lost.cause (field 1) must be Failure struct; got {:?}",
                                other
                            ),
                        }
                    }
                }
                other => panic!(
                    "select' must return ServiceEvent enum; got {:?}",
                    other
                ),
            }
        }
        Err(e) => {
            // At HEAD with the deadlock: the watchdog fires first (_exit(124)).
            // If we somehow get here without deadlock (e.g. some other error path):
            panic!(
                "select' raised instead of returning ServiceEvent::Lost \
                 (expected ServiceEvent::Lost for FrameTooLarge flood): {}",
                e
            );
        }
    }
}

/// Arm a watchdog thread that calls `_exit(124)` after `timeout`.
///
/// If the test deadlocks (parent blocked in `err.recv()` while child is alive),
/// the watchdog terminates the test process with exit code 124. The test harness
/// sees a non-zero exit and reports FAIL.
///
/// If the test completes before the timeout, the watchdog sleeps its full
/// duration — but the test process has already exited normally by then.
fn arm_watchdog(timeout: Duration) {
    std::thread::spawn(move || {
        std::thread::sleep(timeout);
        eprintln!(
            "\n[WATCHDOG] select_prime_flood_no_deadlock: select'() did not return within {:?} \
             — the parent is blocked on err_rxs[0].recv() while the child is blocked on write_all \
             (pipe full, FrameTooLarge). This is the select' FrameTooLarge deadlock bug. \
             Killing test process with exit code 124.",
            timeout
        );
        // SAFETY: _exit is always safe to call; the process exits immediately.
        unsafe { libc::_exit(124) };
    });
}
