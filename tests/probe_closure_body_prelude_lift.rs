//! Arc 170 slice 3 Gap H — probes for closure-extraction prelude-lift.
//!
//! These probes confirm that `extract_closure` lifts leading
//! `define`/`struct`/`enum` forms from a fn body's `do`-prefix INTO the
//! closure's prologue, so that the child's `startup_from_forms` registers
//! them (step 6) before the body is evaluated. Without the lift, the child
//! exits non-zero because `eval_do_tail` encounters `define` at expression
//! position and returns `DefineInExpressionPosition`.
//!
//! ## Why this matters
//!
//! Gap G (commit `021884a`) blocked Path E of `deftest-hermetic` because
//! prelude `define` forms inside a fn body's `do` cannot be evaluated at
//! child runtime. Gap H resolves that by lifting them UPSTREAM (before eval
//! ever sees them), preserving the single mental model "define = top-level
//! registration."
//!
//! ## Probe structure
//!
//! Each probe:
//!   1. Declares a fn whose body is `(:wat::core::do prelude-forms... expr)`
//!   2. Spawns it as a child process via `spawn-process`
//!   3. Waits for child to exit and asserts exit code 0
//!
//! Before Gap H: all probes fail (child exits non-zero, `DefineInExpressionPosition`).
//! After Gap H: all probes pass (lifted forms registered via prologue startup).
//!
//! ## The 5 probes
//!
//! 1. `define` in fn body do-prefix lifts to prologue
//! 2. `struct` in fn body do-prefix lifts to prologue
//! 3. `enum` in fn body do-prefix lifts to prologue
//! 4. mixed prelude (struct + enum + define) all lift in order
//! 5. prefix-termination semantics: only LEADING prelude forms lift;
//!    a prelude form AFTER the first expression does NOT lift (it stays in
//!    the body and would still trigger `DefineInExpressionPosition` if
//!    reached — but the prefix-termination rule correctly limits the lift
//!    to the do's leading run)

use std::sync::Arc;
use wat::ast::WatAST;
use wat::freeze::startup_from_source;
use wat::load::InMemoryLoader;
use wat::runtime::{eval, Environment, ProgramHandleInner};

// ─── helpers ────────────────────────────────────────────────────────────────

fn freeze_ok(src: &str) -> wat::freeze::FrozenWorld {
    match startup_from_source(src, None, Arc::new(InMemoryLoader::new())) {
        Ok(w) => w,
        Err(e) => panic!("freeze should succeed; got: {}", e),
    }
}

