//! FM-2-bis PROBE-LED diagnostic for Arc 249 Stone 249.4 — can `keyword/of` and
//! `for` be reborn as WAT macros over the total-pure engine, replacing their Rust
//! built-ins (expand.rs `construct_keyword_of` + `match_for_comprehension`)?
//!
//! PROBE-LED, not conviction-led (REALIZATIONS §"the practitioner is the failure
//! domain"): attempt the natural wat encoding; let the substrate name the gap.
//!
//! `keyword/of` builds `:Head<arg1,arg2>` from keyword args by munging keyword
//! text. In a wat macro the args bind as `wat__WatAST(Keyword)` FORM-values; to
//! munge text we need keyword-form → text. `keyword/to-string` wants a keyword
//! VALUE — so this is expected to hit the same form-vs-value gap threading hit
//! with first/rest. This probe confirms (or refutes) that, empirically.
//!
//! Run: cargo test --release --test probe_arc249_4_rehome_in_wat -- --ignored --nocapture

use std::sync::Arc;
use wat::freeze::{eval_in_frozen, startup_from_source};
use wat::load::InMemoryLoader;
use wat::runtime::{Environment, Value};

fn with_nil_main(src: &str) -> String {
    format!(
        "{}\n(:wat::core::defn :user::main [] -> :wat::core::nil nil)",
        src
    )
}

/// Eval a String-returning `:user::compute` with body `body`, after `decls`.
fn eval_string_with(decls: &str, body: &str) -> Result<Value, String> {
    let src = format!(
        "{decls}\n(:wat::core::defn :user::compute [] -> :wat::core::String {body})",
    );
    let full = with_nil_main(&src);
    let world = startup_from_source(&full, None, Arc::new(InMemoryLoader::new()))
        .map_err(|e| format!("startup: {:?}", e))?;
    let ast = wat::parse_one!("(:user::compute)").map_err(|e| format!("parse: {:?}", e))?;
    let env = Environment::new();
    eval_in_frozen(&ast, &world, &env)
        .map(|tv| tv.value_owned())
        .map_err(|e| format!("eval: {:?}", e))
}

/// Eval an i64-returning `:user::compute`.
fn eval_i64_with(decls: &str, body: &str) -> Result<Value, String> {
    let src = format!(
        "{decls}\n(:wat::core::defn :user::compute [] -> :wat::core::i64 {body})",
    );
    let full = with_nil_main(&src);
    let world = startup_from_source(&full, None, Arc::new(InMemoryLoader::new()))
        .map_err(|e| format!("startup: {:?}", e))?;
    let ast = wat::parse_one!("(:user::compute)").map_err(|e| format!("parse: {:?}", e))?;
    let env = Environment::new();
    eval_in_frozen(&ast, &world, &env)
        .map(|tv| tv.value_owned())
        .map_err(|e| format!("eval: {:?}", e))
}

// ═══════════════════════════════════════════════════════════════════════════
// C — first/rest over a VECTOR form. The `for` rehome `(for [x xs] tmpl)` must
// decompose the `[x xs]` binder Vector form to extract x + xs. 249.3a-ii added
// first/rest for wat__WatAST(List) ONLY — a Vector form `[10 20]` hits the
// non-List arm → None. `(:test::vec-first [10 20])` → if first-over-Vector
// works, expands to `(i64::+ 10 0)` → 10; if None, Option/expect panics → the
// gap. Expected to FAIL until form-decomposition extends to Vector nodes.
// ═══════════════════════════════════════════════════════════════════════════
const VEC_FIRST_MACRO: &str = "(:wat::core::defmacro :test::vec-first \
     [v <- :wat::holon::HolonAST] -> :AST<wat::holon::HolonAST> \
     `(:wat::core::i64::+ ~(:wat::core::Option/expect -> :wat::holon::HolonAST (:wat::core::first v) \"empty\") 0))";

#[test]
#[ignore = "249.4 diagnostic — run with --ignored to read the gap"]
fn diag_first_over_vector_form() {
    let result = eval_i64_with(VEC_FIRST_MACRO, "(:test::vec-first [10 20])");
    println!("\n=== diag_first_over_vector_form ===\nexpect Ok(10):\n{:#?}\n", result);
    let _ = result;
}

