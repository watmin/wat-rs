//! Strict-rejection tests: every spec-mandated `MUST NOT` exercised.
//! Locks in the conformance fixes from the /ignorant ward audit.

use wat_edn::{parse, Value};

// ─── CRIT-1: discard at end of collection ───────────────────────

#[test]
fn discard_at_end_of_vector() {
    assert_eq!(
        parse("[1 #_2]").unwrap(),
        Value::Vector(vec![Value::Integer(1)])
    );
}

#[test]
fn discard_at_end_of_list() {
    assert_eq!(
        parse("(1 #_2)").unwrap(),
        Value::List(vec![Value::Integer(1)])
    );
}

#[test]
fn discard_at_end_of_set() {
    assert_eq!(
        parse("#{1 #_2}").unwrap(),
        Value::Set(vec![Value::Integer(1)])
    );
}

#[test]
fn discard_inside_map_after_value() {
    let v = parse("{:k :v #_:dangling}").unwrap();
    if let Value::Map(entries) = v {
        assert_eq!(entries.len(), 1);
    } else {
        panic!("expected Map");
    }
}

#[test]
fn discard_chained() {
    // `[#_#_a b c]` discards two forms (a and b), leaves c.
    assert_eq!(
        parse("[#_#_a b c]").unwrap(),
        Value::Vector(vec![Value::Symbol(wat_edn::Symbol::new("c"))])
    );
}

#[test]
fn discard_top_level_then_value() {
    assert_eq!(parse("#_42 7").unwrap(), Value::Integer(7));
}

#[test]
fn discard_only_at_top_level_errors() {
    assert!(parse("#_42").is_err());
}

// ─── CRIT-2: leading zeros ──────────────────────────────────────

#[test]
fn rejects_leading_zero_int() {
    assert!(parse("01").is_err());
    assert!(parse("-007").is_err());
    assert!(parse("+0123").is_err());
}

#[test]
fn rejects_leading_zero_bigint() {
    assert!(parse("00N").is_err());
    assert!(parse("007N").is_err());
}

#[test]
fn rejects_leading_zero_bigdec() {
    assert!(parse("007M").is_err());
}

#[test]
fn allows_legitimate_zero_forms() {
    assert_eq!(parse("0").unwrap(), Value::Integer(0));
    assert_eq!(parse("-0").unwrap(), Value::Integer(0));
    assert_eq!(parse("0.5").unwrap(), Value::Float(0.5));
    assert_eq!(parse("0e10").unwrap(), Value::Float(0.0));
}

// ─── CRIT-3: `:/` is not a legal keyword ────────────────────────

#[test]
fn rejects_slash_keyword() {
    assert!(parse(":/").is_err());
}

#[test]
fn rejects_slash_anything_keyword() {
    assert!(parse(":/foo").is_err());
}

#[test]
fn allows_bare_slash_symbol() {
    assert_eq!(
        parse("/").unwrap(),
        Value::Symbol(wat_edn::Symbol::new("/"))
    );
}

// ─── CRIT-4: backslash-then-whitespace ──────────────────────────

#[test]
fn rejects_backslash_space() {
    assert!(parse("\\ ").is_err());
}

#[test]
fn rejects_backslash_tab() {
    assert!(parse("\\\t").is_err());
}

#[test]
fn rejects_backslash_newline() {
    assert!(parse("\\\n").is_err());
}

#[test]
fn rejects_backslash_comma() {
    // Per spec, comma is whitespace.
    assert!(parse("\\,").is_err());
}

// ─── CRIT-5: numeric first char in symbol/keyword name ──────────

#[test]
fn rejects_numeric_name_in_namespaced_symbol() {
    assert!(parse("ns/123").is_err());
}

#[test]
fn rejects_numeric_name_in_namespaced_keyword() {
    assert!(parse(":ns/123").is_err());
}

#[test]
fn rejects_plus_digit_name() {
    assert!(parse("ns/+42").is_err());
}

