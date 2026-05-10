//! Arc 170 slice 1b — Rust closure extraction substrate primitive.
//!
//! These tests exercise `wat::closure_extract::extract_closure` on a
//! variety of fn shapes (top-level defns, inline lambdas, factory
//! results, captures with mixed types, recursive types, transitive
//! deps, non-portable captures). Each test:
//!
//! 1. Composes a parent program (a wat source string) and freezes it.
//! 2. Pulls the entry fn out of the parent's symbol table (or via
//!    `apply_function` for factory-pattern shapes that build the fn
//!    dynamically).
//! 3. Calls `extract_closure` to produce a `ClosurePackage`.
//! 4. Asserts the package shape: `prologue` (Vec<WatAST> — captured
//!    environment) and `entry_form` (a WatAST that evaluates to a fn
//!    Value).
//! 5. Re-freezes a fresh world from `package.prologue`.
//! 6. Evaluates `package.entry_form` in the fresh world to obtain a
//!    fn Value, then applies it; compares against invoking the
//!    original fn directly in the parent world.
//!
//! Slice 1b reshape (vs slice 1):
//!   - `pkg.entry: String` retired → `pkg.entry_form: WatAST`
//!   - `pkg.forms` renamed to `pkg.prologue`
//!   - For inline-lambda input: `entry_form` is the reconstructed
//!     fn-form AST `(:wat::core::fn [name <- :T ...] -> :Ret body)`;
//!     prologue carries no entry-define
//!   - For keyword-path input: `entry_form` is a Keyword AST naming
//!     the entry; the entry's define lives in prologue alongside
//!     other user deps

use std::sync::Arc;
use wat::ast::WatAST;
use wat::closure_extract::{extract_closure, ClosurePackage, ExtractionError};
use wat::freeze::{startup_from_forms, startup_from_source};
use wat::load::InMemoryLoader;
use wat::runtime::{apply_function, eval, Environment, Value};
use wat::span::Span;

// ─── helpers ────────────────────────────────────────────────────────────

fn freeze(src: &str) -> wat::freeze::FrozenWorld {
    startup_from_source(src, None, Arc::new(InMemoryLoader::new()))
        .expect("parent freeze should succeed")
}

fn re_freeze(forms: Vec<WatAST>) -> wat::freeze::FrozenWorld {
    startup_from_forms(forms, None, Arc::new(InMemoryLoader::new()))
        .expect("re-freeze should succeed")
}

fn lookup_fn(world: &wat::freeze::FrozenWorld, path: &str) -> Value {
    let func = world
        .symbols()
        .get(path)
        .unwrap_or_else(|| panic!("fn {} not registered", path));
    Value::wat__core__fn(func.clone())
}

fn extract(
    world: &wat::freeze::FrozenWorld,
    fn_value: &Value,
    entry_name: Option<&str>,
) -> ClosurePackage {
    let types = world.symbols().types().expect("TypeEnv attached").clone();
    extract_closure(fn_value, entry_name, world.symbols(), &types)
        .expect("extract_closure should succeed")
}

fn extract_err(
    world: &wat::freeze::FrozenWorld,
    fn_value: &Value,
    entry_name: Option<&str>,
) -> ExtractionError {
    let types = world.symbols().types().expect("TypeEnv attached").clone();
    extract_closure(fn_value, entry_name, world.symbols(), &types)
        .expect_err("extract_closure should fail")
}

/// Slice 1b consumer pattern: re-freeze prologue, then `eval`
/// entry_form in the frozen world to obtain the fn Value, then
/// `apply_function` it to the args.
fn invoke_via_entry_form(
    fresh: &wat::freeze::FrozenWorld,
    entry_form: &WatAST,
    args: Vec<Value>,
) -> Value {
    let env = Environment::new();
    let fn_value = eval(entry_form, &env, fresh.symbols())
        .expect("entry_form eval should succeed");
    let func = match fn_value {
        Value::wat__core__fn(f) => f,
        other => panic!("entry_form did not evaluate to a fn Value; got {:?}", other),
    };
    apply_function(func, args, fresh.symbols(), Span::unknown())
        .expect("apply_function should succeed")
}

fn invoke_in_parent(
    world: &wat::freeze::FrozenWorld,
    fn_value: &Value,
    args: Vec<Value>,
) -> Value {
    let func = match fn_value {
        Value::wat__core__fn(f) => f.clone(),
        other => panic!("expected fn value, got {:?}", other),
    };
    apply_function(func, args, world.symbols(), Span::unknown())
        .expect("apply_function should succeed")
}

fn assert_i64(v: &Value, expected: i64) {
    match v {
        Value::i64(n) => assert_eq!(*n, expected),
        other => panic!("expected i64({}); got {:?}", expected, other),
    }
}

fn assert_string(v: &Value, expected: &str) {
    match v {
        Value::String(s) => assert_eq!(s.as_str(), expected),
        other => panic!("expected String({:?}); got {:?}", expected, other),
    }
}

/// Get a synthesized lambda by calling a top-level zero-arg factory
/// in the parent world. The factory's body builds the lambda and
/// returns it as a `Value::wat__core__fn`.
fn synth_lambda(world: &wat::freeze::FrozenWorld, factory_path: &str) -> Value {
    let factory = world
        .symbols()
        .get(factory_path)
        .unwrap_or_else(|| panic!("factory {} not registered", factory_path))
        .clone();
    apply_function(factory, Vec::new(), world.symbols(), Span::unknown())
        .expect("factory call ok")
}

// ─── entry_form-shape assertion helpers ────────────────────────────────

/// Assert that `entry_form` is a Keyword AST whose name equals `expected`.
fn assert_entry_form_keyword(entry_form: &WatAST, expected: &str) {
    match entry_form {
        WatAST::Keyword(k, _) => assert_eq!(
            k, expected,
            "expected entry_form Keyword({}); got Keyword({})",
            expected, k
        ),
        other => panic!(
            "expected entry_form to be Keyword({}); got {:?}",
            expected, other
        ),
    }
}

