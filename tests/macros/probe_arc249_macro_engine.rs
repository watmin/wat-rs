//! FM-2-bis probe for Arc 249 Stone 249.2b — the total-pure macro-eval engine.
//!
//! ROW STATUS:
//!   - A REGRESSION (GREEN): a PURE computed-unquote still expands identically.
//!   - B MINT: an IMPURE computed-unquote must be REJECTED.
//!   - C MINT: a macro body that is an `if` (not a bare quasiquote) must expand.
//!   - D MINT: a fold-shaped program body must expand.
//!   - E HYGIENE BOUND: a program body with literal binder in quasiquote is REFUSED.
//!   - F MIRROR WALL (arc 255 Stone expand-only-the-mirror-wall): an `ExpandOnly` head
//!     found OUTSIDE a macro body is REFUSED; found INSIDE one, unchanged.
//!
//! Run: cargo nextest run --release -E 'binary(macros)' -F probe_arc249_macro_engine

use wat::freeze::{startup_from_file, StartupError};
use wat::macros::{MacroError, MacroErrorKind};
use wat::runtime::{apply_function, Value};

// just-eval (rubric): each `*.wat` fixture defines a zero-arg `:user::compute`; fetch it from
// the frozen world and `apply_function` it — no inline wat driver. (Path-based rather than
// `call_beside_value` because this probe drives several distinct co-located fixtures from one `.rs`.)
fn compute_from_file(fixture: &str) -> Value {
    let world = startup_from_file(fixture).expect("startup");
    let func = world
        .symbols()
        .get(":user::compute")
        .unwrap_or_else(|| panic!("no :user::compute in {fixture:?}"))
        .clone();
    apply_function(func, vec![], world.symbols(), wat::rust_caller_span!()).expect("eval")
}

// ═══════════════════════════════════════════════════════════════════════════
// A — REGRESSION
// ═══════════════════════════════════════════════════════════════════════════
#[test]
fn regression_pure_computed_unquote_preserved() {
    let result = compute_from_file("tests/macros/probe_arc249_macro_engine_regression.wat");
    assert_eq!(result, Value::bool(true));
}

// ═══════════════════════════════════════════════════════════════════════════
// B — F5 CLOSURE: an IMPURE computed-unquote must be REJECTED.
// ═══════════════════════════════════════════════════════════════════════════
#[test]
fn mint_impure_computed_unquote_rejected() {
    let result = startup_from_file("tests/macros/probe_arc249_macro_engine_impure.wat.bad");
    // Phase 3 (296-L) repair: the fixture's return-type spelling `(:AST :- [:wat::holon::HolonAST])`
    // failed defmacro SIGNATURE PARSING (`MalformedDefmacro "expected return-type keyword after
    // \`->\`"`) before the F5 purity gate under test ever ran. Respelled to `:wat::WatAST` — the
    // bare keyword defmacro's own parser requires for a macro's return type (a macro always
    // expands to a form) — which is also what the working fixtures in this directory use
    // (`probe_arc249_macro_engine_regression.wat`). The gate now fires for real.
    wat::assert_startup_error!(result,
        StartupError::Macro(MacroError { kind: MacroErrorKind::RefusedInMacro { head }, .. })
            if head == ":wat::kernel::stopped?"
    );
}

// ═══════════════════════════════════════════════════════════════════════════
// C — MINIMAL PROGRAM BODY: a macro body that is an `if` (not a bare quasiquote).
// ═══════════════════════════════════════════════════════════════════════════
#[test]
fn mint_program_body_if() {
    let result = compute_from_file("tests/macros/probe_arc249_macro_engine_prog_if.wat");
    assert_eq!(result, Value::bool(true));
}

// ═══════════════════════════════════════════════════════════════════════════
// D — FOLD-SHAPED PROGRAM BODY.
// ═══════════════════════════════════════════════════════════════════════════
#[test]
fn mint_program_body_fold() {
    let result = compute_from_file("tests/macros/probe_arc249_macro_engine_prog_fold.wat");
    assert_eq!(result, Value::bool(true));
}

