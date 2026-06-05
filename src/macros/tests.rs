use super::*;
use super::expand;
use crate::ast::WatAST;
use crate::identifier::Identifier;

fn expand(src: &str) -> Result<Vec<WatAST>, MacroError> {
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
fn expand_keeping_defmacros(src: &str) -> Result<Vec<WatAST>, MacroError> {
    let forms = crate::parse_all!(src).expect("parse ok");
    let mut reg = MacroRegistry::new();
    let rest = register_defmacros(forms, &mut reg)?;
    let env = crate::runtime::Environment::default();
    let sym = crate::runtime::SymbolTable::default();
    let mut out = Vec::with_capacity(rest.len());
    for form in rest {
        out.push(expand::expand_form(form, &reg, 0, &env, &sym)?);
    }
    Ok(out)
}

// ─── Pure alias macro ───────────────────────────────────────────────

#[test]
fn alias_macro_expands_to_primitive() {
    let forms = expand(
        r#"
        (:wat::core::defmacro :my::vocab::Concurrent
          [xs <- :AST<List<wat::holon::HolonAST>>]
          -> :AST<wat::holon::HolonAST>
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
    let forms = expand(
        r#"
        (:wat::core::defmacro :my::vocab::Subtract
          [x <- :AST<wat::holon::HolonAST>
           y <- :AST<wat::holon::HolonAST>]
          -> :AST<wat::holon::HolonAST>
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
    let forms = expand(
        r#"
        (:wat::core::defmacro :my::vocab::SumAll
          [xs <- :AST<List<wat::holon::HolonAST>>]
          -> :AST<wat::holon::HolonAST>
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
    let forms = expand(
        r#"
        (:wat::core::defmacro :my::outer [x <- :AST] -> :AST `(:my::inner ~x))
        (:wat::core::defmacro :my::inner [x <- :AST] -> :AST `(:wat::holon::Atom ~x))
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

#[test]
fn template_identifier_carries_macro_scope() {
    let forms = expand(
        r#"
        (:wat::core::defmacro :my::vocab::WithTmp
          [body <- :AST]
          -> :AST
          `(:wat::core::let ((tmp 1)) ~body))
        (:my::vocab::WithTmp tmp)
        "#,
    )
    .unwrap();
    // Expansion: (:wat::core::let ((tmp[macro-scope] 1)) tmp[user-empty])
    // The two `tmp`s must have DIFFERENT Identifiers.
    let list = match &forms[0] {
        WatAST::List(items, _) => items,
        _ => panic!("expected list"),
    };
    // ((tmp 1)) — new canonical shape: drill through the bindings list
    // and the binding pair to reach tmp directly at position 0.
    let bindings = match &list[1] {
        WatAST::List(bs, _) => bs,
        _ => panic!("expected bindings list"),
    };
    let first_binding = match &bindings[0] {
        WatAST::List(b, _) => b,
        _ => panic!("expected binding pair"),
    };
    let template_tmp = match &first_binding[0] {
        WatAST::Symbol(i, _) => i,
        _ => panic!("expected Symbol at binding name position"),
    };
    // The body position's `tmp` — user-supplied argument, not macro-origin.
    let user_tmp = match &list[2] {
        WatAST::Symbol(i, _) => i,
        _ => panic!("expected Symbol in body"),
    };
    assert_eq!(template_tmp.name, "tmp");
    assert_eq!(user_tmp.name, "tmp");
    assert!(
        !template_tmp.scopes.is_empty(),
        "template tmp must have macro scope attached"
    );
    assert!(
        user_tmp.scopes.is_empty(),
        "user-argument tmp must NOT have the macro scope"
    );
    assert_ne!(
        template_tmp, user_tmp,
        "template and user tmp must be DIFFERENT Identifiers"
    );
}

// ─── Argument identifiers are preserved unchanged ──────────────────

#[test]
fn argument_identifiers_pass_through_unchanged() {
    // User passes a symbol; the macro should splice it verbatim.
    let forms = expand(
        r#"
        (:wat::core::defmacro :my::wrap [v <- :AST] -> :AST `(:wat::holon::Atom ~v))
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
    assert_eq!(v_arg.name, "some-var");
    assert!(
        v_arg.scopes.is_empty(),
        "argument identifier should have no macro scope"
    );
}

// ─── Classic capture: two macros introduce the same template name ─

#[test]
fn two_macro_invocations_get_distinct_scopes() {
    let forms = expand(
        r#"
        (:wat::core::defmacro :my::twice
          [x <- :AST]
          -> :AST
          `(:wat::core::let (((t :i64) ~x)) t))
        (:my::twice 1)
        (:my::twice 2)
        "#,
    )
    .unwrap();
    // Both expansions bind `t` in the template; each invocation should
    // tag its `t` with a FRESH scope. The two `t`s differ.
    let extract_binding_sym = |f: &WatAST| -> Identifier {
        let outer = if let WatAST::List(items, _) = f {
            items.clone()
        } else {
            panic!("expected list")
        };
        let bindings = if let WatAST::List(b, _) = &outer[1] {
            b.clone()
        } else {
            panic!()
        };
        let pair = if let WatAST::List(b, _) = &bindings[0] {
            b.clone()
        } else {
            panic!()
        };
        let typed_name = if let WatAST::List(tn, _) = &pair[0] {
            tn.clone()
        } else {
            panic!()
        };
        if let WatAST::Symbol(i, _) = &typed_name[0] {
            i.clone()
        } else {
            panic!()
        }
    };
    let t1 = extract_binding_sym(&forms[0]);
    let t2 = extract_binding_sym(&forms[1]);
    assert_eq!(t1.name, "t");
    assert_eq!(t2.name, "t");
    assert_ne!(t1, t2, "each invocation should mint a fresh macro scope");
}

// ─── Error paths ────────────────────────────────────────────────────

#[test]
fn reserved_prefix_macro_rejected() {
    let err = expand(
        r#"(:wat::core::defmacro :wat::std::MyMacro [x <- :AST] -> :AST `~x)"#,
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
    let err = expand(
        r#"
        (:wat::core::defmacro :my::m [x <- :AST] -> :AST `~x)
        (:wat::core::defmacro :my::m [x <- :AST] -> :AST `(:wat::core::Vector ~x))
        "#,
    )
    .unwrap_err();
    assert!(matches!(err, MacroError { kind: MacroErrorKind::DuplicateMacro(_), .. }));
}

#[test]
fn duplicate_defmacro_byte_equivalent_is_noop() {
    // Arc 054: two byte-equivalent defmacro forms — same name,
    // params, body. Second registration is a no-op. The macro
    // expands normally afterward.
    let result = expand(
        r#"
        (:wat::core::defmacro :my::m [x <- :AST] -> :AST `~x)
        (:wat::core::defmacro :my::m [x <- :AST] -> :AST `~x)
        (:my::m 42)
        "#,
    );
    assert!(result.is_ok(), "byte-equivalent re-decl should succeed; got {:?}", result);
}

#[test]
fn macro_arity_mismatch() {
    let err = expand(
        r#"
        (:wat::core::defmacro :my::two
          [x <- :AST y <- :AST]
          -> :AST
          `(:wat::core::Vector ~x ~y))
        (:my::two 1)
        "#,
    )
    .unwrap_err();
    assert!(matches!(err, MacroError { kind: MacroErrorKind::ArityMismatch { .. }, .. }));
}

#[test]
fn non_quasiquote_body_rejected() {
    // Arc 249 stone 249.2b-ii: a program body (non-quasiquote) IS evaluated
    // by macro_eval. A body that produces a non-AST result (e.g. a Vec) still
    // errors — with MalformedTemplate (value_to_watast rejects Vec).
    let err = expand(
        r#"
        (:wat::core::defmacro :my::m [x <- :AST] -> :AST
          (:wat::core::Vector :bogus x))
        (:my::m 1)
        "#,
    )
    .unwrap_err();
    assert!(matches!(err, MacroError { kind: MacroErrorKind::MalformedTemplate { .. }, .. }));
}

#[test]
fn splice_non_list_arg_rejected() {
    let err = expand(
        r#"
        (:wat::core::defmacro :my::s [xs <- :AST] -> :AST `(:wat::core::Vector ~@xs))
        (:my::s 42)
        "#,
    )
    .unwrap_err();
    assert!(matches!(err, MacroError { kind: MacroErrorKind::SpliceNotSequence { .. }, .. }));
}

// ─── Non-macro forms pass through unchanged ─────────────────────────

#[test]
fn non_macro_forms_unchanged() {
    let forms = expand(r#"(:wat::holon::Atom "hello") 42 "world""#).unwrap();
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
          [name <- :AST<()>]
          -> :AST<()>
          `(:wat::core::defmacro
             ~name
             [x <- :AST]
             -> :AST
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
          [v <- :AST<i64>]
          -> :AST<()>
          `(:wat::core::defmacro
             :my::configured
             []
             -> :AST
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
    let lit = WatAST::IntLit(99, crate::span::Span::unknown());
    let out = expand::unquote_argument(&lit, &bindings, &env, &sym).unwrap();
    match out {
        WatAST::IntLit(n, _) => assert_eq!(n, 99),
        _ => panic!("expected IntLit"),
    }
    // A list whose head is NOT a keyword — treated as already-substituted
    // literal (backward-compat heuristic: head must be Keyword to eval).
    let list = WatAST::List(
        vec![WatAST::IntLit(1, crate::span::Span::unknown()), WatAST::IntLit(2, crate::span::Span::unknown())],
        crate::span::Span::unknown(),
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
          [name <- :AST<()>]
          -> :AST<()>
          `(:wat::core::defmacro
             ~name
             [xs <- :AST]
             -> :AST
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

// Note: ,,@X (double unquote-splicing) is NOT yet supported. The
// combined shape (:wat::core::unquote (:wat::core::unquote-splicing X))
// at depth 2 would need special-case handling that lets the outer
// substitution hand a concrete list down to an outer-level splice
// wrapper. `make-deftest`'s implementation uses `,,default-prelude`
// (non-splicing double unquote) where the list value is placed as
// deftest's prelude argument — the splicing happens inside deftest's
// own template, not at make-deftest's level. If a future use case
// forces `,,@`, extend `walk_template` to recognize
// `(unquote (unquote-splicing X))` at depth 2 as "substitute + wrap
// in unquote-splicing" (outer wrapper replaced by the inner).

#[test]
fn make_deftest_shaped_template_expands_through_two_passes() {
    // The canonical forcing case — a macro-generating-macro that
    // configures dims + mode + default-prelude and registers a
    // new macro; then the user calls the new macro.
    let forms = expand(
        r#"
        (:wat::core::defmacro :my::make-mac
          [name   <- :AST<()>
           dims   <- :AST<i64>
           mode   <- :AST<wat::core::keyword>
           extras <- :AST]
          -> :AST<()>
          `(:wat::core::defmacro
             ~name
             [test-name <- :AST<()>
              body      <- :AST<()>]
             -> :AST<()>
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
    // The call-site form is parsed with `parse_all` which labels spans
    // `<test>:<line>:<col>`. The MacroError Display arm prefixes the
    // span via `span_prefix`, so the rendered message must contain
    // `<test>:` when the variant's span is known.
    let err = expand(
        r#"
        (:wat::core::defmacro :my::two
          [x <- :AST
           y <- :AST]
          -> :AST
          `(:wat::core::Vector ~x ~y))
        (:my::two 1)
        "#,
    )
    .unwrap_err();
    let rendered = format!("{}", err);
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

/// `substitute_bindings` helper: replaces Symbols bound in the map,
/// recurses into Lists, passes keywords + literals through unchanged.
#[test]
fn substitute_bindings_replaces_symbols_and_recurses() {
    let mut bindings = std::collections::HashMap::new();
    let span = crate::span::Span::unknown();
    bindings.insert("x".into(), WatAST::IntLit(42, span.clone()));
    // Symbol that IS in bindings → replaced.
    let sym = WatAST::Symbol(
        Identifier::bare("x"),
        span.clone(),
    );
    let out = expand::substitute_bindings(&sym, &bindings);
    assert!(matches!(out, WatAST::IntLit(42, _)));
    // Symbol NOT in bindings → passes through as-is.
    let sym2 = WatAST::Symbol(
        Identifier::bare("y"),
        span.clone(),
    );
    let out2 = expand::substitute_bindings(&sym2, &bindings);
    assert!(matches!(out2, WatAST::Symbol(_, _)));
    // List containing the symbol — recursive replacement.
    let list = WatAST::List(
        vec![
            WatAST::Keyword(":head".into(), span.clone()),
            sym.clone(),
        ],
        span.clone(),
    );
    let out3 = expand::substitute_bindings(&list, &bindings);
    match out3 {
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
    let span = crate::span::Span::unknown();
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
    let forms = expand(
        r#"
        (:wat::core::defmacro :my::computed-test
          []
          -> :AST
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
    let forms = expand(
        r#"
        (:wat::core::defmacro :my::succ
          [n <- :AST<i64>]
          -> :AST
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
    let forms = expand(
        r#"
        (:wat::core::defmacro :my::trio
          []
          -> :AST
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
          [name <- :AST<()>]
          -> :AST<()>
          `(:wat::core::defmacro
             ~name
             []
             -> :AST
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
    let err = expand(
        r#"
        (:wat::core::defmacro :my::inf [x <- :AST] -> :AST `(:my::inf ~x))
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
    let forms = crate::parse_all!(
        r#"
        (:wat::core::defmacro :my::outer [x <- :AST] -> :AST `(:my::inner ~x))
        (:wat::core::defmacro :my::inner [x <- :AST] -> :AST `(:wat::holon::Atom ~x))
        (:my::outer 42)
        "#
    )
    .expect("parse ok");
    let mut reg = MacroRegistry::new();
    let rest = register_defmacros(forms, &mut reg).unwrap();
    let env = crate::runtime::Environment::default();
    let sym = crate::runtime::SymbolTable::default();
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

/// `register_stdlib` bypasses the reserved-prefix gate and can register
/// a `:wat::*` macro that `register` would reject.
#[test]
fn register_stdlib_bypasses_reserved_prefix_gate() {
    let mut reg = MacroRegistry::new();
    let env = crate::runtime::Environment::default();
    let sym = crate::runtime::SymbolTable::default();

    // Attempt via the normal path — must be rejected.
    let user_forms = crate::parse_all!(
        r#"(:wat::core::defmacro :wat::std::TestMacro [x <- :AST] -> :AST `~x)"#
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
        r#"(:wat::core::defmacro :wat::std::TestMacro [x <- :AST] -> :AST `~x)"#
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