/// Assert that `entry_form` is a fn-form AST
/// `(:wat::core::fn [<param-triples>] -> :Ret <body>)`. Returns the
/// inner Vec items (params-vector triples, ret-keyword) for callers
/// that want to dig further.
struct FnFormShape {
    /// The flat-vector of triples `name <- :T name <- :T ...`.
    params_vector: Vec<WatAST>,
    /// The keyword text of the return type (e.g. `:wat::core::i64`).
    ret_type_kw: String,
    /// The (possibly do-wrapped) body AST after the signature.
    /// Held for completeness / future shape assertions; current tests
    /// don't introspect the body (behavior equivalence covers it).
    #[allow(dead_code)]
    body: WatAST,
}

fn assert_entry_form_fn_form(entry_form: &WatAST) -> FnFormShape {
    let items = match entry_form {
        WatAST::List(items, _) => items,
        other => panic!("expected entry_form to be a List (fn-form); got {:?}", other),
    };
    assert!(
        items.len() >= 5,
        "fn-form must have >= 5 elements (head, args-vec, ->, :Ret, body); got {}",
        items.len()
    );
    match &items[0] {
        WatAST::Keyword(k, _) => assert_eq!(
            k, ":wat::core::fn",
            "fn-form head must be :wat::core::fn; got {}",
            k
        ),
        other => panic!("fn-form head must be Keyword; got {:?}", other),
    }
    let params_vector = match &items[1] {
        WatAST::Vector(v, _) => v.clone(),
        other => panic!(
            "fn-form args-position must be Vector [name <- :T ...]; got {:?}",
            other
        ),
    };
    match &items[2] {
        WatAST::Symbol(s, _) => assert_eq!(
            s.as_str(),
            "->",
            "fn-form must have `->` between args-vector and ret type"
        ),
        other => panic!("fn-form expected `->` Symbol; got {:?}", other),
    }
    let ret_type_kw = match &items[3] {
        WatAST::Keyword(k, _) => k.clone(),
        other => panic!("fn-form ret-type must be Keyword; got {:?}", other),
    };
    // Body is items[4]; if there were multiple body forms, the
    // closure-extract path passes a single (already-do-collapsed)
    // node — the rewriter doesn't re-wrap. Either way, items[4] is
    // the body node we hand to the consumer.
    let body = items[4].clone();
    FnFormShape {
        params_vector,
        ret_type_kw,
        body,
    }
}

/// Walk the params-vector in chunks of 3 (name <- :T) and return
/// (param-name, param-type-kw) pairs.
fn fn_form_param_pairs(shape: &FnFormShape) -> Vec<(String, String)> {
    let mut out = Vec::new();
    let mut i = 0;
    while i + 2 < shape.params_vector.len() {
        let name = match &shape.params_vector[i] {
            WatAST::Symbol(s, _) => s.name.clone(),
            other => panic!("fn-form param[{}] name must be Symbol; got {:?}", i / 3, other),
        };
        match &shape.params_vector[i + 1] {
            WatAST::Symbol(s, _) => assert_eq!(
                s.as_str(),
                "<-",
                "fn-form param[{}] must have `<-` arrow",
                i / 3
            ),
            other => panic!("fn-form param[{}] arrow slot must be Symbol; got {:?}", i / 3, other),
        }
        let ty = match &shape.params_vector[i + 2] {
            WatAST::Keyword(k, _) => k.clone(),
            other => panic!("fn-form param[{}] type slot must be Keyword; got {:?}", i / 3, other),
        };
        out.push((name, ty));
        i += 3;
    }
    out
}

// ─── T1. top-level defn, no deps, no captures ───────────────────────────

