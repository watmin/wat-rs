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
use wat::freeze::{invoke_user_main, startup_from_source};
use wat::io::{StringIoReader, StringIoWriter, WatReader, WatWriter};
use wat::load::InMemoryLoader;
use wat::runtime::Value;

fn run(src: &str) -> Vec<String> {
    let world = startup_from_source(
        src,
        Some(concat!(file!(), ":", line!())),
        Arc::new(InMemoryLoader::new()),
    )
    .expect("startup");
    let stdin: Arc<dyn WatReader> = Arc::new(StringIoReader::from_string(String::new()));
    let stdout = Arc::new(StringIoWriter::new());
    let stderr = Arc::new(StringIoWriter::new());
    let stdout_dyn: Arc<dyn WatWriter> = stdout.clone();
    let stderr_dyn: Arc<dyn WatWriter> = stderr.clone();
    let args = vec![
        Value::io__IOReader(stdin),
        Value::io__IOWriter(stdout_dyn),
        Value::io__IOWriter(stderr_dyn),
    ];
    invoke_user_main(&world, args).expect("main");
    let bytes = stdout.snapshot_bytes().expect("snapshot");
    let s = String::from_utf8(bytes).expect("utf8");
    if s.is_empty() {
        return Vec::new();
    }
    let mut lines: Vec<String> = s.split('\n').map(String::from).collect();
    if s.ends_with('\n') {
        lines.pop();
    }
    lines
}

/// Drive the three reflection primitives at a special-form name and
/// emit three lines:
///   line 1: rendered EDN of `lookup-define <name>`
///   line 2: rendered EDN of `signature-of <name>`
///   line 3: "body-pass" iff `body-of <name>` returns :None,
///           "body-fail" otherwise
///
/// `name_keyword` is the keyword form passed to the primitives
/// (e.g., `:wat::core::if`).
fn three_probes(name_keyword: &str) -> Vec<String> {
    let src = format!(
        r##"
        (:wat::core::define
          (:user::main
            (stdin  :wat::io::IOReader)
            (stdout :wat::io::IOWriter)
            (stderr :wat::io::IOWriter)
            -> :wat::core::unit)
          (:wat::core::let*
            (((def-opt :wat::core::Option<wat::holon::HolonAST>)
              (:wat::runtime::lookup-define {name}))
             ((def-rendered :wat::core::String)
              (:wat::edn::write def-opt))
             ((sig-opt :wat::core::Option<wat::holon::HolonAST>)
              (:wat::runtime::signature-of {name}))
             ((sig-rendered :wat::core::String)
              (:wat::edn::write sig-opt))
             ((body-opt :wat::core::Option<wat::holon::HolonAST>)
              (:wat::runtime::body-of {name})))
            (:wat::core::let*
              (((_ :wat::core::unit)
                (:wat::io::IOWriter/println stdout def-rendered))
               ((_ :wat::core::unit)
                (:wat::io::IOWriter/println stdout sig-rendered)))
              (:wat::core::match body-opt
                -> :wat::core::unit
                ((:wat::core::Some _) (:wat::io::IOWriter/println stdout "body-fail"))
                (:wat::core::None    (:wat::io::IOWriter/println stdout "body-pass"))))))
    "##,
        name = name_keyword
    );
    run(&src)
}

/// Common assertions on the three-probe output:
///   - 3 lines emitted
///   - lookup-define rendered AST contains the slice-1 sentinel head
///     `:wat::core::__internal/special-form` and the form's name
///   - signature-of rendered AST contains the form's name (its
///     bundle head)
///   - body-of returned :None ("body-pass")
fn assert_special_form(name_keyword: &str, name_no_colon_prefix: &str) {
    let out = three_probes(name_keyword);
    assert_eq!(
        out.len(),
        3,
        "expected 3 lines (lookup-define / signature-of / body-of), got {:?}",
        out
    );
    let define_line = &out[0];
    let signature_line = &out[1];
    let body_line = &out[2];
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
    assert_eq!(
        body_line, "body-pass",
        "body-of for {} should be :None; got: {}",
        name_keyword, body_line
    );
}

// ─── Per-group coverage (one test per representative special form) ──────────

#[test]
fn lookup_form_if_returns_special_form() {
    let out = three_probes(":wat::core::if");
    assert_eq!(out.len(), 3);
    let define_line = &out[0];
    let signature_line = &out[1];
    let body_line = &out[2];
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
    assert_eq!(body_line, "body-pass", "body-of should be :None");
}

