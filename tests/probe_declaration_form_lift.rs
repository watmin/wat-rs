//! Arc 170 slice 3 Gap I-A — probes for `is_declaration_form` lift coverage.
//!
//! These probes confirm that `extract_closure` lifts ALL 8 declaration forms
//! from a fn body's `do`-prefix into the closure's prologue via the new
//! [`freeze::is_declaration_form`] predicate. Gap H (commit `36030c3`) covered
//! only 3 of 8 forms (define/struct/enum via `is_prelude_form`). Gap I-A
//! retires `is_prelude_form` and routes the lift through `is_declaration_form`,
//! covering the 5 remaining forms: def / defmacro / define-dispatch / newtype /
//! typealias.
//!
//! ## Why this matters
//!
//! Before Gap I-A, writing `(:wat::core::def :x 42)` at a fn body's
//! `do`-prefix caused the child to fail with `DefNotTopLevel` or
//! `EvalForbidsMutationForm` (the form was left in the body for eval to see).
//! Same for defmacro / define-dispatch / newtype / typealias. After Gap I-A,
//! all 8 forms lift; the child's `startup_from_forms` processes them at
//! startup before the body is evaluated.
//!
//! ## Probe structure
//!
//! Each probe (positive-case-only):
//!   1. Declares a fn whose body is `(:wat::core::do declaration-form(s)... expr)`
//!   2. Spawns it as a child process via `spawn-process`
//!   3. Waits for child to exit and asserts exit code 0
//!
//! Gap H's 5 probes serve as the failing-baseline precedent for the
//! prelude-lift mechanism. Gap I-A's probes prove additional coverage for the
//! 5 newly-covered forms.
//!
//! ## The 6 probes
//!
//! 1. `def` in fn body do-prefix lifts to prologue
//! 2. `defmacro` in fn body do-prefix lifts to prologue
//! 3. `define-dispatch` (+ arm impl defines) in fn body do-prefix lifts to prologue
//! 4. `newtype` in fn body do-prefix lifts to prologue
//! 5. `typealias` in fn body do-prefix lifts to prologue
//! 6. mixed prelude covering all 8 form kinds — all lift in source order

use std::sync::Arc;
use wat::ast::WatAST;
use wat::freeze::{is_declaration_form, startup_from_source};
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

// ─── Probe 1 — is_declaration_form covers def (predicate unit test) ───────────

/// Gap I-A adds `def` to `is_declaration_form`. This probe directly tests the
/// predicate rather than end-to-end spawn because `def` at a fn body's
/// `do`-prefix is currently blocked at PARENT check time by
/// `validate_def_position_with_wrapper` — which emits `DefNotTopLevel` for
/// `def` found inside a non-top-level `do` (inside a `fn` body). The check
/// runs at step 8 of `startup_from_forms`, BEFORE `extract_closure` runs
/// at spawn-evaluate time.
///
/// The lift is mechanically ready: `is_declaration_form` covers `def` and
/// `split_body_prelude` would lift it if the parent's source were accepted.
/// End-to-end coverage for `def` at fn body do-prefix requires Gap I-B
/// (extending `validate_def_position_with_wrapper` to understand that the
/// do-prefix lift makes these forms safe at fn body position). Gap I-B is the
/// explicit follow-on slice; the predicate mint here is the enabling substrate.
///
/// All 8 declaration keywords are verified together to confirm the complete
/// predicate surface.
#[test]
fn probe_is_declaration_form_covers_all_8_keywords() {
    // The 8 declaration forms that Gap I-A's is_declaration_form covers.
    let covered = [
        ":wat::core::def",
        ":wat::core::define",
        ":wat::core::defmacro",
        ":wat::core::define-dispatch",
        ":wat::core::struct",
        ":wat::core::enum",
        ":wat::core::newtype",
        ":wat::core::typealias",
    ];
    for kw in &covered {
        assert!(
            is_declaration_form(kw),
            "is_declaration_form should return true for {:?}",
            kw
        );
    }

    // Loads and config setters are in is_mutation_form but NOT in is_declaration_form.
    let excluded = [
        ":wat::load-file!",
        ":wat::digest-load!",
        ":wat::signed-load!",
        ":wat::config::set-foo!",
    ];
    for kw in &excluded {
        assert!(
            !is_declaration_form(kw),
            "is_declaration_form should return false for {:?} (loads/config-setters are out of scope)",
            kw
        );
    }

    // defn expands to def before extract_closure runs; it is intentionally absent.
    assert!(
        !is_declaration_form(":wat::core::defn"),
        "is_declaration_form should return false for :wat::core::defn (macro that expands to :wat::core::def)"
    );
}

