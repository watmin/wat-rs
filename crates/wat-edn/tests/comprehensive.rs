//! Comprehensive coverage — organized by category, every spec corner.
//!
//! Tests every literal type the EDN spec defines, every boundary, every
//! pathological input. No prose; tests are the verification.

use wat_edn::{parse, parse_all, write, Error, ErrorKind, Keyword, Symbol, Tag, Value};

// ═══════════════════════════════════════════════════════════════════════
// NUMERIC EDGE CASES
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn integer_zero() {
    assert_eq!(parse("0").unwrap(), Value::Integer(0));
    assert_eq!(parse("-0").unwrap(), Value::Integer(0));
    assert_eq!(parse("+0").unwrap(), Value::Integer(0));
}

#[test]
fn integer_max() {
    assert_eq!(
        parse("9223372036854775807").unwrap(),
        Value::Integer(i64::MAX)
    );
}

#[test]
fn integer_min() {
    assert_eq!(
        parse("-9223372036854775808").unwrap(),
        Value::Integer(i64::MIN)
    );
}

#[test]
fn integer_overflow_errors() {
    // i64::MAX + 1 — should error since we don't auto-promote to BigInt
    assert!(parse("9223372036854775808").is_err());
}

#[test]
fn integer_overflow_promoted_via_n_suffix() {
    let v = parse("9223372036854775808N").unwrap();
    assert!(matches!(v, Value::BigInt(_)));
}

#[test]
fn integer_signs() {
    assert_eq!(parse("42").unwrap(), Value::Integer(42));
    assert_eq!(parse("-42").unwrap(), Value::Integer(-42));
    assert_eq!(parse("+42").unwrap(), Value::Integer(42));
}

#[test]
fn float_basic() {
    assert_eq!(parse("2.5").unwrap(), Value::Float(2.5));
    assert_eq!(parse("-2.5").unwrap(), Value::Float(-2.5));
    assert_eq!(parse("0.0").unwrap(), Value::Float(0.0));
}

#[test]
fn float_scientific() {
    assert_eq!(parse("1e10").unwrap(), Value::Float(1e10));
    assert_eq!(parse("1E10").unwrap(), Value::Float(1e10));
    assert_eq!(parse("1e+10").unwrap(), Value::Float(1e10));
    assert_eq!(parse("1e-10").unwrap(), Value::Float(1e-10));
    assert_eq!(parse("1.5e3").unwrap(), Value::Float(1500.0));
    assert_eq!(parse("1.5e-3").unwrap(), Value::Float(0.0015));
}

#[test]
fn float_no_int_part_rejected() {
    // Per spec, leading dot is not allowed.
    assert!(parse(".5").is_err());
}

#[test]
fn float_no_frac_after_dot_works() {
    // `42.` — our impl treats trailing `.` as token-terminator since
    // there's no digit after, so this lexes as int 42 followed by junk.
    // Verify the integer comes through.
    let v = parse_all("42.").unwrap();
    assert_eq!(v[0], Value::Integer(42));
}

#[test]
fn float_negative_zero() {
    // -0.0 != 0.0 in IEEE bits but == in our equality.
    let v = parse("-0.0").unwrap();
    assert!(matches!(v, Value::Float(_)));
}

#[test]
fn bigint_basic() {
    let v = parse("42N").unwrap();
    assert!(matches!(v, Value::BigInt(_)));
}

#[test]
fn bigint_signed() {
    let v = parse("-42N").unwrap();
    assert!(matches!(v, Value::BigInt(_)));
    let v = parse("+42N").unwrap();
    assert!(matches!(v, Value::BigInt(_)));
}

#[test]
fn bigint_huge() {
    let v = parse("123456789012345678901234567890N").unwrap();
    assert!(matches!(v, Value::BigInt(_)));
}

#[test]
fn bigdec_basic() {
    let v = parse("3.14M").unwrap();
    assert!(matches!(v, Value::BigDec(_)));
}

#[test]
fn bigdec_exponent() {
    let v = parse("1.5e-3M").unwrap();
    assert!(matches!(v, Value::BigDec(_)));
}

#[test]
fn bigdec_negative() {
    let v = parse("-3.14M").unwrap();
    assert!(matches!(v, Value::BigDec(_)));
}

#[test]
fn rejects_leading_zero_in_int() {
    assert!(parse("01").is_err());
    assert!(parse("007").is_err());
    assert!(parse("-007").is_err());
    assert!(parse("+0123").is_err());
}

#[test]
fn allows_zero_dot_anything() {
    assert_eq!(parse("0.5").unwrap(), Value::Float(0.5));
    assert_eq!(parse("0e10").unwrap(), Value::Float(0.0));
    assert!(matches!(parse("0M").unwrap(), Value::BigDec(_)));
    assert!(matches!(parse("0N").unwrap(), Value::BigInt(_)));
}

#[test]
fn rejects_n_suffix_on_float() {
    assert!(parse("3.14N").is_err());
}

#[test]
fn rejects_invalid_exponent() {
    assert!(parse("1e").is_err());
    assert!(parse("1e+").is_err());
}

