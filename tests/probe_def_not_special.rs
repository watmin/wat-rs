//! Arc 170 slice 3 Gap I-B — probes confirming `def` is no longer special.
//!
//! Gap I-B closes the asymmetry between `def` and the other 7 declaration forms:
//!
//! - **Check-time:** The validator's `:wat::core::def` arm is retired. `def` falls
//!   through to the `_ =>` arm like the other 7 forms — silent at check time.
//! - **Runtime:** The permissive eval arm (evaluate RHS, return Unit) is replaced
//!   by a `DeclarationInExpressionPosition` error — loud rejection symmetric with
//!   `define`'s prior behavior.
//! - **End-to-end:** `def` at fn-body do-prefix in a closure flowing to
//!   `spawn-process` now compiles at parent check-time (no `DefNotTopLevel`)
//!   and the child's startup registers it via `register_runtime_defs`.
//!
//! ## The 5 probes
//!
//! 1. `probe_def_at_fn_body_do_prefix_lifts_to_prologue_end_to_end` — the
//!    spawn probe Gap I-A's probe 1 couldn't deliver (blocked at parent check time).
//!    After Gap I-B, the parent accepts it; the child registers the def binding;
//!    the body references it successfully.
//! 2. `probe_def_at_expression_position_emits_position_error_at_runtime` — def
//!    buried inside a function body; calling the function emits
//!    `DeclarationInExpressionPosition` at runtime.
//! 3. `probe_def_at_top_level_still_works` — regression; top-level def unaffected.
//! 4. `probe_define_at_expression_position_still_emits_error` — regression; define's
//!    position discipline unchanged (now via `DeclarationInExpressionPosition`).
//! 5. `probe_mixed_declaration_prelude_now_includes_def` — the mixed 8-form prelude
//!    from Gap I-A probe 6, extended to include `def`. All 8 declaration forms
//!    lift together.

use std::sync::Arc;
use wat::ast::WatAST;
use wat::freeze::{eval_in_frozen, startup_from_source};
use wat::load::InMemoryLoader;
use wat::runtime::{Environment, ProgramHandleInner, RuntimeError, Value};

// ─── helpers ─────────────────────────────────────────────────────────────────

fn freeze_ok(src: &str) -> wat::freeze::FrozenWorld {
    match startup_from_source(src, None, Arc::new(InMemoryLoader::new())) {
        Ok(w) => w,
        Err(e) => panic!("freeze should succeed; got: {}", e),
    }
}

/// Drain the stderr field (index 2) of a Process Struct value.
fn drain_stderr(process: &Value) -> String {
    match process {
        Value::Struct(s) if s.type_name == ":wat::kernel::Process" => match &s.fields[2] {
            Value::io__IOReader(rdr) => {
                let mut all = String::new();
                while let Ok(Some(line)) = rdr.read_line(wat::span::Span::unknown()) {
                    all.push_str(&line);
                }
                all
            }
            _ => "<stderr field not IOReader>".into(),
        },
        _ => "<not a Process Struct>".into(),
    }
}

/// Evaluate `(:my::launch)` in the frozen world, fork the child, wait for
/// it to exit, and return (exit_code, stderr_text).
fn run_launch(world: &wat::freeze::FrozenWorld) -> (i64, String) {
    let call = WatAST::List(
        vec![WatAST::Keyword(
            ":my::launch".into(),
            wat::span::Span::unknown(),
        )],
        wat::span::Span::unknown(),
    );
    let env = Environment::new();
    let process = wat::runtime::eval(&call, &env, world.symbols())
        .expect("launch should evaluate");
    let handle = match &process {
        Value::Struct(s) if s.type_name == ":wat::kernel::Process" => match &s.fields[3] {
            Value::wat__kernel__ProgramHandle(h) => h.clone(),
            other => panic!("expected ProgramHandle field at index 3; got {:?}", other),
        },
        other => panic!("expected Process Struct from launch; got {:?}", other),
    };
    let exit_code: i64 = match handle.as_ref() {
        ProgramHandleInner::Forked(child) => child.wait_or_cached(),
        other => panic!("expected Forked handle; got {:?}", other),
    };
    let stderr = drain_stderr(&process);
    (exit_code, stderr)
}

// ─── Probe 1 — def at fn body do-prefix lifts to prologue end-to-end ─────────

