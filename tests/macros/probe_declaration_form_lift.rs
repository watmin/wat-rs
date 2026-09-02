//! Arc 170 slice 3 Gap I-A — probes for `is_liftable_declaration_head` lift coverage.
//!
//! These probes confirm that `extract_closure` lifts ALL 8 declaration forms
//! from a fn body's `do`-prefix into the closure's prologue via the new
//! [`freeze::is_liftable_declaration_head`] predicate. Gap H (commit `36030c3`) covered
//! only 3 of 8 forms (define/struct/enum via `is_prelude_form`). Gap I-A
//! retires `is_prelude_form` and routes the lift through `is_liftable_declaration_head`,
//! covering the 5 remaining forms: def / defmacro / defclause / newtype /
//! typealias.
//!
//! Wat source lives in the co-located fixture: probe_declaration_form_lift.wat
//! (slurped via startup_beside(file!())). Four named launch functions
//! (:my::launch-defmacro, :my::launch-newtype, :my::launch-typealias,
//! :my::launch-mixed) are called by name per test.

use wat::freeze::{is_liftable_declaration_head, startup_beside};
use wat::runtime::Value;

// ─── helpers ────────────────────────────────────────────────────────────────

/// Apply the named launch fn and return the i64 the child derived from the
/// declaration under test and sent back over the peer wire.
///
/// Arc 278 IPC de-prime — the old form field-poked the concrete `Process`
/// struct (`fields[2]` stderr, `fields[3]` handle → exit code), an observation
/// model the opaque `Process'` peer has no analog for. The peer model observes
/// the same property more strongly: a registration failure surfaces as a
/// `Lost` cause carrying the child's real reason rather than a bare non-zero
/// exit code.
fn run_named_launch(world: &wat::freeze::FrozenWorld, name: &str) -> i64 {
    let launcher = world.symbols().get(name).unwrap_or_else(|| panic!("{name} defined"));
    let result = wat::runtime::apply_function(
        launcher.clone(),
        Vec::new(),
        world.symbols(),
        wat::rust_caller_span!(),
    )
    .unwrap_or_else(|e| panic!("{name} runs (spawn-program' + recv'); got: {e:?}"));
    match result {
        Value::i64(n) => n,
        other => panic!("expected i64 from {name}; got {other:?}"),
    }
}

// ─── Probe 1 — is_liftable_declaration_head covers def (predicate unit test) ───────────

