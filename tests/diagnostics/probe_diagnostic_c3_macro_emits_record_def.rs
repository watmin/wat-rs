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

use wat::freeze::{eval_in_frozen, startup_beside};
use wat::runtime::{Environment, Value};

#[test]
fn macro_output_reexpands_record_def_and_enum_wraps_it() {
    let world = startup_beside(file!())
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
