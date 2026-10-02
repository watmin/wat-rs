//! Arc 255.81 amend 3 — a retired hard-primitive name is never silent.
//!
//! `type-equal?` of a keyword built from the old spelling raises the retirement
//! remedy (it must not answer `false`). A constructor head of that spelling
//! raises the same remedy (it must not be read as a field of its element).
//!
//! The programs live in the co-located `.wat`. The constructor form is a string
//! the fixture returns: a checked program of that head never reaches
//! `eval_in_frozen`, because check refuses it first.

use wat::freeze::{call_beside_value, startup_beside};
use wat::runtime::RuntimeErrorKind;
use wat::Value;

fn expect_retirement(fn_name: &str, head: &str, reason: &str) {
    match call_beside_value(file!(), fn_name) {
        Err(err) => match err.kind() {
            RuntimeErrorKind::MalformedForm { head: got_head, reason: got_reason } => {
                assert_eq!(got_head, head);
                assert_eq!(got_reason, reason);
            }
            other => panic!("expected MalformedForm, got {other:?}"),
        },
        Ok(v) => panic!("retired name must not return, got {v:?}"),
    }
}

#[test]
fn type_equal_of_a_retired_keyword_names_the_new_spelling() {
    expect_retirement(
        ":user::type-equal",
        ":wat::core::i64",
        "':wat::core::i64' is retired (arc 255.81); use 'wat.type/i64' instead \
         (wat.type/ holds exactly the 24 hard primitives, and this is one \
         of them — the old :wat::core:: home no longer resolves in a type \
         position)",
    );
}

#[test]
fn metadata_of_a_retired_keyword_names_the_new_spelling() {
    expect_retirement(
        ":user::metadata",
        ":wat::core::char",
        "':wat::core::char' is retired (arc 255.81); use 'wat.type/char' instead \
         (wat.type/ holds exactly the 24 hard primitives, and this is one \
         of them — the old :wat::core:: home no longer resolves in a type \
         position)",
    );
}

#[test]
fn render_doc_of_a_retired_keyword_names_the_new_spelling() {
    expect_retirement(
        ":user::render-doc",
        ":wat::core::char",
        "':wat::core::char' is retired (arc 255.81); use 'wat.type/char' instead \
         (wat.type/ holds exactly the 24 hard primitives, and this is one \
         of them — the old :wat::core:: home no longer resolves in a type \
         position)",
    );
}

#[test]
fn retired_constructor_head_names_itself() {
    let src = match call_beside_value(file!(), ":user::constructor-src") {
        Ok(Value::String(s)) => (*s).clone(),
        other => panic!("constructor source must be a string, got {other:?}"),
    };
    let world = startup_beside(file!()).expect("fixture freezes");
    let ast = wat::parse_one_with_file(&src, "constructor-src").expect("parse");
    match wat::freeze::eval_in_frozen(&ast, &world, &wat::runtime::Environment::new()) {
        Err(err) => match err.kind() {
            RuntimeErrorKind::MalformedForm { head: got_head, reason: got_reason } => {
                assert_eq!(got_head, ":wat::core::PersistentVector");
                assert_eq!(
                    got_reason,
                    "':wat::core::PersistentVector' is retired (arc 255.81); use 'wat.type/PersistentVector' instead \
                     (wat.type/ holds exactly the 24 hard primitives, and this is one \
                     of them — the old :wat::core:: home no longer resolves in a type \
                     position)"
                );
            }
            other => panic!("expected MalformedForm, got {other:?}"),
        },
        Ok(v) => panic!("retired name must not return, got {v:?}"),
    }
}