/// The spawn probe Gap I-A's probe 1 couldn't deliver.
///
/// Before Gap I-B, `def` at a fn body's `do`-prefix was blocked at PARENT
/// check time by `validate_def_position_with_wrapper` (which emitted
/// `DefNotTopLevel`), preventing `extract_closure` from ever running.
///
/// After Gap I-B:
/// 1. Parent check-time: validator's def arm retired → no `DefNotTopLevel`
/// 2. `extract_closure` (Gap I-A): `split_body_prelude` lifts the `def` form
///    to the closure prologue via `is_declaration_form`
/// 3. Child startup: `register_runtime_defs_form` processes the lifted `def`,
///    binding `:h::local-answer = 42` in the child's SymbolTable
/// 4. Child body: references `:h::local-answer` — resolves to 42 → exits 0
#[test]
fn probe_def_at_fn_body_do_prefix_lifts_to_prologue_end_to_end() {
    let src = r#"
        (:wat::core::define
          (:my::launch -> :wat::kernel::Process<wat::core::nil,wat::core::nil>)
          (:wat::kernel::spawn-process
            (:wat::core::fn
              []
              -> :wat::core::nil
              (:wat::core::do
                (:wat::core::def :h::local-answer 42)
                (:wat::core::let
                  [v :h::local-answer]
                  :wat::core::nil)))))

        (:wat::core::define (:user::main -> :wat::core::nil) :wat::core::nil)
    "#;
    let world = freeze_ok(src);
    let (exit_code, stderr) = run_launch(&world);
    assert_eq!(
        exit_code, 0i64,
        "child should exit 0 (def in do-prefix lifted to prologue; :h::local-answer = 42 resolved); stderr:\n{}",
        stderr
    );
}

// ─── Probe 2 — def at expression position emits position error at runtime ─────

/// `def` buried inside a function body (not a do-prefix prelude position)
/// passes check-time after Gap I-B but is rejected at runtime with
/// `DeclarationInExpressionPosition` when the function is called.
///
/// This probes the tightened `":wat::core::def"` arm in `dispatch_keyword_head`
/// (runtime.rs). The function `:my::bad` has body `(:wat::core::def :x 1)`.
/// Startup succeeds (no check-time error); calling `(:my::bad)` emits
/// `DeclarationInExpressionPosition { head: ":wat::core::def", .. }`.
#[test]
fn probe_def_at_expression_position_emits_position_error_at_runtime() {
    let src = r#"
        (:wat::core::define (:my::bad -> :wat::core::nil)
          (:wat::core::def :x 1))

        (:wat::core::define (:user::main -> :wat::core::nil) :wat::core::nil)
    "#;
    // Startup must succeed after Gap I-B (check-time validator arm retired).
    let world = freeze_ok(src);

    // Calling (:my::bad) evaluates the body which hits the tightened def arm.
    let call = wat::parse_one!("(:my::bad)").expect("parse");
    let env = Environment::new();
    let result = eval_in_frozen(&call, &world, &env);
    match result {
        Err(RuntimeError::DeclarationInExpressionPosition(ref head, _)) => {
            assert_eq!(
                head, ":wat::core::def",
                "expected head ':wat::core::def'; got: {}",
                head
            );
        }
        Err(other) => panic!(
            "expected DeclarationInExpressionPosition; got: {:?}",
            other
        ),
        Ok(v) => panic!(
            "expected runtime error; got Ok({:?})",
            v
        ),
    }
}

// ─── Probe 3 — def at top-level still works (regression) ─────────────────────

/// Regression: top-level `def` is unaffected by Gap I-B.
///
/// The validator's def arm was the ONLY source of check-time errors for def
/// in non-top-level positions; top-level def never triggered it (it was in
/// `TopLevel` context, not `NonTopLevel`). Retiring the arm changes nothing
/// for top-level defs.
///
/// `register_runtime_defs_form` still processes top-level defs at freeze time.
/// The bound value is available at runtime.
#[test]
fn probe_def_at_top_level_still_works() {
    let src = r#"
        (:wat::core::def :my-answer 42)

        (:wat::core::define (:my::compute -> :wat::core::i64)
          :my-answer)

        (:wat::core::define (:user::main -> :wat::core::nil) :wat::core::nil)
    "#;
    let world = freeze_ok(src);
    let call = wat::parse_one!("(:my::compute)").expect("parse");
    let env = Environment::new();
    let v = eval_in_frozen(&call, &world, &env).expect("compute should succeed");
    match v {
        Value::i64(n) => assert_eq!(n, 42, "expected 42; got {}", n),
        other => panic!("expected Value::i64(42); got {:?}", other),
    }
}

