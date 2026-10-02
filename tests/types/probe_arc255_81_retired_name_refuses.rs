// rune:lint(no-inlined-wat) — each string is the retired name under test. A co-located
// `.wat` of the bare head is refused at check, so the runtime door (type-equal?,
// metadata-of, render-doc, and an unchecked constructor head) is only reachable
// from source the checker does not load as the program.
//! Arc 255.81 amend 3 — a retired hard-primitive name is never silent.
//!
//! `type-equal?` of a keyword built from the old spelling raises the retirement
//! remedy (it must not answer `false`). A constructor head of that spelling
//! raises the same remedy (it must not be read as a field of its element).

use std::sync::Arc;
use wat::freeze::{invoke_user_main, startup_from_source};
use wat::load::loader::InMemoryLoader;
use wat::runtime::RuntimeErrorKind;

fn run(src: &str) -> Result<(), wat::runtime::RuntimeError> {
    let world = startup_from_source(src, Some(concat!(file!(), ":", line!())), Arc::new(InMemoryLoader::new()))
        .expect("startup");
    invoke_user_main(&world, Vec::new()).map(|_| ())
}

fn expect_retirement(src: &str, head: &str, reason: &str) {
    match run(src) {
        Err(err) => match err.kind() {
            RuntimeErrorKind::MalformedForm { head: got_head, reason: got_reason } => {
                assert_eq!(got_head, head);
                assert_eq!(got_reason, reason);
            }
            other => panic!("expected MalformedForm, got {other:?}"),
        },
        Ok(()) => panic!("retired name must not return"),
    }
}

/// `eval_in_frozen` does not type-check (the rete `format!` templates reach dispatch
/// this way). A checked program never gets here: the retirement table already
/// refuses the head at check time.
fn expect_retirement_unchecked(form: &str, head: &str, reason: &str) {
    let world = startup_from_source(
        "(:wat::core::defn :user::anchor [] -> wat.type/i64 1)",
        Some(concat!(file!(), ":", line!())),
        Arc::new(InMemoryLoader::new()),
    )
    .expect("startup");
    let ast = wat::parse_one!(form).expect("parse");
    match wat::freeze::eval_in_frozen(&ast, &world, &wat::runtime::Environment::new()) {
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
    let src = r#"
(:wat::core::defn :user::main [] -> wat.type/nil
  (:wat::core::type-equal?
    (:wat::core::keyword-node ":wat::type::i64")
    (:wat::core::keyword-node ":wat::core::i64")))
"#;
    expect_retirement(
        src,
        ":wat::core::i64",
        "':wat::core::i64' is retired (arc 255.81); use 'wat.type/i64' instead \
         (wat.type/ holds exactly the 24 hard primitives, and this is one \
         of them — the old :wat::core:: home no longer resolves in a type \
         position)",
    );
}

#[test]
fn metadata_of_a_retired_keyword_names_the_new_spelling() {
    let src = r#"
(:wat::core::defn :user::main [] -> wat.type/nil
  (:wat::runtime::metadata-of (:wat::core::keyword-node ":wat::core::char")))
"#;
    expect_retirement(
        src,
        ":wat::core::char",
        "':wat::core::char' is retired (arc 255.81); use 'wat.type/char' instead \
         (wat.type/ holds exactly the 24 hard primitives, and this is one \
         of them — the old :wat::core:: home no longer resolves in a type \
         position)",
    );
}

#[test]
fn render_doc_of_a_retired_keyword_names_the_new_spelling() {
    let src = r#"
(:wat::core::defn :user::main [] -> wat.type/nil
  (:wat::kernel::println
    (:wat::core::render-doc (:wat::core::keyword-node ":wat::core::char"))))
"#;
    expect_retirement(
        src,
        ":wat::core::char",
        "':wat::core::char' is retired (arc 255.81); use 'wat.type/char' instead \
         (wat.type/ holds exactly the 24 hard primitives, and this is one \
         of them — the old :wat::core:: home no longer resolves in a type \
         position)",
    );
}

#[test]
fn retired_constructor_head_names_itself() {
    expect_retirement_unchecked(
        "(:wat::core::PersistentVector :- [:wat::type::i64] 1)",
        ":wat::core::PersistentVector",
        "':wat::core::PersistentVector' is retired (arc 255.81); use 'wat.type/PersistentVector' instead \
         (wat.type/ holds exactly the 24 hard primitives, and this is one \
         of them — the old :wat::core:: home no longer resolves in a type \
         position)",
    );
}
