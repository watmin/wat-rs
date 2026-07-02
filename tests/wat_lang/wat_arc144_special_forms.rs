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
    // rune:lint(loose-assert) — property-over-variable-set: called from multiple test fns with different `probe` values; `define_line` differs per form but every valid special form must carry this sentinel
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
    assert_eq!(
        signature_line,
        r#"#wat.core.Option/Some #wat-edn.holon/Bundle [#wat-edn.holon/Keyword :wat::core::if #wat-edn.holon/Symbol "<cond>" #wat-edn.holon/Symbol "<then>" #wat-edn.holon/Symbol "<else>"]"#,
        "if signature must carry <cond>/<then>/<else> slots"
    );
}

#[test]
fn lookup_form_let_returns_special_form() {
    assert_special_form("let", ":wat::core::let", ":wat::core::let");
    let signature_line = sig_str("let");
    assert_eq!(
        signature_line,
        r#"#wat.core.Option/Some #wat-edn.holon/Bundle [#wat-edn.holon/Keyword :wat::core::let #wat-edn.holon/Symbol "<bindings>" #wat-edn.holon/Symbol "<body>+"]"#,
        "let signature must carry <bindings>/<body>+ slots"
    );
}

#[test]
fn lookup_form_fn_returns_special_form() {
    assert_special_form("fn", ":wat::core::fn", ":wat::core::fn");
    let sig = sig_str("fn");
    assert_eq!(
        sig,
        r#"#wat.core.Option/Some #wat-edn.holon/Bundle [#wat-edn.holon/Keyword :wat::core::fn #wat-edn.holon/Symbol "<params>" #wat-edn.holon/Symbol "<body>+"]"#,
        "fn signature must carry <params>/<body>+ slots"
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
    assert_eq!(
        sig,
        r#"#wat.core.Option/Some #wat-edn.holon/Bundle [#wat-edn.holon/Keyword :wat::core::match #wat-edn.holon/Symbol "<scrutinee>" #wat-edn.holon/Symbol "->" #wat-edn.holon/Symbol "<T>" #wat-edn.holon/Symbol "<arm>+"]"#,
        "match signature must carry <scrutinee>/<arm>+ slots"
    );
}

#[test]
fn lookup_form_quasiquote_returns_special_form() {
    assert_special_form("quasiquote", ":wat::core::quasiquote", ":wat::core::quasiquote");
    let sig = sig_str("quasiquote");
    assert_eq!(
        sig,
        r#"#wat.core.Option/Some #wat-edn.holon/Bundle [#wat-edn.holon/Keyword :wat::core::quasiquote #wat-edn.holon/Symbol "<template>"]"#,
        "quasiquote signature must carry <template> slot"
    );
}

#[test]
fn lookup_form_struct_returns_special_form() {
    // Arc 293.2-parity: :wat::core::defstruct is now a WAT MACRO (not a special form).
    // lookup-define returns the macro definition (head :wat::core::defmacro); the macro
    // body expands all args through to :wat::core::structtype (the new low-level primitive).
    let define_line = def_str("defstruct");
    assert_eq!(
        define_line,
        r#"#wat.core.Option/Some #wat-edn.holon/Bundle [#wat-edn.holon/Keyword :wat::core::defmacro #wat-edn.holon/Keyword :wat::core::defstruct #wat-edn.holon/Bundle [#wat-edn.holon/Symbol "&" #wat-edn.holon/Symbol "args" #wat-edn.holon/Symbol "<-" #wat-edn.holon/Keyword :AST<Vec<wat::WatAST>>] #wat-edn.holon/Symbol "->" #wat-edn.holon/Keyword :AST<wat::WatAST> #wat-edn.holon/Bundle [#wat-edn.holon/Keyword :wat::core::quasiquote #wat-edn.holon/Bundle [#wat-edn.holon/Keyword :wat::core::structtype #wat-edn.holon/Bundle [#wat-edn.holon/Keyword :wat::core::unquote-splicing #wat-edn.holon/Symbol "args"]]]]"#,
        "Arc 293.2-parity: defstruct must be a macro expanding to structtype"
    );
}

#[test]
fn lookup_form_kernel_spawn_returns_special_form() {
    assert_special_form("spawn", ":wat::kernel::spawn", ":wat::kernel::spawn");
    let sig = sig_str("spawn");
    assert_eq!(
        sig,
        r#"#wat.core.Option/Some #wat-edn.holon/Bundle [#wat-edn.holon/Keyword :wat::kernel::spawn #wat-edn.holon/Symbol "<retired-use-spawn-thread>"]"#,
        "spawn signature must carry :wat::kernel::spawn head"
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
