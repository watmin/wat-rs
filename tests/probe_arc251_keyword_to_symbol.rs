//! FM 2-bis probe — arc 251 head role-inversion: `keyword/to-symbol`.
//!
//! The grammar-inverse primitive that turns a wat rust-scheme call-head KEYWORD into a
//! faithful-Clojure SYMBOL. It is the inverse of `ns_to_wat_path` (edn_shim) — the `::`↔`.`/`/`
//! grammar lives in ONE place; this verb calls its inverse, never re-encodes it.
//!
//! The rule (derived + pressure-tested by an intueri cast, total over the corpus): strip `:`;
//! split on `::`; the last segment is the NAME unless it is `Type/method` (a `/` with a
//! non-empty part before it), in which case `Type` folds into the namespace and `method` is
//! the name; join the namespace with `.`; result `namespace/name`.
//!
//! C01: simple head    — :wat::core::if                       -> wat.core/if
//! C02: division        — :wat::core::/                        -> wat.core//   (the clojure.core// shape)
//! C03: Type/method     — :wat::core::Option/expect           -> wat.core.Option/expect (Type folds into ns)
//! C04: deep + nested   — :wat::kernel::services::StdErrService::Rep/new
//!                                                              -> wat.kernel.services.StdErrService.Rep/new
//! C05: kind change     — the result node IS a Symbol, not a Keyword.
//!
//! RED at HEAD: the verb `:wat::core::keyword/to-symbol` does not exist yet (UnknownFunction).
//!
//! Run: `cargo test --release --test probe_arc251_keyword_to_symbol`

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

/// `(ast-name (keyword/to-symbol (keyword-node "<kw>")))` — convert and read the result's token.
fn convert(kw: &str) -> Result<String, String> {
    eval_string(&format!(
        "(:wat::core::ast-name (:wat::core::keyword/to-symbol (:wat::core::keyword-node \"{kw}\")))"
    ))
}

#[test]
fn contract_01_simple_head() {
    assert_eq!(convert(":wat::core::if"), Ok("wat.core/if".into()));
    assert_eq!(convert(":wat::holon::HolonAST"), Ok("wat.holon/HolonAST".into()));
    assert_eq!(convert(":user::main"), Ok("user/main".into()));
}

#[test]
fn contract_02_division_is_clojure_core_slashslash() {
    assert_eq!(convert(":wat::core::/"), Ok("wat.core//".into()));
    assert_eq!(convert(":wat::core::+"), Ok("wat.core/+".into()));
}

#[test]
fn contract_03_type_method_folds_type_into_namespace() {
    assert_eq!(convert(":wat::core::Option/expect"), Ok("wat.core.Option/expect".into()));
    assert_eq!(convert(":wat::core::HashMap/dissoc"), Ok("wat.core.HashMap/dissoc".into()));
}

#[test]
fn contract_04_deep_and_nested() {
    assert_eq!(
        convert(":wat::kernel::services::StdErrService/handle"),
        Ok("wat.kernel.services.StdErrService/handle".into())
    );
    assert_eq!(
        convert(":wat::kernel::services::StdErrService::Rep/new"),
        Ok("wat.kernel.services.StdErrService.Rep/new".into())
    );
}

#[test]
fn contract_05_result_is_a_symbol_not_a_keyword() {
    assert_eq!(
        eval_string(
            "(:wat::core::ast-kind (:wat::core::keyword/to-symbol (:wat::core::keyword-node \":wat::core::if\")))"
        ),
        Ok("symbol".into()),
        "the converted head is a Symbol node (a call head), not a Keyword"
    );
}