// ─── CRIT-6: leading-dot-then-digit ─────────────────────────────

#[test]
fn rejects_dot_digit_symbol() {
    assert!(parse(".5").is_err());
    assert!(parse(".0").is_err());
}

#[test]
fn allows_dot_letter_symbol() {
    assert_eq!(
        parse(".foo").unwrap(),
        Value::Symbol(wat_edn::Symbol::new(".foo"))
    );
}

#[test]
fn allows_double_dot_symbol() {
    // `..1` is OK: first char `.`, second char `.` (non-numeric).
    assert_eq!(
        parse("..1").unwrap(),
        Value::Symbol(wat_edn::Symbol::new("..1"))
    );
}

// ─── INV-3: non-finite float round-trip via sentinel tags ──────

#[test]
fn nan_round_trips() {
    let v = Value::Float(f64::NAN);
    let s = wat_edn::write(&v);
    assert_eq!(s, "#wat-edn.float/nan nil");
    let v2 = parse(&s).unwrap();
    if let Value::Float(f) = v2 {
        assert!(f.is_nan());
    } else {
        panic!("expected Float(NaN)");
    }
}

#[test]
fn pos_inf_round_trips() {
    let v = Value::Float(f64::INFINITY);
    let s = wat_edn::write(&v);
    assert_eq!(s, "#wat-edn.float/inf nil");
    let v2 = parse(&s).unwrap();
    assert_eq!(v2, Value::Float(f64::INFINITY));
}

#[test]
fn neg_inf_round_trips() {
    let v = Value::Float(f64::NEG_INFINITY);
    let s = wat_edn::write(&v);
    assert_eq!(s, "#wat-edn.float/neg-inf nil");
    let v2 = parse(&s).unwrap();
    assert_eq!(v2, Value::Float(f64::NEG_INFINITY));
}

// ─── INV-5: UUID strict canonical form ──────────────────────────

#[test]
fn rejects_uuid_simple_form() {
    assert!(parse(r#"#uuid "550e8400e29b41d4a716446655440000""#).is_err());
}

#[test]
fn rejects_uuid_urn_form() {
    assert!(
        parse(r#"#uuid "urn:uuid:550e8400-e29b-41d4-a716-446655440000""#).is_err()
    );
}

#[test]
fn accepts_canonical_uuid() {
    let v = parse(r#"#uuid "550e8400-e29b-41d4-a716-446655440000""#).unwrap();
    assert!(matches!(v, Value::Uuid(_)));
}

// ─── Spec example heterogeneous map ─────────────────────────────

#[test]
fn heterogeneous_map_keys_parse() {
    let v = parse(r#"{:a 1, "foo" :bar, [1 2 3] four}"#).unwrap();
    if let Value::Map(entries) = v {
        assert_eq!(entries.len(), 3);
    } else {
        panic!("expected Map");
    }
}

// ─── Multi-line strings ─────────────────────────────────────────

#[test]
fn multi_line_string() {
    assert_eq!(
        parse("\"line1\nline2\"").unwrap(),
        Value::String("line1\nline2".into())
    );
}

// ─── Comment without trailing newline ───────────────────────────

#[test]
fn comment_without_newline_at_eof() {
    assert_eq!(parse("42 ; trailing").unwrap(), Value::Integer(42));
}

// ─── Tag with no element inside collection ──────────────────────

#[test]
fn rejects_tag_without_element_inside_vector() {
    assert!(parse("[#myapp/Foo]").is_err());
}

// ─── Signed bigint / bigdec ─────────────────────────────────────

#[test]
fn signed_bigint() {
    let v = parse("-42N").unwrap();
    assert!(matches!(v, Value::BigInt(_)));
}

#[test]
fn signed_bigdec() {
    let v = parse("-1.5e-3M").unwrap();
    assert!(matches!(v, Value::BigDec(_)));
}