/// Gap I-A adds `def` to `is_liftable_declaration_head`. This probe directly tests the
/// predicate rather than end-to-end spawn because `def` at a fn body's
/// `do`-prefix is currently blocked at PARENT check time by
/// `validate_def_position_with_wrapper` — which emits `DefNotTopLevel` for
/// `def` found inside a non-top-level `do` (inside a `fn` body). The check
/// runs at step 8 of `startup_from_forms`, BEFORE `extract_closure` runs
/// at spawn-evaluate time.
///
/// The lift is mechanically ready: `is_liftable_declaration_head` covers `def` and
/// `split_body_prelude` would lift it if the parent's source were accepted.
/// End-to-end coverage for `def` at fn body do-prefix requires Gap I-B
/// (extending `validate_def_position_with_wrapper` to understand that the
/// do-prefix lift makes these forms safe at fn body position). Gap I-B is the
/// explicit follow-on slice; the predicate mint here is the enabling substrate.
///
/// ⛔ CORRECTED 2026-09-02 (arc 255) — this probe asserted SEVEN keywords while the predicate
/// held NINE. `:wat::core::structtype` and `:wat::core::defsurface` were never in the list, so
/// the probe's own name ("all_7") was a claim about a population it had not measured, and two
/// live arms had no coverage at all. Stone 241.13 retired `define-dispatch`; Stone 241.16
/// retired `define`; `defalias` (Stone 241.12) took a slot — and the two additions were never
/// mirrored here. The list below is now read FROM the `matches!` arms, all nine.
///
/// ⚠ This is still a hand-list, and it can still rot the same way. Arc 255's 1a-β-i mints the
/// gate that cannot: a bidirectional meter over the predicate's ACTUAL domain, asserting each
/// name is registered with a `SpecialFormRole::Declare` impl. When that lands, this probe's
/// membership half is subsumed by a check nobody has to remember to update.
///
/// ⛔ CORRECTED AGAIN 2026-09-02 (arc 255, Stone 1a-β-i-b) — `:wat::core::defstruct` LEFT the
/// domain (nine → eight) and this probe's name moves with it. `defstruct` was never wrong to be
/// listed here — it genuinely used to be one of the `matches!` arms — but it could never actually
/// be exercised: `defstruct` is a stdlib `defmacro` (`wat/core.wat:2030`) that `expand_all`
/// rewrites to `structtype` before this predicate's only caller
/// (`closure_extract::split_body_prelude`, itself POST-expansion) ever runs, so no real program
/// could reach this predicate with a literal `defstruct` head. The arm was removed from
/// `is_liftable_declaration_head` itself (`src/freeze.rs`); `defstruct` now belongs in `excluded`,
/// below, alongside the other heads the predicate correctly refuses.
#[test]
fn probe_liftable_declaration_head_covers_all_eight_keywords() {
    // All EIGHT arms of `is_liftable_declaration_head`, transcribed from the `matches!` itself.
    let covered = [
        ":wat::core::def",
        ":wat::core::defmacro",
        // Arc 293.2-parity — the low-level primitive `defstruct` (a macro) expands to.
        ":wat::core::structtype",
        ":wat::core::defenum",
        ":wat::core::newtype",
        ":wat::core::typealias",
        // Stone 241.12 — defalias is a declaration form.
        ":wat::core::defalias",
        // Arc 293 — a surface declaration is liftable like any other type declaration.
        ":wat::core::defsurface",
    ];
    for kw in &covered {
        assert!(
            is_liftable_declaration_head(kw),
            "is_liftable_declaration_head should return true for {:?}",
            kw
        );
    }

    // Loads and config setters are in is_mutation_form but NOT in is_liftable_declaration_head.
    // `:wat::core::defstruct` joined this list Stone 255.1a-β-i-b — it is a macro `expand_all`
    // always rewrites to `structtype` before this predicate's caller runs, so the arm was dead
    // and removed; this assertion is the un-sweepable pin against it silently coming back.
    let excluded = [
        ":wat::load-file!",
        ":wat::digest-load!",
        ":wat::signed-load!",
        ":wat::config::set-foo!",
        ":wat::core::defstruct",
    ];
    for kw in &excluded {
        assert!(
            !is_liftable_declaration_head(kw),
            "is_liftable_declaration_head should return false for {:?} (loads/config-setters are out of scope; defstruct is a macro, dead post-expansion)",
            kw
        );
    }

    // defn expands to def before extract_closure runs; it is intentionally absent.
    assert!(
        !is_liftable_declaration_head(":wat::core::defn"),
        "is_liftable_declaration_head should return false for :wat::core::defn (macro that expands to :wat::core::def)"
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
    let world = startup_beside(file!()).expect("freeze should succeed");
    assert_eq!(
        run_named_launch(&world, ":my::launch-defmacro"),
        5,
        "child should expand (:h::id-macro 5) → 5 (defmacro in do-prefix registered AND expanded)"
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
    let world = startup_beside(file!()).expect("freeze should succeed");
    assert_eq!(
        run_named_launch(&world, ":my::launch-newtype"),
        100,
        "child should read 100 back via :h::LocalAmount/0 (newtype registered, constructor + accessor synthesized)"
    );
}

// ─── Probe 5 — typealias in fn body do-prefix lifts to prologue ──────────────

/// A `typealias` form at the head of a fn body's `do` lifts into the closure's
/// prologue. The child's `startup_from_forms` step 5 (`register_types`) processes
/// the typealias. The body's `define` uses the alias as a return type annotation;
/// the child type-checks it successfully and exits 0.
#[test]
fn probe_typealias_in_fn_body_do_prefix_lifts_to_prologue() {
    let world = startup_beside(file!()).expect("freeze should succeed");
    assert_eq!(
        run_named_launch(&world, ":my::launch-typealias"),
        7,
        "child should return 7 through the aliased return type (typealias + its consumer fn registered)"
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
/// `extract_closure` ever runs. The predicate (`is_liftable_declaration_head`) covers
/// `def` — verified in probe 1 — but the end-to-end lift for `def` requires
/// Gap I-B (extending the check-time validator). Gap I-B is the follow-on
/// slice; this probe confirms the lift works for the 6 forms not blocked by
/// the check-time validator.
#[test]
fn probe_mixed_declaration_prelude_all_lift() {
    let world = startup_beside(file!()).expect("freeze should succeed");
    assert_eq!(
        run_named_launch(&world, ":my::launch-mixed"),
        30,
        "child should fold every declaration into one value: (1+2 struct) + (10 enum) + (10 newtype) + (7 typealias-fn via macro) \
         — 6 of 7 kinds lifted; def excluded pending Gap I-B"
    );
}