// ═══════════════════════════════════════════════════════════════════════════
// E — HYGIENE BOUND.
// ═══════════════════════════════════════════════════════════════════════════
#[test]
fn hygiene_bound_program_body_literal_binder_refused() {
    let result = startup_from_file("tests/macros/probe_arc249_macro_engine_hygiene.wat.bad");
    // Phase 3 (296-L) repair: same defect class as `mint_impure_computed_unquote_rejected`
    // above — the outer defmacro's param type `:wat::holon::HolonAST` and return type
    // `(:AST :- [:wat::holon::HolonAST])` both failed defmacro's own signature constraint (a
    // macro param/return always binds/produces a FORM, so it must be `:wat::WatAST`), before
    // the hygiene-bound logic under test ever ran. Respelled the outer signature to
    // `:wat::WatAST` (leaving the inner `if`'s unrelated `-> (:AST :- [...])` annotation
    // alone — that's a value-level type ascription on the `if`, not part of defmacro's own
    // signature). The gate now fires for real.
    wat::assert_startup_error!(result,
        StartupError::Macro(MacroError {
            kind: MacroErrorKind::ProgramBodyIntroducesName { macro_name, binder },
            ..
        }) if macro_name == ":my::capturing" && binder == "tmp"
    );
}

// ═══════════════════════════════════════════════════════════════════════════
// F — THE MIRROR WALL (arc 255 Stone expand-only-the-mirror-wall): `is_expand_time_legal`
// above (B, E) refuses a head found INSIDE a macro body; this is its mirror — refuse an
// `ExpandTime::ExpandOnly` head found OUTSIDE one. `:wat::core::macro-error` is, today,
// the sole `ExpandOnly` declarer.
// ═══════════════════════════════════════════════════════════════════════════

/// CASE A — the control, written first: `macro-error` as the entire program body of a
/// `defmacro`, never invoked, must still load and compute unchanged. If this ever goes
/// red, the wall fired at macro-error's only legitimate call site and the mirror wall is
/// worse than not shipping — do not adjust this fixture to match the wall.
#[test]
fn mirror_wall_control_macro_error_inside_defmacro_still_legal() {
    let result = compute_from_file("tests/macros/probe_arc255_mirror_wall_control.wat");
    assert_eq!(result, Value::bool(true));
}

/// CASE B — the target: a direct `macro-error` call in a `defn` body (ordinary program
/// code, no enclosing `defmacro`) is refused at expand time.
#[test]
fn mirror_wall_direct_call_outside_macro_refused() {
    let result = startup_from_file("tests/macros/probe_arc255_mirror_wall_target.wat.bad");
    wat::assert_startup_error!(result,
        StartupError::Macro(MacroError { kind: MacroErrorKind::ExpandOnlyOutsideMacro { head }, .. })
            if head == ":wat::core::macro-error"
    );
}

/// CASE C — the quoted-template defect: a macro whose TEMPLATE quotes a `macro-error`
/// call registers cleanly (its own program body is a bare quasiquote — legal), but
/// expanding a call to it splices the literal call into ordinary program code, where it
/// would otherwise only raise at runtime. Refused at expand time, same error kind as B.
#[test]
fn mirror_wall_quoted_template_emitting_macro_error_refused() {
    let result = startup_from_file("tests/macros/probe_arc255_mirror_wall_quoted_template.wat.bad");
    wat::assert_startup_error!(result,
        StartupError::Macro(MacroError { kind: MacroErrorKind::ExpandOnlyOutsideMacro { head }, .. })
            if head == ":wat::core::macro-error"
    );
}

// ═══════════════════════════════════════════════════════════════════════════
// E2 — 255.11: GATE E READS BOTH SPELLINGS OF THE QUASIQUOTE HEAD.
//
// The wall audit (`docs/arc/2026/06/255-builtin-registry/SCORE-STONE-255.11-the-wall-audit.md`)
// measured `quasiquote_inner` + `check_quasiquote_for_literal_binders` FAIL-OPEN: both
// read `WatAST::Keyword` only, so a macro whose NESTED quasiquote (and its `let` head)
// were written in the faithful-Clojure spelling was walked as ordinary program code and
// the binder scan never ran. Measured at `eb860cb96`, one file pair differing only in
// that spelling: keyword → rc 1 `ProgramBodyIntroducesName`, symbol → rc 0, SILENTLY.
// `` ` ``/`~` sugar was never affected (reader-synthesized leaves survive the codemod);
// the EXPLICIT spelling is the reachable one, and 255.10's census called it "LOUD" — it
// is SILENT.
//
// ⚠ `is_quasiquote_form` — the sibling ROUTER — is deliberately still keyword-only:
// teaching it the symbol spelling would SKIP `validate_macro_definition`, the more
// permissive direction. Reported for the builder, not cured.
// ═══════════════════════════════════════════════════════════════════════════

