//! Forward-proof probe — arc 251 Stone 251.5 / Slice 4.1: the declaration migrator.
//!
//! Verifies `fix-form` (`:migrate::fix-form`) closes the two gaps left by
//! `fix.wat`'s `fix-source` in DECLARATION and DEFN forms.
//!
//! Loading: `wat-migrate/fix-decl.wat` is NOT blessed (not in STDLIB_FILES) — it is a
//! throwaway that retires with the hard-cut. The test reads it from disk and prepends
//! its content to the program string, so `startup_from_source` sees it. The blessed
//! `wat/fix.wat` (`:wat::fix::fix-source`) IS auto-loaded via STDLIB_FILES.
//!
//! Cases:
//!   C01 (gap A)  — typealias bare type-slot → `keyword/to-type-form`, not symbol.
//!   C02 (gap B)  — defn with `<T>` name → PLAIN symbol (`<T>` dropped).
//!   C03          — generic decl name + parametric target.
//!   C04          — preservation: user-type targets already correct under fix-source.
//!   C05          — newtype: bare type-slot handled; name-fix on plain name.
//!
//! Run: `cargo test --release --test probe_arc251_decl_migrator`

use std::sync::Arc;
use wat::freeze::{eval_in_frozen, startup_from_source};
use wat::load::InMemoryLoader;
use wat::runtime::{Environment, Value};

/// Path to the non-blessed migrator, relative to the cargo workspace root.
const FIX_DECL_PATH: &str = "wat-migrate/fix-decl.wat";

/// Helper: load the migrator source from disk.
fn migrator_source() -> String {
    std::fs::read_to_string(FIX_DECL_PATH)
        .unwrap_or_else(|e| panic!("cannot read {FIX_DECL_PATH}: {e}"))
}

/// topform helper — reads the first top-level form from a source string.
const TOPFORM: &str = r#"
(:wat::core::defn :user::topform [src <- :wat::core::String] -> :wat::WatAST
  (:wat::core::first (:wat::core::ast->children (:wat::core::read-string src))))
"#;

/// Build a complete program source: migrator + topform helper + a `compute` defn + a
/// `main` stub.
fn make_src(compute_body: &str) -> String {
    let migrator = migrator_source();
    format!(
        "{migrator}\n\
         {TOPFORM}\n\
         (:wat::core::defn :user::compute [] -> :wat::core::String {compute_body})\n\
         (:wat::core::defn :user::main [] -> :wat::core::nil nil)",
    )
}

