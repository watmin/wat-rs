//! Arc 170 slice 3 Gap I-A — probes for the fn-body-do-prefix declaration LIFT
//! (`closure_extract::split_body_prelude`).
//!
//! These probes confirm that `extract_closure` lifts declaration forms from a fn body's
//! `do`-prefix into the closure's prologue. Gap H (commit `36030c3`) covered only 3 of 8
//! forms (define/struct/enum via `is_prelude_form`). Gap I-A retired `is_prelude_form` and
//! routed the lift through the (now-DELETED) `freeze::is_liftable_declaration_head`
//! hand-list, covering the 5 remaining forms: def / defmacro / defclause / newtype /
//! typealias.
//!
//! ⛔ Arc 255 Stone 1a-β-ii — `is_liftable_declaration_head` and its bidirectional meter
//! (`liftable_declaration_head_missing_and_foreign`, `src/intrinsic/mod.rs`) are DELETED, the
//! campaign's first hand-list kill: `split_body_prelude` now asks
//! `crate::intrinsic::is_declare_role_head` (a registry query — does the head's row name a
//! `role = declare` impl?) instead of a `matches!` hand-list. This file's former "probe 1"
//! (`probe_liftable_declaration_head_covers_all_eight_keywords`) unit-tested the deleted
//! predicate's MEMBERSHIP directly — that subject no longer exists, so the probe is
//! retired with it, not rewritten against the registry: the registry's own membership is
//! covered where it is asserted, `src/intrinsic/mod.rs`'s registration + doc-string tests,
//! not a second copy here. The probes below, which exercise the LIFT end-to-end (spawn a
//! child, verify the declaration registered and its effect is observable), are UNCHANGED —
//! the lift itself, and the population it admits, did not change: every name
//! `is_liftable_declaration_head` used to cover still carries a `role = declare` row today.
//!
//! Wat source lives in the co-located fixture: probe_declaration_form_lift.wat
//! (slurped via startup_beside(file!())). Four named launch functions
//! (:my::launch-defmacro, :my::launch-newtype, :my::launch-typealias,
//! :my::launch-mixed) are called by name per test.

use wat::freeze::startup_beside;
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
/// `extract_closure` ever runs. The lift itself is mechanically ready for `def`
/// (arc 255 Stone 1a-β-ii registered it with a `role = declare` impl, so
/// `crate::intrinsic::is_declare_role_head(":wat::core::def")` answers `true`), but the
/// end-to-end lift for `def` requires Gap I-B (extending the check-time validator).
/// Gap I-B is the follow-on slice; this probe confirms the lift works for the 6 forms not
/// blocked by the check-time validator.
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

// ─── The NEGATIVE direction — restored, at the behavioural layer ──────────────

/// ⛔ Arc 255 Stone 1a-β-ii — THE ONLY STANDING ASSERTION THAT THE LIFT BOUNDARY
/// DISCRIMINATES, and it exists because deleting the old one left a hole.
///
/// `split_body_prelude` used to consult `freeze::is_liftable_declaration_head`, a nine-name
/// hand-list. It now asks the registry (`crate::intrinsic::is_declare_role_head`). This stone's
/// retired probe-1 was the only place that asserted the predicate answers **false** for anything
/// — loads, config setters, `defn` — and retiring it with its subject removed the whole negative
/// half. `[[feedback_retiring_a_name_disarms_every_bare_is_err_test]]`
///
/// ★ Measured, before this test existed: stubbing `is_declare_role_head` to `true` for every
/// input left the ENTIRE floor green at 5122/5122. Every sibling probe above checks only that
/// declarations DO lift, so an accessor that lifts everything passes all of them. The trade the
/// stone would otherwise have made is a hand-list for an untested boundary.
///
/// ⚠ AND THE FIRST TWO VERSIONS OF THIS TEST DID NOT CATCH IT EITHER. The fixture's
/// non-declaration first form was the literal `5`, which has no keyword head — so
/// `split_body_prelude`'s scan stopped before the predicate was consulted at all, and the
/// stub still passed. Only a first form that is a CALL (`(:wat::i64::+ 2 3)`) reaches the
/// predicate. Re-aimed, the stub now reports 3 where 1 is correct.
/// `[[feedback_a_probe_answers_the_question_you_asked_not_the_one_you_meant]]`
///
/// The check is end-to-end and deliberately NOT a unit test of the predicate: `fn-forms` reifies
/// a closure through `extract_closure` → `split_body_prelude`, so this survives any future
/// re-spelling of the predicate, which is exactly what killed its ancestor.
#[test]
fn probe_non_declaration_prefix_does_not_lift() {
    let world = startup_beside(file!()).expect("freeze should succeed");
    assert_eq!(
        run_named_launch(&world, ":my::launch-lift-count"),
        2,
        "control: a fn body whose FIRST form IS a declaration must still lift — without this \
         the negative assertion below could pass by the lift being broken outright"
    );
    assert_eq!(
        run_named_launch(&world, ":my::launch-no-lift"),
        1,
        "a fn body whose FIRST form is not a declaration must lift nothing: the prelude scan \
         stops at the first non-declaration child, so the body stays whole. A `true`-for-\
         everything predicate reports a lift here and this is the assertion that catches it."
    );
}
