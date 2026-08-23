use super::*;
use super::expand;
use crate::ast::WatAST;
use crate::scope::Identifier;

/// Parse `src`, register defmacros, and return `(registry, rest_forms, env, sym)`.
/// Panics on parse or registration failure — for success-path setup only.
/// Tests that exercise error paths call `expand_src` directly (which propagates errors).
fn expand_setup(src: &str) -> (MacroRegistry, Vec<WatAST>, crate::runtime::Environment, crate::runtime::SymbolTable) {
    let forms = crate::parse_all!(src).expect("parse ok");
    let mut reg = MacroRegistry::new();
    let rest = register_defmacros(forms, &mut reg).expect("register_defmacros ok");
    let env = crate::runtime::Environment::default();
    let sym = crate::runtime::SymbolTable::default();
    (reg, rest, env, sym)
}

fn expand_src(src: &str) -> super::ExpandBatch {
    let forms = crate::parse_all!(src).expect("parse ok");
    let mut reg = MacroRegistry::new();
    let rest = register_defmacros(forms, &mut reg)?;
    let env = crate::runtime::Environment::default();
    let sym = crate::runtime::SymbolTable::default();
    expand_all(rest, &mut reg, &env, &sym)
}

/// Like `expand`, but DOES NOT strip generated defmacros from the
/// output. Arc 029 slice 1 tests use this to inspect the body of
/// a defmacro produced by an outer macro-generating-macro call.
fn expand_keeping_defmacros(src: &str) -> super::ExpandBatch {
    let forms = crate::parse_all!(src).expect("parse ok");
    let mut reg = MacroRegistry::new();
    let rest = register_defmacros(forms, &mut reg)?;
    let env = crate::runtime::Environment::default();
    let sym = crate::runtime::SymbolTable::default();
    let mut out = Vec::with_capacity(rest.len());
    for form in rest {
        out.push(expand::expand_form(form, &mut reg, 0, &env, &sym, crate::resolve::Privilege::User)?);
    }
    Ok(out)
}

// ─── Arc 278: stdlib registration privilege (expansion-born :wat:: macros) ───

/// The `expand_all` stdlib privilege: a `:wat::`-prefixed defmacro — what a baked
/// `defservice`'s expansion-born `…/start` companion looks like — registers via
/// `register` ONLY when the registry is stdlib-privileged (set around the stdlib
/// expansion pass in `freeze/env.rs`). User expansion stays gated, so a mis-namespaced
/// user macro still halts. Before this, a baked `:wat::` defservice broke stdlib load.
#[test]
fn stdlib_privilege_bypasses_reserved_prefix_on_register() {
    let forms = crate::parse_all!(
        "(:wat::core::defmacro :wat::query::probe-privilege [] -> :wat::WatAST (:wat::core::quasiquote :ok))"
    )
    .expect("parse ok");
    let def = crate::macros::parse::parse_defmacro_form(forms.into_iter().next().unwrap())
        .expect("parse defmacro");

    // Unprivileged (the user-expansion path): a :wat:: macro still halts — the gate holds.
    let mut reg = MacroRegistry::new();
    match reg.register(def.clone(), crate::resolve::Privilege::User) {
        Err(MacroError { kind: MacroErrorKind::ReservedPrefix(_), .. }) => {}
        other => panic!("expected ReservedPrefix without privilege; got {other:?}"),
    }

    // Privileged (the stdlib-expansion path): the same :wat:: macro registers — the fix.
    reg.register(def, crate::resolve::Privilege::Stdlib)
        .expect("a :wat:: macro must register when stdlib-privileged");
}

// ─── Quasiquote discriminant regression (item 1) ───────────────────

/// A defmacro whose body is `(:wat::core::quasiquote a b)` (quasiquote head
/// but wrong arity — 2 body forms instead of 1) must fail with
/// `MalformedTemplate` at expansion time, not silently misroute to the
/// program-body path. Regression for the parse-vs-expand discriminant unification.
#[test]
fn malformed_quasiquote_body_wrong_arity_fails_with_malformed_template() {
    let err = expand_src(
        r#"
        (:wat::core::defmacro :my::bad-quasi
          [x <- :wat::WatAST]
          -> :wat::WatAST
          (:wat::core::quasiquote a b))
        (:my::bad-quasi 1)
        "#,
    )
    .unwrap_err();
    assert!(
        matches!(err, MacroError { kind: MacroErrorKind::MalformedTemplate { .. }, .. }),
        "expected MalformedTemplate for (:wat::core::quasiquote a b) body (wrong arity); got: {:?}",
        err
    );
}

// ─── expand_keeping_defmacros contract ─────────────────────────────

/// Contract: a source whose expansion emits a defmacro form shows up in
/// `expand_keeping_defmacros`'s output but NOT in `expand_src`'s output.
/// Also proves that `expand_keeping_defmacros` uses `expand_form` directly
/// (not `expand_all`): the generated defmacro is preserved verbatim rather
/// than being registered and stripped.
#[test]
fn expand_keeping_defmacros_keeps_vs_expand_src_strips() {
    // A macro-generating-macro: invoking `:my::mkmac` produces a defmacro form.
    let src = r#"
    (:wat::core::defmacro :my::mkmac
      [name <- :wat::WatAST]
      -> :wat::WatAST
      `(:wat::core::defmacro
         ~name
         []
         -> :wat::WatAST
         `(:sentinel)))
    (:my::mkmac :my::generated)
    "#;

    // expand_keeping_defmacros: the generated defmacro survives in output.
    let kept = expand_keeping_defmacros(src).expect("expand_keeping_defmacros ok");
    assert_eq!(kept.len(), 1, "one form in output (the generated defmacro)");
    assert!(
        matches!(&kept[0], WatAST::List(items, _)
            if matches!(items.first(), Some(WatAST::Keyword(k, _)) if k == ":wat::core::defmacro")),
        "expand_keeping_defmacros must preserve the generated defmacro in output; got: {:?}",
        kept[0]
    );

    // expand_src: the generated defmacro is registered and stripped — no output forms.
    let stripped = expand_src(src).expect("expand_src ok");
    assert_eq!(
        stripped.len(), 0,
        "expand_src must strip the generated defmacro (registers it instead); got: {:?}",
        stripped
    );
}

// ─── Pure alias macro ───────────────────────────────────────────────

#[test]
fn alias_macro_expands_to_primitive() {
    let forms = expand_src(
        r#"
        (:wat::core::defmacro :my::vocab::Concurrent
          [xs <- :wat::WatAST]
          -> :wat::WatAST
          `(:wat::holon::Bundle ~xs))
        (:my::vocab::Concurrent (:wat::core::Vector :wat::holon::HolonAST a b c))
        "#,
    )
    .unwrap();
    assert_eq!(forms.len(), 1);
    // Expansion: (:wat::holon::Bundle (:wat::core::Vector :wat::holon::HolonAST a b c))
    match &forms[0] {
        WatAST::List(items, _) => {
            assert_eq!(items.len(), 2);
            assert!(matches!(&items[0], WatAST::Keyword(k, _) if k == ":wat::holon::Bundle"));
        }
        _ => panic!("expected List after expansion"),
    }
}

// ─── Transforming macro with multiple params ────────────────────────

#[test]
fn subtract_macro_expansion() {
    let forms = expand_src(
        r#"
        (:wat::core::defmacro :my::vocab::Subtract
          [x <- :wat::WatAST
           y <- :wat::WatAST]
          -> :wat::WatAST
          `(:wat::holon::Blend ~x ~y 1 -1))
        (:my::vocab::Subtract foo bar)
        "#,
    )
    .unwrap();
    // (:wat::holon::Blend foo bar 1 -1)
    match &forms[0] {
        WatAST::List(items, _) => {
            assert_eq!(items.len(), 5);
            assert!(matches!(&items[0], WatAST::Keyword(k, _) if k == ":wat::holon::Blend"));
            assert!(matches!(&items[1], WatAST::Symbol(i, _) if i.as_str() == "foo"));
            assert!(matches!(&items[2], WatAST::Symbol(i, _) if i.as_str() == "bar"));
            assert!(matches!(items[3], WatAST::IntLit(1, _)));
            assert!(matches!(items[4], WatAST::IntLit(-1, _)));
        }
        _ => panic!("expected List"),
    }
}