#[test]
fn t1_toplevel_defn_no_deps_no_captures() {
    let src = r#"
        (:wat::core::define (:my::add-one (n :wat::core::i64) -> :wat::core::i64)
          (:wat::core::i64::+'2 n 1))
        (:wat::core::define (:user::main -> :wat::core::nil)
          :wat::core::nil)
    "#;
    let parent = freeze(src);
    let fn_value = lookup_fn(&parent, ":my::add-one");
    let package = extract(&parent, &fn_value, Some(":my::add-one"));
    // Keyword-path entry: entry_form is the Keyword reference.
    assert_entry_form_keyword(&package.entry_form, ":my::add-one");
    // No user types, no extra deps, no captures: prologue contains
    // exactly the entry's define (it ships in prologue as a regular
    // dep so the entry_form's Keyword AST resolves at eval-time).
    let names: Vec<String> = package.prologue.iter().filter_map(extract_define_name).collect();
    assert_eq!(
        names,
        vec![":my::add-one".to_string()],
        "expected prologue to contain only :my::add-one's define"
    );
    let fresh = re_freeze(package.prologue);
    let result =
        invoke_via_entry_form(&fresh, &package.entry_form, vec![Value::i64(41)]);
    assert_i64(&result, 42);
}

// ─── T2. top-level defn, calls other top-level defns ────────────────────

#[test]
fn t2_toplevel_defn_calls_other_defns() {
    let src = r#"
        (:wat::core::define (:my::times-two (n :wat::core::i64) -> :wat::core::i64)
          (:wat::core::i64::*'2 n 2))
        (:wat::core::define (:my::times-four (n :wat::core::i64) -> :wat::core::i64)
          (:my::times-two (:my::times-two n)))
        (:wat::core::define (:user::main -> :wat::core::nil)
          :wat::core::nil)
    "#;
    let parent = freeze(src);
    let fn_value = lookup_fn(&parent, ":my::times-four");
    let package = extract(&parent, &fn_value, Some(":my::times-four"));
    assert_entry_form_keyword(&package.entry_form, ":my::times-four");
    // Should contain :my::times-two then :my::times-four in
    // topological order (times-two before times-four; entry's
    // define lands last as a regular dep).
    let names: Vec<String> = package
        .prologue
        .iter()
        .filter_map(extract_define_name)
        .collect();
    assert_eq!(
        names,
        vec![":my::times-two".to_string(), ":my::times-four".to_string()],
        "expected topological order with entry last in prologue"
    );
    let fresh = re_freeze(package.prologue);
    let result =
        invoke_via_entry_form(&fresh, &package.entry_form, vec![Value::i64(3)]);
    assert_i64(&result, 12);
}

// ─── T3. top-level defn uses user types ─────────────────────────────────

#[test]
fn t3_toplevel_defn_uses_user_types() {
    let src = r#"
        (:wat::core::struct :my::Point
          (x :wat::core::i64)
          (y :wat::core::i64))
        (:wat::core::enum :my::Side
          :Left
          :Right)
        (:wat::core::newtype :my::PriceUsd :wat::core::f64)
        (:wat::core::typealias :my::Coord :wat::core::i64)
        (:wat::core::define (:my::compute (p :my::Point) -> :wat::core::i64)
          (:wat::core::i64::+'2 (:my::Point/x p) (:my::Point/y p)))
        (:wat::core::define (:user::main -> :wat::core::nil)
          :wat::core::nil)
    "#;
    let parent = freeze(src);
    let fn_value = lookup_fn(&parent, ":my::compute");
    let package = extract(&parent, &fn_value, Some(":my::compute"));
    assert_entry_form_keyword(&package.entry_form, ":my::compute");
    // The fn signature mentions `:my::Point`; expect that struct + the
    // accessor fn to be in the prologue.
    let type_decls = collect_type_decl_names(&package.prologue);
    assert!(type_decls.contains(&":my::Point".to_string()),
            "Point struct must be extracted; got {:?}", type_decls);
    // PriceUsd, Side, Coord are not referenced — should NOT be extracted.
    assert!(!type_decls.contains(&":my::PriceUsd".to_string()));
    assert!(!type_decls.contains(&":my::Side".to_string()));
    assert!(!type_decls.contains(&":my::Coord".to_string()));
    let fresh = re_freeze(package.prologue);
    // Build a Point in the fresh world directly via the constructor.
    let new_func = fresh.symbols().get(":my::Point/new").expect("Point/new").clone();
    let point = apply_function(
        new_func,
        vec![Value::i64(3), Value::i64(4)],
        fresh.symbols(),
        Span::unknown(),
    )
    .expect("Point/new ok");
    let result = invoke_via_entry_form(&fresh, &package.entry_form, vec![point]);
    assert_i64(&result, 7);
}

// ─── T4. inline lambda, no captures ─────────────────────────────────────

#[test]
fn t4_inline_lambda_no_captures() {
    // Factory returns a lambda; we extract it.
    let src = r#"
        (:wat::core::define (:my::factory -> :wat::core::Fn(wat::core::i64)->wat::core::i64)
          (:wat::core::fn [n <- :wat::core::i64] -> :wat::core::i64
            (:wat::core::i64::+'2 n 7)))
        (:wat::core::define (:user::main -> :wat::core::nil)
          :wat::core::nil)
    "#;
    let parent = freeze(src);
    let lambda = synth_lambda(&parent, ":my::factory");
    let package = extract(&parent, &lambda, None);
    // Inline lambda: entry_form is the reconstructed fn-form AST.
    let shape = assert_entry_form_fn_form(&package.entry_form);
    let pairs = fn_form_param_pairs(&shape);
    assert_eq!(
        pairs,
        vec![("n".to_string(), ":wat::core::i64".to_string())],
        "fn-form param signature mismatch"
    );
    assert_eq!(shape.ret_type_kw, ":wat::core::i64");
    // Prologue should be empty (no types, no captures, no deps).
    assert!(
        package.prologue.is_empty(),
        "expected empty prologue for no-capture lambda; got {:#?}",
        package.prologue
    );
    // Behavior equivalence.
    let parent_result = invoke_in_parent(&parent, &lambda, vec![Value::i64(1)]);
    let fresh = re_freeze(package.prologue);
    let fresh_result =
        invoke_via_entry_form(&fresh, &package.entry_form, vec![Value::i64(1)]);
    assert_i64(&parent_result, 8);
    assert_i64(&fresh_result, 8);
}

// ─── T5. inline lambda captures let-scope value ─────────────────────────

#[test]
fn t5_inline_lambda_captures_let_scope_struct() {
    let src = r#"
        (:wat::core::struct :my::Config
          (offset :wat::core::i64))
        (:wat::core::define (:my::make-adder -> :wat::core::Fn(wat::core::i64)->wat::core::i64)
          (:wat::core::let
            [cfg (:my::Config/new 10)]
            (:wat::core::fn [n <- :wat::core::i64] -> :wat::core::i64
              (:wat::core::i64::+'2 n (:my::Config/offset cfg)))))
        (:wat::core::define (:user::main -> :wat::core::nil)
          :wat::core::nil)
    "#;
    let parent = freeze(src);
    let lambda = synth_lambda(&parent, ":my::make-adder");
    let package = extract(&parent, &lambda, None);
    // Expect: type def for :my::Config, capture binding for `cfg` in
    // prologue. entry_form is a fn-form AST.
    let shape = assert_entry_form_fn_form(&package.entry_form);
    let pairs = fn_form_param_pairs(&shape);
    assert_eq!(
        pairs,
        vec![("n".to_string(), ":wat::core::i64".to_string())]
    );
    assert_eq!(shape.ret_type_kw, ":wat::core::i64");
    let type_decls = collect_type_decl_names(&package.prologue);
    assert!(type_decls.contains(&":my::Config".to_string()));
    let captures = collect_def_names(&package.prologue);
    assert!(captures.iter().any(|n| n.starts_with(":__captured_cfg")),
            "expected `cfg` capture; got {:?}", captures);
    // Behavior equivalence.
    let fresh = re_freeze(package.prologue);
    let parent_result = invoke_in_parent(&parent, &lambda, vec![Value::i64(5)]);
    let fresh_result =
        invoke_via_entry_form(&fresh, &package.entry_form, vec![Value::i64(5)]);
    assert_i64(&parent_result, 15);
    assert_i64(&fresh_result, 15);
}

// ─── T6. lambda captures multiple values, mixed types ───────────────────

#[test]
fn t6_lambda_captures_multiple_mixed_types() {
    let src = r#"
        (:wat::core::struct :my::Cfg
          (label :wat::core::String))
        (:wat::core::define (:my::make-multi -> :wat::core::Fn(wat::core::i64)->wat::core::i64)
          (:wat::core::let
            [n 7
             cfg (:my::Cfg/new "ok")
             xs (:wat::core::Vector :wat::core::i64 1 2 3)]
            (:wat::core::fn [m <- :wat::core::i64] -> :wat::core::i64
              (:wat::core::i64::+'2 m
                (:wat::core::i64::+'2 n
                  (:wat::core::Vector/length xs))))))
        (:wat::core::define (:user::main -> :wat::core::nil)
          :wat::core::nil)
    "#;
    let parent = freeze(src);
    let lambda = synth_lambda(&parent, ":my::make-multi");
    let package = extract(&parent, &lambda, None);
    // entry_form is fn-form AST; prologue holds types + captures.
    let _shape = assert_entry_form_fn_form(&package.entry_form);
    let captures = collect_def_names(&package.prologue);
    // We expect captures for n and xs (cfg may also be captured even
    // though the body doesn't reference it — capture collection is
    // driven by closed_env). Verify n and xs are present; cfg is
    // optional.
    assert!(captures.iter().any(|c| c.starts_with(":__captured_n")),
            "missing :__captured_n; got {:?}", captures);
    assert!(captures.iter().any(|c| c.starts_with(":__captured_xs")),
            "missing :__captured_xs; got {:?}", captures);
    let fresh = re_freeze(package.prologue);
    // n=7, length(xs)=3, m=10 => 10+7+3 = 20.
    let result =
        invoke_via_entry_form(&fresh, &package.entry_form, vec![Value::i64(10)]);
    assert_i64(&result, 20);
}

// ─── T7. factory pattern ────────────────────────────────────────────────

#[test]
fn t7_factory_pattern() {
    let src = r#"
        (:wat::core::struct :my::Cfg
          (val :wat::core::i64))
        (:wat::core::define
          (:my::factory (config :my::Cfg) -> :wat::core::Fn(wat::core::i64)->wat::core::i64)
          (:wat::core::fn [n <- :wat::core::i64] -> :wat::core::i64
            (:wat::core::i64::+'2 n (:my::Cfg/val config))))
        (:wat::core::define (:my::make -> :wat::core::Fn(wat::core::i64)->wat::core::i64)
          (:my::factory (:my::Cfg/new 100)))
        (:wat::core::define (:user::main -> :wat::core::nil)
          :wat::core::nil)
    "#;
    let parent = freeze(src);
    let lambda = synth_lambda(&parent, ":my::make");
    let package = extract(&parent, &lambda, None);
    // entry_form is fn-form AST (factory result is a synthesized
    // lambda; it has no canonical name).
    let _shape = assert_entry_form_fn_form(&package.entry_form);
    let captures = collect_def_names(&package.prologue);
    assert!(captures.iter().any(|c| c.starts_with(":__captured_config")),
            "expected `config` capture (the factory's arg); got {:?}", captures);
    let fresh = re_freeze(package.prologue);
    let result =
        invoke_via_entry_form(&fresh, &package.entry_form, vec![Value::i64(7)]);
    assert_i64(&result, 107);
}

// ─── T8. lambda captures non-portable Sender (NEGATIVE) ─────────────────

#[test]
fn t8_lambda_captures_sender_is_non_portable() {
    // The lambda captures `tx` (a Sender) by closing over it but
    // never reads/writes the channel — the send call would trigger
    // CommCallOutOfPosition at type-check, which is a separate
    // discipline. We're only testing extraction's portability gate
    // here. Capturing the Sender in the closed env is enough to
    // surface NonPortableCapture.
    let src = r#"
        (:wat::core::define
          (:my::make-snd -> :wat::core::Fn(wat::core::i64)->wat::core::i64)
          (:wat::core::let
            [[tx rx] (:wat::kernel::make-bounded-channel :wat::core::i64 1)
             dropped rx]
            (:wat::core::fn [n <- :wat::core::i64] -> :wat::core::i64
              (:wat::core::do
                tx
                n))))
        (:wat::core::define (:user::main -> :wat::core::nil)
          :wat::core::nil)
    "#;
    let parent = freeze(src);
    let lambda = synth_lambda(&parent, ":my::make-snd");
    let err = extract_err(&parent, &lambda, None);
    match &err {
        ExtractionError::NonPortableCapture { name, type_name, path: _ } => {
            assert_eq!(name, "tx");
            assert!(type_name.contains("Sender"), "type_name={}", type_name);
        }
        other => panic!("expected NonPortableCapture; got {:?}", other),
    }
    // Verify the Display rendering carries the substrate-as-teacher
    // diagnostic. The report shape mandates a verbatim sample.
    let msg = format!("{}", err);
    assert!(msg.contains("`tx`"), "missing capture name: {}", msg);
    assert!(msg.contains("Sender"), "missing type: {}", msg);
    assert!(msg.contains("Channel-bearing types cannot cross"),
            "missing teacher hint: {}", msg);
    assert!(msg.contains("stdin/stdout/stderr"),
            "missing pipes pointer: {}", msg);
}

// ─── T9. captured struct holds Sender field (NEGATIVE) ──────────────────
//
// Slice 1 surfaces this case if the substrate admits a struct holding a
// Sender field. Since the substrate's struct field-types are validated
// against TypeEnv at type-check, defining such a struct requires the
// Sender type be admissible. Lab-side substrate admits this; the
// extraction surface refuses it.

#[test]
fn t9_captured_struct_holds_sender_field_nested() {
    // The substrate admits structs holding kernel-channel types as
    // fields (the type system has Sender<T> as a parametric type).
    // The captured value is a struct; encoding walks fields and the
    // Sender field surfaces as NonPortableCapture.
    let src = r#"
        (:wat::core::struct :my::Pack
          (tx :wat::kernel::Sender<wat::core::i64>))
        (:wat::core::define
          (:my::make-pack -> :wat::core::Fn(wat::core::i64)->wat::core::i64)
          (:wat::core::let
            [[tx rx] (:wat::kernel::make-bounded-channel :wat::core::i64 1)
             pack (:my::Pack/new tx)
             unused rx]
            (:wat::core::fn [n <- :wat::core::i64] -> :wat::core::i64
              (:wat::core::do pack n))))
        (:wat::core::define (:user::main -> :wat::core::nil)
          :wat::core::nil)
    "#;
    let parent = match startup_from_source(src, None, Arc::new(InMemoryLoader::new())) {
        Ok(w) => w,
        Err(_e) => {
            // Substrate may refuse Sender<T> as a struct field type.
            // In that case T9 is vacuous — the lab can't construct
            // the failing input shape. Skip via early return.
            eprintln!("T9 skipped: substrate rejects Sender<T> as struct field");
            return;
        }
    };
    let lambda = synth_lambda(&parent, ":my::make-pack");
    let err = extract_err(&parent, &lambda, None);
    match err {
        ExtractionError::NonPortableCapture { name: _, type_name, path } => {
            assert!(type_name.contains("Sender"), "type_name={}", type_name);
            assert!(!path.is_empty(), "expected nested path naming the offending field");
        }
        other => panic!("expected NonPortableCapture; got {:?}", other),
    }
}

// ─── T10. captures with type alias ──────────────────────────────────────

#[test]
fn t10_captures_with_type_alias() {
    let src = r#"
        (:wat::core::typealias :my::Coord :wat::core::i64)
        (:wat::core::define (:my::compute (c :my::Coord) -> :wat::core::i64)
          (:wat::core::i64::+'2 c 1))
        (:wat::core::define (:user::main -> :wat::core::nil)
          :wat::core::nil)
    "#;
    let parent = freeze(src);
    let fn_value = lookup_fn(&parent, ":my::compute");
    let package = extract(&parent, &fn_value, Some(":my::compute"));
    assert_entry_form_keyword(&package.entry_form, ":my::compute");
    let type_decls = collect_type_decl_names(&package.prologue);
    assert!(type_decls.contains(&":my::Coord".to_string()),
            "expected :my::Coord to be extracted; got {:?}", type_decls);
    let fresh = re_freeze(package.prologue);
    let result =
        invoke_via_entry_form(&fresh, &package.entry_form, vec![Value::i64(41)]);
    assert_i64(&result, 42);
}

// ─── T11. recursive struct (via Vec<Self>) ──────────────────────────────

#[test]
fn t11_captures_with_recursive_struct() {
    // Recursive type via Vector — `:my::Tree` holds a `:Vector<:my::Tree>`.
    let src = r#"
        (:wat::core::struct :my::Tree
          (value :wat::core::i64)
          (children :wat::core::Vector<my::Tree>))
        (:wat::core::define (:my::root-value (t :my::Tree) -> :wat::core::i64)
          (:my::Tree/value t))
        (:wat::core::define (:user::main -> :wat::core::nil)
          :wat::core::nil)
    "#;
    let parent = freeze(src);
    let fn_value = lookup_fn(&parent, ":my::root-value");
    let package = extract(&parent, &fn_value, Some(":my::root-value"));
    assert_entry_form_keyword(&package.entry_form, ":my::root-value");
    let type_decls = collect_type_decl_names(&package.prologue);
    let count_tree = type_decls.iter().filter(|n| *n == ":my::Tree").count();
    assert_eq!(count_tree, 1, "Tree must appear exactly once; got {:?}", type_decls);
    let fresh = re_freeze(package.prologue);
    let new_func = fresh.symbols().get(":my::Tree/new").expect("Tree/new").clone();
    let empty_children = Value::Vec(Arc::new(Vec::new()));
    let tree = apply_function(
        new_func,
        vec![Value::i64(99), empty_children],
        fresh.symbols(),
        Span::unknown(),
    )
    .expect("Tree/new ok");
    let result = invoke_via_entry_form(&fresh, &package.entry_form, vec![tree]);
    assert_i64(&result, 99);
}

// ─── T12. body uses macro that expanded to substrate primitives only ────

#[test]
fn t12_body_uses_expanded_substrate_primitive_macro() {
    // `:wat::core::cond` is a defmacro that expands to substrate
    // primitives. After expansion, the body references only :wat::core::*.
    // We verify the body's expanded form makes it through extraction
    // and re-freezes cleanly.
    let src = r#"
        (:wat::core::define (:my::classify (n :wat::core::i64) -> :wat::core::String)
          (:wat::core::cond -> :wat::core::String
            ((:wat::core::< n 0) "negative")
            ((:wat::core::= n 0) "zero")
            (:else "positive")))
        (:wat::core::define (:user::main -> :wat::core::nil)
          :wat::core::nil)
    "#;
    let parent = freeze(src);
    let fn_value = lookup_fn(&parent, ":my::classify");
    let package = extract(&parent, &fn_value, Some(":my::classify"));
    assert_entry_form_keyword(&package.entry_form, ":my::classify");
    let fresh = re_freeze(package.prologue);
    let r1 = invoke_via_entry_form(&fresh, &package.entry_form, vec![Value::i64(-5)]);
    assert_string(&r1, "negative");
    let r2 = invoke_via_entry_form(&fresh, &package.entry_form, vec![Value::i64(0)]);
    assert_string(&r2, "zero");
    let r3 = invoke_via_entry_form(&fresh, &package.entry_form, vec![Value::i64(7)]);
    assert_string(&r3, "positive");
}

// ─── T13. body uses user-defined macro ──────────────────────────────────

#[test]
fn t13_body_uses_user_defined_macro_post_expansion() {
    // User defmacro expands to a substrate-primitive call. Post
    // expansion the body references only substrate; the user macro
    // itself does NOT need to be in `package.prologue` (no runtime
    // dependency).
    let src = r#"
        (:wat::core::defmacro (:my::triple (x))
          (:wat::core::quasiquote
            (:wat::core::i64::*'2 (:wat::core::unquote x) 3)))
        (:wat::core::define (:my::compute (n :wat::core::i64) -> :wat::core::i64)
          (:my::triple n))
        (:wat::core::define (:user::main -> :wat::core::nil)
          :wat::core::nil)
    "#;
    let parent = freeze(src);
    let fn_value = lookup_fn(&parent, ":my::compute");
    let package = extract(&parent, &fn_value, Some(":my::compute"));
    assert_entry_form_keyword(&package.entry_form, ":my::compute");
    // The user macro `:my::triple` is post-expanded; the body has no
    // reference to it. The package should NOT include a defmacro form.
    for form in &package.prologue {
        if let WatAST::List(items, _) = form {
            if let Some(WatAST::Keyword(k, _)) = items.first() {
                assert_ne!(k, ":wat::core::defmacro",
                           "macro defs should NOT be in the closure package");
            }
        }
    }
    let fresh = re_freeze(package.prologue);
    let result = invoke_via_entry_form(&fresh, &package.entry_form, vec![Value::i64(4)]);
    assert_i64(&result, 12);
}

// ─── T14. transitive 3-level dep chain ──────────────────────────────────

#[test]
fn t14_transitive_three_level_dep_chain() {
    let src = r#"
        (:wat::core::define (:my::a (n :wat::core::i64) -> :wat::core::i64)
          (:wat::core::i64::+'2 n 1))
        (:wat::core::define (:my::b (n :wat::core::i64) -> :wat::core::i64)
          (:my::a (:my::a n)))
        (:wat::core::define (:my::c (n :wat::core::i64) -> :wat::core::i64)
          (:my::b (:my::b n)))
        (:wat::core::define (:user::main -> :wat::core::nil)
          :wat::core::nil)
    "#;
    let parent = freeze(src);
    let fn_value = lookup_fn(&parent, ":my::c");
    let package = extract(&parent, &fn_value, Some(":my::c"));
    assert_entry_form_keyword(&package.entry_form, ":my::c");
    let names: Vec<String> = package.prologue.iter().filter_map(extract_define_name).collect();
    // Topological order: a before b before c. Entry's define lands
    // last in prologue (it's a regular dep that the entry_form's
    // Keyword AST resolves to).
    let pa = names.iter().position(|n| n == ":my::a").expect("a missing");
    let pb = names.iter().position(|n| n == ":my::b").expect("b missing");
    let pc = names.iter().position(|n| n == ":my::c").expect("c missing");
    assert!(pa < pb && pb < pc, "expected topological a<b<c; got {:?}", names);
    let fresh = re_freeze(package.prologue);
    let result = invoke_via_entry_form(&fresh, &package.entry_form, vec![Value::i64(0)]);
    // c(0) = b(b(0)) = b(a(a(0))) = b(2) = a(a(2)) = 4 ; b(b(0)) = b(2) = 4
    // c(0) calls b twice: b(b(0)). b(0) = a(a(0)) = 2. b(2) = a(a(2)) = 4.
    assert_i64(&result, 4);
}

// ─── T15. behavior equivalence end-to-end across T1-T7 ──────────────────

#[test]
fn t15_behavior_equivalence_across_shapes() {
    // Re-run the extraction + re-freeze for several of the shapes
    // from T1-T7 and verify the end-to-end output matches original
    // invocation in every case.
    //
    // T1 — top-level defn no captures.
    let src1 = r#"
        (:wat::core::define (:my::add-one (n :wat::core::i64) -> :wat::core::i64)
          (:wat::core::i64::+'2 n 1))
        (:wat::core::define (:user::main -> :wat::core::nil) :wat::core::nil)
    "#;
    let p1 = freeze(src1);
    let f1 = lookup_fn(&p1, ":my::add-one");
    let pkg1 = extract(&p1, &f1, Some(":my::add-one"));
    let fr1 = re_freeze(pkg1.prologue);
    for x in &[-5_i64, 0, 17, 99] {
        let parent = invoke_in_parent(&p1, &f1, vec![Value::i64(*x)]);
        let fresh =
            invoke_via_entry_form(&fr1, &pkg1.entry_form, vec![Value::i64(*x)]);
        match (parent, fresh) {
            (Value::i64(a), Value::i64(b)) => assert_eq!(a, b, "input {}", x),
            other => panic!("non-i64: {:?}", other),
        }
    }
    // T5 — captures struct.
    let src5 = r#"
        (:wat::core::struct :my::Config (offset :wat::core::i64))
        (:wat::core::define (:my::make-adder -> :wat::core::Fn(wat::core::i64)->wat::core::i64)
          (:wat::core::let
            [cfg (:my::Config/new 99)]
            (:wat::core::fn [n <- :wat::core::i64] -> :wat::core::i64
              (:wat::core::i64::+'2 n (:my::Config/offset cfg)))))
        (:wat::core::define (:user::main -> :wat::core::nil) :wat::core::nil)
    "#;
    let p5 = freeze(src5);
    let lambda5 = synth_lambda(&p5, ":my::make-adder");
    let pkg5 = extract(&p5, &lambda5, None);
    let fr5 = re_freeze(pkg5.prologue);
    for x in &[-3_i64, 0, 100] {
        let parent = invoke_in_parent(&p5, &lambda5, vec![Value::i64(*x)]);
        let fresh =
            invoke_via_entry_form(&fr5, &pkg5.entry_form, vec![Value::i64(*x)]);
        match (parent, fresh) {
            (Value::i64(a), Value::i64(b)) => assert_eq!(a, b, "input {}", x),
            other => panic!("non-i64: {:?}", other),
        }
    }
}

// ─── T16-T21. Slice 1d — match-arm + wildcard binder coverage ──────────
//
// Slice 1d extends `walk_free_symbols` so `(:wat::core::match scrut -> :T
// (pattern body) ...)` introduces pattern bindings into the arm body's
// scope. Pre-slice-1d, every name bound by a match-arm pattern surfaced
// as a free symbol — the 162 deftest_* failures the brief tracks.
//
// Tests assert each pattern-binding category does NOT surface as a
// free symbol; the package re-freezes; behavior matches.
//
// Helper that drives extraction expecting the entry to be a
// keyword-path defn whose body uses match.
fn extract_and_invoke(src: &str, entry: &str, args: Vec<Value>) -> (Value, Value) {
    let parent = freeze(src);
    let fn_value = lookup_fn(&parent, entry);
    let package = extract(&parent, &fn_value, Some(entry));
    let parent_result = invoke_in_parent(&parent, &fn_value, args.clone());
    let fresh = re_freeze(package.prologue);
    let fresh_result = invoke_via_entry_form(&fresh, &package.entry_form, args);
    (parent_result, fresh_result)
}

// ─── T16. match arm with `(:wat::core::Some name)` pattern binding ─────

#[test]
fn t16_match_some_pattern_binds_name() {
    // Body uses `(match opt -> :i64 ((Some n) n) (None 0))`. Pre-fix,
    // `n` surfaced as a free symbol; post-fix, `n` is bound by the
    // arm pattern and resolves locally.
    let src = r#"
        (:wat::core::define
          (:my::option-or-zero (opt :wat::core::Option<wat::core::i64>) -> :wat::core::i64)
          (:wat::core::match opt -> :wat::core::i64
            ((:wat::core::Some n) n)
            (:wat::core::None    0)))
        (:wat::core::define (:user::main -> :wat::core::nil) :wat::core::nil)
    "#;
    let parent = freeze(src);
    let fn_value = lookup_fn(&parent, ":my::option-or-zero");
    let package = extract(&parent, &fn_value, Some(":my::option-or-zero"));
    let fresh = re_freeze(package.prologue);
    // Some(7) → 7
    let some_seven = Value::Option(Arc::new(Some(Value::i64(7))));
    let r = invoke_via_entry_form(&fresh, &package.entry_form, vec![some_seven]);
    assert_i64(&r, 7);
    // None → 0
    let none = Value::Option(Arc::new(None));
    let r = invoke_via_entry_form(&fresh, &package.entry_form, vec![none]);
    assert_i64(&r, 0);
}

// ─── T17. match arm with `_` wildcard does not surface as free ─────────

#[test]
fn t17_match_wildcard_does_not_surface_as_free() {
    // The `_` wildcard binds nothing; pre-fix, `_` was pushed onto the
    // free-symbol queue and triggered UnresolvedSymbol. Post-fix, `_`
    // is filtered at the Symbol arm and ignored at pattern position.
    let src = r#"
        (:wat::core::define
          (:my::is-some? (opt :wat::core::Option<wat::core::i64>) -> :wat::core::bool)
          (:wat::core::match opt -> :wat::core::bool
            ((:wat::core::Some _) true)
            (:wat::core::None     false)))
        (:wat::core::define (:user::main -> :wat::core::nil) :wat::core::nil)
    "#;
    let (parent_v, fresh_v) = extract_and_invoke(
        src,
        ":my::is-some?",
        vec![Value::Option(Arc::new(Some(Value::i64(42))))],
    );
    match (parent_v, fresh_v) {
        (Value::bool(a), Value::bool(b)) => {
            assert_eq!(a, true);
            assert_eq!(b, true);
        }
        other => panic!("expected bool match; got {:?}", other),
    }
    // None case
    let (parent_v, fresh_v) = extract_and_invoke(
        src,
        ":my::is-some?",
        vec![Value::Option(Arc::new(None))],
    );
    match (parent_v, fresh_v) {
        (Value::bool(a), Value::bool(b)) => {
            assert_eq!(a, false);
            assert_eq!(b, false);
        }
        other => panic!("expected bool match; got {:?}", other),
    }
}

// ─── T18. match arm with `(:Ok b)` / `(:Err _)` Result patterns ────────

#[test]
fn t18_match_result_patterns_bind_arm_names() {
    // Both Ok and Err patterns; Ok-arm binds `b`, Err-arm has wildcard.
    // This is the dominant shape in the failing eval-coincident tests.
    let src = r#"
        (:wat::core::define
          (:my::unwrap-or-false (r :wat::core::Result<wat::core::bool,wat::core::String>) -> :wat::core::bool)
          (:wat::core::match r -> :wat::core::bool
            ((:wat::core::Ok b)  b)
            ((:wat::core::Err _) false)))
        (:wat::core::define (:user::main -> :wat::core::nil) :wat::core::nil)
    "#;
    let (p, f) = extract_and_invoke(
        src,
        ":my::unwrap-or-false",
        vec![Value::Result(Arc::new(Ok(Value::bool(true))))],
    );
    match (p, f) {
        (Value::bool(a), Value::bool(b)) => {
            assert!(a && b, "Ok(true) → true");
        }
        other => panic!("non-bool: {:?}", other),
    }
    let (p, f) = extract_and_invoke(
        src,
        ":my::unwrap-or-false",
        vec![Value::Result(Arc::new(Err(Value::String(Arc::new(
            "boom".to_string(),
        )))))],
    );
    match (p, f) {
        (Value::bool(a), Value::bool(b)) => {
            assert!(!a && !b, "Err(_) → false");
        }
        other => panic!("non-bool: {:?}", other),
    }
}

// ─── T19. nested match: arm body uses an inner let referencing arm-bound name

#[test]
fn t19_match_arm_body_with_inner_let() {
    // The arm body opens an inner let whose RHS uses the arm-bound
    // name. Pre-fix, the inner let walked under the OUTER scope (no
    // arm bindings) and `i` surfaced as free. The time.wat /
    // iso8601 tests exercise exactly this shape.
    let src = r#"
        (:wat::core::define
          (:my::inc-or-default (opt :wat::core::Option<wat::core::i64>) -> :wat::core::i64)
          (:wat::core::match opt -> :wat::core::i64
            ((:wat::core::Some i)
             (:wat::core::let
               [s (:wat::core::i64::+'2 i 1)]
               s))
            (:wat::core::None 0)))
        (:wat::core::define (:user::main -> :wat::core::nil) :wat::core::nil)
    "#;
    let (p, f) = extract_and_invoke(
        src,
        ":my::inc-or-default",
        vec![Value::Option(Arc::new(Some(Value::i64(41))))],
    );
    match (p, f) {
        (Value::i64(a), Value::i64(b)) => {
            assert_eq!(a, 42);
            assert_eq!(b, 42);
        }
        other => panic!("non-i64: {:?}", other),
    }
    let (p, f) =
        extract_and_invoke(src, ":my::inc-or-default", vec![Value::Option(Arc::new(None))]);
    match (p, f) {
        (Value::i64(a), Value::i64(b)) => {
            assert_eq!(a, 0);
            assert_eq!(b, 0);
        }
        other => panic!("non-i64: {:?}", other),
    }
}

// ─── T20. match arm with user-enum tagged variant pulls the enum into prologue

#[test]
fn t20_match_user_enum_variant_records_type_dep() {
    // Pattern `(:my::Color::Red)` etc. is a unit-variant; pattern
    // `(:my::Shape::Rect (w h))` is a tagged variant whose payload
    // sub-patterns introduce two bindings. The user-enum's type defn
    // must land in prologue (closure-extraction's existing
    // unit-variants resolution stays); the bindings must NOT surface
    // as free symbols.
    let src = r#"
        (:wat::core::enum :my::Shape
          (Rect
            (w :wat::core::i64)
            (h :wat::core::i64))
          (Circle
            (r :wat::core::i64)))
        (:wat::core::define
          (:my::shape-area (s :my::Shape) -> :wat::core::i64)
          (:wat::core::match s -> :wat::core::i64
            ((:my::Shape::Rect w h) (:wat::core::i64::*'2 w h))
            ((:my::Shape::Circle r) (:wat::core::i64::*'2 r r))))
        (:wat::core::define (:user::main -> :wat::core::nil) :wat::core::nil)
    "#;
    let parent = freeze(src);
    let fn_value = lookup_fn(&parent, ":my::shape-area");
    let package = extract(&parent, &fn_value, Some(":my::shape-area"));
    // Type defn must be in prologue.
    let type_names = collect_type_decl_names(&package.prologue);
    assert!(
        type_names.contains(&":my::Shape".to_string()),
        "expected :my::Shape in prologue type defs; got {:?}",
        type_names
    );
    // Re-freeze + invoke.
    let fresh = re_freeze(package.prologue);
    // Build an enum value Rect(3, 4) → 12. Use the parent's accessors
    // to construct via apply_function.
    let rect_ctor = parent.symbols().get(":my::Shape::Rect").expect("Rect ctor");
    let rect = apply_function(
        rect_ctor.clone(),
        vec![Value::i64(3), Value::i64(4)],
        parent.symbols(),
        Span::unknown(),
    )
    .expect("Rect/new");
    let res = invoke_via_entry_form(&fresh, &package.entry_form, vec![rect]);
    assert_i64(&res, 12);
}

// ─── T21. Pattern bindings shadow outer let-scope names ────────────────

#[test]
fn t21_match_arm_binding_shadows_outer_let() {
    // Outer let binds `n`; match arm's pattern introduces a fresh
    // `n` that shadows. Body uses arm-bound `n`. Pre-fix the walker
    // had `n` in outer locals so no false free-symbol fire — but
    // post-fix we still need the shadowing to be a no-op (locals are
    // BTreeSet so re-inserting an already-bound name is harmless).
    let src = r#"
        (:wat::core::define
          (:my::shadow-test (opt :wat::core::Option<wat::core::i64>) -> :wat::core::i64)
          (:wat::core::let
            [n 100]
            (:wat::core::match opt -> :wat::core::i64
              ((:wat::core::Some n) n)
              (:wat::core::None     n))))
        (:wat::core::define (:user::main -> :wat::core::nil) :wat::core::nil)
    "#;
    let (p, f) = extract_and_invoke(
        src,
        ":my::shadow-test",
        vec![Value::Option(Arc::new(Some(Value::i64(7))))],
    );
    match (p, f) {
        (Value::i64(a), Value::i64(b)) => {
            assert_eq!(a, 7);
            assert_eq!(b, 7);
        }
        other => panic!("non-i64: {:?}", other),
    }
    let (p, f) = extract_and_invoke(
        src,
        ":my::shadow-test",
        vec![Value::Option(Arc::new(None))],
    );
    match (p, f) {
        (Value::i64(a), Value::i64(b)) => {
            // None arm: `n` resolves to outer let's `n` = 100.
            assert_eq!(a, 100);
            assert_eq!(b, 100);
        }
        other => panic!("non-i64: {:?}", other),
    }
}

// ─── helpers for form inspection ────────────────────────────────────────

/// Pull the canonical name out of a `(:wat::core::define <sig> body)`
/// form. Returns None for non-define forms.
fn extract_define_name(form: &WatAST) -> Option<String> {
    if let WatAST::List(items, _) = form {
        if items.len() == 3 {
            if let Some(WatAST::Keyword(head, _)) = items.first() {
                if head == ":wat::core::define" {
                    if let WatAST::List(sig_items, _) = &items[1] {
                        if let Some(WatAST::Keyword(name, _)) = sig_items.first() {
                            // Strip any `<T,U>` suffix; canonical name is
                            // the keyword path without type-params.
                            let canonical = match name.find('<') {
                                Some(idx) => name[..idx].to_string(),
                                None => name.clone(),
                            };
                            return Some(canonical);
                        }
                    }
                }
            }
        }
    }
    None
}

/// Pull the binding name out of a `(:wat::core::def :name expr)` form.
fn extract_def_name(form: &WatAST) -> Option<String> {
    if let WatAST::List(items, _) = form {
        if items.len() == 3 {
            if let Some(WatAST::Keyword(head, _)) = items.first() {
                if head == ":wat::core::def" {
                    if let WatAST::Keyword(name, _) = &items[1] {
                        return Some(name.clone());
                    }
                }
            }
        }
    }
    None
}

fn collect_def_names(forms: &[WatAST]) -> Vec<String> {
    forms.iter().filter_map(extract_def_name).collect()
}

/// Pull the names of every type declaration form (`struct`/`enum`/
/// `newtype`/`typealias`) out of a forms vec, stripping any `<T>` suffix.
fn collect_type_decl_names(forms: &[WatAST]) -> Vec<String> {
    forms
        .iter()
        .filter_map(|form| {
            if let WatAST::List(items, _) = form {
                if items.len() >= 2 {
                    if let Some(WatAST::Keyword(head, _)) = items.first() {
                        let is_type_decl = matches!(
                            head.as_str(),
                            ":wat::core::struct"
                                | ":wat::core::enum"
                                | ":wat::core::newtype"
                                | ":wat::core::typealias"
                        );
                        if is_type_decl {
                            if let WatAST::Keyword(name, _) = &items[1] {
                                let canonical = match name.find('<') {
                                    Some(idx) => name[..idx].to_string(),
                                    None => name.clone(),
                                };
                                return Some(canonical);
                            }
                        }
                    }
                }
            }
            None
        })
        .collect()
}
