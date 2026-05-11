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
//!   - `(:wat::runtime::signature-of :SOMETHING)` returns
//!     `Some(<HolonAST>)` whose head matches the form's keyword and
//!     whose body slots match the audited grammar.
//!   - `(:wat::runtime::body-of :SOMETHING)` returns `:None` —
//!     special forms are syntactic operations, not data with a body.
//!
//! The bonus test pins `lookup_form` returning None on a
//! deliberately-not-registered name; the registry is intentional, not
//! a wildcard catch-all.

use std::sync::Arc;
use wat::freeze::{eval_in_frozen, startup_from_source};
use wat::load::InMemoryLoader;
use wat::runtime::{Environment, Value};

fn with_nil_main(src: &str) -> String {
    format!(
        "{}\n(:wat::core::define (:user::main -> :wat::core::nil) :wat::core::nil)",
        src
    )
}

fn eval_string(src: &str) -> String {
    let src = with_nil_main(src);
    let world = startup_from_source(
        &src,
        Some(concat!(file!(), ":", line!())),
        Arc::new(InMemoryLoader::new()),
    )
    .expect("startup");
    let ast = wat::parse_one!("(:user::compute)").expect("parse compute call");
    let env = Environment::new();
    match eval_in_frozen(&ast, &world, &env).expect("compute") {
        Value::String(s) => s.as_str().to_owned(),
        other => panic!("expected String; got {:?}", other),
    }
}

fn eval_bool(src: &str) -> bool {
    let src = with_nil_main(src);
    let world = startup_from_source(
        &src,
        Some(concat!(file!(), ":", line!())),
        Arc::new(InMemoryLoader::new()),
    )
    .expect("startup");
    let ast = wat::parse_one!("(:user::compute)").expect("parse compute call");
    let env = Environment::new();
    match eval_in_frozen(&ast, &world, &env).expect("compute") {
        Value::bool(b) => b,
        other => panic!("expected bool; got {:?}", other),
    }
}

/// Drive the three reflection primitives at a special-form name and
/// return (def_rendered, sig_rendered, body_is_none).
fn three_probes(name_keyword: &str) -> (String, String, bool) {
    let def_rendered = eval_string(&format!(
        r##"
        (:wat::core::define (:user::compute -> :wat::core::String)
          (:wat::edn::write (:wat::runtime::lookup-define {name})))
        "##,
        name = name_keyword
    ));
    let sig_rendered = eval_string(&format!(
        r##"
        (:wat::core::define (:user::compute -> :wat::core::String)
          (:wat::edn::write (:wat::runtime::signature-of {name})))
        "##,
        name = name_keyword
    ));
    let body_is_none = eval_bool(&format!(
        r##"
        (:wat::core::define (:user::compute -> :wat::core::bool)
          (:wat::core::match
            (:wat::runtime::body-of {name})
            -> :wat::core::bool
            ((:wat::core::Some _) false)
            (:wat::core::None    true)))
        "##,
        name = name_keyword
    ));
    (def_rendered, sig_rendered, body_is_none)
}

/// Common assertions on the three-probe output:
///   - lookup-define rendered AST contains the slice-1 sentinel head
///     `:wat::core::__internal/special-form` and the form's name
///   - signature-of rendered AST contains the form's name (its
///     bundle head)
///   - body-of returned :None
fn assert_special_form(name_keyword: &str, name_no_colon_prefix: &str) {
    let (define_line, signature_line, body_is_none) = three_probes(name_keyword);
    assert!(
        define_line.contains(":wat::core::__internal/special-form"),
        "lookup-define for {} should emit the slice-1 special-form sentinel; got: {}",
        name_keyword,
        define_line
    );
    assert!(
        define_line.contains(name_no_colon_prefix),
        "lookup-define for {} should mention the form name {}; got: {}",
        name_keyword,
        name_no_colon_prefix,
        define_line
    );
    assert!(
        signature_line.contains(name_no_colon_prefix),
        "signature-of for {} should render the form's name as its bundle head; got: {}",
        name_keyword,
        signature_line
    );
    assert!(
        body_is_none,
        "body-of for {} should be :None",
        name_keyword
    );
}

// ─── Per-group coverage (one test per representative special form) ──────────

#[test]
fn lookup_form_if_returns_special_form() {
    let (define_line, signature_line, body_is_none) = three_probes(":wat::core::if");
    // lookup-define emits the SpecialForm sentinel
    // `(:wat::core::__internal/special-form :wat::core::if)`.
    assert!(
        define_line.contains(":wat::core::__internal/special-form"),
        "expected sentinel head, got: {}",
        define_line
    );
    assert!(
        define_line.contains(":wat::core::if"),
        "expected form name, got: {}",
        define_line
    );
    // signature-of emits the registry's synthetic Bundle: head =
    // `:wat::core::if`, slots = `<cond>`/`<then>`/`<else>`. The slots
    // are the load-bearing evidence that slice 2 populated the
    // registry's `signature` field (vs slice 1 returning None for
    // SpecialForm).
    assert!(
        signature_line.contains(":wat::core::if"),
        "expected form keyword in signature, got: {}",
        signature_line
    );
    assert!(
        signature_line.contains("<cond>")
            && signature_line.contains("<then>")
            && signature_line.contains("<else>"),
        "expected <cond>/<then>/<else> slots in signature, got: {}",
        signature_line
    );
    // body-of returns :None.
    assert!(body_is_none, "body-of should be :None");
}