// ─── Unquote-splicing ───────────────────────────────────────────────

#[test]
fn splice_list_arg_into_template() {
    let forms = expand_src(
        r#"
        (:wat::core::defmacro :my::vocab::SumAll
          [xs <- :wat::WatAST]
          -> :wat::WatAST
          `(:wat::holon::Bundle ~@xs))
        (:my::vocab::SumAll (a b c))
        "#,
    )
    .unwrap();
    // (:wat::holon::Bundle a b c) — the list elements are spliced in.
    match &forms[0] {
        WatAST::List(items, _) => {
            assert_eq!(items.len(), 4);
            assert!(matches!(&items[0], WatAST::Keyword(k, _) if k == ":wat::holon::Bundle"));
            assert!(matches!(&items[1], WatAST::Symbol(i, _) if i.as_str() == "a"));
            assert!(matches!(&items[2], WatAST::Symbol(i, _) if i.as_str() == "b"));
            assert!(matches!(&items[3], WatAST::Symbol(i, _) if i.as_str() == "c"));
        }
        _ => panic!("expected List"),
    }
}

// ─── Nested macros (fixpoint) ───────────────────────────────────────

#[test]
fn nested_macro_expands_to_fixpoint() {
    let forms = expand_src(
        r#"
        (:wat::core::defmacro :my::outer [x <- :wat::WatAST] -> :wat::WatAST `(:my::inner ~x))
        (:wat::core::defmacro :my::inner [x <- :wat::WatAST] -> :wat::WatAST `(:wat::holon::Atom ~x))
        (:my::outer 42)
        "#,
    )
    .unwrap();
    // (:wat::holon::Atom 42) after fixpoint.
    match &forms[0] {
        WatAST::List(items, _) => {
            assert_eq!(items.len(), 2);
            assert!(matches!(&items[0], WatAST::Keyword(k, _) if k == ":wat::holon::Atom"));
            assert!(matches!(items[1], WatAST::IntLit(42, _)));
        }
        _ => panic!("expected List"),
    }
}

// ─── Hygiene — template-origin identifiers get the macro scope ─────

/// Drill into a plain `(:wat::core::let ((binder val) ...) …)` form and return
/// the first binder Identifier. Shape: `form[1]` = bindings-list; `bindings[0]` =
/// pair; `pair[0]` = Symbol. Each arm carries a descriptive panic message.
fn drill_let_binder_ident(form: &WatAST) -> &Identifier {
    let list = match form {
        WatAST::List(items, _) => items,
        _ => panic!("drill_let_binder_ident: expected let-List; got non-List"),
    };
    let bindings = match &list[1] {
        WatAST::List(bs, _) => bs,
        _ => panic!("drill_let_binder_ident: expected bindings-list at list[1]; got non-List"),
    };
    let pair = match &bindings[0] {
        WatAST::List(b, _) => b,
        _ => panic!("drill_let_binder_ident: expected binding-pair at bindings[0]; got non-List"),
    };
    match &pair[0] {
        WatAST::Symbol(i, _) => i,
        _ => panic!("drill_let_binder_ident: expected Symbol at pair[0]; got non-Symbol"),
    }
}

#[test]
fn drill_let_binder_ident_on_minimal_form() {
    // Prove the helper on a hand-built (:let ((tmp 1)) tmp) form.
    let span = crate::rust_caller_span!();
    let tmp_ident = Identifier::bare("tmp");
    let pair = WatAST::List(
        vec![
            WatAST::Symbol(tmp_ident.clone(), span.clone()),
            WatAST::IntLit(1, span.clone()),
        ],
        span.clone(),
    );
    let bindings = WatAST::List(vec![pair], span.clone());
    let form = WatAST::List(
        vec![
            WatAST::Keyword(":wat::core::let".into(), span.clone()),
            bindings,
            WatAST::Symbol(tmp_ident.clone(), span.clone()),
        ],
        span.clone(),
    );
    let extracted = drill_let_binder_ident(&form);
    assert_eq!(extracted.as_str(), "tmp");
    assert_eq!(extracted, &tmp_ident);
}

#[test]
fn template_identifier_carries_macro_scope() {
    let forms = expand_src(
        r#"
        (:wat::core::defmacro :my::vocab::WithTmp
          [body <- :wat::WatAST]
          -> :wat::WatAST
          `(:wat::core::let ((tmp 1)) ~body))
        (:my::vocab::WithTmp tmp)
        "#,
    )
    .unwrap();
    // Expansion: (:wat::core::let ((tmp[macro-scope] 1)) tmp[user-empty])
    // The two `tmp`s must have DIFFERENT Identifiers.
    let template_tmp = drill_let_binder_ident(&forms[0]);
    // The body position's `tmp` — user-supplied argument, not macro-origin.
    let list = match &forms[0] {
        WatAST::List(items, _) => items,
        _ => panic!("expected list"),
    };
    let user_tmp = match &list[2] {
        WatAST::Symbol(i, _) => i,
        _ => panic!("expected Symbol in body"),
    };
    assert_eq!(template_tmp.as_str(), "tmp");
    assert_eq!(user_tmp.as_str(), "tmp");
    assert!(
        !template_tmp.scopes().is_empty(),
        "template tmp must have macro scope attached"
    );
    assert!(
        user_tmp.scopes().is_empty(),
        "user-argument tmp must NOT have the macro scope"
    );
    assert_ne!(
        template_tmp, user_tmp,
        "template and user tmp must be DIFFERENT Identifiers"
    );
}

// ─── walk_template uniformity: binder + body-ref carry identical scope sets ──
//
// This is the load-bearing premise for `env_key` exact-match resolution:
// `walk_template` adds ONE macro scope uniformly to EVERY template-origin
// identifier in a single pass; a binder (`let [tmp …]`) and every reference
// to it in the same template (`tmp` in the body) must therefore carry EXACTLY
// the SAME scope set — so `env_key(binder) == env_key(body_ref)` and the
// runtime lookup succeeds.

#[test]
fn binder_and_reference_carry_identical_scope_sets() {
    let forms = expand_src(
        r#"
        (:wat::core::defmacro :my::vocab::WalkUniformity
          []
          -> :wat::WatAST
          `(:wat::core::let ((tmp 1)) tmp))
        (:my::vocab::WalkUniformity)
        "#,
    )
    .unwrap();
    // Expansion shape: (:wat::core::let ((tmp 1)) tmp)
    //   list[0] = :wat::core::let keyword
    //   list[1] = bindings list ((tmp 1))
    //   list[2] = body reference `tmp`
    let binder = drill_let_binder_ident(&forms[0]);
    let list = match &forms[0] {
        WatAST::List(items, _) => items,
        _ => panic!("expected let list"),
    };
    // list[2] = body reference `tmp`
    let body_ref = match &list[2] {
        WatAST::Symbol(i, _) => i,
        _ => panic!("expected Symbol at body-reference position"),
    };
    assert_eq!(binder.as_str(), "tmp");
    assert_eq!(body_ref.as_str(), "tmp");
    assert!(
        !binder.scopes().is_empty(),
        "binder `tmp` must carry the macro scope (non-empty scope set)"
    );
    assert!(
        !body_ref.scopes().is_empty(),
        "body-reference `tmp` must carry the macro scope (non-empty scope set)"
    );
    assert_eq!(
        binder.scopes(),
        body_ref.scopes(),
        "binder and body-reference must carry IDENTICAL scope sets; \
         any divergence means env_key(binder) ≠ env_key(body_ref) → lookup failure"
    );
}

// ─── Argument identifiers are preserved unchanged ──────────────────

#[test]
fn argument_identifiers_pass_through_unchanged() {
    // User passes a symbol; the macro should splice it verbatim.
    let forms = expand_src(
        r#"
        (:wat::core::defmacro :my::wrap [v <- :wat::WatAST] -> :wat::WatAST `(:wat::holon::Atom ~v))
        (:my::wrap some-var)
        "#,
    )
    .unwrap();
    let list = match &forms[0] {
        WatAST::List(items, _) => items,
        _ => panic!("expected list"),
    };
    let v_arg = match &list[1] {
        WatAST::Symbol(i, _) => i,
        _ => panic!("expected Symbol at arg position"),
    };
    // Argument identifier — no macro scope added.
    assert_eq!(v_arg.as_str(), "some-var");
    assert!(
        v_arg.scopes().is_empty(),
        "argument identifier should have no macro scope"
    );
}

