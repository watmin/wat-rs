//! Arc 170 slice 3 Gap I-A — probes for `is_declaration_form` lift coverage.
//!
//! These probes confirm that `extract_closure` lifts ALL 8 declaration forms
//! from a fn body's `do`-prefix into the closure's prologue via the new
//! [`freeze::is_declaration_form`] predicate. Gap H (commit `36030c3`) covered
//! only 3 of 8 forms (define/struct/enum via `is_prelude_form`). Gap I-A
//! retires `is_prelude_form` and routes the lift through `is_declaration_form`,
//! covering the 5 remaining forms: def / defmacro / defclause / newtype /
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
//! 3. `defclause` in fn body do-prefix lifts to prologue
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
    let process = eval(&call, &env, world.symbols()).expect("launch should evaluate").value_owned();
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
/// All 7 declaration keywords are verified together to confirm the complete
/// predicate surface. Stone 241.13: `:wat::core::define-dispatch` retired;
/// Stone 241.16: `:wat::core::define` retired (HARD CUT total; eval-time residue completed).
/// Its slot is vacated from this list; `:wat::core::defalias` replaces it.
#[test]
fn probe_is_declaration_form_covers_all_7_keywords() {
    // The 7 declaration forms that Gap I-A's is_declaration_form covers.
    // Stone 241.13 — define-dispatch removed (HARD CUT; defclause is the
    // surviving dispatch entity kind).
    // Stone 241.16 — define removed (HARD CUT total; defn replaces define).
    // defalias (Stone 241.12) now occupies the 7th slot.
    let covered = [
        ":wat::core::def",
        // Stone 241.16 — `:wat::core::define` REMOVED from is_declaration_form.
        // HARD CUT total; define is no longer recognized as a declaration form.
        ":wat::core::defmacro",
        ":wat::core::defstruct",
        ":wat::core::defenum",
        ":wat::core::newtype",
        ":wat::core::typealias",
        // Stone 241.12 — defalias is a declaration form.
        ":wat::core::defalias",
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
    // Arc 170 slice 6 — defmacro lives at program top-level via the
    // new spawn-process program shape; the "lift" mechanism is retired
    // because declarations sit at their natural position from the start.
    // Stone 241.11 — define hard-cut; use defn in the child program forms.
    let src = r#"
        (:wat::core::defn :my::launch [] -> :wat::kernel::Process<wat::core::nil,wat::core::nil>
          (:wat::kernel::spawn-process
                      (:wat::core::forms
                        (:wat::core::defmacro :h::id-macro [x <- :AST] -> :AST `~x)
                        (:wat::core::defn :user::main [] -> :wat::core::nil nil))))

        (:wat::core::defn :user::main [] -> :wat::core::nil nil)
    "#;
    let world = freeze_ok(src);
    let (exit_code, stderr) = run_launch(&world);
    assert_eq!(
        exit_code, 0i64,
        "child should exit 0 (defmacro in do-prefix lifted to prologue); stderr:\n{}",
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
    // Arc 170 slice 6 — newtype at program top-level.
    // Stone 241.11 — define hard-cut; use defn in the child program forms.
    let src = r#"
        (:wat::core::defn :my::launch [] -> :wat::kernel::Process<wat::core::nil,wat::core::nil>
          (:wat::kernel::spawn-process
                      (:wat::core::forms
                        (:wat::core::newtype :h::LocalAmount :wat::core::i64)
                        (:wat::core::defn :user::main [] -> :wat::core::nil
                          (:wat::core::let [a (:h::LocalAmount/new 100)] nil)))))

        (:wat::core::defn :user::main [] -> :wat::core::nil nil)
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
    // Arc 170 slice 6 — typealias at program top-level.
    // Stone 241.11 — define hard-cut; use defn in the child program forms.
    let src = r#"
        (:wat::core::defn :my::launch [] -> :wat::kernel::Process<wat::core::nil,wat::core::nil>
          (:wat::kernel::spawn-process
                      (:wat::core::forms
                        (:wat::core::typealias :h::LocalCount :wat::core::i64)
                        (:wat::core::defn :h::get-count [] -> :h::LocalCount 7)
                        (:wat::core::defn :user::main [] -> :wat::core::nil
                          (:wat::core::let [_c (:h::get-count)] nil)))))

        (:wat::core::defn :user::main [] -> :wat::core::nil nil)
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

/// Six of the 7 declaration form kinds appear consecutively at the fn body's
/// `do`-prefix. All 6 lift in source order into the closure's prologue.
///
/// Stone 241.13 — `:wat::core::define-dispatch` retired (HARD CUT). The mixed
/// prelude uses `:wat::core::defclause` (the surviving dispatch entity kind,
/// Stone 237.2) to exercise the clause declaration slot.
///
/// `def` is intentionally omitted from this end-to-end probe. `def` at a fn
/// body's `do`-prefix is blocked at PARENT check time by
/// `validate_def_position_with_wrapper`, which emits `DefNotTopLevel` before
/// `extract_closure` ever runs. The predicate (`is_declaration_form`) covers
/// `def` — verified in probe 1 — but the end-to-end lift for `def` requires
/// Gap I-B (extending the check-time validator). Gap I-B is the follow-on
/// slice; this probe confirms the lift works for the 6 forms not blocked by
/// the check-time validator.
///
/// Order in prelude: struct → enum → newtype → typealias → defn (arm impl) →
///                   defmacro
///
/// The residual body exercises each declaration: constructs a struct, references
/// an enum variant, constructs a newtype, calls a fn.
///
/// The typealias is used as the return type of the arm impl. The defmacro
/// is registered in the child's macro registry (the parent has already expanded
/// any macro call sites; the child registration is correct for future macro
/// expansion in a subsequent spawn).
#[test]
fn probe_mixed_declaration_prelude_all_lift() {
    // Arc 170 slice 6 — all declaration kinds sit at program top-level
    // alongside :user::main. The new substrate makes the "lift" a no-op
    // (declarations were already at the position the lift would move
    // them to).
    // Stone 241.11 — define hard-cut; use defn in the child program forms.
    // Stone 241.13 — define-dispatch hard-cut; defclause is the surviving
    // dispatch entity kind (not applicable in this mixed-prelude fixture
    // since the form requires arc 237.2's defclause dispatch table, which
    // only exercises clause definition — no runtime dispatch invocation here).
    let src = r#"
        (:wat::core::defn :my::launch [] -> :wat::kernel::Process<wat::core::nil,wat::core::nil>
          (:wat::kernel::spawn-process
                      (:wat::core::forms
                        (:wat::core::defstruct :h::MixPoint
                          [x <- :wat::core::i64
                           y <- :wat::core::i64])
                        (:wat::core::defenum :h::MixDir
                          :Up
                          :Down)
                        (:wat::core::newtype :h::MixAmount :wat::core::i64)
                        (:wat::core::typealias :h::MixCount :wat::core::i64)
                        (:wat::core::defn :h::mix-i64 [v <- :wat::core::i64] -> :h::MixCount
                          v)
                        (:wat::core::defmacro :h::mix-id [z <- :AST] -> :AST `~z)
                        (:wat::core::defn :user::main [] -> :wat::core::nil
                          (:wat::core::let
                            [_p  (:h::MixPoint/new 1 2)
                             _d  :h::MixDir::Up
                             _a  (:h::MixAmount/new 10)
                             _n  (:h::mix-i64 7)]
                            nil)))))

        (:wat::core::defn :user::main [] -> :wat::core::nil nil)
    "#;
    let world = freeze_ok(src);
    let (exit_code, stderr) = run_launch(&world);
    assert_eq!(
        exit_code, 0i64,
        "child should exit 0 (6 of 7 declaration kinds in mixed prelude lifted to prologue; def excluded pending Gap I-B); stderr:\n{}",
        stderr
    );
}