/// Evaluate `compute_body` (which must return a `:String`) via `startup_from_source`.
fn eval_string(compute_body: &str) -> Result<String, String> {
    let src = make_src(compute_body);
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

/// Escape a wat snippet for embedding inside a wat string literal (fed to `read-string`).
fn embed(payload: &str) -> String {
    payload.replace('\\', "\\\\").replace('"', "\\\"")
}

/// Run `(write-forms (fix-form (topform "<dirty>")))` and normalize whitespace.
fn fix_and_render(dirty: &str) -> Result<String, String> {
    let body = format!(
        "(:wat::core::write-forms (:migrate::fix-form (:user::topform \"{}\")))",
        embed(dirty)
    );
    eval_string(&body).map(normalize)
}

/// Normalize internal whitespace to single spaces (write-forms may use consistent
/// spacing, but we guard against any trailing/leading variation).
fn normalize(s: String) -> String {
    s.split_whitespace().collect::<Vec<_>>().join(" ")
}

// ── C01: Gap A — typealias bare type-slot must use keyword/to-type-form ──────────────
//
// `:wat::core::i64` is a core scalar in a bare type-slot (child[2] of typealias).
// fix-source alone gives `wat.core/i64` (hit head-keyword? → symbol).
// fix-form must give `wat.type/i64` (type-slot → keyword/to-type-form).
#[test]
fn c01_typealias_bare_type_slot_uses_type_form() {
    let dirty = "(:wat::core::typealias :svc::Alias :wat::core::i64)";
    let got = fix_and_render(dirty).expect("C01 fix-form");
    assert_eq!(
        got,
        "(wat.core/typealias svc/Alias wat.type/i64)",
        "C01 (gap A): typealias core-scalar type-slot must render as wat.type/i64"
    );
}

// ── C02: Gap B — defn with <T> name must produce a PLAIN symbol ───────────────────────
//
// `:wat::stream::map<T>` has `<T>`, so fix-source misfires through
// `type-shaped-keyword?` → a parametric FORM `(wat.stream/map T)`.
// fix-form strips `<T>` via name-fix → plain symbol `wat.stream/map`.
#[test]
fn c02_defn_generic_name_drops_type_params() {
    let dirty = "(:wat::core::defn :wat::stream::map<T> [x <- :T] -> :T x)";
    let got = fix_and_render(dirty).expect("C02 fix-form");
    assert_eq!(
        got,
        "(wat.core/defn wat.stream/map [x :- T] :- T x)",
        "C02 (gap B, LOAD-BEARING): defn name with <T> must be a plain symbol, <T> dropped"
    );
}

// ── C03: generic decl name + parametric target ────────────────────────────────────────
//
// `:Foo<T>` name → name-fix drops `<T>` → plain symbol `Foo`.
// `:wat::core::Vector<wat::core::i64>` in type-slot[2] → `keyword/to-type-form`
//   → `(wat.type/Vector wat.type/i64)`.
#[test]
fn c03_generic_decl_name_and_parametric_target() {
    let dirty = "(:wat::core::typealias :Foo<T> :wat::core::Vector<wat::core::i64>)";
    let got = fix_and_render(dirty).expect("C03 fix-form");
    assert_eq!(
        got,
        "(wat.core/typealias Foo (wat.type/Vector wat.type/i64))",
        "C03: generic typealias name stripped + parametric target converted"
    );
}

// ── C04: preservation — user-type targets already correct ────────────────────────────
//
// `:wat::holon::HolonAST` is a user type; `keyword/to-type-form` and `keyword/to-symbol`
// both render it with the namespace preserved. C04 confirms no regression.
#[test]
fn c04_user_type_target_preserved() {
    let dirty = "(:wat::core::typealias :wat::edn::Tagged :wat::holon::HolonAST)";
    let got = fix_and_render(dirty).expect("C04 fix-form");
    assert_eq!(
        got,
        "(wat.core/typealias wat.edn/Tagged wat.holon/HolonAST)",
        "C04 (preservation): user-type target must preserve namespace"
    );
}

// ── C05: newtype — bare type-slot handled; name plain ────────────────────────────────
//
// `newtype` is in type-slot-2? set → child[2] goes through `keyword/to-type-form`.
// `:wat::edn::NoTag` has no `<T>` → name-fix strips nothing → `wat.edn/NoTag`.
#[test]
fn c05_newtype_type_slot_handled() {
    let dirty = "(:wat::core::newtype :wat::edn::NoTag :wat::holon::HolonAST)";
    let got = fix_and_render(dirty).expect("C05 fix-form");
    assert_eq!(
        got,
        "(wat.core/newtype wat.edn/NoTag wat.holon/HolonAST)",
        "C05 (newtype): bare type-slot in newtype must use keyword/to-type-form"
    );
}

// ── C06: typeunion — member VECTOR of non-arrow'd core-scalar types ───────────────────
//
// `(typeunion :my::Foo [:wat::core::i64 :wat::core::f64])` — child[2] is a VECTOR of bare
// member types. fix-seq alone routes each through head-keyword? → wat.core/i64 (WRONG).
// fix-form maps keyword/to-type-form over the vector → [wat.type/i64 wat.type/f64].
#[test]
fn c06_typeunion_core_member_vector_uses_type_form() {
    let dirty = "(:wat::core::typeunion :my::Foo [:wat::core::i64 :wat::core::f64])";
    let got = fix_and_render(dirty).expect("C06 fix-form");
    assert_eq!(
        got,
        "(wat.core/typeunion my/Foo [wat.type/i64 wat.type/f64])",
        "C06: typeunion core-scalar members must render as wat.type/ in the member vector"
    );
}

// ── C07: typeunion — user-type members preserve namespace ─────────────────────────────
#[test]
fn c07_typeunion_user_member_vector_preserved() {
    let dirty = "(:wat::core::typeunion :my::Shape [:my::Circle :my::Square])";
    let got = fix_and_render(dirty).expect("C07 fix-form");
    assert_eq!(
        got,
        "(wat.core/typeunion my/Shape [my/Circle my/Square])",
        "C07: typeunion user-type members must preserve their namespace"
    );
}

// ── C08: defenum — name fixed, VARIANT TAGS stay keywords (data), field types converted ─
//
// Variant tags (`:Provision`) are data constructors, NOT types — they STAY keywords.
// Field types live in arrow'd field vectors → fix-seq converts them via the arrow rule.
#[test]
fn c08_defenum_variant_tags_stay_fields_converted() {
    let dirty = "(:wat::core::defenum :counter::AdminReq :Provision [initial <- :wat::core::i64])";
    let got = fix_and_render(dirty).expect("C08 fix-form");
    assert_eq!(
        got,
        "(wat.core/defenum counter/AdminReq :Provision [initial :- wat.type/i64])",
        "C08 (defenum): variant tags stay keywords; field types convert via the arrow rule"
    );
}