// ─── Classic capture: two macros introduce the same template name ─

/// Extract the typed-binding symbol from a `(:wat::core::let (((t :i64) val)) …)` form.
///
/// Drills: outer-list[1] = bindings-list → bindings[0] = pair → pair[0] = typed-name-list
/// → typed-name[0] = Symbol. Each arm carries a descriptive panic message naming the layer.
fn extract_typed_binding_sym(form: &WatAST) -> Identifier {
    let outer = match form {
        WatAST::List(items, _) => items,
        _ => panic!("extract_typed_binding_sym: expected outer let-List; got non-List"),
    };
    let bindings = match &outer[1] {
        WatAST::List(b, _) => b,
        _ => panic!("extract_typed_binding_sym: expected bindings-list at outer[1]; got non-List"),
    };
    let pair = match &bindings[0] {
        WatAST::List(b, _) => b,
        _ => panic!("extract_typed_binding_sym: expected binding-pair at bindings[0]; got non-List"),
    };
    let typed_name = match &pair[0] {
        WatAST::List(tn, _) => tn,
        _ => panic!("extract_typed_binding_sym: expected typed-name-list at pair[0]; got non-List"),
    };
    match &typed_name[0] {
        WatAST::Symbol(i, _) => i.clone(),
        _ => panic!("extract_typed_binding_sym: expected Symbol at typed_name[0]; got non-Symbol"),
    }
}

#[test]
fn extract_typed_binding_sym_on_minimal_form() {
    // Build a minimal (:let (((t :i64) 1)) …) form by hand and assert the helper
    // returns the `t` Identifier — proving the helper on data we fully control.
    let span = crate::rust_caller_span!();
    let t_ident = Identifier::bare("t");
    let typed_name = WatAST::List(
        vec![
            WatAST::Symbol(t_ident.clone(), span.clone()),
            WatAST::Keyword(":i64".into(), span.clone()),
        ],
        span.clone(),
    );
    let pair = WatAST::List(
        vec![typed_name, WatAST::IntLit(1, span.clone())],
        span.clone(),
    );
    let bindings = WatAST::List(vec![pair], span.clone());
    let outer = WatAST::List(
        vec![
            WatAST::Keyword(":wat::core::let".into(), span.clone()),
            bindings,
            WatAST::Symbol(t_ident.clone(), span.clone()),
        ],
        span.clone(),
    );
    let extracted = extract_typed_binding_sym(&outer);
    assert_eq!(extracted.as_str(), "t");
    assert_eq!(extracted, t_ident);
}

#[test]
fn two_macro_invocations_get_distinct_scopes() {
    let forms = expand_src(
        r#"
        (:wat::core::defmacro :my::twice
          [x <- :wat::WatAST]
          -> :wat::WatAST
          `(:wat::core::let (((t :i64) ~x)) t))
        (:my::twice 1)
        (:my::twice 2)
        "#,
    )
    .unwrap();
    // Both expansions bind `t` in the template; each invocation should
    // tag its `t` with a FRESH scope. The two `t`s differ.
    let t1 = extract_typed_binding_sym(&forms[0]);
    let t2 = extract_typed_binding_sym(&forms[1]);
    assert_eq!(t1.as_str(), "t");
    assert_eq!(t2.as_str(), "t");
    assert_ne!(t1, t2, "each invocation should mint a fresh macro scope");
}

// ─── Error paths ────────────────────────────────────────────────────

#[test]
fn reserved_prefix_macro_rejected() {
    let err = expand_src(
        r#"(:wat::core::defmacro :wat::std::MyMacro [x <- :wat::WatAST] -> :wat::WatAST `~x)"#,
    )
    .unwrap_err();
    assert!(matches!(err, MacroError { kind: MacroErrorKind::ReservedPrefix(_), .. }));
}

#[test]
fn duplicate_defmacro_with_divergent_body_rejected() {
    // Arc 054: byte-equivalent re-declaration is a no-op (tested
    // separately). Divergent re-declaration still errors. This
    // test exercises the divergent-body path — same name, two
    // distinct templates.
    let err = expand_src(
        r#"
        (:wat::core::defmacro :my::m [x <- :wat::WatAST] -> :wat::WatAST `~x)
        (:wat::core::defmacro :my::m [x <- :wat::WatAST] -> :wat::WatAST `(:wat::core::Vector ~x))
        "#,
    )
    .unwrap_err();
    assert!(matches!(err, MacroError { kind: MacroErrorKind::DuplicateMacro(_), .. }));
}

#[test]
fn duplicate_defmacro_structurally_equivalent_is_noop() {
    // Arc 054: two structurally-equivalent defmacro forms — same name,
    // params, body. Second registration is a no-op. The macro
    // expands normally afterward.
    let result = expand_src(
        r#"
        (:wat::core::defmacro :my::m [x <- :wat::WatAST] -> :wat::WatAST `~x)
        (:wat::core::defmacro :my::m [x <- :wat::WatAST] -> :wat::WatAST `~x)
        (:my::m 42)
        "#,
    );
    assert!(result.is_ok(), "byte-equivalent re-decl should succeed; got {:?}", result);
}

#[test]
fn macro_arity_mismatch() {
    let err = expand_src(
        r#"
        (:wat::core::defmacro :my::two
          [x <- :wat::WatAST y <- :wat::WatAST]
          -> :wat::WatAST
          `(:wat::core::Vector ~x ~y))
        (:my::two 1)
        "#,
    )
    .unwrap_err();
    assert!(matches!(err, MacroError { kind: MacroErrorKind::ArityMismatch { .. }, .. }));
}

#[test]
fn variadic_macro_arity_too_few_uses_arity_too_few_variant() {
    // A variadic macro declares two fixed params + a rest-param.
    // Calling it with one arg (below the fixed minimum of 2) must
    // error with ArityTooFew and display "expects at least 2 arguments".
    let err = expand_src(
        r#"
        (:wat::core::defmacro :my::variadic
          [x <- :wat::WatAST y <- :wat::WatAST & rest <- (:wat::core::Vector :- [:wat::WatAST])]
          -> :wat::WatAST
          `(:wat::core::Vector ~x ~y ~@rest))
        (:my::variadic 1)
        "#,
    )
    .unwrap_err();
    assert!(
        matches!(err, MacroError { kind: MacroErrorKind::ArityTooFew { .. }, .. }),
        "expected ArityTooFew for variadic macro called with too-few args; got: {:?}",
        err
    );
    let rendered = format!("{}", err);
    // rune:lint(loose-assert) — Display embeds a Rust-derived source span in the error prefix
    // (e.g. "src/macros/tests.rs:N:col:end_col: macro ..."); the file path shifts if the test
    // file is moved or renamed, making full assert_eq! infeasible
    assert!(
        rendered.contains("expects at least 2 arguments"),
        "ArityTooFew Display must read 'expects at least 2 arguments'; got: {}",
        rendered
    );
}

#[test]
fn program_body_producing_non_ast_rejected() {
    // Arc 249 stone 249.2b-ii: a program body (non-quasiquote) IS evaluated
    // by macro_eval. A body that produces a non-AST result (e.g. a Vec) still
    // errors — with MalformedTemplate (value_to_watast rejects Vec).
    let err = expand_src(
        r#"
        (:wat::core::defmacro :my::m [x <- :wat::WatAST] -> :wat::WatAST
          (:wat::core::Vector :bogus x))
        (:my::m 1)
        "#,
    )
    .unwrap_err();
    assert!(matches!(err, MacroError { kind: MacroErrorKind::MalformedTemplate { .. }, .. }));
}

