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

use wat::freeze::{eval_in_frozen, startup_beside};
use wat::runtime::{Environment, Value};

fn run_expr(expr: &str) -> Value {
    let world = startup_beside(file!()).expect("startup");
    let ast = wat::parse_one!(expr).expect("parse expr");
    eval_in_frozen(&ast, &world, &Environment::new())
        .expect("eval should succeed")
        .value_owned()
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
    unwrap_string(run_expr(&format!("(:t::def-{})", probe)), probe)
}
fn sig_str(probe: &str) -> String {
    unwrap_string(run_expr(&format!("(:t::sig-{})", probe)), probe)
}
fn body_none(probe: &str) -> bool {
    unwrap_bool(run_expr(&format!("(:t::body-{})", probe)), probe)
}

/// Common assertions on the three-probe output.
fn assert_special_form(probe: &str, name_keyword: &str, name_fragment: &str) {
    let define_line = def_str(probe);
    let signature_line = sig_str(probe);
    let body_is_none = body_none(probe);
    assert!(
        define_line.contains(":wat::core::__internal/special-form"),
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
    assert_special_form("if", ":wat::core::if", ":wat::core::if");
    let signature_line = sig_str("if");
    assert!(
        signature_line.contains("<cond>")
            && signature_line.contains("<then>")
            && signature_line.contains("<else>"),
        "expected <cond>/<then>/<else> slots in signature, got: {}",
        signature_line
    );
}

#[test]
fn lookup_form_let_returns_special_form() {
    assert_special_form("let", ":wat::core::let", ":wat::core::let");
    let signature_line = sig_str("let");
    assert!(
        signature_line.contains(":wat::core::let")
            && signature_line.contains("<bindings>")
            && signature_line.contains("<body>+"),
        "expected let signature with <bindings>/<body>+, got: {}",
        signature_line
    );
}

#[test]
fn lookup_form_fn_returns_special_form() {
    assert_special_form("fn", ":wat::core::fn", ":wat::core::fn");
    let sig = sig_str("fn");
    assert!(
        sig.contains("<params>") && sig.contains("<body>+"),
        "expected <params>/<body>+ in fn signature, got: {}",
        sig
    );
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
    assert_special_form("match", ":wat::core::match", ":wat::core::match");
    let sig = sig_str("match");
    assert!(
        sig.contains("<scrutinee>") && sig.contains("<arm>+"),
        "expected <scrutinee>/<arm>+ in match signature, got: {}",
        sig
    );
}

#[test]
fn lookup_form_quasiquote_returns_special_form() {
    assert_special_form("quasiquote", ":wat::core::quasiquote", ":wat::core::quasiquote");
    let sig = sig_str("quasiquote");
    assert!(
        sig.contains("<template>"),
        "expected <template> in quasiquote signature, got: {}",
        sig
    );
}

#[test]
fn lookup_form_struct_returns_special_form() {
    // Arc 293.2-parity: :wat::core::defstruct is now a WAT MACRO (not a special form).
    // lookup-define returns the macro definition (head :wat::core::defmacro); the macro
    // body expands all args through to :wat::core::structtype (the new low-level primitive).
    let define_line = def_str("defstruct");
    assert!(
        define_line.contains(":wat::core::defmacro"),
        "Arc 293.2-parity: defstruct should now be a macro; lookup-define should contain \
         :wat::core::defmacro; got: {}",
        define_line
    );
    assert!(
        define_line.contains(":wat::core::structtype"),
        "Arc 293.2-parity: defstruct macro body should expand to :wat::core::structtype; \
         got: {}",
        define_line
    );
}

#[test]
fn lookup_form_kernel_spawn_returns_special_form() {
    assert_special_form("spawn", ":wat::kernel::spawn", ":wat::kernel::spawn");
    let sig = sig_str("spawn");
    assert!(
        sig.contains(":wat::kernel::spawn"),
        "expected spawn keyword as signature head, got: {}",
        sig
    );
}

// ─── Bonus: unknown special-form name returns None ──────────────────────────

#[test]
fn lookup_form_unknown_special_form_name_returns_none() {
    assert!(
        unwrap_bool(run_expr("(:t::all-none-not-a-sf)"), "all-none"),
        "unknown name should return None for all three primitives"
    );
}
