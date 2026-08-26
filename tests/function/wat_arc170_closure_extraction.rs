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
use wat::closure_extract::{extract_closure, ClosurePackage};
use wat::freeze::{startup_from_file, startup_from_forms};
use wat::runtime::{apply_function, eval, Environment, Value};

// ─── helpers ────────────────────────────────────────────────────────────

fn freeze(path: &str) -> wat::freeze::FrozenWorld {
    startup_from_file(path)
        .expect("parent freeze should succeed")
}

fn re_freeze(forms: Vec<WatAST>) -> wat::freeze::FrozenWorld {
    startup_from_forms(forms, None, Arc::new(wat::load::loader::InMemoryLoader::new()))
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
        .expect("entry_form eval should succeed").value_owned();
    let func = match fn_value {
        Value::wat__core__fn(f) => f,
        other => panic!("entry_form did not evaluate to a fn Value; got {:?}", other),
    };
    apply_function(func, args, fresh.symbols(), wat::rust_caller_span!())
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
    apply_function(func, args, world.symbols(), wat::rust_caller_span!())
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
    apply_function(factory, Vec::new(), world.symbols(), wat::rust_caller_span!())
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
            WatAST::Symbol(s, _) => s.as_str().to_owned(),
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
    let parent = freeze("tests/function/wat_arc170_closure_extraction_t1.wat");
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
    let parent = freeze("tests/function/wat_arc170_closure_extraction_t2.wat");
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
    let parent = freeze("tests/function/wat_arc170_closure_extraction_t3.wat");
    let fn_value = lookup_fn(&parent, ":my::compute");
    let package = extract(&parent, &fn_value, Some(":my::compute"));
    assert_entry_form_keyword(&package.entry_form, ":my::compute");
    // The fn signature mentions `:my::Point`; expect that struct + the
    // accessor fn to be in the prologue.
    let type_decls = collect_type_decl_names(&package.prologue);
    assert!(type_decls.iter().any(|x| x == ":my::Point"),
            "Point struct must be extracted; got {:?}", type_decls);
    // Arc 170 slice 3 Gap F-3: extract_closure now sweeps the WHOLE parent
    // user type registry into the prologue, not just types statically
    // referenced by the fn signature / body. PriceUsd, Side, and Coord are
    // not referenced by :my::compute, but they ARE in the parent's TypeEnv
    // and therefore appear in the prologue after the Gap F-3 fix. This
    // ensures the child subprocess's TypeEnv matches the parent's for any
    // dynamic type lookup (e.g., edn::read on tagged EDN forms).
    assert!(type_decls.iter().any(|x| x == ":my::PriceUsd"),
            "PriceUsd must be in prologue (whole-registry sweep); got {:?}", type_decls);
    assert!(type_decls.iter().any(|x| x == ":my::Side"),
            "Side must be in prologue (whole-registry sweep); got {:?}", type_decls);
    assert!(type_decls.iter().any(|x| x == ":my::Coord"),
            "Coord must be in prologue (whole-registry sweep); got {:?}", type_decls);
    let fresh = re_freeze(package.prologue);
    // Build a Point in the fresh world directly via the constructor.
    let new_func = fresh.symbols().get(":my::Point'").expect("Point ctor (positional prime)").clone();
    let point = apply_function(
        new_func,
        vec![Value::i64(3), Value::i64(4)],
        fresh.symbols(),
        wat::rust_caller_span!(),
    )
    .expect("Point/new ok");
    let result = invoke_via_entry_form(&fresh, &package.entry_form, vec![point]);
    assert_i64(&result, 7);
}

// ─── T4. inline lambda, no captures ─────────────────────────────────────