// ═══════════════════════════════════════════════════════════════════════
// STRING COVERAGE
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn string_empty() {
    assert_eq!(parse(r#""""#).unwrap(), Value::String("".into()));
}

#[test]
fn string_simple() {
    assert_eq!(parse(r#""hello""#).unwrap(), Value::String("hello".into()));
}

#[test]
fn string_with_spaces() {
    assert_eq!(
        parse(r#""hello world""#).unwrap(),
        Value::String("hello world".into())
    );
}

#[test]
fn string_escape_newline() {
    assert_eq!(
        parse(r#""line1\nline2""#).unwrap(),
        Value::String("line1\nline2".into())
    );
}

#[test]
fn string_escape_tab() {
    assert_eq!(
        parse(r#""a\tb""#).unwrap(),
        Value::String("a\tb".into())
    );
}

#[test]
fn string_escape_return() {
    assert_eq!(
        parse(r#""a\rb""#).unwrap(),
        Value::String("a\rb".into())
    );
}

#[test]
fn string_escape_backslash() {
    assert_eq!(
        parse(r#""a\\b""#).unwrap(),
        Value::String("a\\b".into())
    );
}

#[test]
fn string_escape_quote() {
    assert_eq!(
        parse(r#""a\"b""#).unwrap(),
        Value::String("a\"b".into())
    );
}

#[test]
fn string_escape_slash() {
    // Extension: \/ is JSON-aligned, accepted by our reader.
    assert_eq!(
        parse(r#""a\/b""#).unwrap(),
        Value::String("a/b".into())
    );
}

#[test]
fn string_escape_unicode() {
    assert_eq!(
        parse(r#""é""#).unwrap(),
        Value::String("é".into())
    );
}

#[test]
fn string_escape_unicode_uppercase() {
    assert_eq!(
        parse(r#""é""#).unwrap(),
        parse(r#""é""#).unwrap()
    );
}

#[test]
fn string_multibyte_passthrough() {
    assert_eq!(parse(r#""é""#).unwrap(), Value::String("é".into()));
    assert_eq!(parse(r#""日本語""#).unwrap(), Value::String("日本語".into()));
}

#[test]
fn string_emoji() {
    let v = parse(r#""🦀""#).unwrap();
    assert_eq!(v, Value::String("🦀".into()));
}

#[test]
fn string_multi_line() {
    // Spec: strings may span multiple lines.
    assert_eq!(
        parse("\"line1\nline2\"").unwrap(),
        Value::String("line1\nline2".into())
    );
}

#[test]
fn string_long() {
    let s = "x".repeat(10_000);
    let input = format!("\"{}\"", s);
    assert_eq!(parse(&input).unwrap(), Value::String(s.into()));
}

#[test]
fn string_invalid_escape_errors() {
    assert!(parse(r#""\x""#).is_err()); // \x is not a valid escape
}

#[test]
fn string_unclosed_errors() {
    assert!(parse(r#""abc"#).is_err());
}

#[test]
fn string_truncated_unicode_errors() {
    assert!(parse(r#""\u00""#).is_err()); // need 4 hex digits
}

#[test]
fn string_invalid_unicode_codepoint_errors() {
    // U+D800 is a surrogate, not a valid scalar.
    assert!(parse(r#""\uD800""#).is_err());
}

// ═══════════════════════════════════════════════════════════════════════
// CHARACTER LITERALS
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn char_single_letter() {
    assert_eq!(parse(r"\a").unwrap(), Value::Char('a'));
    assert_eq!(parse(r"\Z").unwrap(), Value::Char('Z'));
}

#[test]
fn char_single_digit() {
    assert_eq!(parse(r"\1").unwrap(), Value::Char('1'));
}

#[test]
fn char_punctuation() {
    assert_eq!(parse(r"\(").unwrap(), Value::Char('('));
    assert_eq!(parse(r"\)").unwrap(), Value::Char(')'));
    assert_eq!(parse(r"\[").unwrap(), Value::Char('['));
    assert_eq!(parse(r"\]").unwrap(), Value::Char(']'));
    assert_eq!(parse(r"\{").unwrap(), Value::Char('{'));
    assert_eq!(parse(r"\}").unwrap(), Value::Char('}'));
    assert_eq!(parse(r"\!").unwrap(), Value::Char('!'));
    assert_eq!(parse(r"\?").unwrap(), Value::Char('?'));
}

#[test]
fn char_named_newline() {
    assert_eq!(parse(r"\newline").unwrap(), Value::Char('\n'));
}

#[test]
fn char_named_space() {
    assert_eq!(parse(r"\space").unwrap(), Value::Char(' '));
}

#[test]
fn char_named_tab() {
    assert_eq!(parse(r"\tab").unwrap(), Value::Char('\t'));
}

#[test]
fn char_named_return() {
    assert_eq!(parse(r"\return").unwrap(), Value::Char('\r'));
}

#[test]
fn char_named_formfeed_extension() {
    assert_eq!(parse(r"\formfeed").unwrap(), Value::Char('\u{000C}'));
}

#[test]
fn char_named_backspace_extension() {
    assert_eq!(parse(r"\backspace").unwrap(), Value::Char('\u{0008}'));
}

#[test]
fn char_unicode_escape() {
    // Spec: `\uNNNN` for any Unicode scalar value.
    assert_eq!(parse("\\u00E9").unwrap(), Value::Char('é'));
    assert_eq!(parse("\\u0041").unwrap(), Value::Char('A'));
    assert_eq!(parse("\\u0000").unwrap(), Value::Char('\0'));
    assert_eq!(parse("\\u20AC").unwrap(), Value::Char('€'));
}

#[test]
fn char_multibyte_utf8() {
    assert_eq!(parse("\\é").unwrap(), Value::Char('é'));
    assert_eq!(parse("\\日").unwrap(), Value::Char('日'));
    assert_eq!(parse("\\🦀").unwrap(), Value::Char('🦀'));
}

#[test]
fn char_backslash_then_whitespace_errors() {
    assert!(parse("\\ ").is_err());
    assert!(parse("\\\t").is_err());
    assert!(parse("\\\n").is_err());
    assert!(parse("\\,").is_err());
}

#[test]
fn char_eof_after_backslash_errors() {
    assert!(parse(r"\").is_err());
}

// ═══════════════════════════════════════════════════════════════════════
// SYMBOLS
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn symbol_simple() {
    assert_eq!(parse("foo").unwrap(), Value::Symbol(Symbol::new("foo")));
}

#[test]
fn symbol_with_dash() {
    assert_eq!(
        parse("foo-bar").unwrap(),
        Value::Symbol(Symbol::new("foo-bar"))
    );
}

#[test]
fn symbol_with_question() {
    assert_eq!(
        parse("foo?").unwrap(),
        Value::Symbol(Symbol::new("foo?"))
    );
}

#[test]
fn symbol_with_bang() {
    assert_eq!(
        parse("set!").unwrap(),
        Value::Symbol(Symbol::new("set!"))
    );
}

#[test]
fn symbol_namespaced() {
    assert_eq!(
        parse("ns/foo").unwrap(),
        Value::Symbol(Symbol::ns("ns", "foo"))
    );
}

#[test]
fn symbol_namespaced_with_dots() {
    assert_eq!(
        parse("my.app.events/OrderPlaced").unwrap(),
        Value::Symbol(Symbol::ns("my.app.events", "OrderPlaced"))
    );
}

#[test]
fn symbol_bare_slash() {
    assert_eq!(parse("/").unwrap(), Value::Symbol(Symbol::new("/")));
}

#[test]
fn symbol_with_angle_brackets() {
    assert_eq!(
        parse("Vec<i64>").unwrap(),
        Value::Symbol(Symbol::new("Vec<i64>"))
    );
}

#[test]
fn symbol_with_underscore_separator() {
    assert_eq!(
        parse("HashMap<String_i64>").unwrap(),
        Value::Symbol(Symbol::new("HashMap<String_i64>"))
    );
}

#[test]
fn symbol_starts_with_dot_letter() {
    assert_eq!(
        parse(".foo").unwrap(),
        Value::Symbol(Symbol::new(".foo"))
    );
}

#[test]
fn symbol_starts_with_dash_letter() {
    assert_eq!(
        parse("-foo").unwrap(),
        Value::Symbol(Symbol::new("-foo"))
    );
}

#[test]
fn symbol_starts_with_plus_letter() {
    assert_eq!(
        parse("+foo").unwrap(),
        Value::Symbol(Symbol::new("+foo"))
    );
}

#[test]
fn symbol_just_dot_dot() {
    assert_eq!(parse("..").unwrap(), Value::Symbol(Symbol::new("..")));
}

#[test]
fn symbol_just_plus() {
    assert_eq!(parse("+").unwrap(), Value::Symbol(Symbol::new("+")));
}

#[test]
fn symbol_just_minus() {
    assert_eq!(parse("-").unwrap(), Value::Symbol(Symbol::new("-")));
}

// ═══════════════════════════════════════════════════════════════════════
// KEYWORDS
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn keyword_simple() {
    assert_eq!(
        parse(":foo").unwrap(),
        Value::Keyword(Keyword::new("foo"))
    );
}

#[test]
fn keyword_namespaced() {
    assert_eq!(
        parse(":ns/foo").unwrap(),
        Value::Keyword(Keyword::ns("ns", "foo"))
    );
}

#[test]
fn keyword_with_dotted_namespace() {
    assert_eq!(
        parse(":my.ns/foo").unwrap(),
        Value::Keyword(Keyword::ns("my.ns", "foo"))
    );
}

#[test]
fn keyword_double_colon_rejected() {
    // Spec: keyword cannot begin with ::
    assert!(parse("::foo").is_err());
}

#[test]
fn keyword_slash_rejected() {
    // Spec: :/ is not a legal keyword.
    assert!(parse(":/").is_err());
}

#[test]
fn keyword_slash_anything_rejected() {
    assert!(parse(":/foo").is_err());
}

#[test]
fn keyword_starts_with_dash() {
    assert_eq!(
        parse(":-foo").unwrap(),
        Value::Keyword(Keyword::new("-foo"))
    );
}

#[test]
fn keyword_ends_with_question() {
    assert_eq!(
        parse(":valid?").unwrap(),
        Value::Keyword(Keyword::new("valid?"))
    );
}

// ═══════════════════════════════════════════════════════════════════════
// COLLECTIONS
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn empty_list() {
    assert_eq!(parse("()").unwrap(), Value::List(vec![]));
}

#[test]
fn empty_vector() {
    assert_eq!(parse("[]").unwrap(), Value::Vector(vec![]));
}

#[test]
fn empty_map() {
    assert_eq!(parse("{}").unwrap(), Value::Map(vec![]));
}

#[test]
fn empty_set() {
    assert_eq!(parse("#{}").unwrap(), Value::Set(vec![]));
}

#[test]
fn single_element_collections() {
    assert_eq!(parse("(1)").unwrap(), Value::List(vec![Value::Integer(1)]));
    assert_eq!(parse("[1]").unwrap(), Value::Vector(vec![Value::Integer(1)]));
    assert_eq!(parse("#{1}").unwrap(), Value::Set(vec![Value::Integer(1)]));
}

#[test]
fn vector_one_hundred_elements() {
    let inner: Vec<String> = (0..100).map(|i| i.to_string()).collect();
    let input = format!("[{}]", inner.join(" "));
    let parsed = parse(&input).unwrap();
    if let Value::Vector(items) = parsed {
        assert_eq!(items.len(), 100);
    } else {
        panic!("expected Vector");
    }
}

#[test]
fn vector_deeply_nested_50_levels() {
    let mut s = String::new();
    for _ in 0..50 {
        s.push('[');
    }
    s.push('1');
    for _ in 0..50 {
        s.push(']');
    }
    let v = parse(&s).unwrap();
    // walk down 50 levels
    let mut current = &v;
    for _ in 0..50 {
        match current {
            Value::Vector(items) if items.len() == 1 => current = &items[0],
            _ => panic!("expected nested Vector"),
        }
    }
    assert_eq!(*current, Value::Integer(1));
}

#[test]
fn map_basic() {
    assert_eq!(
        parse("{:a 1 :b 2}").unwrap(),
        Value::Map(vec![
            (Value::Keyword(Keyword::new("a")), Value::Integer(1)),
            (Value::Keyword(Keyword::new("b")), Value::Integer(2)),
        ])
    );
}

#[test]
fn map_heterogeneous_keys() {
    let v = parse(r#"{:a 1, "foo" :bar, [1 2 3] four}"#).unwrap();
    if let Value::Map(entries) = v {
        assert_eq!(entries.len(), 3);
    } else {
        panic!("expected Map");
    }
}

#[test]
fn map_unequal_lengths_not_equal() {
    let a = parse("{:a 1}").unwrap();
    let b = parse("{:a 1 :b 2}").unwrap();
    assert_ne!(a, b);
}

#[test]
fn map_equality_unordered() {
    // CRITICAL: spec mandates unordered map equality.
    let a = parse("{:a 1 :b 2 :c 3}").unwrap();
    let b = parse("{:c 3 :a 1 :b 2}").unwrap();
    let c = parse("{:b 2 :c 3 :a 1}").unwrap();
    assert_eq!(a, b);
    assert_eq!(a, c);
    assert_eq!(b, c);
}

#[test]
fn map_different_values_not_equal() {
    let a = parse("{:a 1 :b 2}").unwrap();
    let b = parse("{:a 1 :b 99}").unwrap();
    assert_ne!(a, b);
}

#[test]
fn map_with_collection_keys() {
    let a = parse("{[1 2] :pair, [3 4] :other}").unwrap();
    let b = parse("{[3 4] :other, [1 2] :pair}").unwrap();
    assert_eq!(a, b);
}

#[test]
fn map_odd_elements_errors() {
    assert!(parse("{:a 1 :b}").is_err());
}

#[test]
fn set_basic() {
    let v = parse("#{1 2 3}").unwrap();
    assert_eq!(
        v,
        Value::Set(vec![
            Value::Integer(1),
            Value::Integer(2),
            Value::Integer(3),
        ])
    );
}

#[test]
fn set_equality_unordered() {
    // CRITICAL: spec mandates unordered set equality.
    let a = parse("#{1 2 3}").unwrap();
    let b = parse("#{3 2 1}").unwrap();
    let c = parse("#{2 1 3}").unwrap();
    assert_eq!(a, b);
    assert_eq!(a, c);
    assert_eq!(b, c);
}

#[test]
fn set_unequal_lengths_not_equal() {
    let a = parse("#{1 2}").unwrap();
    let b = parse("#{1 2 3}").unwrap();
    assert_ne!(a, b);
}

#[test]
fn set_with_collections() {
    let a = parse("#{[1] [2] [3]}").unwrap();
    let b = parse("#{[3] [1] [2]}").unwrap();
    assert_eq!(a, b);
}

#[test]
fn list_equality_ordered() {
    // Spec: lists are ordered.
    let a = parse("(1 2 3)").unwrap();
    let b = parse("(3 2 1)").unwrap();
    assert_ne!(a, b);
}

#[test]
fn vector_equality_ordered() {
    let a = parse("[1 2 3]").unwrap();
    let b = parse("[3 2 1]").unwrap();
    assert_ne!(a, b);
}

#[test]
fn collection_unclosed_errors() {
    assert!(parse("[1 2 3").is_err());
    assert!(parse("(1 2 3").is_err());
    assert!(parse("{:a 1").is_err());
    assert!(parse("#{1 2").is_err());
}

#[test]
fn collection_mismatched_delim_errors() {
    assert!(parse("[1 2 3)").is_err());
    assert!(parse("(1 2 3]").is_err());
}

// ═══════════════════════════════════════════════════════════════════════
// TAGS
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn user_tag_basic() {
    let v = parse(r#"#myapp/Person {:name "Fred"}"#).unwrap();
    match v {
        Value::Tagged(tag, _) => assert_eq!(tag, Tag::ns("myapp", "Person")),
        _ => panic!("expected Tagged"),
    }
}

#[test]
fn user_tag_dotted_namespace() {
    let v = parse(r#"#my.app/Order {}"#).unwrap();
    match v {
        Value::Tagged(tag, _) => assert_eq!(tag, Tag::ns("my.app", "Order")),
        _ => panic!("expected Tagged"),
    }
}

#[test]
fn user_tag_with_generics_in_name() {
    let v = parse(r#"#wat.core/Vec<i64> [1 2 3]"#).unwrap();
    match v {
        Value::Tagged(tag, body) => {
            assert_eq!(tag.namespace(), "wat.core");
            assert_eq!(tag.name(), "Vec<i64>");
            assert!(matches!(*body, Value::Vector(_)));
        }
        _ => panic!("expected Tagged"),
    }
}

#[test]
fn user_tag_must_have_namespace() {
    assert!(parse("#bareTag 42").is_err());
}

#[test]
fn user_tag_nested() {
    let v = parse(r#"#a/B #c/D 42"#).unwrap();
    match v {
        Value::Tagged(t1, body) => {
            assert_eq!(t1.name(), "B");
            match *body {
                Value::Tagged(t2, inner) => {
                    assert_eq!(t2.name(), "D");
                    assert_eq!(*inner, Value::Integer(42));
                }
                _ => panic!("expected nested Tagged"),
            }
        }
        _ => panic!("expected Tagged"),
    }
}

#[test]
fn user_tag_dangling_at_eof_errors() {
    let r = parse("#myapp/Foo");
    assert!(r.is_err());
}

#[test]
fn user_tag_dangling_in_vector_errors() {
    let r = parse("[#myapp/Foo]");
    assert!(r.is_err());
    // And the error message should name TagWithoutElement, not UnexpectedEof.
    if let Err(Error::Parse { kind, .. }) = r {
        assert!(matches!(kind, ErrorKind::TagWithoutElement(_)));
    } else {
        panic!("expected Parse error");
    }
}

#[test]
fn user_tag_dangling_in_map_errors() {
    assert!(parse("{#myapp/Foo}").is_err());
}

#[test]
fn user_tag_dangling_in_set_errors() {
    assert!(parse("#{#myapp/Foo}").is_err());
}

#[test]
fn inst_canonicalizes() {
    let v = parse(r#"#inst "2026-04-26T14:30:00Z""#).unwrap();
    assert!(matches!(v, Value::Inst(_)));
}

#[test]
fn inst_with_fractional_seconds() {
    let v = parse(r#"#inst "2026-04-26T14:30:00.123Z""#).unwrap();
    assert!(matches!(v, Value::Inst(_)));
}

#[test]
fn inst_with_offset() {
    let v = parse(r#"#inst "2026-04-26T14:30:00+05:00""#).unwrap();
    assert!(matches!(v, Value::Inst(_)));
}

#[test]
fn inst_invalid_format_errors() {
    assert!(parse(r#"#inst "not-a-date""#).is_err());
    assert!(parse(r#"#inst "2026-13-45""#).is_err());
}

#[test]
fn inst_wrong_body_type_errors() {
    assert!(parse(r#"#inst 42"#).is_err());
}

#[test]
fn uuid_canonical() {
    let v = parse(r#"#uuid "550e8400-e29b-41d4-a716-446655440000""#).unwrap();
    assert!(matches!(v, Value::Uuid(_)));
}

#[test]
fn uuid_simple_form_rejected() {
    // Spec wants canonical 8-4-4-4-12.
    assert!(parse(r#"#uuid "550e8400e29b41d4a716446655440000""#).is_err());
}

#[test]
fn uuid_urn_form_rejected() {
    assert!(parse(r#"#uuid "urn:uuid:550e8400-e29b-41d4-a716-446655440000""#).is_err());
}

#[test]
fn uuid_invalid_hex_rejected() {
    assert!(parse(r#"#uuid "550e8400-e29b-41d4-a716-44665544000Z""#).is_err());
}

#[test]
fn uuid_wrong_length_rejected() {
    assert!(parse(r#"#uuid "550e8400-e29b-41d4-a716-44665544""#).is_err());
}

// ═══════════════════════════════════════════════════════════════════════
// DISCARD `#_`
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn discard_top_level_then_value() {
    assert_eq!(parse("#_42 7").unwrap(), Value::Integer(7));
}

#[test]
fn discard_at_end_of_top_level_errors() {
    assert!(parse("#_42").is_err());
}

#[test]
fn discard_inside_vector_middle() {
    assert_eq!(
        parse("[1 #_99 2 3]").unwrap(),
        Value::Vector(vec![
            Value::Integer(1),
            Value::Integer(2),
            Value::Integer(3),
        ])
    );
}

#[test]
fn discard_inside_vector_start() {
    assert_eq!(
        parse("[#_99 1 2]").unwrap(),
        Value::Vector(vec![Value::Integer(1), Value::Integer(2)])
    );
}

#[test]
fn discard_inside_vector_end() {
    assert_eq!(
        parse("[1 2 #_99]").unwrap(),
        Value::Vector(vec![Value::Integer(1), Value::Integer(2)])
    );
}

#[test]
fn discard_inside_list_end() {
    assert_eq!(
        parse("(1 2 #_99)").unwrap(),
        Value::List(vec![Value::Integer(1), Value::Integer(2)])
    );
}

#[test]
fn discard_inside_set_end() {
    assert_eq!(
        parse("#{1 2 #_99}").unwrap(),
        Value::Set(vec![Value::Integer(1), Value::Integer(2)])
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
fn discard_inside_map_between_pairs() {
    let v = parse("{:k1 :v1 #_:dropped :k2 :v2}").unwrap();
    if let Value::Map(entries) = v {
        assert_eq!(entries.len(), 2);
    } else {
        panic!("expected Map");
    }
}

#[test]
fn discard_chained_two() {
    assert_eq!(
        parse("[#_#_a b c]").unwrap(),
        Value::Vector(vec![Value::Symbol(Symbol::new("c"))])
    );
}

#[test]
fn discard_chained_three() {
    assert_eq!(
        parse("[#_#_#_a b c d]").unwrap(),
        Value::Vector(vec![Value::Symbol(Symbol::new("d"))])
    );
}

#[test]
fn discard_complex_form() {
    // Discarding a whole sub-tree.
    assert_eq!(
        parse("[1 #_[a b c] 2]").unwrap(),
        Value::Vector(vec![Value::Integer(1), Value::Integer(2)])
    );
}

// ═══════════════════════════════════════════════════════════════════════
// COMMENTS AND WHITESPACE
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn comment_to_eol() {
    assert_eq!(parse("; comment\n42").unwrap(), Value::Integer(42));
}

#[test]
fn comment_at_eof_no_newline() {
    assert_eq!(parse("42 ; trailing").unwrap(), Value::Integer(42));
}

#[test]
fn multiple_consecutive_comments() {
    let input = "; one\n; two\n; three\n42";
    assert_eq!(parse(input).unwrap(), Value::Integer(42));
}

#[test]
fn comment_inside_collection() {
    assert_eq!(
        parse("[1 ; comment\n 2 3]").unwrap(),
        Value::Vector(vec![
            Value::Integer(1),
            Value::Integer(2),
            Value::Integer(3),
        ])
    );
}

#[test]
fn comma_is_whitespace() {
    assert_eq!(
        parse("[1, 2, 3]").unwrap(),
        Value::Vector(vec![
            Value::Integer(1),
            Value::Integer(2),
            Value::Integer(3),
        ])
    );
}

#[test]
fn lots_of_whitespace() {
    let input = "  \t\n\r , \n\t  42  \r\n";
    assert_eq!(parse(input).unwrap(), Value::Integer(42));
}

#[test]
fn leading_trailing_whitespace_in_collections() {
    assert_eq!(
        parse("[ 1 , 2 , 3 ]").unwrap(),
        Value::Vector(vec![
            Value::Integer(1),
            Value::Integer(2),
            Value::Integer(3),
        ])
    );
}

// ═══════════════════════════════════════════════════════════════════════
// NON-FINITE FLOAT ROUND-TRIP
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn nan_writes_as_sentinel() {
    let s = write(&Value::Float(f64::NAN));
    assert_eq!(s, "#wat-edn.float/nan nil");
}

#[test]
fn nan_round_trips() {
    let v1 = Value::Float(f64::NAN);
    let s = write(&v1); let v2 = parse(&s).unwrap();
    assert_eq!(v1, v2);
}

#[test]
fn pos_inf_round_trips() {
    let v1 = Value::Float(f64::INFINITY);
    let s = write(&v1); let v2 = parse(&s).unwrap();
    assert_eq!(v1, v2);
}

#[test]
fn neg_inf_round_trips() {
    let v1 = Value::Float(f64::NEG_INFINITY);
    let s = write(&v1); let v2 = parse(&s).unwrap();
    assert_eq!(v1, v2);
}

// ═══════════════════════════════════════════════════════════════════════
// TOP-LEVEL parse_all
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn parse_all_empty_input() {
    assert_eq!(parse_all("").unwrap(), Vec::<Value>::new());
}

#[test]
fn parse_all_only_whitespace() {
    assert_eq!(parse_all("   \n\t").unwrap(), Vec::<Value>::new());
}

#[test]
fn parse_all_only_comments() {
    assert_eq!(parse_all("; just a comment\n").unwrap(), Vec::<Value>::new());
}

#[test]
fn parse_all_three_forms() {
    assert_eq!(
        parse_all("1 2 3").unwrap(),
        vec![Value::Integer(1), Value::Integer(2), Value::Integer(3)]
    );
}

#[test]
fn parse_all_mixed_forms() {
    assert_eq!(
        parse_all("nil :foo \"bar\"").unwrap(),
        vec![
            Value::Nil,
            Value::Keyword(Keyword::new("foo")),
            Value::String("bar".into()),
        ]
    );
}

// ═══════════════════════════════════════════════════════════════════════
// ROUND-TRIP IDENTITY
// ═══════════════════════════════════════════════════════════════════════

fn roundtrip(input: &str) -> Value<'_> {
    let v1 = parse(input).expect("parse 1");
    let s = write(&v1);
    let v2 = parse(&s).expect("re-parse");
    assert_eq!(v1, v2, "round-trip mismatch for {} → {}", input, s);
    v1
}

#[test]
fn rt_primitives() {
    roundtrip("nil");
    roundtrip("true");
    roundtrip("false");
    roundtrip("0");
    roundtrip("42");
    roundtrip("-7");
    roundtrip("3.14");
    roundtrip(r#""""#);
    roundtrip(r#""hello""#);
    roundtrip(":foo");
    roundtrip(":ns/foo");
    roundtrip("foo");
    roundtrip("ns/foo");
}

#[test]
fn rt_collections() {
    roundtrip("[]");
    roundtrip("[1]");
    roundtrip("[1 2 3]");
    roundtrip("()");
    roundtrip("(1 2 3)");
    roundtrip("{}");
    roundtrip("{:a 1}");
    roundtrip("#{}");
    roundtrip("#{1 2 3}");
}

#[test]
fn rt_nested() {
    roundtrip("[[1 2] [3 4] [5 6]]");
    roundtrip("{:outer {:inner 42}}");
    roundtrip("[{:k :v} {:k :v2}]");
}

#[test]
fn rt_tagged() {
    roundtrip("#myapp/Foo {:bar 1}");
    roundtrip("#a/B #c/D 42");
}

#[test]
fn rt_inst_uuid() {
    roundtrip(r#"#inst "2026-04-26T14:30:00Z""#);
    roundtrip(r#"#uuid "550e8400-e29b-41d4-a716-446655440000""#);
}

#[test]
fn rt_strings_with_escapes() {
    roundtrip(r#""line1\nline2""#);
    roundtrip(r#""tab\there""#);
    roundtrip(r#""quote\"inside""#);
    roundtrip(r#""back\\slash""#);
}

#[test]
fn rt_unicode_strings() {
    roundtrip(r#""é""#);
    roundtrip(r#""日本語""#);
    roundtrip(r#""🦀 rust""#);
}

#[test]
fn rt_realistic_blob() {
    roundtrip(
        r#"
        #enterprise.observer.market/TradeSignal
        {:asset       :BTC
         :side        :Buy
         :size        0.025
         :confidence  0.73
         :reasoning   #wat.core/Vec<wat.holon.HolonAST>
                        [#wat.holon/Atom :rsi-rising
                         #wat.holon/Atom :flow-positive
                         #wat.holon/Bind [:flow :positive]]
         :proposed-at #inst "2026-04-26T14:30:00Z"
         :id          #uuid "550e8400-e29b-41d4-a716-446655440000"}
        "#,
    );
}

#[test]
fn rt_signed_bigint() {
    let v1 = parse("-123456789012345678901234567890N").unwrap();
    let s = write(&v1); let v2 = parse(&s).unwrap();
    assert_eq!(v1, v2);
}

#[test]
fn rt_signed_bigdec() {
    let v1 = parse("-3.14M").unwrap();
    let s = write(&v1); let v2 = parse(&s).unwrap();
    assert_eq!(v1, v2);
}

#[test]
fn rt_chars() {
    roundtrip(r"\a");
    roundtrip(r"\newline");
    roundtrip(r"\space");
    roundtrip("\\é");
}

// ═══════════════════════════════════════════════════════════════════════
// ERROR POSITIONS
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn error_position_unexpected_byte() {
    let r = parse("@invalid");
    if let Err(Error::Parse { pos, .. }) = r {
        assert_eq!(pos, 0);
    } else {
        panic!("expected Parse error");
    }
}

#[test]
fn error_position_unclosed_string() {
    let r = parse(r#""abc"#);
    assert!(matches!(
        r,
        Err(Error::Parse {
            kind: ErrorKind::UnclosedString,
            ..
        })
    ));
}

// ═══════════════════════════════════════════════════════════════════════
// SPEC-EXAMPLE BLOBS
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn spec_example_person() {
    let v = parse(r#"#myapp/Person {:first "Fred" :last "Mertz"}"#).unwrap();
    match v {
        Value::Tagged(tag, body) => {
            assert_eq!(tag, Tag::ns("myapp", "Person"));
            if let Value::Map(entries) = *body {
                assert_eq!(entries.len(), 2);
            } else {
                panic!("expected Map body");
            }
        }
        _ => panic!("expected Tagged"),
    }
}

#[test]
fn deeply_mixed_data() {
    let input = r#"
    {:users [{:name "Alice" :age 30 :uuid #uuid "550e8400-e29b-41d4-a716-446655440000"}
             {:name "Bob"   :age 25 :friends #{:carol :dave}}]
     :counts {:active 12, :idle 3}
     :tags   #{:alpha :beta}
     :ratio  3.14M
     :ts     #inst "2026-04-26T14:30:00Z"}
    "#;
    let v = parse(input).unwrap();
    assert!(matches!(v, Value::Map(_)));
}

// ═══════════════════════════════════════════════════════════════════════
// PARTIAL EQ — DEEP STRUCTURAL
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn deep_map_equality_unordered() {
    let a = parse(r#"{:a {:nested 1 :other 2} :b {:x 10}}"#).unwrap();
    let b = parse(r#"{:b {:x 10} :a {:other 2 :nested 1}}"#).unwrap();
    assert_eq!(a, b);
}

#[test]
fn deep_set_equality_unordered() {
    let a = parse("#{[1 2] [3 4] [5 6]}").unwrap();
    let b = parse("#{[5 6] [1 2] [3 4]}").unwrap();
    assert_eq!(a, b);
}

#[test]
fn nested_map_in_set_unordered() {
    let a = parse(r#"#{{:a 1} {:b 2}}"#).unwrap();
    let b = parse(r#"#{{:b 2} {:a 1}}"#).unwrap();
    assert_eq!(a, b);
}

// ═══════════════════════════════════════════════════════════════════════
// DISCARD SUPPRESSES BUILT-IN HANDLERS (G2 from /ignorant)
// ═══════════════════════════════════════════════════════════════════════
//
// Per spec L267: "A reader should not call user-supplied tag handlers
// during the processing of the element to be discarded." wat-edn
// extends this to its built-in handlers too — `#inst`/`#uuid`
// validators do NOT run under `#_`, so semantically-bad bodies don't
// cause errors when the entire form is being discarded.

#[test]
fn discard_suppresses_inst_validation() {
    // A bad RFC3339 string would normally error on `#inst`, but
    // under `#_` the validator must not run.
    assert_eq!(
        parse(r#"[1 #_#inst "not-a-date" 2]"#).unwrap(),
        Value::Vector(vec![Value::Integer(1), Value::Integer(2)])
    );
}

#[test]
fn discard_suppresses_uuid_validation() {
    assert_eq!(
        parse(r#"[1 #_#uuid "garbage" 2]"#).unwrap(),
        Value::Vector(vec![Value::Integer(1), Value::Integer(2)])
    );
}

#[test]
fn discard_suppresses_user_tag_namespace_check() {
    // A user tag without a namespace would normally error, but
    // under `#_` it must be discarded silently.
    assert_eq!(
        parse(r#"[1 #_#bareTag 999 2]"#).unwrap(),
        Value::Vector(vec![Value::Integer(1), Value::Integer(2)])
    );
}

// ═══════════════════════════════════════════════════════════════════════
// VALUE SIZE — temper measurement
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn value_size_is_reasonable() {
    let size = std::mem::size_of::<Value>();
    // Symbol/Keyword have Option<String> + String = 48 bytes; that
    // floor + tag/padding is the lower bound. Box<BigInt> /
    // Box<BigDecimal> shrink the BigInt and BigDec variants to one
    // pointer so they don't widen the enum past the
    // Symbol/Keyword/Tagged baseline.
    eprintln!("size_of::<Value>() = {} bytes", size);
    assert!(
        size <= 64,
        "Value enum has grown to {} bytes — investigate before merge",
        size
    );
}

// ═══════════════════════════════════════════════════════════════════════
// DEL (0x7F) string-escape behavior
// ═══════════════════════════════════════════════════════════════════════
//
// Spec doesn't mandate escaping DEL (0x7F is technically "delete"
// but C0 controls are 0x00-0x1F per Unicode). wat-edn emits it
// literally; round-trip works since DEL is a single ASCII byte.

#[test]
fn del_byte_in_string_round_trips() {
    let v = Value::String("a\u{007F}b".into());
    let s = write(&v);
    let v2 = parse(&s).unwrap();
    assert_eq!(v, v2);
}

// ═══════════════════════════════════════════════════════════════════════
// SYMBOL/KEYWORD/TAG CONSTRUCTOR VALIDATION
// ═══════════════════════════════════════════════════════════════════════
//
// Constructors validate per spec; invalid input panics. `try_*`
// constructors return Result for fallible callers.

use wat_edn::{Keyword as Kw, Symbol as Sym};

#[test]
fn symbol_try_new_validates() {
    assert!(Sym::try_new("foo").is_ok());
    assert!(Sym::try_new("foo-bar?").is_ok());
    assert!(Sym::try_new("").is_err());
    assert!(Sym::try_new("123").is_err());
    assert!(Sym::try_new("+1").is_err());
}

#[test]
fn symbol_try_ns_validates() {
    assert!(Sym::try_ns("ns", "foo").is_ok());
    assert!(Sym::try_ns("", "foo").is_err());
    assert!(Sym::try_ns("ns", "").is_err());
    assert!(Sym::try_ns("ns", "123").is_err());
}

#[test]
fn keyword_try_new_validates() {
    assert!(Kw::try_new("foo").is_ok());
    assert!(Kw::try_new("").is_err());
    assert!(Kw::try_new("123").is_err());
}

#[test]
fn tag_try_ns_validates() {
    assert!(Tag::try_ns("myapp", "Person").is_ok());
    assert!(Tag::try_ns("", "Person").is_err());
    assert!(Tag::try_ns("myapp", "").is_err());
}

#[test]
#[should_panic(expected = "invalid symbol name")]
fn symbol_new_panics_on_invalid() {
    let _ = Sym::new("123abc");
}

#[test]
#[should_panic(expected = "invalid keyword name")]
fn keyword_new_panics_on_invalid() {
    let _ = Kw::new("");
}

#[test]
#[should_panic(expected = "invalid tag")]
fn tag_ns_panics_on_invalid() {
    let _ = Tag::ns("myapp", "+1");
}

#[test]
fn symbol_accessors() {
    let s = Sym::ns("myapp", "Order");
    assert_eq!(s.namespace(), Some("myapp"));
    assert_eq!(s.name(), "Order");

    let s = Sym::new("foo");
    assert_eq!(s.namespace(), None);
    assert_eq!(s.name(), "foo");
}