#[test]
fn splice_non_list_arg_rejected() {
    let err = expand_src(
        r#"
        (:wat::core::defmacro :my::s [xs <- :wat::WatAST] -> :wat::WatAST `(:wat::core::Vector ~@xs))
        (:my::s 42)
        "#,
    )
    .unwrap_err();
    assert!(matches!(err, MacroError { kind: MacroErrorKind::SpliceNotSequence { .. }, .. }));
}

// ─── Non-macro forms pass through unchanged ─────────────────────────

#[test]
fn non_macro_forms_unchanged() {
    let forms = expand_src(r#"(:wat::holon::Atom "hello") 42 "world""#).unwrap();
    assert_eq!(forms.len(), 3);
    assert!(matches!(forms[1], WatAST::IntLit(42, _)));
    assert!(matches!(&forms[2], WatAST::StringLit(s, _) if s == "world"));
}

// ─── Nested quasiquote — arc 029 slice 1 ────────────────────────────

/// Helper: find the `:wat::core::quasiquote` body inside a
/// `(:wat::core::defmacro ...)` form. Used by nested-quasi tests
/// to assert the generated macro's body.
fn find_defmacro_body(form: &WatAST) -> &WatAST {
    match form {
        WatAST::List(items, _) => {
            assert!(matches!(&items[0], WatAST::Keyword(k, _) if k == ":wat::core::defmacro"));
            // Stone 241.17 canonical shape (6 items):
            //   items[0] = :wat::core::defmacro head
            //   items[1] = macro name keyword
            //   items[2] = argspec Vector
            //   items[3] = -> symbol
            //   items[4] = return-type keyword
            //   items[5] = body — a (:wat::core::quasiquote ...)
            let body = &items[5];
            match body {
                WatAST::List(b, _) => {
                    assert!(matches!(&b[0],
                        WatAST::Keyword(k, _) if k == ":wat::core::quasiquote"));
                    &b[1]
                }
                _ => panic!("expected quasiquote body"),
            }
        }
        _ => panic!("expected defmacro list"),
    }
}

#[test]
fn find_defmacro_body_returns_quasiquote_inner() {
    // Build a minimal (:wat::core::defmacro :name [x <- :AST] -> :AST `body-sentinel)
    // by hand; assert find_defmacro_body returns the quasiquote inner (the body-sentinel).
    let span = crate::rust_caller_span!();
    let sentinel = WatAST::IntLit(999, span.clone());
    let quasi = WatAST::List(
        vec![
            WatAST::Keyword(":wat::core::quasiquote".into(), span.clone()),
            sentinel.clone(),
        ],
        span.clone(),
    );
    let defmacro_form = WatAST::List(
        vec![
            WatAST::Keyword(":wat::core::defmacro".into(), span.clone()),
            WatAST::Keyword(":my::name".into(), span.clone()),
            WatAST::Vector(vec![], span.clone()),
            WatAST::Symbol(Identifier::bare("->"), span.clone()),
            WatAST::Keyword(":AST".into(), span.clone()),
            quasi,
        ],
        span.clone(),
    );
    let inner = find_defmacro_body(&defmacro_form);
    assert!(
        matches!(inner, WatAST::IntLit(999, _)),
        "find_defmacro_body must return the quasiquote inner (the body content); got: {:?}",
        inner
    );
}

/// Helper: assert `form` is `(:wat::core::unquote <arg>)` and
/// return the inner arg.
fn expect_unquote(form: &WatAST) -> &WatAST {
    match form {
        WatAST::List(items, _) if items.len() == 2 => {
            assert!(matches!(&items[0],
                WatAST::Keyword(k, _) if k == ":wat::core::unquote"));
            &items[1]
        }
        _ => panic!("expected (:wat::core::unquote ...)"),
    }
}

#[test]
fn nested_quasiquote_preserves_inner_unquote() {
    // Outer macro body contains a nested quasiquote with an
    // unquote referencing an INNER parameter (not bound at outer
    // expansion). The unquote should survive into the generated
    // defmacro's body.
    let forms = expand_keeping_defmacros(
        r#"
        (:wat::core::defmacro :my::mkmac
          [name <- :wat::WatAST]
          -> :wat::WatAST
          `(:wat::core::defmacro
             ~name
             [x <- :wat::WatAST]
             -> :wat::WatAST
             `(:wat::holon::Atom ~x)))
        (:my::mkmac :my::wrap)
        "#,
    )
    .unwrap();
    // After outer expansion: a defmacro registration for :my::wrap
    // whose body is (:wat::core::quasiquote (:wat::holon::Atom
    // (:wat::core::unquote x))) — the inner `,x` preserved.
    let body = find_defmacro_body(&forms[0]);
    // body = (:wat::holon::Atom (:wat::core::unquote x))
    let body_items = match body {
        WatAST::List(items, _) => items,
        _ => panic!("expected list body"),
    };
    assert_eq!(body_items.len(), 2);
    assert!(matches!(&body_items[0],
        WatAST::Keyword(k, _) if k == ":wat::holon::Atom"));
    let inner = expect_unquote(&body_items[1]);
    assert!(matches!(inner, WatAST::Symbol(i, _) if i.as_str() == "x"));
}

#[test]
fn double_unquote_substitutes_at_outer_level() {
    // ,,X at depth 2: outer unquote drops to depth 1; inner
    // unquote at depth 1 substitutes X's outer binding. Result
    // is (:wat::core::unquote <value>) — the value sits wrapped
    // in an unquote that fires on the inner expansion pass.
    let forms = expand_keeping_defmacros(
        r#"
        (:wat::core::defmacro :my::mkmac
          [v <- :wat::WatAST]
          -> :wat::WatAST
          `(:wat::core::defmacro
             :my::configured
             []
             -> :wat::WatAST
             `(:wat::holon::Atom ~~v)))
        (:my::mkmac 42)
        "#,
    )
    .unwrap();
    let body = find_defmacro_body(&forms[0]);
    let body_items = match body {
        WatAST::List(items, _) => items,
        _ => panic!("expected list"),
    };
    assert_eq!(body_items.len(), 2);
    assert!(matches!(&body_items[0],
        WatAST::Keyword(k, _) if k == ":wat::holon::Atom"));
    // body_items[1] = (:wat::core::unquote 42) — the value
    // substituted at outer expansion.
    let inner = expect_unquote(&body_items[1]);
    assert!(matches!(inner, WatAST::IntLit(42, _)));
}

#[test]
fn unquote_of_literal_returns_literal() {
    // Direct check on unquote_argument: if the arg is already a
    // concrete value (from a prior substitution pass), return
    // as-is. Supports the `,,X` two-pass resolution.
    let bindings = std::collections::HashMap::new();
    let env = crate::runtime::Environment::default();
    let sym = crate::runtime::SymbolTable::default();
    // A literal int — not a symbol, no binding needed.
    let lit = WatAST::IntLit(99, crate::rust_caller_span!());
    let out = expand::unquote_argument(&lit, &bindings, &env, &sym).unwrap();
    match out {
        WatAST::IntLit(n, _) => assert_eq!(n, 99),
        _ => panic!("expected IntLit"),
    }
    // A list whose head is NOT a keyword — treated as already-substituted
    // literal (backward-compat heuristic: head must be Keyword to eval).
    let list = WatAST::List(
        vec![WatAST::IntLit(1, crate::rust_caller_span!()), WatAST::IntLit(2, crate::rust_caller_span!())],
        crate::rust_caller_span!(),
    );
    let out = expand::unquote_argument(&list, &bindings, &env, &sym).unwrap();
    assert!(matches!(out, WatAST::List(_, _)));
}

