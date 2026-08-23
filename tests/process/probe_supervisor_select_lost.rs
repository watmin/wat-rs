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

/// The child program's own file tag, as passed to `parse_all_with_file` below.
/// Named once so the standing span-carriage assertion (stone J, arc 296) and
/// the parse call that establishes it cannot drift apart.
const CHILD_PROGRAM_FILE: &str = "<spawn-process-program>";

/// Build `(:wat::kernel::spawn-program (:wat::spawn::process) (:wat::core::forms <forms>...))`
fn build_spawn_process_call(child_program_src: &str) -> WatAST {
    let child_forms = wat::parser::parse_all_with_file(child_program_src, CHILD_PROGRAM_FILE)
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

/// Recursively collect the `:file` value of every `#wat.kernel/Location {…}`
/// node found anywhere in `v` — structural, never a substring/`.contains()`
/// scan of the rendered text (`no_loose_string_assert` is armed and has fired
/// on this arc twice).
fn find_location_files(v: &wat_edn::OwnedValue, out: &mut Vec<String>) {
    use wat_edn::Value::*;
    if let Tagged(t, body) = v {
        if t.namespace() == "wat.kernel" && t.name() == "Location" {
            if let Map(fields) = body.as_ref() {
                for (k, fv) in fields {
                    if let (Keyword(kw), String(s)) = (k, fv) {
                        if kw.namespace().is_none() && kw.name() == "file" {
                            out.push(s.as_ref().to_string());
                        }
                    }
                }
            }
        }
    }
    match v {
        Tagged(_, body) => find_location_files(body, out),
        List(xs) | Vector(xs) | Set(xs) => xs.iter().for_each(|x| find_location_files(x, out)),
        Map(kvs) => kvs.iter().for_each(|(k, v)| {
            find_location_files(k, out);
            find_location_files(v, out);
        }),
        _ => {}
    }
}

/// STANDING ASSERTION (independent of the golden below) — stone J/296's whole
/// claim for this probe: the crashed child's `:location` must name the
/// CHILD's own source, never this decoder's Rust line. Unlike the golden,
/// recapturing this file cannot silently erase the property: it is a fixed
/// structural check, not a byte-for-byte oracle that a blind `UPDATE_EDN=1`
/// would happily re-stamp over a regression.
fn assert_location_names_the_child_not_the_decoder(msg: &str) {
    let parsed = wat_edn::parse_owned(msg)
        .unwrap_or_else(|e| panic!("crash message must itself be valid EDN: {e} — msg: {msg}"));
    let mut files = Vec::new();
    find_location_files(&parsed, &mut files);
    assert_eq!(
        files.len(),
        1,
        "expected exactly one #wat.kernel/Location in the crash chain — msg: {msg}"
    );
    assert_eq!(
        files[0], CHILD_PROGRAM_FILE,
        "STANDING (stone J): :location's :file must be the CRASHED CHILD's own \
         source file, not this decoder's. If this regresses, span carriage broke \
         and every diagnostic from a spawned child will point at wat-rs's own \
         Rust source instead of the user's — msg: {msg}"
    );
    let is_src_rs_path = files[0].starts_with("src/") && files[0].ends_with(".rs");
    assert!(
        !is_src_rs_path,
        "STANDING (stone J): :location's :file must NEVER be a `src/*.rs` path \
         — got {:?}, which means the span carriage dropped the child's real \
         span and fell back to the decoder's own `rust_caller_span!()` — msg: {msg}",
        files[0]
    );
}

/// The process child immediately panics with "boom" via `Option/expect` on `None`.
const CRASHING_CHILD_SRC: &str = r#"
    (:wat::core::defn :user::main [] -> :wat::core::nil
      (:wat::core::Option/expect :wat::core::None "boom"))
"#;

/// `select'` over `[child]` where child immediately crashes.
/// At HEAD this raises; after the strike it returns `ServiceEvent::Lost{idx=0}`.
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
    // Note: we use a plain vector literal [child] via embedded wat. Arc 109 "the comma dies in
    // the reader" retired the comma-carrying `Process<nil,nil>` spelling; the `:-` binder form
    // (already live in the stdlib — wat/cache.wat, wat/spawn.wat) is the replacement.
    let select_call = wat::parse_one!(
        r#"
        (:wat::kernel::select (:wat::core::Vector (:wat::kernel::Process :- [:wat::core::nil :wat::core::nil]) child))
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
                                s.class.as_ref(), "wat::kernel::Failure",
                                "cause must be Failure struct; got {:?}",
                                s.class
                            );
                            // Arc 278 the string-wrap annihilation — Failure.fields[0] is
                            // the mandatory `error` (Fault); its fields[0] is the message String.
                            match s.fields.first() {
                                Some(Value::Aggregate(err)) => match err.fields.first() {
                                    Some(Value::String(msg)) => {
                                        // STANDING, independent of the golden below (stone J,
                                        // arc 296): the crash's :location must name the child's
                                        // own source, never this decoder's Rust file. A wholesale
                                        // golden recapture cannot silently erase this — it is
                                        // asserted structurally on the parsed EDN, separately.
                                        assert_location_names_the_child_not_the_decoder(msg.as_str());
                                        wat::assert_edn_matches_file!(msg.as_str().to_string(), "probe_supervisor_select_lost__process_panics.edn", "Failure.error.message must match the process crash sentinel golden");
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
