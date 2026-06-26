//! FM-2-bis diagnostic probe (arc 209 C.3) — the ONE novel composition C.3 rests on.
//!
//! C.3 makes `defservice` (a defmacro) emit `:wat::core::defrecord` calls — and defrecord is
//! ITSELF a defmacro. No existing macro emits a macro-CALL in its output (C.2 emits only the
//! `defenum`/`defn` special forms). So the load-bearing, unproven claim is:
//!
//!   **Does the expander re-expand a macro-call (`defrecord`) that appears in a macro's
//!   output, AND can a `defenum` variant's field type be a record minted by that emitted
//!   defrecord, with the record emitted BEFORE the enum (the ordering wat/program.wat:8
//!   requires)?**
//!
//! This probe mints a tiny defmacro `:t::mk` whose output is EXACTLY the C.3 shape in miniature:
//! a `(do (defrecord Req) (defenum Op wraps Req) (defn go uses Op + Req accessor))`. If the
//! world builds and `(:t::go 5)` returns 5, the whole composition is proven and the C.3 strike
//! is "generate this shape." If it fails, the gap is here — found before the BRIEF, not after.
//!
//! GREEN at HEAD expected (this is the proof the composition works; the C.3 GATE probe, which
//! drives the real generated client face, is RED at HEAD by contrast).
//!
//! Run: cargo test --release -p wat --test probe_diagnostic_c3_macro_emits_record_def

use std::sync::Arc;
use wat::freeze::{eval_in_frozen, startup_from_source};
use wat::load::InMemoryLoader;
use wat::runtime::{Environment, Value};

const PROGRAM: &str = r#"
;; A defmacro whose OUTPUT contains a defrecord macro-call (must re-expand) + a defenum
;; wrapping that record as a variant field type (record must precede the enum) + a defn using
;; the Op variant and the Record accessor. This mirrors the C.3 target expansion in miniature.
(:wat::core::defmacro :t::mk [base <- :wat::WatAST] -> :wat::WatAST
  (:wat::core::let
    [base-str (:wat::core::keyword/to-string base)
     req-name (:wat::core::keyword/from-string (:wat::core::string::concat base-str "::Req"))
     op-name  (:wat::core::keyword/from-string (:wat::core::string::concat base-str "::Op"))
     go-name  (:wat::core::keyword/from-string (:wat::core::string::concat base-str "/go"))
     ;; the wrapped-record field type keyword for the Op variant: :<base>::Req
     req-ty   (:wat::core::keyword/from-string (:wat::core::string::concat base-str "::Req"))
     ;; the accessor: :<base>::Req/n
     acc-name (:wat::core::keyword/from-string (:wat::core::string::concat base-str "::Req/n"))
     ;; the Op::Go variant constructor keyword: :<base>::Op::Go
     go-var   (:wat::core::keyword/from-string (:wat::core::string::concat base-str "::Op::Go"))]
    `(:wat::core::do
       (:wat::core::defrecord ~req-name [n <- :wat::core::i64])
       (:wat::core::defenum ~op-name :Go [req <- ~req-ty])
       (:wat::core::defn ~go-name [n <- :wat::core::i64] -> :wat::core::i64
         (:wat::core::match (~go-var (~req-name n)) -> :wat::core::i64
           ((~go-var req) (~acc-name req)))))))

(:t::mk :demo)

(:wat::core::defn :user::main [] -> :wat::core::nil nil)
"#;

#[test]
fn macro_output_reexpands_record_def_and_enum_wraps_it() {
    let world = startup_from_source(PROGRAM, None, Arc::new(InMemoryLoader::new()))
        .expect("startup should succeed: a defmacro emitting a defrecord call must re-expand, \
                 and a defenum variant field may be the emitted record (record emitted first)");
    let ast = wat::parse_one!("(:demo/go 5)").expect("parse");
    let got = eval_in_frozen(&ast, &world, &Environment::new())
        .map(|tv| tv.value_owned())
        .unwrap_or_else(|e| panic!("demo/go raised: {e:?}"));
    assert!(
        matches!(got, Value::i64(5)),
        "expected 5: :t::mk's macro output (defrecord Req + defenum Op wraps Req + defn go) must \
         re-expand and round-trip n=5 through the wrapped record + accessor; got {got:?}"
    );
}