#[test]
fn lookup_form_let_star_returns_special_form() {
    let out = three_probes(":wat::core::let*");
    assert_eq!(out.len(), 3);
    let define_line = &out[0];
    let signature_line = &out[1];
    assert!(
        define_line.contains(":wat::core::__internal/special-form"),
        "expected sentinel head, got: {}",
        define_line
    );
    assert!(
        define_line.contains(":wat::core::let*"),
        "expected form name, got: {}",
        define_line
    );
    assert!(
        signature_line.contains(":wat::core::let*")
            && signature_line.contains("<bindings>")
            && signature_line.contains("<body>+"),
        "expected let* signature with <bindings>/<body>+, got: {}",
        signature_line
    );
    assert_eq!(out[2], "body-pass");
}

#[test]
fn lookup_form_lambda_returns_special_form() {
    assert_special_form(":wat::core::lambda", ":wat::core::lambda");
    // Pin the load-bearing slot.
    let out = three_probes(":wat::core::lambda");
    assert!(
        out[1].contains("<params>") && out[1].contains("<body>+"),
        "expected <params>/<body>+ in lambda signature, got: {}",
        out[1]
    );
}

#[test]
fn lookup_form_define_returns_special_form() {
    assert_special_form(":wat::core::define", ":wat::core::define");
    let out = three_probes(":wat::core::define");
    assert!(
        out[1].contains("<head>") && out[1].contains("<body>"),
        "expected <head>/<body> in define signature, got: {}",
        out[1]
    );
}

#[test]
fn lookup_form_match_returns_special_form() {
    assert_special_form(":wat::core::match", ":wat::core::match");
    let out = three_probes(":wat::core::match");
    // The `->` and `<T>` slots are part of the match grammar's
    // surface form — verify they made it into the sketch.
    assert!(
        out[1].contains("<scrutinee>") && out[1].contains("<arm>+"),
        "expected <scrutinee>/<arm>+ in match signature, got: {}",
        out[1]
    );
}

#[test]
fn lookup_form_quasiquote_returns_special_form() {
    assert_special_form(":wat::core::quasiquote", ":wat::core::quasiquote");
    let out = three_probes(":wat::core::quasiquote");
    assert!(
        out[1].contains("<template>"),
        "expected <template> in quasiquote signature, got: {}",
        out[1]
    );
}

#[test]
fn lookup_form_struct_returns_special_form() {
    assert_special_form(":wat::core::struct", ":wat::core::struct");
    let out = three_probes(":wat::core::struct");
    assert!(
        out[1].contains("<name>") && out[1].contains("<field>+"),
        "expected <name>/<field>+ in struct signature, got: {}",
        out[1]
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
    let out = three_probes(":wat::kernel::spawn");
    assert!(
        out[1].contains(":wat::kernel::spawn"),
        "expected spawn keyword as signature head, got: {}",
        out[1]
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
        (:wat::core::define
          (:user::main
            (stdin  :wat::io::IOReader)
            (stdout :wat::io::IOWriter)
            (stderr :wat::io::IOWriter)
            -> :wat::core::unit)
          (:wat::core::let*
            (((d-opt :wat::core::Option<wat::holon::HolonAST>)
              (:wat::runtime::lookup-define :wat::core::not-a-special-form))
             ((s-opt :wat::core::Option<wat::holon::HolonAST>)
              (:wat::runtime::signature-of :wat::core::not-a-special-form))
             ((b-opt :wat::core::Option<wat::holon::HolonAST>)
              (:wat::runtime::body-of    :wat::core::not-a-special-form)))
            (:wat::core::match d-opt
              -> :wat::core::unit
              ((:wat::core::Some _) (:wat::io::IOWriter/println stdout "fail-d"))
              (:wat::core::None
                (:wat::core::match s-opt
                  -> :wat::core::unit
                  ((:wat::core::Some _) (:wat::io::IOWriter/println stdout "fail-s"))
                  (:wat::core::None
                    (:wat::core::match b-opt
                      -> :wat::core::unit
                      ((:wat::core::Some _) (:wat::io::IOWriter/println stdout "fail-b"))
                      (:wat::core::None    (:wat::io::IOWriter/println stdout "pass")))))))))
    "##;
    assert_eq!(run(src), vec!["pass".to_string()]);
}
