//! Locks the `Value::as_*` facade. Each accessor returns Some on
//! its matching variant and None on every other variant. Surfaced
//! by the /reap ward — without these tests the accessors ship
//! without proof of life.

use bigdecimal::BigDecimal;
use chrono::{TimeZone, Utc};
use num_bigint::BigInt;
use std::str::FromStr;
use uuid::Uuid;
use wat_edn::{Keyword, Symbol, Tag, Value};

fn all_variants() -> Vec<(&'static str, Value<'static>)> {
    vec![
        ("nil", Value::Nil),
        ("bool", Value::Bool(true)),
        ("integer", Value::Integer(42)),
        ("bigint", Value::BigInt(Box::new(BigInt::from(123)))),
        ("float", Value::Float(2.5)),
        ("bigdec", Value::BigDec(Box::new(BigDecimal::from_str("2.5").unwrap()))),
        ("string", Value::String("hello".into())),
        ("char", Value::Char('a')),
        ("symbol", Value::Symbol(Symbol::new("foo"))),
        ("keyword", Value::Keyword(Keyword::new("foo"))),
        ("list", Value::List(vec![Value::Integer(1)])),
        ("vector", Value::Vector(vec![Value::Integer(1)])),
        ("map", Value::Map(vec![(Value::Integer(1), Value::Integer(2))])),
        ("set", Value::Set(vec![Value::Integer(1)])),
        (
            "tagged",
            Value::Tagged(Tag::ns("myapp", "Foo"), Box::new(Value::Nil)),
        ),
        ("inst", Value::Inst(Utc.timestamp_opt(0, 0).unwrap())),
        ("uuid", Value::Uuid(Uuid::nil())),
    ]
}

#[test]
fn type_name_per_variant() {
    let names: Vec<_> = all_variants().iter().map(|(n, _)| *n).collect();
    let computed: Vec<_> = all_variants().iter().map(|(_, v)| v.type_name()).collect();
    assert_eq!(names, computed);
}

#[test]
fn is_nil_only_on_nil() {
    for (name, v) in all_variants() {
        assert_eq!(v.is_nil(), name == "nil", "is_nil for {}", name);
    }
}

#[test]
fn as_bool_matches() {
    for (name, v) in all_variants() {
        let r = v.as_bool();
        if name == "bool" {
            assert_eq!(r, Some(true));
        } else {
            assert!(r.is_none(), "{} should not unwrap as bool", name);
        }
    }
}

#[test]
fn as_i64_matches() {
    for (name, v) in all_variants() {
        let r = v.as_i64();
        if name == "integer" {
            assert_eq!(r, Some(42));
        } else {
            assert!(r.is_none(), "{} should not unwrap as i64", name);
        }
    }
}

#[test]
fn as_f64_matches() {
    for (name, v) in all_variants() {
        let r = v.as_f64();
        if name == "float" {
            assert_eq!(r, Some(2.5));
        } else {
            assert!(r.is_none(), "{} should not unwrap as f64", name);
        }
    }
}

#[test]
fn as_str_matches() {
    for (name, v) in all_variants() {
        let r = v.as_str();
        if name == "string" {
            assert_eq!(r, Some("hello"));
        } else {
            assert!(r.is_none(), "{} should not unwrap as str", name);
        }
    }
}

#[test]
fn as_char_matches() {
    for (name, v) in all_variants() {
        let r = v.as_char();
        if name == "char" {
            assert_eq!(r, Some('a'));
        } else {
            assert!(r.is_none(), "{} should not unwrap as char", name);
        }
    }
}

#[test]
fn as_symbol_matches() {
    for (name, v) in all_variants() {
        let r = v.as_symbol();
        if name == "symbol" {
            assert_eq!(r.unwrap().name(), "foo");
        } else {
            assert!(r.is_none(), "{} should not unwrap as symbol", name);
        }
    }
}

#[test]
fn as_keyword_matches() {
    for (name, v) in all_variants() {
        let r = v.as_keyword();
        if name == "keyword" {
            assert_eq!(r.unwrap().name(), "foo");
        } else {
            assert!(r.is_none(), "{} should not unwrap as keyword", name);
        }
    }
}

#[test]
fn as_list_matches() {
    for (name, v) in all_variants() {
        let r = v.as_list();
        if name == "list" {
            assert_eq!(r.unwrap().len(), 1);
        } else {
            assert!(r.is_none(), "{} should not unwrap as list", name);
        }
    }
}

#[test]
fn as_vector_matches() {
    for (name, v) in all_variants() {
        let r = v.as_vector();
        if name == "vector" {
            assert_eq!(r.unwrap().len(), 1);
        } else {
            assert!(r.is_none(), "{} should not unwrap as vector", name);
        }
    }
}

#[test]
fn as_map_matches() {
    for (name, v) in all_variants() {
        let r = v.as_map();
        if name == "map" {
            assert_eq!(r.unwrap().len(), 1);
        } else {
            assert!(r.is_none(), "{} should not unwrap as map", name);
        }
    }
}

#[test]
fn as_set_matches() {
    for (name, v) in all_variants() {
        let r = v.as_set();
        if name == "set" {
            assert_eq!(r.unwrap().len(), 1);
        } else {
            assert!(r.is_none(), "{} should not unwrap as set", name);
        }
    }
}

#[test]
fn as_tagged_matches() {
    for (name, v) in all_variants() {
        let r = v.as_tagged();
        if name == "tagged" {
            let (tag, _body) = r.unwrap();
            assert_eq!(tag.namespace(), "myapp");
            assert_eq!(tag.name(), "Foo");
        } else {
            assert!(r.is_none(), "{} should not unwrap as tagged", name);
        }
    }
}

#[test]
fn as_inst_matches() {
    for (name, v) in all_variants() {
        let r = v.as_inst();
        if name == "inst" {
            assert!(r.is_some());
        } else {
            assert!(r.is_none(), "{} should not unwrap as inst", name);
        }
    }
}

#[test]
fn as_uuid_matches() {
    for (name, v) in all_variants() {
        let r = v.as_uuid();
        if name == "uuid" {
            assert_eq!(r.unwrap(), &Uuid::nil());
        } else {
            assert!(r.is_none(), "{} should not unwrap as uuid", name);
        }
    }
}