/// Drain the stderr field (index 2) of a Process Struct value.
fn drain_stderr(process: &wat::runtime::Value) -> String {
    match process {
        wat::runtime::Value::Struct(s) if s.type_name == ":wat::kernel::Process" => {
            match &s.fields[2] {
                wat::runtime::Value::io__IOReader(rdr) => {
                    let mut all = String::new();
                    while let Ok(Some(line)) = rdr.read_line(wat::span::Span::unknown()) {
                        all.push_str(&line);
                    }
                    all
                }
                _ => "<stderr field not IOReader>".into(),
            }
        }
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
    let process = eval(&call, &env, world.symbols()).expect("launch should evaluate");
    let handle = match &process {
        wat::runtime::Value::Struct(s) if s.type_name == ":wat::kernel::Process" => {
            match &s.fields[3] {
                wat::runtime::Value::wat__kernel__ProgramHandle(h) => h.clone(),
                other => panic!("expected ProgramHandle field at index 3; got {:?}", other),
            }
        }
        other => panic!("expected Process Struct from launch; got {:?}", other),
    };
    let exit_code: i64 = match handle.as_ref() {
        ProgramHandleInner::Forked(child) => child.wait_or_cached(),
        other => panic!("expected Forked handle; got {:?}", other),
    };
    let stderr = drain_stderr(&process);
    (exit_code, stderr)
}

// ─── Probe 1 — define in fn body do-prefix lifts to prologue ─────────────────

/// A `define` form at the head of a fn body's `do` must lift into the
/// closure's prologue so the child's `startup_from_forms` registers it at
/// step 6 (before the body runs). Without the lift the child exits non-zero
/// (`DefineInExpressionPosition`). After Gap H the child exits 0.
///
/// The child fn's body is:
///   `(:wat::core::do
///      (:wat::core::define (:h::helper -> :wat::core::i64) 42)
///      (:wat::core::let [v (:h::helper)] :wat::core::nil))`
///
/// `:h::helper` is declared inside the fn body; the lift makes it available
/// in the child's SymbolTable so the `let`-bound call succeeds.
#[test]
fn probe_define_in_fn_body_do_prefix_lifts_to_prologue() {
    // Arc 170 slice 6 — under the new spawn-process program shape, the
    // prelude declarations sit at the program's TOP LEVEL alongside
    // :user::main. The "lift" mechanism that pre-slice-6 moved
    // declarations from the fn body's do-prefix to the closure prologue
    // is retired; the natural shape replaces it (declarations live at
    // their natural top-level position from the start).
    let src = r#"
        (:wat::core::define
          (:my::launch -> :wat::kernel::Process<wat::core::nil,wat::core::nil>)
          (:wat::kernel::spawn-process
            (:wat::core::forms
              (:wat::core::define (:h::helper -> :wat::core::i64) 42)
              (:wat::core::define (:user::main -> :wat::core::nil)
                (:wat::core::let [v (:h::helper)] :wat::core::nil)))))

        (:wat::core::define (:user::main -> :wat::core::nil) :wat::core::nil)
    "#;
    let world = freeze_ok(src);
    let (exit_code, stderr) = run_launch(&world);
    assert_eq!(
        exit_code, 0i64,
        "child should exit 0 (define in do-prefix lifted to prologue); stderr:\n{}",
        stderr
    );
}

// ─── Probe 2 — struct in fn body do-prefix lifts to prologue ─────────────────

/// A `struct` declaration at the head of a fn body's `do` lifts into the
/// prologue. The child's `startup_from_forms` step 5 registers the struct
/// into its TypeEnv; step 6a synthesizes the `/new` constructor and field
/// accessors. The body then calls `(:h::LocalPoint/new 3 4)` successfully.
#[test]
fn probe_struct_in_fn_body_do_prefix_lifts_to_prologue() {
    // Arc 170 slice 6 — struct sits at program top-level via spawn-process's
    // program shape (no lift required; the natural shape supersedes it).
    let src = r#"
        (:wat::core::define
          (:my::launch -> :wat::kernel::Process<wat::core::nil,wat::core::nil>)
          (:wat::kernel::spawn-process
            (:wat::core::forms
              (:wat::core::struct :h::LocalPoint
                (x :wat::core::i64)
                (y :wat::core::i64))
              (:wat::core::define (:user::main -> :wat::core::nil)
                (:wat::core::let [p (:h::LocalPoint/new 3 4)] :wat::core::nil)))))

        (:wat::core::define (:user::main -> :wat::core::nil) :wat::core::nil)
    "#;
    let world = freeze_ok(src);
    let (exit_code, stderr) = run_launch(&world);
    assert_eq!(
        exit_code, 0i64,
        "child should exit 0 (struct in do-prefix lifted to prologue); stderr:\n{}",
        stderr
    );
}

// ─── Probe 3 — enum in fn body do-prefix lifts to prologue ───────────────────

/// An `enum` declaration at the head of a fn body's `do` lifts into the
/// prologue. The child's step 5 registers the enum; step 6.5 synthesizes
/// variant constructors. The body then references `:h::LocalDir::North`
/// successfully.
#[test]
fn probe_enum_in_fn_body_do_prefix_lifts_to_prologue() {
    // Arc 170 slice 6 — enum at program top-level.
    let src = r#"
        (:wat::core::define
          (:my::launch -> :wat::kernel::Process<wat::core::nil,wat::core::nil>)
          (:wat::kernel::spawn-process
            (:wat::core::forms
              (:wat::core::enum :h::LocalDir
                :North
                :South)
              (:wat::core::define (:user::main -> :wat::core::nil)
                (:wat::core::let [d :h::LocalDir::North] :wat::core::nil)))))

        (:wat::core::define (:user::main -> :wat::core::nil) :wat::core::nil)
    "#;
    let world = freeze_ok(src);
    let (exit_code, stderr) = run_launch(&world);
    assert_eq!(
        exit_code, 0i64,
        "child should exit 0 (enum in do-prefix lifted to prologue); stderr:\n{}",
        stderr
    );
}

// ─── Probe 4 — mixed prelude (struct + enum + define) all lift in order ──────

/// A mixed prelude — struct, then enum, then define — at the head of a fn
/// body's `do`. All three lift into the prologue in order. The body uses
/// all three: constructs a LocalItem struct, references a LocalKind enum
/// variant, and calls a local helper define.
#[test]
fn probe_mixed_prelude_lift() {
    // Arc 170 slice 6 — mixed prelude (struct + enum + define) all live
    // at program top-level via the new spawn-process program shape.
    let src = r#"
        (:wat::core::define
          (:my::launch -> :wat::kernel::Process<wat::core::nil,wat::core::nil>)
          (:wat::kernel::spawn-process
            (:wat::core::forms
              (:wat::core::struct :h::LocalItem
                (value :wat::core::i64))
              (:wat::core::enum :h::LocalKind
                :A
                :B)
              (:wat::core::define (:h::make-item -> :h::LocalItem)
                (:h::LocalItem/new 99))
              (:wat::core::define (:user::main -> :wat::core::nil)
                (:wat::core::let
                  [item (:h::make-item)
                   kind :h::LocalKind::A]
                  :wat::core::nil)))))

        (:wat::core::define (:user::main -> :wat::core::nil) :wat::core::nil)
    "#;
    let world = freeze_ok(src);
    let (exit_code, stderr) = run_launch(&world);
    assert_eq!(
        exit_code, 0i64,
        "child should exit 0 (mixed prelude: struct+enum+define all lifted); stderr:\n{}",
        stderr
    );
}

// ─── Probe 5 — prefix-termination semantics ──────────────────────────────────

/// Only LEADING prelude forms (consecutive define/struct/enum at the do's
/// prefix, before any expression) lift into the prologue.
///
/// The body's do has:
///   - `(:wat::core::define (:h::early-helper -> :wat::core::i64) 1)` (LEADING — lifts)
///   - `(:wat::core::let [_x (:h::early-helper)] :wat::core::nil)` (expression — stops prefix)
///   - `(:wat::core::define (:h::late-helper -> :wat::core::i64) 2)` (AFTER expression — stays)
///
/// The late define is NOT lifted. It stays in the residual do body and would
/// hit `DefineInExpressionPosition` if the body reaches that form. However,
/// the let-form before it returns `:wat::core::nil`, making the body a two-form
/// do whose final form is the late define. At eval time the late define IS
/// reached (it's the last form in the do, not after the final form).
///
/// To keep probe 5 strictly about prefix-termination semantics without
/// triggering the error from the late define, we structure the body so the
/// late define is unreachable: the let returns nil and there are no more
/// forms after it. We verify that the early define DID lift (child exits 0).
///
/// This probe confirms: split_prelude_prefix stops at the first non-prelude
/// form. The lift is prefix-only, not full-body-define-hoisting.
#[test]
fn probe_prelude_prefix_terminates_at_first_expression() {
    // Arc 170 slice 6 — the prefix-termination semantics retire under
    // the new substrate: declarations sit at program top-level naturally
    // and there is no "prefix" concept. The probe migrates to the
    // top-level shape; the early define is registered as a normal
    // top-level form alongside :user::main.
    let src = r#"
        (:wat::core::define
          (:my::launch -> :wat::kernel::Process<wat::core::nil,wat::core::nil>)
          (:wat::kernel::spawn-process
            (:wat::core::forms
              (:wat::core::define (:h::counted-helper -> :wat::core::i64) 7)
              (:wat::core::define (:user::main -> :wat::core::nil)
                (:wat::core::let [_v (:h::counted-helper)] :wat::core::nil)))))

        (:wat::core::define (:user::main -> :wat::core::nil) :wat::core::nil)
    "#;
    let world = freeze_ok(src);
    let (exit_code, stderr) = run_launch(&world);
    assert_eq!(
        exit_code, 0i64,
        "child should exit 0 (prefix-terminating define lifted; expression after is nil); stderr:\n{}",
        stderr
    );
}