#[test]
fn unquote_splicing_at_depth_two_preserves() {
    // ,@X at depth 2: preserve the unquote-splicing wrapper,
    // walk X at depth 1. X is an inner-macro parameter, so
    // it should appear as-is (symbol) inside the preserved
    // wrapper.
    let forms = expand_keeping_defmacros(
        r#"
        (:wat::core::defmacro :my::mkmac
          [name <- :wat::WatAST]
          -> :wat::WatAST
          `(:wat::core::defmacro
             ~name
             [xs <- :wat::WatAST]
             -> :wat::WatAST
             `(:wat::holon::Bundle ~@xs)))
        (:my::mkmac :my::wrap)
        "#,
    )
    .unwrap();
    let body = find_defmacro_body(&forms[0]);
    let body_items = match body {
        WatAST::List(items, _) => items,
        _ => panic!("expected list"),
    };
    assert_eq!(body_items.len(), 2);
    assert!(matches!(&body_items[0],
        WatAST::Keyword(k, _) if k == ":wat::holon::Bundle"));
    // body_items[1] = (:wat::core::unquote-splicing xs)
    match &body_items[1] {
        WatAST::List(items, _) => {
            assert_eq!(items.len(), 2);
            assert!(matches!(&items[0],
                WatAST::Keyword(k, _) if k == ":wat::core::unquote-splicing"));
            assert!(matches!(&items[1], WatAST::Symbol(i, _) if i.as_str() == "xs"));
        }
        _ => panic!("expected unquote-splicing wrapper"),
    }
}

// Double unquote-splicing (,,@X) is out of arc 249's scope — the depth-protocol
// treats the inner ,@ as a preserved unquote-splicing wrapper inside the nested
// quasiquote (depth > 1 peel path), so ,,@xs in an outer template leaves
// (:wat::core::unquote-splicing xs) intact in the generated macro's body for
// the inner expansion pass to splice. Not tracked elsewhere: no consumer has
// surfaced the form.

// rune:complectens(inline-fixtures) — body dominated by an embedded 3-macro program source + AST shape assertions; outer logical bindings = 1; the visual length is data, not composition.
#[test]
fn make_deftest_shaped_template_expands_through_two_passes() {
    // The canonical forcing case — a macro-generating-macro that
    // configures dims + mode + default-prelude and registers a
    // new macro; then the user calls the new macro.
    let forms = expand_src(
        r#"
        (:wat::core::defmacro :my::make-mac
          [name   <- :wat::WatAST
           dims   <- :wat::WatAST
           mode   <- :wat::WatAST
           extras <- :wat::WatAST]
          -> :wat::WatAST
          `(:wat::core::defmacro
             ~name
             [test-name <- :wat::WatAST
              body      <- :wat::WatAST]
             -> :wat::WatAST
             `(:wat::holon::configured
                ~test-name
                ~~dims
                ~~mode
                ~~extras
                ~body)))

        (:my::make-mac :my::tdef 1024 :error ((load-a) (load-b)))

        (:my::tdef :my::run-1 (body-expr))
        "#,
    )
    .unwrap();
    // After both expansions, the final form should be:
    // (:wat::holon::configured :my::run-1 1024 :error ((load-a) (load-b)) (body-expr))
    assert_eq!(forms.len(), 1);
    match &forms[0] {
        WatAST::List(items, _) => {
            assert_eq!(items.len(), 6);
            assert!(matches!(&items[0],
                WatAST::Keyword(k, _) if k == ":wat::holon::configured"));
            assert!(matches!(&items[1],
                WatAST::Keyword(k, _) if k == ":my::run-1"));
            assert!(matches!(&items[2], WatAST::IntLit(1024, _)));
            assert!(matches!(&items[3],
                WatAST::Keyword(k, _) if k == ":error"));
            // items[4] = ((load-a) (load-b))
            match &items[4] {
                WatAST::List(l, _) => assert_eq!(l.len(), 2),
                _ => panic!("expected extras list"),
            }
            // items[5] = (body-expr)
            match &items[5] {
                WatAST::List(l, _) => assert_eq!(l.len(), 1),
                _ => panic!("expected body list"),
            }
        }
        _ => panic!("expected final list"),
    }
}

// ─── Arc 138 canary ─────────────────────────────────────────────────

#[test]
fn arc138_macro_error_message_carries_span() {
    // Trigger ArityMismatch — a two-param macro called with one arg.
    // The call-site form is parsed with `parse_all!` which labels spans
    // using file!() — e.g. "src/macros/tests.rs:N:M". The MacroError
    // Display arm prefixes the span via `span_prefix`, so the rendered
    // message must contain "src/" or ".rs:" when the variant's span is known.
    let err = expand_src(
        r#"
        (:wat::core::defmacro :my::two
          [x <- :wat::WatAST
           y <- :wat::WatAST]
          -> :wat::WatAST
          `(:wat::core::Vector ~x ~y))
        (:my::two 1)
        "#,
    )
    .unwrap_err();
    let rendered = format!("{}", err);
    // rune:lint(loose-assert) — variable Rust source file path embedded in error Display output via macro call-site span (varies by build environment)
    assert!(
        rendered.contains("src/") || rendered.contains(".rs:"),
        "expected MacroError Display to carry real source coordinates (file:line:col); got: {}",
        rendered
    );
    assert!(
        matches!(err, MacroError { kind: MacroErrorKind::ArityMismatch { .. }, .. }),
        "expected ArityMismatch, got: {:?}",
        err
    );
}

// ─── Arc 143 slice 2 — computed unquote ─────────────────────────────

/// `substitute_bindings`: a Symbol whose name is in the bindings map is replaced
/// with the bound AST value.
#[test]
fn substitute_bindings_bound_symbol_is_replaced() {
    let mut bindings = std::collections::HashMap::new();
    let span = crate::rust_caller_span!();
    bindings.insert("x".into(), WatAST::IntLit(42, span.clone()));
    let sym = WatAST::Symbol(Identifier::bare("x"), span.clone());
    let out = expand::substitute_bindings(&sym, &bindings);
    assert!(matches!(out, WatAST::IntLit(42, _)));
}

/// `substitute_bindings`: a Symbol whose name is NOT in the bindings map passes
/// through unchanged.
#[test]
fn substitute_bindings_unbound_symbol_passes_through() {
    let bindings = std::collections::HashMap::new();
    let span = crate::rust_caller_span!();
    let sym = WatAST::Symbol(Identifier::bare("y"), span.clone());
    let out = expand::substitute_bindings(&sym, &bindings);
    assert!(matches!(out, WatAST::Symbol(_, _)));
}

/// `substitute_bindings`: a List containing a bound Symbol is recursed into;
/// the Symbol is replaced inside the List.
#[test]
fn substitute_bindings_recurses_into_list() {
    let mut bindings = std::collections::HashMap::new();
    let span = crate::rust_caller_span!();
    bindings.insert("x".into(), WatAST::IntLit(42, span.clone()));
    let sym = WatAST::Symbol(Identifier::bare("x"), span.clone());
    let list = WatAST::List(
        vec![
            WatAST::Keyword(":head".into(), span.clone()),
            sym,
        ],
        span.clone(),
    );
    let out = expand::substitute_bindings(&list, &bindings);
    match out {
        WatAST::List(items, _) => {
            assert!(matches!(&items[0], WatAST::Keyword(k, _) if k == ":head"));
            assert!(matches!(&items[1], WatAST::IntLit(42, _)));
        }
        _ => panic!("expected List"),
    }
}

/// `,(some-list-with-non-keyword-head)` — the backward-compat heuristic
/// treats a List whose head is NOT a Keyword as an already-substituted
/// literal. It is returned as-is, not evaluated.
#[test]
fn computed_unquote_non_keyword_head_list_is_literal() {
    let bindings = std::collections::HashMap::new();
    let env = crate::runtime::Environment::default();
    let sym = crate::runtime::SymbolTable::default();
    let span = crate::rust_caller_span!();
    // Head is an IntLit, not a Keyword — must return as-is.
    let list = WatAST::List(
        vec![
            WatAST::IntLit(1, span.clone()),
            WatAST::IntLit(2, span.clone()),
        ],
        span.clone(),
    );
    let out = expand::unquote_argument(&list, &bindings, &env, &sym).unwrap();
    match out {
        WatAST::List(items, _) => {
            assert_eq!(items.len(), 2);
            assert!(matches!(items[0], WatAST::IntLit(1, _)));
            assert!(matches!(items[1], WatAST::IntLit(2, _)));
        }
        _ => panic!("expected List returned as-is"),
    }
}

/// `,(substrate-primitive-call)` in a macro body evaluates the call
/// at expand-time with a bare SymbolTable. Uses `:wat::core::i64::+`
/// which dispatches as a substrate primitive (no sym.functions needed).
#[test]
fn computed_unquote_evaluates_substrate_call() {
    let forms = expand_src(
        r#"
        (:wat::core::defmacro :my::computed-test
          []
          -> :wat::WatAST
          `(:result ~(:wat::core::i64::+ 10 32)))
        (:my::computed-test)
        "#,
    )
    .unwrap();
    // Expansion: (:result 42)
    assert_eq!(forms.len(), 1);
    match &forms[0] {
        WatAST::List(items, _) => {
            assert_eq!(items.len(), 2);
            assert!(matches!(&items[0], WatAST::Keyword(k, _) if k == ":result"));
            assert!(matches!(&items[1], WatAST::IntLit(42, _)));
        }
        _ => panic!("expected List"),
    }
}