// ─── Probe 4 — define at expression position still emits error (regression) ───

/// Regression: `define` at expression position is still rejected with
/// `DeclarationInExpressionPosition` (now via the unified variant, carrying
/// `":wat::core::define"` as the head).
///
/// Gap I-B routes `define` through `DeclarationInExpressionPosition` instead
/// of the retired `DefineInExpressionPosition` variant. The behavior is
/// identical from the user's perspective (loud rejection with a clear message),
/// but the variant is now symmetric with `def`'s treatment.
#[test]
fn probe_define_at_expression_position_still_emits_error() {
    let src = r#"
        (:wat::core::define (:my::bad-define -> :wat::core::nil)
          (:wat::core::define (:my::inner -> :wat::core::nil) :wat::core::nil))

        (:wat::core::define (:user::main -> :wat::core::nil) :wat::core::nil)
    "#;
    // Startup succeeds (check-time validator silent for define-at-expression too;
    // define has always been caught at runtime, not check-time).
    let world = freeze_ok(src);

    // Calling (:my::bad-define) evaluates the body which hits the define arm.
    let call = wat::parse_one!("(:my::bad-define)").expect("parse");
    let env = Environment::new();
    let result = eval_in_frozen(&call, &world, &env);
    match result {
        Err(RuntimeError::DeclarationInExpressionPosition(ref head, _)) => {
            assert_eq!(
                head, ":wat::core::define",
                "expected head ':wat::core::define'; got: {}",
                head
            );
        }
        Err(other) => panic!(
            "expected DeclarationInExpressionPosition; got: {:?}",
            other
        ),
        Ok(v) => panic!(
            "expected runtime error; got Ok({:?})",
            v
        ),
    }
}

// ─── Probe 5 — mixed prelude now includes def (all 8 forms lift) ───────────────

/// The mixed-prelude probe from Gap I-A (probe 6) extended to include `def`.
///
/// Gap I-A's probe 6 covered 7 of 8 declaration forms, explicitly excluding
/// `def` because the parent check-time validator blocked it. After Gap I-B,
/// `def` is no longer blocked. This probe adds `def` to the mixed prelude and
/// verifies all 8 declaration forms lift together.
///
/// Prelude order: def → struct → enum → newtype → typealias → define (arm impl) →
///               define-dispatch → defmacro
///
/// The body references the def-bound value (`:h::def-answer = 99`), constructs
/// a struct, references an enum variant, constructs a newtype, calls the dispatch.
#[test]
fn probe_mixed_declaration_prelude_now_includes_def() {
    let src = r#"
        (:wat::core::define
          (:my::launch -> :wat::kernel::Process<wat::core::nil,wat::core::nil>)
          (:wat::kernel::spawn-process
            (:wat::core::fn
              []
              -> :wat::core::nil
              (:wat::core::do
                (:wat::core::def :h::def-answer 99)
                (:wat::core::struct :h::MixPoint8
                  (x :wat::core::i64)
                  (y :wat::core::i64))
                (:wat::core::enum :h::MixDir8
                  :Up
                  :Down)
                (:wat::core::newtype :h::MixAmount8 :wat::core::i64)
                (:wat::core::typealias :h::MixCount8 :wat::core::i64)
                (:wat::core::define
                  (:h::mix-i64-arm8 (v :wat::core::i64) -> :h::MixCount8)
                  v)
                (:wat::core::define-dispatch :h::mix-count8
                  ((:wat::core::i64) :h::mix-i64-arm8))
                (:wat::core::defmacro (:h::mix-id8 (z :AST) -> :AST) `~z)
                (:wat::core::let
                  [_ans :h::def-answer
                   _p   (:h::MixPoint8/new 1 2)
                   _d   :h::MixDir8::Up
                   _a   (:h::MixAmount8/new 10)
                   _n   (:h::mix-count8 7)]
                  :wat::core::nil)))))

        (:wat::core::define (:user::main -> :wat::core::nil) :wat::core::nil)
    "#;
    let world = freeze_ok(src);
    let (exit_code, stderr) = run_launch(&world);
    assert_eq!(
        exit_code, 0i64,
        "child should exit 0 (all 8 declaration forms in mixed prelude lifted to prologue — including def); stderr:\n{}",
        stderr
    );
}