#[test]
fn lookup_form_let_returns_special_form() {
    let (define_line, signature_line, body_is_none) = three_probes(":wat::core::let");
    assert!(
        define_line.contains(":wat::core::__internal/special-form"),
        "expected sentinel head, got: {}",
        define_line
    );
    assert!(
        define_line.contains(":wat::core::let"),
        "expected form name, got: {}",
        define_line
    );
    assert!(
        signature_line.contains(":wat::core::let")
            && signature_line.contains("<bindings>")
            && signature_line.contains("<body>+"),
        "expected let signature with <bindings>/<body>+, got: {}",
        signature_line
    );
    assert!(body_is_none, "body-of should be :None");
}

#[test]
fn lookup_form_fn_returns_special_form() {
    // Arc 155: `:wat::core::fn` is the canonical operator form for function
    // values (replaced `:wat::core::lambda`). The registry entry carries
    // the same shape: params + body.
    assert_special_form(":wat::core::fn", ":wat::core::fn");
    // Pin the load-bearing slot.
    let (_, sig, _) = three_probes(":wat::core::fn");
    assert!(
        sig.contains("<params>") && sig.contains("<body>+"),
        "expected <params>/<body>+ in fn signature, got: {}",
        sig
    );
}

#[test]
fn lookup_form_define_returns_special_form() {
    assert_special_form(":wat::core::define", ":wat::core::define");
    let (_, sig, _) = three_probes(":wat::core::define");
    assert!(
        sig.contains("<head>") && sig.contains("<body>"),
        "expected <head>/<body> in define signature, got: {}",
        sig
    );
}

#[test]
fn lookup_form_match_returns_special_form() {
    assert_special_form(":wat::core::match", ":wat::core::match");
    let (_, sig, _) = three_probes(":wat::core::match");
    // The `->` and `<T>` slots are part of the match grammar's
    // surface form — verify they made it into the sketch.
    assert!(
        sig.contains("<scrutinee>") && sig.contains("<arm>+"),
        "expected <scrutinee>/<arm>+ in match signature, got: {}",
        sig
    );
}

#[test]
fn lookup_form_quasiquote_returns_special_form() {
    assert_special_form(":wat::core::quasiquote", ":wat::core::quasiquote");
    let (_, sig, _) = three_probes(":wat::core::quasiquote");
    assert!(
        sig.contains("<template>"),
        "expected <template> in quasiquote signature, got: {}",
        sig
    );
}

#[test]
fn lookup_form_struct_returns_special_form() {
    assert_special_form(":wat::core::struct", ":wat::core::struct");
    let (_, sig, _) = three_probes(":wat::core::struct");
    assert!(
        sig.contains("<name>") && sig.contains("<field>+"),
        "expected <name>/<field>+ in struct signature, got: {}",
        sig
    );
}

#[test]
fn lookup_form_kernel_spawn_returns_special_form() {
    // `:wat::kernel::spawn` is RETIRED (arc 114 Pattern 2 poison) but
    // still has dispatch in `infer_list` redirecting to
    // `:wat::kernel::spawn-thread`. Reflection should still find it —
    // "nothing is special, even retired forms" — so a future
    // `(help :wat::kernel::spawn)` can render the migration redirect.
    assert_special_form(":wat::kernel::spawn", ":wat::kernel::spawn");
    let (_, sig, _) = three_probes(":wat::kernel::spawn");
    assert!(
        sig.contains(":wat::kernel::spawn"),
        "expected spawn keyword as signature head, got: {}",
        sig
    );
}

// ─── Bonus: unknown special-form name returns None ──────────────────────────

#[test]
fn lookup_form_unknown_special_form_name_returns_none() {
    // The registry is intentional, not a wildcard catch-all. A name
    // that LOOKS like a special form (`:wat::core::not-a-special-form`)
    // but isn't registered yields None across all three reflection
    // primitives — same shape as slice 1's
    // `all_three_primitives_return_none_on_unknown_name` test.
    let src = r##"
        (:wat::core::define (:user::compute -> :wat::core::bool)
          (:wat::core::let
            [d-opt
              (:wat::runtime::lookup-define :wat::core::not-a-special-form)
             s-opt
              (:wat::runtime::signature-of :wat::core::not-a-special-form)
             b-opt
              (:wat::runtime::body-of    :wat::core::not-a-special-form)]
            (:wat::core::match d-opt
              -> :wat::core::bool
              ((:wat::core::Some _) false)
              (:wat::core::None
                (:wat::core::match s-opt
                  -> :wat::core::bool
                  ((:wat::core::Some _) false)
                  (:wat::core::None
                    (:wat::core::match b-opt
                      -> :wat::core::bool
                      ((:wat::core::Some _) false)
                      (:wat::core::None    true))))))))
    "##;
    assert!(eval_bool(src), "unknown name should return None for all three primitives");
}
