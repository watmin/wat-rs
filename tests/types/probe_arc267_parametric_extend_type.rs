//! Arc 267 — parametric `extend-type` / parametric protocol bounds.
//!
//! Arc 232 made a protocol `:P` a usable bound, but only for NON-parametric extenders (its probes
//! used plain Records — Robot/Dog). It explicitly scoped out "Parametric protocols … OUT of v1
//! unless a strike proves them load-bearing" (DESIGN.md:99) — "if/when [a caller] does, a NEW arc
//! opens" (INSCRIPTION.md:41). The arc-209 host seam is that caller: the spawn handles
//! (`Thread'<I,O>`/`Process'<I,O>`) are PARAMETRIC and must satisfy a plain handle protocol.
//!
//! THE GAP (this probe isolates it): `assignable` (check.rs:13681) consults `is_subtype` ONLY when
//! BOTH actual and expected are `TypeExpr::Path`. A `Parametric` actual (`Box<i64>`) against a `Path`
//! protocol bound (`:t::Tagged`) skips the subtype check → falls through to `unify` → rejected, even
//! though `Box` `extend-type`s `:t::Tagged`.
//!
//! THE FIX: a `Foo<…>` satisfies `:P` iff the CONSTRUCTOR `Foo` extend-types `:P` — `assignable`
//! must match a `Parametric` actual to a `Path` protocol by its head. (Edge keys carry the leading
//! colon — register_subtype(":wat::holon::Record", …) types.rs:1402; `Parametric.head` does NOT — so
//! the head is reconciled as `format!(":{head}")` before the `is_subtype` lookup.)
//!
//! RED at HEAD: `(:user::tag-of (:t::Box/new 5))` fails to check — TypeMismatch, expected
//! `:t::Tagged`, got `:t::Box<wat::core::i64>`. GREEN once `assignable` consults the parametric head.
//!
//! Run: cargo test --release -p wat --test probe_arc267_parametric_extend_type

use wat::freeze::{eval_in_frozen, startup_beside};
use wat::runtime::{Environment, Value};

#[test]
fn parametric_type_satisfies_a_plain_protocol_bound() {
    let world = startup_beside(file!())
        .expect("startup should succeed (267: a parametric Box<i64> satisfies the :t::Tagged bound)");
    let ast = wat::parse_one!("(:user::go)").expect("parse");
    let got = eval_in_frozen(&ast, &world, &Environment::new())
        .map(|tv| tv.value_owned())
        .unwrap_or_else(|e| panic!("go raised: {e:?}"));
    assert!(
        matches!(&got, Value::String(s) if s.as_str() == "box"),
        "expected \"box\": Box<i64> (a Parametric actual) passed through a :t::Tagged-typed param must \
         be accepted via the extend-type edge on the Box constructor, and dispatch to Box's impl; got {got:?}"
    );
}
