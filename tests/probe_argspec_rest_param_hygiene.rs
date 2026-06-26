//! FM-2-bis DIAGNOSTIC PROBE — Stone 249.5d (ArgSpec carries the Identifier).
//!
//! Does a macro-generated `defclause` WITH a rest param (`& rest <- :T`) resolve
//! its fixed params at call time? This is the disconfirming contract for the
//! ArgSpec strip-and-re-walk root fix.
//!
//! THE BUG (at HEAD): `scope::scoped_arg_names` (src/scope/resolution.rs:122)
//! guards `if items.len() % 3 != 0 { return fallback }`. A rest-binder argspec
//! has `3n + 4` items — `3n` fixed (n triples) + `&` (1) + the 3-item rest triple
//! — and `3n + 4 ≡ 1 (mod 3)`. The guard fires, so the FIXED-param names fall
//! back to BARE. But `walk_template` (macros/expand.rs) tagged every
//! template-origin symbol — the binders AND the body references — with the
//! macro's fresh scope. So the defclause binds `x`/`y` BARE while the body looks
//! them up SCOPED (`x\u{1}<scope>`) → bind-key ≠ lookup-key → `UnboundSymbol` at
//! the first param reference inside the clause body.
//!
//! The pre-rest hygiene probe (`probe_macro_hygiene_capture.rs`) uses NO rest
//! param (6-item argspec, `% 3 == 0`), so it never exercised this guard — the
//! rest-param case is the unexercised path the root fix closes.
//!
//! THE FIX (Stone 249.5d): `ArgSpec.fixed_params`/`rest_param` carry the
//! `Identifier` (not a bare `String`); the defclause registration derives each
//! scoped key via `env_key` over the parsed identifiers — no re-walk, no `% 3`
//! guard. Both fixed and rest params resolve.
//!
//! RED at HEAD (`UnboundSymbol`); GREEN after the fix (returns 10 = 1+2+3+4).
//!
//! Run: cargo test --release --test probe_argspec_rest_param_hygiene -- --nocapture

use std::sync::Arc;
use wat::freeze::{eval_in_frozen, startup_from_source};
use wat::load::InMemoryLoader;
use wat::runtime::{Environment, Value};

fn eval_i64(decls: &str, body: &str) -> Result<Value, String> {
    let src = format!(
        "{decls}\n(:wat::core::defn :user::compute [] -> :wat::core::i64 {body})\n\
         (:wat::core::defn :user::main [] -> :wat::core::nil nil)",
    );
    let world = startup_from_source(&src, None, Arc::new(InMemoryLoader::new()))
        .map_err(|e| format!("startup: {:?}", e))?;
    let ast = wat::parse_one!("(:user::compute)").map_err(|e| format!("parse: {:?}", e))?;
    let env = Environment::new();
    eval_in_frozen(&ast, &world, &env)
        .map(|tv| tv.value_owned())
        .map_err(|e| format!("eval: {:?}", e))
}

// A macro that, called at top-level, expands to a defclause WITH a rest param.
// `walk_template` tags EVERY template-origin symbol — the fixed binders x/y, the
// rest binder `rest`, AND their body references — with the macro's fresh scope.
// The body mirrors the stdlib variadic idiom (wat/core.wat:44-53): fold over the
// rest, seeded with the fixed params, so x, y, and rest all must resolve.
const MAKE_REST_SUM: &str = "\
(:wat::core::defmacro :test::make-rest-sum \
  [] -> :wat::WatAST \
  `(:wat::core::defclause :test::rest-sum \
     ([x <- :wat::core::i64 y <- :wat::core::i64 \
       & rest <- :wat::core::Vector<wat::core::i64>] -> :wat::core::i64 \
       (:wat::core::foldl \
         (:wat::core::fn [acc <- :wat::core::i64 n <- :wat::core::i64] -> :wat::core::i64 \
           (:wat::core::i64::+ acc n)) \
         (:wat::core::i64::+ x y) \
         rest))))";

// Call the macro at top-level so it expands to and registers the defclause.
const CALL_MAKE_REST_SUM: &str = "(:test::make-rest-sum)";

/// REST-PARAM HYGIENE GUARD — a macro-generated defclause WITH a rest param must
/// resolve its scope-tagged fixed params at call time.
///
/// At HEAD the `% 3` guard in `scoped_arg_names` ejects the rest-binder argspec,
/// baring the fixed-param bind keys while the body looks them up scoped →
/// `UnboundSymbol`. Stone 249.5d (ArgSpec carries the Identifier; `env_key` over
/// the parsed identifiers; the re-walk deleted) makes bind-key == lookup-key.
#[test]
fn macro_generated_defclause_with_rest_resolves_params() {
    let decls = format!("{MAKE_REST_SUM}\n{CALL_MAKE_REST_SUM}");
    let body = "(:test::rest-sum 1 2 3 4)";
    let result = eval_i64(&decls, body).expect(
        "macro-generated defclause WITH a rest param must evaluate without \
         UnboundSymbol; failure = the `% 3` guard bared the fixed params while \
         the scope-tagged body looked them up scoped (Stone 249.5d root fix)",
    );
    assert_eq!(
        result,
        Value::i64(10),
        "REST HYGIENE: macro-generated defclause body must resolve x, y AND rest \
         (1 + 2 + 3 + 4 = 10). Got {:?}",
        result,
    );
}