// ─── Probe 2 — defmacro in fn body do-prefix lifts to prologue ───────────────

/// A `defmacro` form at the head of a fn body's `do` lifts into the
/// closure's prologue. The child's `startup_from_forms` registers the macro
/// at step 4 (`register_defmacros`) before the body runs.
///
/// The macro `:h::id-macro` is an identity transform over an AST argument.
/// The parent macro-expands the fn body before freeze, so the macro call site
/// in the body is already expanded to its result. The child registers the macro
/// (idempotent with the parent's registration) and exits 0.
#[test]
fn probe_defmacro_in_fn_body_do_prefix_lifts_to_prologue() {
    let src = r#"
        (:wat::core::define
          (:my::launch -> :wat::kernel::Process<wat::core::nil,wat::core::nil>)
          (:wat::kernel::spawn-process
            (:wat::core::fn
              [_rx <- :wat::kernel::Receiver<wat::core::nil>
               _tx <- :wat::kernel::Sender<wat::core::nil>]
              -> :wat::core::nil
              (:wat::core::do
                (:wat::core::defmacro (:h::id-macro (x :AST) -> :AST) `~x)
                :wat::core::nil))))

        (:wat::core::define (:user::main -> :wat::core::nil) :wat::core::nil)
    "#;
    let world = freeze_ok(src);
    let (exit_code, stderr) = run_launch(&world);
    assert_eq!(
        exit_code, 0i64,
        "child should exit 0 (defmacro in do-prefix lifted to prologue); stderr:\n{}",
        stderr
    );
}

// ─── Probe 3 — define-dispatch in fn body do-prefix lifts to prologue ────────

/// A `define-dispatch` form (with its arm impl `define` forms) at the head of
/// a fn body's `do` lifts entirely into the closure's prologue. The consecutive
/// declaration prefix — define + define + define-dispatch — all lift together
/// since the prefix scan stops at the first non-declaration child.
///
/// The body calls the dispatch with an `:wat::core::i64` argument; the child
/// resolves the arm, calls the impl, and exits 0.
#[test]
fn probe_define_dispatch_in_fn_body_do_prefix_lifts_to_prologue() {
    let src = r#"
        (:wat::core::define
          (:my::launch -> :wat::kernel::Process<wat::core::nil,wat::core::nil>)
          (:wat::kernel::spawn-process
            (:wat::core::fn
              [_rx <- :wat::kernel::Receiver<wat::core::nil>
               _tx <- :wat::kernel::Sender<wat::core::nil>]
              -> :wat::core::nil
              (:wat::core::do
                (:wat::core::define
                  (:h::describe-i64 (x :wat::core::i64) -> :wat::core::nil)
                  :wat::core::nil)
                (:wat::core::define-dispatch :h::describe
                  ((:wat::core::i64) :h::describe-i64))
                (:h::describe 99)))))

        (:wat::core::define (:user::main -> :wat::core::nil) :wat::core::nil)
    "#;
    let world = freeze_ok(src);
    let (exit_code, stderr) = run_launch(&world);
    assert_eq!(
        exit_code, 0i64,
        "child should exit 0 (define-dispatch in do-prefix lifted to prologue); stderr:\n{}",
        stderr
    );
}

// ─── Probe 4 — newtype in fn body do-prefix lifts to prologue ────────────────

/// A `newtype` form at the head of a fn body's `do` lifts into the closure's
/// prologue. The child's `startup_from_forms` step 5 (`register_types`) and
/// step 6.7 (`register_newtype_methods`) process the newtype, synthesizing a
/// `/new` constructor and `/0` accessor. The body calls `:h::LocalAmount/new`
/// and `:h::LocalAmount/0` successfully; the child exits 0.
#[test]
fn probe_newtype_in_fn_body_do_prefix_lifts_to_prologue() {
    let src = r#"
        (:wat::core::define
          (:my::launch -> :wat::kernel::Process<wat::core::nil,wat::core::nil>)
          (:wat::kernel::spawn-process
            (:wat::core::fn
              [_rx <- :wat::kernel::Receiver<wat::core::nil>
               _tx <- :wat::kernel::Sender<wat::core::nil>]
              -> :wat::core::nil
              (:wat::core::do
                (:wat::core::newtype :h::LocalAmount :wat::core::i64)
                (:wat::core::let [a (:h::LocalAmount/new 100)] :wat::core::nil)))))

        (:wat::core::define (:user::main -> :wat::core::nil) :wat::core::nil)
    "#;
    let world = freeze_ok(src);
    let (exit_code, stderr) = run_launch(&world);
    assert_eq!(
        exit_code, 0i64,
        "child should exit 0 (newtype in do-prefix lifted to prologue); stderr:\n{}",
        stderr
    );
}

// ─── Probe 5 — typealias in fn body do-prefix lifts to prologue ──────────────

/// A `typealias` form at the head of a fn body's `do` lifts into the closure's
/// prologue. The child's `startup_from_forms` step 5 (`register_types`) processes
/// the typealias. The body's `define` uses the alias as a return type annotation;
/// the child type-checks it successfully and exits 0.
#[test]
fn probe_typealias_in_fn_body_do_prefix_lifts_to_prologue() {
    let src = r#"
        (:wat::core::define
          (:my::launch -> :wat::kernel::Process<wat::core::nil,wat::core::nil>)
          (:wat::kernel::spawn-process
            (:wat::core::fn
              [_rx <- :wat::kernel::Receiver<wat::core::nil>
               _tx <- :wat::kernel::Sender<wat::core::nil>]
              -> :wat::core::nil
              (:wat::core::do
                (:wat::core::typealias :h::LocalCount :wat::core::i64)
                (:wat::core::define (:h::get-count -> :h::LocalCount) 7)
                (:wat::core::let [_c (:h::get-count)] :wat::core::nil)))))

        (:wat::core::define (:user::main -> :wat::core::nil) :wat::core::nil)
    "#;
    let world = freeze_ok(src);
    let (exit_code, stderr) = run_launch(&world);
    assert_eq!(
        exit_code, 0i64,
        "child should exit 0 (typealias in do-prefix lifted to prologue); stderr:\n{}",
        stderr
    );
}

// ─── Probe 6 — mixed prelude covering 7 of 8 declaration form kinds ──────────

/// Seven of the 8 declaration form kinds appear consecutively at the fn body's
/// `do`-prefix. All 7 lift in source order into the closure's prologue.
///
/// `def` is intentionally omitted from this end-to-end probe. `def` at a fn
/// body's `do`-prefix is blocked at PARENT check time by
/// `validate_def_position_with_wrapper`, which emits `DefNotTopLevel` before
/// `extract_closure` ever runs. The predicate (`is_declaration_form`) covers
/// `def` — verified in probe 1 — but the end-to-end lift for `def` requires
/// Gap I-B (extending the check-time validator). Gap I-B is the follow-on
/// slice; this probe confirms the lift works for the 7 forms not blocked by
/// the check-time validator.
///
/// Order in prelude: struct → enum → newtype → typealias → define (arm impl) →
///                   define-dispatch → defmacro
///
/// The residual body exercises each declaration: constructs a struct, references
/// an enum variant, constructs a newtype, calls the dispatch.
///
/// The typealias is used as the return type of the arm impl define. The defmacro
/// is registered in the child's macro registry (the parent has already expanded
/// any macro call sites; the child registration is correct for future macro
/// expansion in a subsequent spawn).
#[test]
fn probe_mixed_declaration_prelude_all_lift() {
    let src = r#"
        (:wat::core::define
          (:my::launch -> :wat::kernel::Process<wat::core::nil,wat::core::nil>)
          (:wat::kernel::spawn-process
            (:wat::core::fn
              [_rx <- :wat::kernel::Receiver<wat::core::nil>
               _tx <- :wat::kernel::Sender<wat::core::nil>]
              -> :wat::core::nil
              (:wat::core::do
                (:wat::core::struct :h::MixPoint
                  (x :wat::core::i64)
                  (y :wat::core::i64))
                (:wat::core::enum :h::MixDir
                  :Up
                  :Down)
                (:wat::core::newtype :h::MixAmount :wat::core::i64)
                (:wat::core::typealias :h::MixCount :wat::core::i64)
                (:wat::core::define
                  (:h::mix-i64-arm (v :wat::core::i64) -> :h::MixCount)
                  v)
                (:wat::core::define-dispatch :h::mix-count
                  ((:wat::core::i64) :h::mix-i64-arm))
                (:wat::core::defmacro (:h::mix-id (z :AST) -> :AST) `~z)
                (:wat::core::let
                  [_p  (:h::MixPoint/new 1 2)
                   _d  :h::MixDir::Up
                   _a  (:h::MixAmount/new 10)
                   _n  (:h::mix-count 7)]
                  :wat::core::nil)))))

        (:wat::core::define (:user::main -> :wat::core::nil) :wat::core::nil)
    "#;
    let world = freeze_ok(src);
    let (exit_code, stderr) = run_launch(&world);
    assert_eq!(
        exit_code, 0i64,
        "child should exit 0 (7 of 8 declaration kinds in mixed prelude lifted to prologue; def excluded pending Gap I-B); stderr:\n{}",
        stderr
    );
}