/// CASE E2-a — the target: the capturing macro of case E, with its nested quasiquote and
/// `let` head spelled as symbols, must be refused by the SAME gate with the SAME binder.
#[test]
fn hygiene_bound_fires_on_a_symbol_spelled_quasiquote_template() {
    let result = startup_from_file("tests/macros/probe_arc255_11_hygiene_symbol_quasiquote.wat.bad");
    wat::assert_startup_error!(result,
        StartupError::Macro(MacroError {
            kind: MacroErrorKind::ProgramBodyIntroducesName { macro_name, binder },
            ..
        }) if macro_name == ":my::capturing" && binder == "tmp"
    );
}

/// CASE E2-b — ⛔ THE POSITIVE CONTROL: the same symbol-spelled nested quasiquote with no
/// literal binder still registers, expands and computes. If this goes red the cure
/// refuses every symbol-spelled quasiquote, which is not an intact wall — do not adjust
/// this fixture to match the wall.
#[test]
fn a_symbol_spelled_quasiquote_without_a_literal_binder_still_expands() {
    let result = compute_from_file("tests/macros/probe_arc255_11_hygiene_symbol_control.wat");
    assert_eq!(result, Value::bool(true));
}

// ═══════════════════════════════════════════════════════════════════════════
// E3 — 251.8d-ii: A TYPE IN A PARAM VECTOR IS NOT A BINDER NAME.
//
// Gate E's `fn` arm used to walk the params vector one item at a time and refuse EVERY
// Symbol that was not a `->`/`<-`/`&` marker. That is only safe while a type is spelled
// as a Keyword (`:wat::core::i64`). In the faithful-Clojure spelling a type is a SYMBOL
// (`wat.gen/Coord`), so the gate read the ANNOTATION as a literal binder and refused
// `wat/gen.wat`'s `record` macro at definition — ONE site, 5 286 failing tests, the whole
// converted stdlib blocked (`SCORE-STONE-251.8d-ii-FOURTH-the-bootstrap.md`).
//
// The cure asks `types::is_param_annotation_arrow` (the same door `argspec::parse_triple`
// uses) and skips the item AFTER an arrow, restoring the triple cadence this gate's own
// header already documented. E3-a/E3-b are the NON-VACUITY rows: a genuine literal binder
// is still refused, in BOTH spellings, with the same located binder. E3-c is the positive
// control — the shape that was wrongly refused.
// ═══════════════════════════════════════════════════════════════════════════

/// CASE E3-a — ⛔ NON-VACUITY, keyword spelling: a template `fn` that introduces the real
/// literal binder `y` is still refused.
#[test]
fn hygiene_bound_still_fires_on_a_keyword_spelled_fn_binder() {
    let result =
        startup_from_file("tests/macros/probe_arc251_8d_hygiene_fn_binder_keyword.wat.bad");
    wat::assert_startup_error!(result,
        StartupError::Macro(MacroError {
            kind: MacroErrorKind::ProgramBodyIntroducesName { macro_name, binder },
            ..
        }) if macro_name == ":my::capturing-param" && binder == "y"
    );
}

/// CASE E3-b — ⛔ NON-VACUITY, symbol spelling: the SAME violation written in the
/// faithful-Clojure spelling is refused by the SAME gate with the SAME binder. If this
/// goes green, the type-skip has eaten the binder scan.
#[test]
fn hygiene_bound_still_fires_on_a_symbol_spelled_fn_binder() {
    let result = startup_from_file("tests/macros/probe_arc251_8d_hygiene_fn_binder_symbol.wat.bad");
    wat::assert_startup_error!(result,
        StartupError::Macro(MacroError {
            kind: MacroErrorKind::ProgramBodyIntroducesName { macro_name, binder },
            ..
        }) if macro_name == ":my::capturing-param" && binder == "y"
    );
}

/// CASE E3-c — ⛔ THE POSITIVE CONTROL: `wat/gen.wat`'s `record` shape in miniature — a
/// `let`-bodied macro whose template is a `fn` with a `~`-spliced hygienic binder and a
/// SYMBOL-spelled type. It must register, expand and compute. Without the cure this is
/// refused with `binder == "wat.core/i64"` — the type read as a name.
#[test]
fn a_symbol_spelled_type_annotation_is_not_a_binder() {
    let result =
        compute_from_file("tests/macros/probe_arc251_8d_hygiene_fn_type_is_not_a_binder.wat");
    assert_eq!(result, Value::bool(true));
}
