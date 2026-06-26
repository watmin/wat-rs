//! Strike 2 (examinare disconfirming probe) — `keyword/to-type-form`: the type-converter.
//!
//! fix-source's hardest rule, as a decomplected Rust primitive: convert an old rust-scheme
//! TYPE keyword into the faithful-Clojure type FORM. It does NOT re-implement the `<>`/`::`
//! grammar — it PARSES via the existing type parser (`parse_type_expr` → `TypeExpr`), then
//! RENDERS the closed `TypeExpr` enum back out as faithful WatAST:
//!   Path(named, has `::`)  -> Symbol `wat.type/Name`   (flat type namespace)
//!   Path(type-var, no `::`) -> Symbol `T`               (bare; type-vars stay symbols)
//!   Parametric{head,args}   -> List `(wat.type/Head …rendered-args)`  (recursive)
//!   Fn{args,ret}            -> Vector `[…args :-> ret]`
//!
//! (Working verb name `keyword/to-type-form` — intueri to ratify.)
//!
//! C01 scalar      : :wat::core::i64                              -> wat.type/i64
//! C02 parametric  : :wat::core::Vector<wat::core::i64>           -> (wat.type/Vector wat.type/i64)
//! C03 nested      : :wat::core::Vector<wat::core::Vector<...>>   -> (wat.type/Vector (wat.type/Vector wat.type/i64))
//! C04 type-var    : :wat::core::Vector<T>                        -> (wat.type/Vector T)   (T stays bare)
//! C05 multi-arg   : :wat::core::HashMap<wat::core::String,...>   -> (wat.type/HashMap wat.type/String wat.type/i64)
//!
//! RED at HEAD: the verb `:wat::core::keyword/to-type-form` does not exist (UnknownFunction).
//!
//! Run: `cargo test --release --test probe_arc251_keyword_to_type_form`

use std::sync::Arc;
use wat::freeze::{eval_in_frozen, startup_from_source};
use wat::load::InMemoryLoader;
use wat::runtime::{Environment, Value};

fn eval_string(body: &str) -> Result<String, String> {
    let src = format!(
        "(:wat::core::defn :user::compute [] -> :wat::core::String {body})\n\
         (:wat::core::defn :user::main [] -> :wat::core::nil nil)",
    );
    let world = startup_from_source(&src, None, Arc::new(InMemoryLoader::new()))
        .map_err(|e| format!("startup/check: {e:?}"))?;
    let ast = wat::parse_one!("(:user::compute)").expect("parse");
    match eval_in_frozen(&ast, &world, &Environment::new())
        .map(|tv| tv.value_owned())
        .map_err(|e| format!("eval: {e:?}"))?
    {
        Value::String(s) => Ok((*s).clone()),
        other => Err(format!("non-string: {other:?}")),
    }
}

/// `(write-forms (keyword/to-type-form (keyword-node "<kw>")))` — render the faithful type.
fn to_type(kw: &str) -> Result<String, String> {
    eval_string(&format!(
        "(:wat::core::write-forms (:wat::core::keyword/to-type-form (:wat::core::keyword-node \"{kw}\")))"
    ))
}

#[test]
fn contract_01_scalar() {
    assert_eq!(to_type(":wat::core::i64"), Ok("wat.type/i64".into()));
    assert_eq!(to_type(":wat::holon::HolonAST"), Ok("wat.holon/HolonAST".into()));
}

#[test]
fn contract_02_parametric() {
    assert_eq!(
        to_type(":wat::core::Vector<wat::core::i64>"),
        Ok("(wat.type/Vector wat.type/i64)".into())
    );
}

#[test]
fn contract_03_nested_parametric() {
    assert_eq!(
        to_type(":wat::core::Vector<wat::core::Vector<wat::core::i64>>"),
        Ok("(wat.type/Vector (wat.type/Vector wat.type/i64))".into())
    );
}

#[test]
fn contract_04_type_var_stays_bare() {
    assert_eq!(
        to_type(":wat::core::Vector<T>"),
        Ok("(wat.type/Vector T)".into()),
        "a type-var (Path with no `::`) renders as a bare symbol, not wat.type/T"
    );
}

#[test]
fn contract_05_multi_arg() {
    assert_eq!(
        to_type(":wat::core::HashMap<wat::core::String,wat::core::i64>"),
        Ok("(wat.type/HashMap wat.type/String wat.type/i64)".into())
    );
}

#[test]
fn contract_06_tuple() {
    assert_eq!(
        to_type(":(wat::core::i64,wat::core::String)"),
        Ok("(wat.type/Tuple wat.type/i64 wat.type/String)".into())
    );
}

#[test]
fn contract_07_empty_tuple_is_not_nil() {
    // `:()` is the 0-tuple — a distinct zero-arity product, NOT unit (`nil` is wat's unit).
    assert_eq!(to_type(":()"), Ok("(wat.type/Tuple)".into()));
}

#[test]
fn contract_08_nested_tuple() {
    assert_eq!(
        to_type(":(wat::core::Vector<T>,wat::core::i64)"),
        Ok("(wat.type/Tuple (wat.type/Vector T) wat.type/i64)".into())
    );
}

#[test]
fn contract_09_tuple_form_round_trips_as_a_type() {
    // The faithful form must PARSE BACK as a tuple type — proving the parser case
    // (`(wat.type/Tuple …)` → TypeExpr::Tuple), so the conversion is a real re-spelling.
    let src = "(:wat::core::defn :user::f [t :- (wat.type/Tuple wat.type/i64 wat.type/String)] \
                  -> :wat::core::nil nil)\n\
               (:wat::core::defn :user::main [] -> :wat::core::nil nil)";
    let r = startup_from_source(src, None, Arc::new(InMemoryLoader::new()))
        .map(|_| ())
        .map_err(|e| format!("{e:?}"));
    assert!(r.is_ok(), "(wat.type/Tuple wat.type/i64 wat.type/String) must parse as a type; got {r:?}");
}