/// Macro params are substituted into the unquoted expression before
/// evaluation. The macro takes a param `n` and uses it inside
/// `,(+ n 1)` — the substituted value is what gets evaluated.
#[test]
fn computed_unquote_substitutes_params_before_eval() {
    let forms = expand_src(
        r#"
        (:wat::core::defmacro :my::succ
          [n <- :wat::WatAST]
          -> :wat::WatAST
          `(:result ~(:wat::core::i64::+ n 1)))
        (:my::succ 41)
        "#,
    )
    .unwrap();
    // Expansion: (:result 42) — param n=41, +1 → 42.
    assert_eq!(forms.len(), 1);
    match &forms[0] {
        WatAST::List(items, _) => {
            assert_eq!(items.len(), 2);
            assert!(matches!(&items[0], WatAST::Keyword(k, _) if k == ":result"));
            assert!(matches!(&items[1], WatAST::IntLit(42, _)));
        }
        _ => panic!("expected (:result 42)"),
    }
}

/// `,@(expr)` — computed unquote-splicing. The expression evaluates
/// to a Vec and its elements are spliced into the surrounding list.
/// Uses `:wat::core::vec` to build the Vec at expand-time.
#[test]
fn computed_unquote_splicing_evaluates_and_splices() {
    let forms = expand_src(
        r#"
        (:wat::core::defmacro :my::trio
          []
          -> :wat::WatAST
          `(:wrapper ~@(:wat::core::Vector :wat::core::i64 1 2 3)))
        (:my::trio)
        "#,
    )
    .unwrap();
    // Expansion: (:wrapper 1 2 3)
    assert_eq!(forms.len(), 1);
    match &forms[0] {
        WatAST::List(items, _) => {
            assert_eq!(items.len(), 4);
            assert!(matches!(&items[0], WatAST::Keyword(k, _) if k == ":wrapper"));
            assert!(matches!(&items[1], WatAST::IntLit(1, _)));
            assert!(matches!(&items[2], WatAST::IntLit(2, _)));
            assert!(matches!(&items[3], WatAST::IntLit(3, _)));
        }
        _ => panic!("expected (:wrapper 1 2 3)"),
    }
}

/// Computed unquote inside a nested quasiquote at depth > 1 does NOT
/// fire at the outer expansion — it is preserved for the inner pass.
/// The expression survives verbatim inside the inner quasiquote body.
#[test]
fn computed_unquote_in_nested_quasiquote_preserved_at_outer() {
    // make-mac creates an inner macro whose body has ,(+ 1 2).
    // At the outer expansion of make-mac, the ,(+ 1 2) is at
    // depth 2 — it should be PRESERVED (not evaluated), because
    // only depth-1 unquotes fire at the outer level.
    let forms = expand_keeping_defmacros(
        r#"
        (:wat::core::defmacro :my::make-inner
          [name <- :wat::WatAST]
          -> :wat::WatAST
          `(:wat::core::defmacro
             ~name
             []
             -> :wat::WatAST
             `(:result ~(:wat::core::i64::+ 1 2))))
        (:my::make-inner :my::inner)
        "#,
    )
    .unwrap();
    // The generated defmacro's body should still contain an
    // unquote wrapping the list expression — NOT the evaluated 3.
    let body = find_defmacro_body(&forms[0]);
    let body_items = match body {
        WatAST::List(items, _) => items,
        _ => panic!("expected list body"),
    };
    assert_eq!(body_items.len(), 2);
    assert!(matches!(&body_items[0], WatAST::Keyword(k, _) if k == ":result"));
    // body_items[1] should be (:wat::core::unquote (:wat::core::i64::+ 1 2))
    // — the unquote survived to the inner macro's body.
    let inner = expect_unquote(&body_items[1]);
    // The inner arg should be the list (:wat::core::i64::+ 1 2),
    // NOT the evaluated IntLit(3).
    assert!(
        matches!(inner, WatAST::List(_, _)),
        "expected the unquote arg to be the unevaluated List, not an IntLit; got {:?}",
        inner
    );
}

// ─── Stone 249.2a-R2 — three new guard tests ────────────────────────

/// Depth-limit guard: a self-recursive macro expanded via `expand_all`
/// must fail with `MacroErrorKind::ExpansionDepthExceeded`. This test
/// goes red if the `depth > EXPANSION_DEPTH_LIMIT` check is removed.
#[test]
fn depth_limit_exceeded_on_self_recursive_macro() {
    let err = expand_src(
        r#"
        (:wat::core::defmacro :my::inf [x <- :wat::WatAST] -> :wat::WatAST `(:my::inf ~x))
        (:my::inf 1)
        "#,
    )
    .unwrap_err();
    assert!(
        matches!(err, MacroError { kind: MacroErrorKind::ExpansionDepthExceeded { .. }, .. }),
        "expected ExpansionDepthExceeded; got: {:?}",
        err
    );
}

/// `expand_once` expands ONE step only — not to fixpoint.
/// Register two chained macros; `expand_once` on the outer call
/// returns a call to the inner macro, NOT the final value.
#[test]
fn expand_once_single_step_not_fixpoint() {
    let (reg, rest, env, sym) = expand_setup(
        r#"
        (:wat::core::defmacro :my::outer [x <- :wat::WatAST] -> :wat::WatAST `(:my::inner ~x))
        (:wat::core::defmacro :my::inner [x <- :wat::WatAST] -> :wat::WatAST `(:wat::holon::Atom ~x))
        (:my::outer 42)
        "#,
    );
    // rest[0] is the (:my::outer 42) call.
    let once = expand_once(rest[0].clone(), &reg, &env, &sym).unwrap();
    // One step: (:my::outer ...) → (:my::inner 42), NOT (:wat::holon::Atom 42).
    match &once {
        WatAST::List(items, _) => {
            assert!(
                matches!(&items[0], WatAST::Keyword(k, _) if k == ":my::inner"),
                "expand_once should produce (:my::inner …), not fixpoint; got head: {:?}",
                items.first()
            );
        }
        other => panic!("expected a List from expand_once; got {:?}", other),
    }
}

// ─── Arc 249 stone 249.2b-i — RefusedInMacro (F5 default-deny gate) ────────

/// Computed-unquote `,(expr)` whose head is an impure `:wat::kernel::*`
/// effectful verb is refused by the default-deny `macro_eval` gate.
/// Exercises the F5-closure path: `unquote_argument` routes through
/// `macro_eval` → `validate_pure_total` → `RefusedInMacro`.
#[test]
fn impure_computed_unquote_refused_with_refused_in_macro() {
    let bindings = std::collections::HashMap::new();
    let env = crate::runtime::Environment::default();
    let sym = crate::runtime::SymbolTable::default();
    let span = crate::rust_caller_span!();
    // (:wat::kernel::send ...) — effectful head; NOT on the pure-total allow-list.
    // unquote_argument routes through macro_eval for any list with a Keyword head.
    let impure_form = WatAST::List(
        vec![
            WatAST::Keyword(":wat::kernel::send".into(), span.clone()),
            WatAST::IntLit(1, span.clone()),
        ],
        span.clone(),
    );
    let err = expand::unquote_argument(&impure_form, &bindings, &env, &sym).unwrap_err();
    assert!(
        matches!(err, MacroError { kind: MacroErrorKind::RefusedInMacro { .. }, .. }),
        "expected RefusedInMacro for impure kernel head; got: {:?}",
        err
    );
}

// ─── UnboundMacroParam — zero coverage ──────────────────────────────────────

/// A macro template that unquotes a typo'd param name (declared `x`,
/// template references `,y`) errors with `UnboundMacroParam { name: "y" }`.
#[test]
fn unquote_of_typo_param_errors_unbound_macro_param() {
    let err = expand_src(
        r#"
        (:wat::core::defmacro :my::typo
          [x <- :wat::WatAST]
          -> :wat::WatAST
          `(:result ~y))
        (:my::typo 42)
        "#,
    )
    .unwrap_err();
    assert!(
        matches!(
            &err,
            MacroError { kind: MacroErrorKind::UnboundMacroParam { name }, .. }
            if name == "y"
        ),
        "expected UnboundMacroParam {{ name: \"y\" }}; got: {:?}",
        err
    );
}

