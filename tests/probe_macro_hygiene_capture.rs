//! FM-2-bis DIAGNOSTIC PROBE — does wat's sets-of-scopes hygiene actually
//! prevent classic macro variable capture at RUNTIME?
//!
//! circumspicere (arc 249 macros re-ward) flagged a CLAIM-vs-CODE contradiction:
//! `mod.rs`/`identifier.rs` claim "lexical-scope lookup compares (name, scope_set)
//! pairs so a macro's `tmp` and a user's `tmp` resolve to distinct bindings" — but
//! `Environment` is `HashMap<String, BoundEntry>` keyed on the BARE name, and
//! `Identifier::add_scope` adds to a separate `.scopes` field, leaving `.name`
//! bare. Scope sets feed AST hashing (program identity) but appear UNUSED at
//! runtime lookup. No hygiene test exists.
//!
//! This probe answers it empirically (let the substrate decide, not conviction):
//!
//! A macro introduces `tmp` in a `let` binder; the caller passes its OWN `tmp`
//! as the unquoted arg. Template: `(let [tmp 100] (i64::+ tmp ~x))`.
//! Caller: `(let [tmp 5] (add-via-tmp tmp))`. Expands to
//! `(let [tmp{macro-scope} 100] (i64::+ tmp{macro-scope} tmp{user-scope}))`.
//!   - HYGIENIC  → the spliced `~x` (user's tmp, =5) stays distinct from the
//!     macro's `tmp` (=100) → 100 + 5 = 105.
//!   - CAPTURED  → the spliced `~x` resolves to the macro's inner `let tmp`
//!     binding (=100) → 100 + 100 = 200.
//!
//! Run: cargo test --release --test probe_macro_hygiene_capture -- --nocapture

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

const CAPTURE_MACRO: &str = "(:wat::core::defmacro :test::add-via-tmp \
     [x <- :wat::holon::HolonAST] -> :AST<wat::holon::HolonAST> \
     `(:wat::core::let [tmp 100] (:wat::core::i64::+ tmp ~x)))";

/// HYGIENE REGRESSION GUARD — proves wat's macro expansion is hygienic: a
/// macro-introduced binding does NOT capture a caller's same-named variable.
///
/// History: this was the RED contract for Stone 249.5b. `walk_template` already
/// TAGGED template symbols with a fresh macro scope (expand.rs:681), but the
/// runtime resolved names string-only (`Environment` = `HashMap<String, _>`), so
/// the tag was inert and the macro's `tmp` CAPTURED the caller's `tmp` — 200, not
/// 105. Stone 249.5b closed it by routing every Identifier-keyed bind/lookup
/// through `scope::resolution::env_key` (the scope set is now load-bearing). This
/// test went 200 → 105; `mod.rs`'s "variable capture is structurally impossible"
/// claim is now TRUE, and this guard keeps it true.
#[test]
fn classic_macro_capture_is_prevented() {
    // Caller binds its own `tmp` to 5, then calls a macro that introduces its
    // own `tmp`-binder; passes the caller's `tmp` as the unquoted arg.
    let body = "(:wat::core::let [tmp 5] (:test::add-via-tmp tmp))";
    let result = eval_i64(CAPTURE_MACRO, body).expect("expansion + eval should succeed");
    assert_eq!(
        result,
        Value::i64(105),
        "HYGIENE: the macro's `let [tmp 100]` must NOT capture the caller's `tmp` (=5). \
         105 = hygienic (user tmp distinct); 200 = captured. Got {:?}",
        result
    );
}