// ═══════════════════════════════════════════════════════════════════════════
// A — keyword-form → text. Can a wat macro turn its keyword ARG (a wat__WatAST
// Keyword form-value) into the keyword's text? `(:test::kw-text :foo::bar)`
// should expand to the string "foo::bar" (or ":foo::bar"). If keyword/to-string
// rejects the form-value, the error names the gap (keyword-form introspection).
// ═══════════════════════════════════════════════════════════════════════════
const KW_TEXT_MACRO: &str = "(:wat::core::defmacro :test::kw-text \
     [k <- :wat::holon::HolonAST] -> :AST<wat::holon::HolonAST> \
     `~(:wat::core::keyword/to-string k))";

#[test]
#[ignore = "249.4 diagnostic — run with --ignored to read the gap"]
fn diag_keyword_to_string_over_form() {
    let result = eval_string_with(
        KW_TEXT_MACRO,
        "(:test::kw-text :foo::bar)",
    );
    println!("\n=== diag_keyword_to_string_over_form ===\n{:#?}\n", result);
    // Diagnostic — read the shape (a string, or the gap error).
    let _ = result;
}

// ═══════════════════════════════════════════════════════════════════════════
// B — FULL keyword/of as a wat macro. `(:test::kw-of :foo :bar :baz)` should
// build the parametric keyword `:foo<bar,baz>` (mirroring the Rust
// construct_keyword_of: {head}<{arg1},{arg2}>, colons stripped). Verify by
// round-tripping its text: keyword/to-string of the built keyword == "foo<bar,baz>".
// Surfaces any gap in string::join arg-order / keyword/from-string colon handling.
// ═══════════════════════════════════════════════════════════════════════════
const KW_OF_MACRO: &str = "(:wat::core::defmacro :test::kw-of \
     [head <- :wat::holon::HolonAST & args <- :AST<wat::holon::Holons>] \
     -> :AST<wat::holon::HolonAST> \
     (:wat::core::let [head-text (:wat::core::keyword/to-string head) \
                       arg-texts (:wat::core::map \
                                   (:wat::core::fn [a <- :wat::holon::HolonAST] -> :wat::core::String \
                                      (:wat::core::keyword/to-string a)) \
                                   args) \
                       joined (:wat::core::string::join \",\" arg-texts) \
                       full (:wat::core::string::concat head-text \
                              (:wat::core::string::concat \"<\" \
                                (:wat::core::string::concat joined \">\")))] \
        `~(:wat::core::keyword/from-string full)))";

#[test]
fn diag_keyword_of_full() {
    let result = eval_string_with(
        KW_OF_MACRO,
        "(:wat::core::keyword/to-string (:test::kw-of :foo :bar :baz))",
    );
    println!("\n=== diag_keyword_of_full ===\nexpect \"foo<bar,baz>\":\n{:#?}\n", result);
    // Read the shape; if Ok("foo<bar,baz>") the rehome works end-to-end.
    let _ = result;
}

// ═══════════════════════════════════════════════════════════════════════════
// D — keyword/of in TEMPLATE POSITION (the KEY RISK — replaces the deleted
// src/macros/tests.rs::keyword_of_inside_macro_template_with_unquote via the
// full-stdlib path). A USER macro's quasiquote template contains a keyword/of
// call with an ~unquoted arg; the user-macro expansion produces
// `(:wat::core::keyword/of :foo :bar)`, which the fixpoint must RE-EXPAND via
// the registered keyword/of MACRO → `:foo<bar>`. If keyword/of (now a macro,
// not a built-in) does NOT fire on the expansion result, this is the
// template-position regression the deleted test would have caught.
// ═══════════════════════════════════════════════════════════════════════════
const KW_OF_TEMPLATE_MACRO: &str = "(:wat::core::defmacro :my::mk \
     [e <- :wat::holon::HolonAST] -> :AST<wat::holon::HolonAST> \
     `(:wat::core::keyword/of :foo ~e))";

#[test]
fn keyword_of_fires_in_template_position() {
    let result = eval_string_with(
        KW_OF_TEMPLATE_MACRO,
        "(:wat::core::keyword/to-string (:my::mk :bar))",
    );
    println!("\n=== keyword_of_fires_in_template_position ===\nexpect Ok(\"foo<bar>\"):\n{:#?}\n", result);
    assert_eq!(
        result.unwrap(),
        Value::String(Arc::new("foo<bar>".to_string())),
        "keyword/of MUST fire in template position (inside another macro's quasiquote) \
         as a registered macro — the deleted keyword_of_inside_macro_template_with_unquote risk"
    );
}