// ─── MalformedDefmacro — 5 untested parse sites ──────────────────────────────

/// (a) Wrong item count — not 6, 7, or the 3-item paren-pair.
#[test]
fn malformed_defmacro_wrong_item_count() {
    let err = expand_src(
        // Only 4 items: head name argvec body (missing -> and rettype).
        r#"(:wat::core::defmacro :my::m [x <- :AST] `~x)"#,
    )
    .unwrap_err();
    assert!(
        matches!(err, MacroError { kind: MacroErrorKind::MalformedDefmacro { .. }, .. }),
        "expected MalformedDefmacro for wrong item count; got: {:?}",
        err
    );
}

/// (b) Non-keyword macro name (e.g. a Symbol instead of a Keyword at item 1).
#[test]
fn malformed_defmacro_non_keyword_name() {
    let err = expand_src(
        // `my-macro` is a Symbol, not a Keyword — should fail.
        r#"(:wat::core::defmacro my-macro [x <- :AST] -> :AST `~x)"#,
    )
    .unwrap_err();
    assert!(
        matches!(err, MacroError { kind: MacroErrorKind::MalformedDefmacro { .. }, .. }),
        "expected MalformedDefmacro for non-keyword name; got: {:?}",
        err
    );
}

/// (c) Non-vector argspec (e.g. a List instead of a Vector).
#[test]
fn malformed_defmacro_non_vector_argspec() {
    let err = expand_src(
        // (x <- :AST) is a List, not a Vector.
        r#"(:wat::core::defmacro :my::m (x <- :AST) -> :AST `~x)"#,
    )
    .unwrap_err();
    assert!(
        matches!(err, MacroError { kind: MacroErrorKind::MalformedDefmacro { .. }, .. }),
        "expected MalformedDefmacro for non-vector argspec; got: {:?}",
        err
    );
}

/// (d) Missing/non-`->` arrow symbol (e.g. `=>` instead of `->`).
#[test]
fn malformed_defmacro_missing_arrow() {
    let err = expand_src(
        r#"(:wat::core::defmacro :my::m [x <- :AST] => :AST `~x)"#,
    )
    .unwrap_err();
    assert!(
        matches!(err, MacroError { kind: MacroErrorKind::MalformedDefmacro { .. }, .. }),
        "expected MalformedDefmacro for missing -> arrow; got: {:?}",
        err
    );
}

/// (e) Non-keyword return-type (e.g. a Symbol instead of a Keyword after `->`).
#[test]
fn malformed_defmacro_non_keyword_return_type() {
    let err = expand_src(
        // `AST` is a Symbol, not a Keyword (missing `:` prefix).
        r#"(:wat::core::defmacro :my::m [x <- :AST] -> AST `~x)"#,
    )
    .unwrap_err();
    assert!(
        matches!(err, MacroError { kind: MacroErrorKind::MalformedDefmacro { .. }, .. }),
        "expected MalformedDefmacro for non-keyword return type; got: {:?}",
        err
    );
}

