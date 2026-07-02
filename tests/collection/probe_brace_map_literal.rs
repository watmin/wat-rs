//! Arc 214 P2 — `{...}` map literal in expression position.
//!
//! Arc 257.2 amendment — ALL `{…}` now parse to `WatAST::Map`. The old
//! content-shape BraceKind dispatch (Symbol-head → StructPattern) is deleted.
//! Binder-position interpretation is check/runtime's job via
//! `classify_map_destructure`.
//!
//! ## The 9 probes
//!
//! 1. Empty `{}` → empty HashMap (length 0)
//! 2. Single pair `{:foo 42}` → length 1, contains :foo
//! 3. Multi pair `{:a 1 :b 2 :c 3}` → length 3, contains :b
//! 4. Nested in expression `(:wat::core::length {:a 1 :b 2})` → 2
//! 5. Map-literal-of-map-literal `{:outer {:inner 42}}` → length 1
//! 6. Non-keyword key `{42 :v}` accepted (arc 215 stone 2)
//! 7. Odd count `{:foo}` → `MalformedBraceLiteral` at parse
//! 8. Arc 257.2 — old `{outcome grace-residue}` form now errors (migrate to `{:keys […]}`)
//! 9. Keyword in binding position `{:foo bar}` rejected at CHECK time

use wat::freeze::{eval_in_frozen, startup_beside, startup_from_file};
use wat::parser::{ParseError, ParseErrorKind};
use wat::runtime::{Environment, Value};

// ─── Probe 1: Empty `{}` → empty HashMap ────────────────────────────────────

#[test]
fn probe_1_empty_brace_is_empty_hashmap() {
    let world = startup_beside(file!()).expect("startup");
    let ast = wat::parse_one!("(:t::p1-empty-len)").expect("parse");
    match eval_in_frozen(&ast, &world, &Environment::new()).expect("eval").value_owned() {
        Value::i64(n) => assert_eq!(n, 0, "empty {{}} must produce a length-0 HashMap"),
        other => panic!("expected i64; got {:?}", other),
    }
}

// ─── Probe 2: Single pair `{:foo 42}` ────────────────────────────────────────

#[test]
fn probe_2_single_pair_length_and_contains() {
    let world = startup_beside(file!()).expect("startup");
    let env = Environment::new();

    let ast = wat::parse_one!("(:t::p2a-single-len)").expect("parse");
    match eval_in_frozen(&ast, &world, &env).expect("eval").value_owned() {
        Value::i64(n) => assert_eq!(n, 1, "single-pair map literal must have length 1"),
        other => panic!("expected i64; got {:?}", other),
    }

    let ast = wat::parse_one!("(:t::p2b-single-contains)").expect("parse");
    match eval_in_frozen(&ast, &world, &env).expect("eval").value_owned() {
        Value::bool(b) => assert!(b, "single-pair map literal must contain :foo"),
        other => panic!("expected bool; got {:?}", other),
    }
}

// ─── Probe 3: Multi pair `{:a 1 :b 2 :c 3}` ─────────────────────────────────

#[test]
fn probe_3_multi_pair_length_and_contains() {
    let world = startup_beside(file!()).expect("startup");
    let env = Environment::new();

    let ast = wat::parse_one!("(:t::p3a-multi-len)").expect("parse");
    match eval_in_frozen(&ast, &world, &env).expect("eval").value_owned() {
        Value::i64(n) => assert_eq!(n, 3, "three-pair map literal must have length 3"),
        other => panic!("expected i64; got {:?}", other),
    }

    let ast = wat::parse_one!("(:t::p3b-multi-contains)").expect("parse");
    match eval_in_frozen(&ast, &world, &env).expect("eval").value_owned() {
        Value::bool(b) => assert!(b, "three-pair map literal must contain :b"),
        other => panic!("expected bool; got {:?}", other),
    }
}

// ─── Probe 4: Nested in expression ───────────────────────────────────────────

#[test]
fn probe_4_nested_in_expression_position() {
    let world = startup_beside(file!()).expect("startup");
    let ast = wat::parse_one!("(:t::p4-nested-expr-len)").expect("parse");
    match eval_in_frozen(&ast, &world, &Environment::new()).expect("eval").value_owned() {
        Value::i64(n) => assert_eq!(n, 2, "map literal nested in expression must yield length 2"),
        other => panic!("expected i64; got {:?}", other),
    }
}

// ─── Probe 5: Map-literal-of-map-literal ─────────────────────────────────────

