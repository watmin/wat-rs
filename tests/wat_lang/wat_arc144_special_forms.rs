//! Integration coverage for arc 144 slice 2 — special-form registry.
//!
//! Slice 1 shipped a 5-variant `Binding` enum + `lookup_form` walking
//! four registries; the SpecialForm path returned None until slice 2
//! populated the registry. Slice 2 added a `OnceLock`-backed
//! `HashMap<String, SpecialFormDef>` covering ~30 special forms
//! identified from the `infer_list` head dispatch + freeze top-level
//! mutation forms + retired-but-poisoned heads kept for migration.
//!
//! These tests verify the end-to-end uniform-reflection promise:
//!   - `(:wat::runtime::lookup-define :SOMETHING)` returns
//!     `Some(<wat::holon::HolonAST>)` for every known special form;
//!     the AST emits the slice-1 sentinel
//!     `(:wat::core::__internal/special-form <name>)`.
//!   - `(:wat::runtime::signature-of-defn :SOMETHING)` returns
//!     `Some(<HolonAST>)` whose head matches the form's keyword and
//!     whose body slots match the audited grammar.
//!   - `(:wat::runtime::body-of :SOMETHING)` returns `:None` —
//!     special forms are syntactic operations, not data with a body.
//!
//! The bonus test pins `lookup_form` returning None on a
//! deliberately-not-registered name; the registry is intentional, not
//! a wildcard catch-all.

use wat::freeze::{call_beside_value, startup_beside};
use wat::runtime::Value;

fn run_expr(name: &str) -> Value {
    call_beside_value(file!(), name).expect("eval should succeed")
}

fn unwrap_string(v: Value, ctx: &str) -> String {
    match v {
        Value::String(s) => (*s).clone(),
        other => panic!("{}: expected String; got {:?}", ctx, other),
    }
}

fn unwrap_bool(v: Value, ctx: &str) -> bool {
    match v {
        Value::bool(b) => b,
        other => panic!("{}: expected bool; got {:?}", ctx, other),
    }
}

fn def_str(probe: &str) -> String {
    unwrap_string(run_expr(&format!(":t::def-{}", probe)), probe)
}
fn sig_str(probe: &str) -> String {
    unwrap_string(run_expr(&format!(":t::sig-{}", probe)), probe)
}
fn body_none(probe: &str) -> bool {
    unwrap_bool(run_expr(&format!(":t::body-{}", probe)), probe)
}

/// Common assertions on the three-probe output.
fn assert_special_form(probe: &str, name_keyword: &str, name_fragment: &str) {
    let define_line = def_str(probe);
    let signature_line = sig_str(probe);
    let body_is_none = body_none(probe);
    // rune:lint(loose-assert) — property-over-variable-set: called from multiple test fns with different `probe` values; `define_line` differs per form but every valid special form must carry this sentinel
    assert!(
        define_line.contains(":wat.core/__internal/special-form"),
        "lookup-define for {} should emit the special-form sentinel; got: {}",
        name_keyword, define_line
    );
    assert!(
        define_line.contains(name_fragment),
        "lookup-define for {} should mention the form name {}; got: {}",
        name_keyword, name_fragment, define_line
    );
    assert!(
        signature_line.contains(name_fragment),
        "signature-of-defn for {} should render the form's name; got: {}",
        name_keyword, signature_line
    );
    assert!(body_is_none, "body-of for {} should be :None", name_keyword);
}

// ─── Per-group coverage (one test per representative special form) ──────────

#[test]
fn lookup_form_if_returns_special_form() {
    assert_special_form("if", ":wat::core::if", ":wat.core/if");
    let signature_line = sig_str("if");
    wat::assert_edn_matches_file!(signature_line, "wat_arc144_special_forms__if.edn", "if signature must carry <cond>/<then>/<else> slots");
}

#[test]
fn lookup_form_let_returns_special_form() {
    assert_special_form("let", ":wat::core::let", ":wat.core/let");
    let signature_line = sig_str("let");
    wat::assert_edn_matches_file!(signature_line, "wat_arc144_special_forms__let.edn", "let signature must carry <bindings>/<body>+ slots");
}

#[test]
fn lookup_form_fn_returns_special_form() {
    assert_special_form("fn", ":wat::core::fn", ":wat.core/fn");
    let sig = sig_str("fn");
    wat::assert_edn_matches_file!(sig, "wat_arc144_special_forms__fn.edn", "fn signature must carry <params>/<body>+ slots");
}