/// `register_stdlib` bypasses the reserved-prefix gate and can register
/// a `:wat::*` macro that `register` would reject.
#[test]
fn register_stdlib_bypasses_reserved_prefix_gate() {
    let mut reg = MacroRegistry::new();
    let env = crate::runtime::Environment::default();
    let sym = crate::runtime::SymbolTable::default();

    // Attempt via the normal path — must be rejected.
    let user_forms = crate::parse_all!(
        r#"(:wat::core::defmacro :wat::std::TestMacro [x <- :wat::WatAST] -> :wat::WatAST `~x)"#
    )
    .expect("parse ok");
    let err = register_defmacros(user_forms, &mut reg).unwrap_err();
    assert!(
        matches!(err, MacroError { kind: MacroErrorKind::ReservedPrefix(_), .. }),
        "expected ReservedPrefix for :wat::std::* via register; got {:?}",
        err
    );

    // Same macro via the privileged stdlib path — must succeed.
    let stdlib_forms = crate::parse_all!(
        r#"(:wat::core::defmacro :wat::std::TestMacro [x <- :wat::WatAST] -> :wat::WatAST `~x)"#
    )
    .expect("parse ok");
    register_stdlib_defmacros(stdlib_forms, &mut reg)
        .expect("register_stdlib_defmacros should succeed for :wat::std::* prefix");

    // The macro is now in the registry and expands correctly.
    let call = crate::parse_all!(r#"(:wat::std::TestMacro 99)"#).expect("parse ok");
    let out = expand_all(call, &mut reg, &env, &sym).unwrap();
    assert_eq!(out.len(), 1);
    assert!(matches!(&out[0], WatAST::IntLit(99, _)), "expected IntLit(99); got {:?}", out[0]);
}

// ─── Arc 249 — is_pure_total deny-list: macroexpand excluded ────────────────

/// `:wat::core::macroexpand-1` is NOT on the `is_pure_total` allow-list and
/// must be refused when it appears as a computed-unquote head inside a macro
/// template. The deny path: `unquote_argument` → `macro_eval` →
/// `validate_pure_total` → `RefusedInMacro { head }`.
///
/// No test previously witnessed this refusal; this test closes the negative
/// space: a future accidental blessing of macroexpand-1 on the allow-list
/// would turn this test RED — the gate bites its author.
///
/// Mirrors `impure_computed_unquote_refused_with_refused_in_macro` (arc 249
/// stone 249.2b-i) in shape; exercises the macroexpand-specific deny path.
#[test]
fn macroexpand_in_computed_unquote_refused_with_refused_in_macro() {
    let bindings = std::collections::HashMap::new();
    let env = crate::runtime::Environment::default();
    let sym = crate::runtime::SymbolTable::default();
    let span = crate::rust_caller_span!();
    // (:wat::core::macroexpand-1 ...) — deliberately excluded from the
    // is_pure_total allow-list (macro-time evaluation must not invoke the
    // macroexpand runtime primitive; see eval.rs is_pure_total deny-list comment).
    let macroexpand_form = WatAST::List(
        vec![
            WatAST::Keyword(":wat::core::macroexpand-1".into(), span.clone()),
            WatAST::List(
                vec![
                    WatAST::Keyword(":wat::core::quote".into(), span.clone()),
                    WatAST::IntLit(1, span.clone()),
                ],
                span.clone(),
            ),
        ],
        span.clone(),
    );
    let err = expand::unquote_argument(&macroexpand_form, &bindings, &env, &sym).unwrap_err();
    assert!(
        matches!(err, MacroError { kind: MacroErrorKind::RefusedInMacro { .. }, .. }),
        "expected RefusedInMacro for :wat::core::macroexpand-1 in computed unquote; got: {:?}",
        err
    );
}

#[test]
fn impure_fn_body_passed_to_hof_refused_with_refused_in_macro() {
    // THE FENCE-HOLE WITNESS (245 long-tail scoring): blessed HOFs (map/foldl)
    // INVOKE fn arguments at expand time, so a kernel-send hidden in a HOF'd
    // fn body is expand-time impurity. A blanket "fn forms are opaque" rule
    // in validate_pure_total would wave this through — this test exists so
    // that hole can never silently reopen.
    let span = crate::rust_caller_span!();
    let impure_fn = WatAST::List(
        vec![
            WatAST::Keyword(":wat::core::fn".into(), span.clone()),
            WatAST::List(
                vec![
                    WatAST::Keyword(":wat::kernel::send".into(), span.clone()),
                    WatAST::IntLit(1, span.clone()),
                ],
                span.clone(),
            ),
        ],
        span.clone(),
    );
    let hof_form = WatAST::List(
        vec![
            WatAST::Keyword(":wat::core::map".into(), span.clone()),
            impure_fn,
        ],
        span.clone(),
    );
    let err = eval::validate_pure_total(&hof_form).unwrap_err();
    assert!(
        matches!(err, MacroError { kind: MacroErrorKind::RefusedInMacro { .. }, .. }),
        "expected RefusedInMacro for kernel head inside a HOF'd fn body; got: {:?}",
        err
    );
}

#[test]
fn signature_of_fn_literal_fn_arg_is_signature_only() {
    // The ONE sound contextual fn-opacity: signature-of-fn only CREATES the
    // closure and reads its SIGNATURE — the body never executes, so user-fn
    // heads inside it (runtime code destined for the expansion's output) are
    // permitted. A NON-fn argument to the same verb is still validated.
    let span = crate::rust_caller_span!();
    // (:wat::runtime::signature-of-fn (fn (:my::user-fn 1))) — OK.
    let fn_with_user_head = WatAST::List(
        vec![
            WatAST::Keyword(":wat::core::fn".into(), span.clone()),
            WatAST::List(
                vec![
                    WatAST::Keyword(":my::user-fn".into(), span.clone()),
                    WatAST::IntLit(1, span.clone()),
                ],
                span.clone(),
            ),
        ],
        span.clone(),
    );
    let reflect_ok = WatAST::List(
        vec![
            WatAST::Keyword(":wat::runtime::signature-of-fn".into(), span.clone()),
            fn_with_user_head,
        ],
        span.clone(),
    );
    assert!(
        eval::validate_pure_total(&reflect_ok).is_ok(),
        "signature-of-fn on a literal fn must be signature-only (body not expand-time code)"
    );
    // (:wat::runtime::signature-of-fn (:my::user-fn 1)) — non-fn arg: validated, refused.
    let reflect_bad = WatAST::List(
        vec![
            WatAST::Keyword(":wat::runtime::signature-of-fn".into(), span.clone()),
            WatAST::List(
                vec![
                    WatAST::Keyword(":my::user-fn".into(), span.clone()),
                    WatAST::IntLit(1, span.clone()),
                ],
                span.clone(),
            ),
        ],
        span.clone(),
    );
    let err = eval::validate_pure_total(&reflect_bad).unwrap_err();
    assert!(
        matches!(err, MacroError { kind: MacroErrorKind::RefusedInMacro { .. }, .. }),
        "non-fn argument to signature-of-fn must still be validated; got: {:?}",
        err
    );
}

#[test]
fn signature_of_fn_impure_body_is_inert() {
    // The DEEP claim behind the contextual exception (sharpened at the 245
    // re-ward): an IMPURE head inside the skipped fn body is safe not because
    // it is allowed but because it is INERT — eval_fn stores the body without
    // executing it, and function_to_signature_ast never reads f.body. If a
    // future code path ever executes the body on the signature-of-fn route,
    // this witness turns red.
    let span = crate::rust_caller_span!();
    let fn_with_kernel_send = WatAST::List(
        vec![
            WatAST::Keyword(":wat::core::fn".into(), span.clone()),
            WatAST::List(
                vec![
                    WatAST::Keyword(":wat::kernel::send".into(), span.clone()),
                    WatAST::IntLit(1, span.clone()),
                ],
                span.clone(),
            ),
        ],
        span.clone(),
    );
    let reflect = WatAST::List(
        vec![
            WatAST::Keyword(":wat::runtime::signature-of-fn".into(), span.clone()),
            fn_with_kernel_send,
        ],
        span.clone(),
    );
    assert!(
        eval::validate_pure_total(&reflect).is_ok(),
        "a kernel head inside signature-of-fn's literal fn arg is inert (body never executes) — the validator must pass it"
    );
}

// ─── parse.rs lines 112-114: 7-item defmacro with metadata-map ───────────────

/// `parse_defmacro_form` accepts a 7-item defmacro form (lines 112-114) that
/// carries a metadata-map `{...}` between the name and the argspec Vector.
/// The metadata is silently dropped by the parser (`_meta` binding); the
/// resulting `MacroDef` must be equivalent to the 6-item form without metadata.
///
/// Source form: `(:wat::core::defmacro :my::meta-mac {:tag 1} [x <- :AST] -> :AST `~x)`
/// Parsed as 7 items because `{:tag 1}` is one `WatAST::List` node.
#[test]
fn defmacro_with_metadata_map_registered_and_expands() {
    let out = expand_src(
        r#"
        (:wat::core::defmacro :my::meta-mac
          {:tag 1}
          [x <- :wat::WatAST]
          -> :wat::WatAST
          `~x)
        (:my::meta-mac 99)
        "#,
    )
    .expect("7-item defmacro with metadata map must parse and expand");
    assert_eq!(out.len(), 1, "expected one expanded form; got: {:?}", out);
    assert!(
        matches!(&out[0], WatAST::IntLit(99, _)),
        "expected IntLit(99) from expansion; got: {:?}",
        out[0]
    );
}

// ─── registry.rs lines 80-83: register_stdlib duplicate DuplicateMacro ───────

/// `MacroRegistry::register_stdlib` (lines 80-83) emits `DuplicateMacro` when
/// the same `:wat::*` name is registered twice with DIVERGENT bodies. The first
/// registration succeeds (bypassing the reserved-prefix gate); the second,
/// structurally distinct form must fail.
///
/// Complements `register_stdlib_bypasses_reserved_prefix_gate` which only tests
/// the happy path (idempotent re-registration). This test exercises the divergent-
/// duplicate error arm that lives at lines 80-83 of registry.rs.
#[test]
fn register_stdlib_duplicate_divergent_body_returns_duplicate_macro_error() {
    let mut reg = MacroRegistry::new();

    // First registration — body is `` `~x `` (quasiquote unquote of x).
    let first_forms = crate::parse_all!(
        r#"(:wat::core::defmacro :wat::std::DivMac [x <- :wat::WatAST] -> :wat::WatAST `~x)"#
    )
    .expect("parse ok");
    register_stdlib_defmacros(first_forms, &mut reg)
        .expect("first registration must succeed");

    // Second registration — body is `42` (a different body, structurally divergent).
    let second_forms = crate::parse_all!(
        r#"(:wat::core::defmacro :wat::std::DivMac [x <- :wat::WatAST] -> :wat::WatAST 42)"#
    )
    .expect("parse ok");
    let err = register_stdlib_defmacros(second_forms, &mut reg)
        .expect_err("second registration with divergent body must fail");
    assert!(
        matches!(err, MacroError { kind: MacroErrorKind::DuplicateMacro(_), .. }),
        "expected DuplicateMacro for divergent re-registration; got: {:?}",
        err
    );
}
