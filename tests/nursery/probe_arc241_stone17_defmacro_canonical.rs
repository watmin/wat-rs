//! FM 2-bis probe for Stone 241.17 — `:wat::core::defmacro` SIGNATURE MIGRATION TO CANONICAL.
//!
//! Stone 241.17 absorbs arc 177's scope: defmacro signature shape migrates from
//! arc 010/150 paren-pair-with-type form to canonical Vector-of-triples form
//! mirroring arc 166 defn shape. The def-family parser unification reaches
//! genuine COMPLETION — Stone 241.1's `parse_argspec_triples` becomes the sole
//! argspec parser across fn/defn/defclause/defmacro (4 entity kinds).
//!
//! User direction 2026-05-29 very late: *"target acquired - annihilation
//! enqueued - this arc lives as long as it must. 177 is closed by our work
//! here - the next 241 stone is the closure for 177."*
//!
//! Shape comparison:
//!
//! OLD (paren-pair-with-type; arc 010/150 lineage):
//! ```scheme
//! (:wat::core::defmacro
//!   (:my::macro (x :Type1) (y :Type2) -> :ReturnType)
//!   body)
//! ```
//! 3 items: head + signature-list + body.
//!
//! NEW (canonical Vector-triple mirroring defn):
//! ```scheme
//! (:wat::core::defmacro :my::macro
//!   [x <- :Type1, y <- :Type2]
//!   -> :ReturnType
//!   body)
//! ```
//! 6 items: head + name + argspec Vector + `->` + return-type + body.
//!
//! HEAD-disconfirmation map:
//! - C01: defmacro with new canonical Vector-triple shape WORKS
//!        ⇒ FAILS at HEAD (parser expects 3-item old form; 6-item new form rejected)
//! - C02: old paren-pair shape REJECTED post-stone
//!        ⇒ FAILS at HEAD (form is accepted via parse_defmacro_signature)
//! - C03: defmacro with `& rest` rest-binder in canonical shape WORKS
//!        ⇒ FAILS at HEAD (same reason as C01)
//!
//! Post-stone: all 3 contracts PASS.
//!
//! Run: `cargo test --release --test probe_arc241_stone17_defmacro_canonical`

use std::sync::Arc;
use wat::freeze::startup_from_source;
use wat::load::InMemoryLoader;

fn try_startup(src: &str) -> Result<(), String> {
    let full = format!(
        "{}\n(:wat::core::defn :user::main [] -> :wat::core::nil nil)",
        src
    );
    startup_from_source(&full, None, Arc::new(InMemoryLoader::new()))
        .map(|_| ())
        .map_err(|e| format!("{:?}", e))
}

fn try_startup_display(src: &str) -> String {
    let full = format!(
        "{}\n(:wat::core::defn :user::main [] -> :wat::core::nil nil)",
        src
    );
    match startup_from_source(&full, None, Arc::new(InMemoryLoader::new())) {
        Ok(_) => String::from("<startup succeeded — no error to display>"),
        Err(e) => format!("{}", e),
    }
}

// ─── C01: defmacro with new canonical Vector-triple shape WORKS ────────────────

#[test]
fn contract_01_defmacro_canonical_shape_works() {
    // New shape: (:wat::core::defmacro :name [arg <- :Type] -> :Ret body)
    // Mirror defn shape per arc 166 / Stone 241.6.
    // At HEAD: parser expects 3-item old form; new 6-item form unrecognized.
    // Post-stone: parse_defmacro_form routes argspec through parse_argspec_triples.
    let src = r#"
        (:wat::core::defmacro :test::wrap
          [x <- :wat::WatAST]
          -> :wat::WatAST
          `(:wat::core::Some ~x))
    "#;
    let result = try_startup(src);
    assert!(
        result.is_ok(),
        "defmacro with new canonical Vector-triple shape must work post-stone; got: {:?}",
        result
    );
}

// ─── C02: old paren-pair shape REJECTED post-stone ─────────────────────────────

#[test]
fn contract_02_old_paren_pair_shape_rejected() {
    // Old shape: (:wat::core::defmacro (:name (arg :Type) -> :Ret) body)
    // Per `feedback_hard_cut_admits_no_bypasses` — no shim; old shape dies.
    // At HEAD: parse_defmacro_form + parse_defmacro_signature accept it.
    // Post-stone: HARD-CUT-rejected with structured reason pointing at
    // canonical Vector-triple form.
    let src = r#"
        (:wat::core::defmacro
          (:test::wrap (x :AST<wat::core::nil>) -> :AST<wat::core::nil>)
          `(:wat::core::Some ~x))
    "#;
    let result = try_startup(src);
    assert!(
        result.is_err(),
        "old paren-pair defmacro shape must be HARD-CUT-rejected post-stone (canonical Vector-triple is the only way); got Ok"
    );
}

// ─── C03: defmacro with `& rest` rest-binder works in canonical shape ──────────

#[test]
fn contract_03_defmacro_canonical_rest_binder_works() {
    // Rest-binder must work in the canonical shape — same `& rest <- :Type`
    // pattern as defn / fn. The defn defmacro definition at wat/core.wat:180
    // uses rest-binder; this contract is load-bearing for that migration.
    let src = r#"
        (:wat::core::defmacro :test::variadic-wrap
          [& items <- :wat::core::Vector<wat::WatAST>]
          -> :wat::WatAST
          `(:wat::core::Vector ~@items))
    "#;
    let result = try_startup(src);
    assert!(
        result.is_ok(),
        "defmacro with canonical rest-binder shape must work post-stone; got: {:?}",
        result
    );
}