#[test]
fn lookup_form_define_is_absent_from_registry() {
    // Stone 241.16 — `:wat::core::define` HARD CUT (eval-time residue completed).
    // The registry entry was DELETED; lookup must return None.
    use wat::special_forms::lookup_special_form;
    assert!(
        lookup_special_form(":wat::core::define").is_none(),
        "expected :wat::core::define to be ABSENT from special_forms registry post-Stone-241.16 (HARD CUT total)"
    );
}

#[test]
fn lookup_form_match_returns_special_form() {
    assert_special_form("match", ":wat::core::match", ":wat.core/match");
    let sig = sig_str("match");
    wat::assert_edn_matches_file!(sig, "wat_arc144_special_forms__match.edn", "match signature must carry <scrutinee>/<arm>+ slots");
}

#[test]
fn lookup_form_quasiquote_returns_special_form() {
    assert_special_form("quasiquote", ":wat::core::quasiquote", ":wat.core/quasiquote");
    let sig = sig_str("quasiquote");
    wat::assert_edn_matches_file!(sig, "wat_arc144_special_forms__quasiquote.edn", "quasiquote signature must carry <template> slot");
}

/// Does this WatAST carry `kw` as a keyword node, at any depth? A structural walk over
/// WatAST — deliberately NOT a string search over a rendered face.
fn watast_carries_keyword(node: &wat::ast::WatAST, kw: &str) -> bool {
    use wat::ast::WatAST;
    match node {
        WatAST::Keyword(k, _) => k == kw,
        WatAST::List(items, _) | WatAST::Vector(items, _) | WatAST::Set(items, _) => {
            items.iter().any(|c| watast_carries_keyword(c, kw))
        }
        WatAST::Map(pairs, _) => pairs
            .iter()
            .any(|(k, v)| watast_carries_keyword(k, kw) || watast_carries_keyword(v, kw)),
        _ => false,
    }
}

#[test]
fn lookup_form_struct_returns_special_form() {
    // Arc 293.2-parity: :wat::core::defstruct is a WAT MACRO, not a special form.
    //
    // Proven WITHOUT the holon-ast face. `lookup-define` still renders through the OLD
    // `watast_to_holon` path (`src/edn/bridge.rs`'s module doc calls it exactly that), and arc 294's
    // own realizations name that face as scar tissue: flaw #3 "the tagged-HolonAST wire
    // family (scar tissue from a hologram-canonical wire)" and #5 "HolonAST-as-the-code-AST
    // vestigial (WatAST took over)" — HolonAST reduces to Hologram. Pinning that rendering
    // (inline or as a golden) would FOSSILIZE the very thing this arc exists to excise, and
    // the golden wall refuses it outright ("STOP-1: refusing to capture a non-EDN face").
    //
    // So assert the CLAIM against the registries directly — WatAST all the way down.

    // NOTE: defstruct is NOT absent from the special-form registry — it is registered
    // there for its SIGNATURE grammar (special_forms.rs:192, `["<name>", "[<field> <-
    // <type>]+"]`). It is BOTH: a signature entry AND a macro; `lookup-define` resolves
    // to the macro. So the parity claim is about what it IS (a macro that lowers to
    // structtype), asserted below against the macro registry — not about registry absence.

    // 1. IS a registered macro — asserted exactly.
    let world = startup_beside(file!()).expect("startup");
    let def = world
        .macros()
        .get(":wat::core::defstruct")
        .expect("Arc 293.2-parity: :wat::core::defstruct must be a REGISTERED MACRO");

    // 2. Its body reaches the low-level primitive. A structural walk of the macro's own
    //    WatAST body — no holon face, no string search over a rendering. The body itself
    //    legitimately grows (9a made defstruct also mint the bare-name kwargs companion;
    //    (C) changed that companion's emit), which is exactly why the OLD byte-pin went
    //    stale twice; what this test owns is that defstruct still lowers to structtype.
    assert!(
        watast_carries_keyword(&def.body, ":wat::core::structtype"),
        "Arc 293.2-parity: defstruct's macro body must expand through to \
         :wat::core::structtype (the low-level primitive)"
    );
}

// ─── Bonus: unknown special-form name returns None ──────────────────────────

#[test]
fn lookup_form_unknown_special_form_name_returns_none() {
    assert!(
        unwrap_bool(run_expr(":t::all-none-not-a-sf"), "all-none"),
        "unknown name should return None for all three primitives"
    );
}
