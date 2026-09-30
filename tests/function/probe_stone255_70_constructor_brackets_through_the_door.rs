//! STONE 255.70 — a constructor's `:-` bracket reads through the type door; `List` takes `:-`.
//! See the co-located `.wat`'s header for the two checker gaps this guards
//! (SCORE-STONE-255.69 § "What 255.69 left").

use wat::freeze::call_beside_value;
use wat::runtime::Value;

#[test]
fn vector_of_tuples_compound_element_type_runs() {
    match call_beside_value(file!(), ":user::vector-of-tuples") {
        Ok(Value::wat__core__PersistentVector(v)) => {
            assert_eq!(v.len(), 2, "a PersistentVector of 2 Tuples must have length 2");
            for elem in v.iter() {
                assert!(matches!(elem, Value::Tuple(t) if t.len() == 2), "each element must be a 2-tuple; got {elem:?}");
            }
        }
        other => panic!("(wat.type/PersistentVector :- [(wat.type/Tuple :- […])] …) must run; got {other:?}"),
    }
}

#[test]
fn map_string_to_vector_compound_element_type_runs() {
    match call_beside_value(file!(), ":user::map-string-to-vector") {
        Ok(Value::wat__core__PersistentMap(m)) => {
            assert_eq!(m.len(), 2, "a PersistentMap of 2 String->PersistentVector pairs must have length 2");
        }
        other => panic!("(wat.type/PersistentMap :- [wat.type/String (wat.type/PersistentVector :- […])] …) must run; got {other:?}"),
    }
}

#[test]
fn list_typed_value_position_runs() {
    match call_beside_value(file!(), ":user::list-typed") {
        Ok(Value::wat__core__List(l)) => {
            assert_eq!(l.len(), 3, "(wat.type/List :- [wat.type/i64] 1 2 3) must yield a length-3 list");
        }
        other => panic!("(wat.type/List :- [T] 1 2 3) must run; got {other:?}"),
    }
}

#[test]
fn list_typed_empty_is_the_empty_list() {
    match call_beside_value(file!(), ":user::list-typed-empty") {
        Ok(Value::wat__core__List(l)) => {
            assert_eq!(l.len(), 0, "(wat.type/List :- [T]) with no elements must be the empty list");
        }
        other => panic!("(wat.type/List :- [T]) must run to the empty list; got {other:?}"),
    }
}

// `list_bracketless_still_runs` MOVED OUT arc 255 STONE 71 (THE WALL): a bracket-less `List`
// call is illegal now (255.70's STOP-2 guarantee — "the bracket stays optional" — was scoped to
// its OWN stone; the wall is the later stone that guarantee named). Its replacement, proving the
// wall refuses it by name, is
// `tests/function/probe_stone255_71_list_bracketless_illegal.rs`.

#[test]
fn list_of_tuples_compound_element_type_runs() {
    match call_beside_value(file!(), ":user::list-of-tuples") {
        Ok(Value::wat__core__List(l)) => {
            assert_eq!(l.len(), 1, "(wat.type/List :- [(wat.type/Tuple :- […])] …) must yield a length-1 list");
        }
        other => panic!("(wat.type/List :- [(Tuple :- […])] …) must run; got {other:?}"),
    }
}