#[test]
fn t4_inline_lambda_no_captures() {
    // Factory returns a lambda; we extract it.
    let parent = freeze("tests/function/wat_arc170_closure_extraction_t4.wat");
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
    let parent = freeze("tests/function/wat_arc170_closure_extraction_t5.wat");
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
    assert!(type_decls.iter().any(|x| x == ":my::Config"));
    let captures = collect_def_names(&package.prologue);
    assert!(captures.iter().any(|n| n == ":user::closure-capture::cfg"),
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
    let parent = freeze("tests/function/wat_arc170_closure_extraction_t6.wat");
    let lambda = synth_lambda(&parent, ":my::make-multi");
    let package = extract(&parent, &lambda, None);
    // entry_form is fn-form AST; prologue holds types + captures.
    let _shape = assert_entry_form_fn_form(&package.entry_form);
    let captures = collect_def_names(&package.prologue);
    // We expect captures for n and xs (cfg may also be captured even
    // though the body doesn't reference it — capture collection is
    // driven by closed_env). Verify n and xs are present; cfg is
    // optional.
    assert!(captures.iter().any(|c| c == ":user::closure-capture::n"),
            "missing :user::closure-capture::n; got {:?}", captures);
    assert!(captures.iter().any(|c| c == ":user::closure-capture::xs"),
            "missing :user::closure-capture::xs; got {:?}", captures);
    let fresh = re_freeze(package.prologue);
    // n=7, length(xs)=3, m=10 => 10+7+3 = 20.
    let result =
        invoke_via_entry_form(&fresh, &package.entry_form, vec![Value::i64(10)]);
    assert_i64(&result, 20);
}

// ─── T7. factory pattern ────────────────────────────────────────────────

#[test]
fn t7_factory_pattern() {
    let parent = freeze("tests/function/wat_arc170_closure_extraction_t7.wat");
    let lambda = synth_lambda(&parent, ":my::make");
    let package = extract(&parent, &lambda, None);
    // entry_form is fn-form AST (factory result is a synthesized
    // lambda; it has no canonical name).
    let _shape = assert_entry_form_fn_form(&package.entry_form);
    let captures = collect_def_names(&package.prologue);
    assert!(captures.iter().any(|c| c == ":user::closure-capture::config"),
            "expected `config` capture (the factory's arg); got {:?}", captures);
    let fresh = re_freeze(package.prologue);
    let result =
        invoke_via_entry_form(&fresh, &package.entry_form, vec![Value::i64(7)]);
    assert_i64(&result, 107);
}

// ─── T10. captures with type alias ──────────────────────────────────────

#[test]
fn t10_captures_with_type_alias() {
    let parent = freeze("tests/function/wat_arc170_closure_extraction_t10.wat");
    let fn_value = lookup_fn(&parent, ":my::compute");
    let package = extract(&parent, &fn_value, Some(":my::compute"));
    assert_entry_form_keyword(&package.entry_form, ":my::compute");
    let type_decls = collect_type_decl_names(&package.prologue);
    assert!(type_decls.iter().any(|x| x == ":my::Coord"),
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
    let parent = freeze("tests/function/wat_arc170_closure_extraction_t11.wat");
    let fn_value = lookup_fn(&parent, ":my::root-value");
    let package = extract(&parent, &fn_value, Some(":my::root-value"));
    assert_entry_form_keyword(&package.entry_form, ":my::root-value");
    let type_decls = collect_type_decl_names(&package.prologue);
    let count_tree = type_decls.iter().filter(|n| *n == ":my::Tree").count();
    assert_eq!(count_tree, 1, "Tree must appear exactly once; got {:?}", type_decls);
    let fresh = re_freeze(package.prologue);
    let new_func = fresh.symbols().get(":my::Tree'").expect("Tree ctor (positional prime)").clone();
    let empty_children = Value::Vec(Arc::new(Vec::new()));
    let tree = apply_function(
        new_func,
        vec![Value::i64(99), empty_children],
        fresh.symbols(),
        wat::rust_caller_span!(),
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
    let parent = freeze("tests/function/wat_arc170_closure_extraction_t12.wat");
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
    let parent = freeze("tests/function/wat_arc170_closure_extraction_t13.wat");
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
    let parent = freeze("tests/function/wat_arc170_closure_extraction_t14.wat");
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
    let p1 = freeze("tests/function/wat_arc170_closure_extraction_t1.wat");
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
    // T5 — captures struct, offset=99.
    let p5 = freeze("tests/function/wat_arc170_closure_extraction_t15b.wat");
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
fn extract_and_invoke(path: &str, entry: &str, args: Vec<Value>) -> (Value, Value) {
    let parent = freeze(path);
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
    let parent = freeze("tests/function/wat_arc170_closure_extraction_t16.wat");
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
    let (parent_v, fresh_v) = extract_and_invoke(
        "tests/function/wat_arc170_closure_extraction_t17.wat",
        ":my::is-some?",
        vec![Value::Option(Arc::new(Some(Value::i64(42))))],
    );
    match (parent_v, fresh_v) {
        (Value::bool(a), Value::bool(b)) => {
            assert!(a);
            assert!(b);
        }
        other => panic!("expected bool match; got {:?}", other),
    }
    // None case
    let (parent_v, fresh_v) = extract_and_invoke(
        "tests/function/wat_arc170_closure_extraction_t17.wat",
        ":my::is-some?",
        vec![Value::Option(Arc::new(None))],
    );
    match (parent_v, fresh_v) {
        (Value::bool(a), Value::bool(b)) => {
            assert!(!a);
            assert!(!b);
        }
        other => panic!("expected bool match; got {:?}", other),
    }
}

// ─── T18. match arm with `(:Ok b)` / `(:Err _)` Result patterns ────────

#[test]
fn t18_match_result_patterns_bind_arm_names() {
    // Both Ok and Err patterns; Ok-arm binds `b`, Err-arm has wildcard.
    // This is the dominant shape in the failing eval-coincident tests.
    let (p, f) = extract_and_invoke(
        "tests/function/wat_arc170_closure_extraction_t18.wat",
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
        "tests/function/wat_arc170_closure_extraction_t18.wat",
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
    let (p, f) = extract_and_invoke(
        "tests/function/wat_arc170_closure_extraction_t19.wat",
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
        extract_and_invoke(
            "tests/function/wat_arc170_closure_extraction_t19.wat",
            ":my::inc-or-default",
            vec![Value::Option(Arc::new(None))],
        );
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
    let parent = freeze("tests/function/wat_arc170_closure_extraction_t20.wat");
    let fn_value = lookup_fn(&parent, ":my::shape-area");
    let package = extract(&parent, &fn_value, Some(":my::shape-area"));
    // Type defn must be in prologue.
    let type_names = collect_type_decl_names(&package.prologue);
    assert!(
        type_names.iter().any(|x| x == ":my::Shape"),
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
        wat::rust_caller_span!(),
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
    let (p, f) = extract_and_invoke(
        "tests/function/wat_arc170_closure_extraction_t21.wat",
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
        "tests/function/wat_arc170_closure_extraction_t21.wat",
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

// ─── T22. top-level defn references a `def`-bound value ─────────────────

/// RED gate for the `def` arm (`closure_extract.rs`'s Keyword walker).
///
/// A top-level `def` read from a fn body currently raises
/// `Internal("captured `def`-bound name … not yet supported by closure
/// extraction (slice 1)")`. That arm's own comment says a future arc opens
/// IFF a caller surfaces wanting it — `defservice` is that caller: every
/// op's `:max-request-bytes` becomes a top-level `def`, so `fn-forms` on a
/// service's `serve` cannot complete.
///
/// The def must ride in the prologue under its ORIGINAL name: the body
/// references it by Keyword, and `rewrite_captures` rewrites only
/// bare-Symbol locals, never Keyword paths.
#[test]
fn t22_toplevel_defn_references_def_bound_value() {
    let parent = freeze("tests/function/wat_arc170_closure_extraction_t22.wat");
    let fn_value = lookup_fn(&parent, ":my::plus-limit");
    let package = extract(&parent, &fn_value, Some(":my::plus-limit"));
    assert_entry_form_keyword(&package.entry_form, ":my::plus-limit");

    // Exact, not a membership check: this fixture captures no locals, so
    // `:my::LIMIT` is the ONLY def the prologue may carry. An exact compare
    // also catches a spurious extra def — a duplicate emission, or a capture
    // synthesised where none belongs — that `contains` would wave through.
    assert_eq!(
        collect_def_names(&package.prologue),
        vec![":my::LIMIT".to_string()],
        "prologue must carry exactly :my::LIMIT's def, under its original name"
    );

    // The gate that matters: the extracted package must STAND ALONE in a
    // fresh world. A prologue that names the def but does not bind it would
    // pass the shape assert above and die here.
    let fresh = re_freeze(package.prologue);
    let result = invoke_via_entry_form(&fresh, &package.entry_form, vec![Value::i64(8)]);
    assert_i64(&result, 520);
}

// ─── helpers for form inspection ────────────────────────────────────────

/// Pull the canonical name out of a `(:wat::core::defn :name [binders] -> :ret body)`
/// form. Returns None for non-defn forms.
///
/// Stone 241.11 — updated from `:wat::core::define` (old 3-item form) to
/// `:wat::core::defn` (6-item form); closure_extract.rs now emits defn.
fn extract_define_name(form: &WatAST) -> Option<String> {
    if let WatAST::List(items, _) = form {
        // defn shape: [head, :name, [binders], ->, :ret, body] = 6 items
        if items.len() == 6 {
            if let Some(WatAST::Keyword(head, _)) = items.first() {
                if head == ":wat::core::defn" {
                    if let Some(WatAST::Keyword(name, _)) = items.get(1) {
                        // `<K,V>` is unexpressible (arc 109 ③'s wall, `src/types.rs:4688`)
                        // — no keyword the reader hands back ever carries a `<...>`
                        // suffix, so `name` is already canonical; used directly, never
                        // stripped (arc 109 "reap the twelve" — found by widening the
                        // rune, not by the original census).
                        return Some(name.clone());
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
/// `newtype`/`typealias`) out of a forms vec.
fn collect_type_decl_names(forms: &[WatAST]) -> Vec<String> {
    forms
        .iter()
        .filter_map(|form| {
            if let WatAST::List(items, _) = form {
                if items.len() >= 2 {
                    if let Some(WatAST::Keyword(head, _)) = items.first() {
                        let is_type_decl = matches!(
                            head.as_str(),
                            ":wat::core::defstruct"
                                | ":wat::core::defenum"
                                | ":wat::core::newtype"
                                | ":wat::core::typealias"
                                // Arc 170 — freeze now ships each user type's RETAINED
                                // source form (captured at registration, post-macroexpansion),
                                // so struct/record/enum sugar arrives under the PRIMITIVE heads
                                // the sugar macros expand to. Recognize those too.
                                | ":wat::core::structtype"
                                | ":wat::core::recordtype"
                                | ":wat::core::aggregatetype"
                        );
                        if is_type_decl {
                            if let WatAST::Keyword(name, _) = &items[1] {
                                // `<K,V>` is unexpressible (arc 109 ③'s wall,
                                // `src/types.rs:4688`) — no keyword the reader hands
                                // back ever carries a `<...>` suffix, so `name` is
                                // already canonical; used directly, never stripped
                                // (arc 109 "reap the twelve" — found by widening the
                                // rune, not by the original census).
                                return Some(name.clone());
                            }
                        }
                    }
                }
            }
            None
        })
        .collect()
}