#[test]
fn probe_5_map_of_map_resolved_by_arc215() {
    let world = startup_beside(file!()).expect("startup");
    let ast = wat::parse_one!("(:t::p5-map-of-map-len)").expect("parse");
    match eval_in_frozen(&ast, &world, &Environment::new()).expect("eval").value_owned() {
        Value::i64(n) => assert_eq!(n, 1, "nested map literal must have outer length 1 (arc 215 resolves P2 Probe 5 limitation)"),
        other => panic!("expected i64; got {:?}", other),
    }
}

// ─── Probe 6: Non-keyword key accepted (arc 215 stone 2) ─────────────────────

#[test]
fn probe_6_non_keyword_key_accepted_with_inferred_k() {
    // Parse check: `{42 :v}` must parse cleanly (no MalformedBraceLiteral).
    let result = wat::parse_one!("{42 :v}");
    assert!(
        result.is_ok(),
        "non-keyword key must parse cleanly after arc 215 stone 2; got: {:?}",
        result
    );

    // Type-check + runtime via co-located fixture.
    let world = startup_beside(file!()).expect("startup");
    let ast = wat::parse_one!("(:t::p6-int-key-len)").expect("parse");
    match eval_in_frozen(&ast, &world, &Environment::new()).expect("eval").value_owned() {
        Value::i64(n) => assert_eq!(n, 1, "int-keyed map must have length 1"),
        other => panic!("expected i64; got {:?}", other),
    }
}

// ─── Probe 7: Odd count ───────────────────────────────────────────────────────

#[test]
fn probe_7_odd_count_rejected_at_parse() {
    let result = wat::parse_one!("{:foo}");
    assert!(
        matches!(result, Err(ParseError { kind: ParseErrorKind::MalformedBraceLiteral { .. }, .. })),
        "odd-count brace-form must produce MalformedBraceLiteral; got: {:?}",
        result
    );
    let err = format!("{}", result.unwrap_err());
    // rune:lint(loose-assert) — parse_one! embeds the absolute Rust source path (file!():line!():col!()) which is machine-specific
    assert!(
        err.contains("alternate") || err.contains("pairs") || err.contains("1"),
        "error must name alternation requirement + count; got: {}",
        err
    );
}

// ─── Probe 8: Arc 257.2 — old bare-symbol struct-pattern form rejected ──────

#[test]
fn probe_8_old_struct_pattern_now_errors() {
    let err = startup_from_file(
        "tests/collection/probe_brace_map_literal_p8_bad.wat",
    )
    .expect_err("arc 257.2: old bare-symbol brace-form must now be rejected");
    let err = format!("{}\n---\n{:?}", err, err);
    assert_eq!(
        err,
        r##"check:
1 type-check error(s):
  - tests/collection/probe_brace_map_literal_p8_bad.wat:10:6: malformed :wat::core::let form: let binder must be a bare symbol (single binding), a vector of symbols (tuple destructure), or a bare-symbol brace-form (struct destructure); got a map in binder position

---
Check(CheckErrors([CheckError { span: Span { file: "tests/collection/probe_brace_map_literal_p8_bad.wat", line: 10, col: 6, end_line: 10, end_col: 29 }, kind: MalformedForm { head: ":wat::core::let", reason: "let binder must be a bare symbol (single binding), a vector of symbols (tuple destructure), or a bare-symbol brace-form (struct destructure); got a map in binder position", remedies: [] } }]))"##,
        "probe_8: old struct-pattern form rejected golden"
    );
}

// ─── Probe 9: Keyword in binding position ────────────────────────────────────

#[test]
fn probe_9_keyword_in_binding_position_rejected() {
    let err = startup_from_file(
        "tests/collection/probe_brace_map_literal_p9_bad.wat",
    )
    .expect_err("keyword in binding position must be rejected");
    let err = format!("{}\n---\n{:?}", err, err);
    assert_eq!(
        err,
        r##"check:
1 type-check error(s):
  - tests/collection/probe_brace_map_literal_p9_bad.wat:5:6: malformed :wat::core::let form: let binder must be a bare symbol (single binding), a vector of symbols (tuple destructure), or a bare-symbol brace-form (struct destructure); got a map in binder position

---
Check(CheckErrors([CheckError { span: Span { file: "tests/collection/probe_brace_map_literal_p9_bad.wat", line: 5, col: 6, end_line: 5, end_col: 16 }, kind: MalformedForm { head: ":wat::core::let", reason: "let binder must be a bare symbol (single binding), a vector of symbols (tuple destructure), or a bare-symbol brace-form (struct destructure); got a map in binder position", remedies: [] } }]))"##,
        "probe_9: keyword-in-binding-position rejected golden"
    );
}
