//! Arc 296 Strike 2a + 3b — behavioral toy tests for `#[to_edn(...)]` attribute DSL.
//!
//! This file is declared as `#[cfg(test)] mod derive_tests;` in
//! `src/edn/mod.rs`, so it is only compiled during test builds. All toy enums are
//! defined here rather than in an integration test because `#[derive(ToEdn)]`
//! generates `impl crate::edn::contract::ToEdn for <Enum>`, which only resolves inside
//! the `wat` crate.
//!
//! ## Coverage
//!
//! A. **`key` rename** — field-level `#[to_edn(key = "call-span")]` on a `Span`
//!    field: emits the renamed key when the span is known, elides entirely when
//!    unknown.
//!
//! B. **Field-level `via`** — `#[to_edn(via = toy_transform)]` on a field whose
//!    type does NOT implement `ToEdn` (`Vec<(usize, Vec<String>)>`). The via
//!    helper receives `&FieldType` and returns `OwnedValue`; the field compiles
//!    without requiring a `ToEdn` impl for the complex type.
//!
//! C. **Variant-level `literal`** — `#[to_edn(literal(k = "v", …))]` on both
//!    a unit variant (prepend only) and a struct variant (prepend + real fields).
//!
//! D. **Variant-level computed `via`** — `#[to_edn(via(key = "hints", fn =
//!    toy_hints, args(a, b)))]`: `Some(v)` emits `:hints […]`, `None` elides
//!    the key entirely.
//!
//! E. **Secondary Span field** — a primary `:span` + a secondary Span with
//!    `#[to_edn(key = "outer-span")]` override (default would be
//!    `"outer-define-span"`). Both elide independently when unknown.
//!
//! F. **Single-field tuple variant** (Strike 3b) — variant-level
//!    `#[to_edn(key = "cause")]` on `Wrap(String)` emits
//!    `#wat.kernel/Wrap {:cause "…"}`. Also tests field-level `via` on the
//!    tuple field for a custom transform.

use crate::edn::contract::ToEdn;
use crate::span::Span;
use std::sync::Arc;
use wat_edn::OwnedValue;
use std::borrow::Cow;

// ── Helper functions referenced by `via` directives ───────────────────────────
//
// These are referenced by name inside `#[to_edn(via = ...)]` annotations.
// The derive emits `fn_name(field_ident)` in the generated match arm, which
// resolves to these functions in the enclosing module.

/// Field-level via helper: takes `&Vec<(usize, Vec<String>)>` (a type without
/// `ToEdn`) and returns a custom `OwnedValue`. Demonstrates that `via` lifts the
/// `ToEdn` constraint on the field type.
fn toy_transform(xs: &[(usize, Vec<String>)]) -> OwnedValue {
    OwnedValue::String(Cow::Owned(format!("count={}", xs.len())))
}

/// Variant-level via helper: returns `Some([a, b])` when `a` is non-empty,
/// `None` otherwise. The `None` branch causes the `:hints` key to be elided
/// entirely from the EDN output.
fn toy_hints(a: &str, b: &str) -> Option<Vec<String>> {
    if a.is_empty() {
        None
    } else {
        Some(vec![a.to_string(), b.to_string()])
    }
}

// ── Toy enums ─────────────────────────────────────────────────────────────────

/// A. `key` rename on a Span field.
///
/// The `span` field's default key would be `:span`; the annotation overrides
/// it to `:call-span`. Arc 298.2: always emitted (no sentinel elision).
#[derive(wat_edn::ToEdn)]
enum KeyRenameTest {
    WithCallSpan {
        #[to_edn(key = "call-span")]
        span: crate::span::Span,
        name: String,
    },
}

/// B. Field-level `via` on a type that has no `ToEdn` impl.
///
/// `Vec<(usize, Vec<String>)>` does not implement `ToEdn`. Without `via`, the
/// derive would generate `xs.to_edn()` which would not compile. With
/// `via = toy_transform`, the field is serialized by calling
/// `toy_transform(xs)` instead — no `ToEdn` bound required.
#[derive(wat_edn::ToEdn)]
enum FieldViaTest {
    Transform {
        #[to_edn(via = toy_transform)]
        xs: Vec<(usize, Vec<String>)>,
        n: usize,
    },
}

/// C. Variant-level `literal(...)`.
///
/// `NilType` is a unit variant with two synthetic constant pairs prepended.
/// `Mixed` is a struct variant where the literal pair is prepended before the
/// real `count` field.
#[derive(wat_edn::ToEdn)]
enum LiteralTest {
    #[to_edn(literal(primitive = ":()", fqdn = ":wat::core::nil"))]
    NilType,
    #[to_edn(literal(tag_name = "mixed"))]
    Mixed { count: usize },
}

/// D. Variant-level computed `via`.
///
/// `toy_hints(a, b)` receives the bound field idents directly and returns
/// `Option<Vec<String>>`. On `Some`, `:hints [...]` is appended. On `None`,
/// the key is elided.
#[derive(wat_edn::ToEdn)]
enum ViaVariantTest {
    #[to_edn(via(key = "hints", fn = toy_hints, args(a, b)))]
    Pair { a: String, b: String },
}

/// E. Primary + secondary Span fields — always emitted (arc 298.2).
///
/// `span` uses the default key `:span`. `outer_define_span` has a key override
/// `#[to_edn(key = "outer-span")]` so the EDN key is `:outer-span` instead of
/// the snake→kebab default `:outer-define-span`. Both fields are always emitted
/// since `rust_caller_span!()` is a real Rust location (sentinel retired).
#[derive(wat_edn::ToEdn)]
enum MultiSpanTest {
    Def {
        span: crate::span::Span,
        #[to_edn(key = "outer-span")]
        outer_define_span: crate::span::Span,
        name: String,
    },
}

// ── Test helpers ──────────────────────────────────────────────────────────────

fn known_span(file: &str, line: i64, col: i64) -> Span {
    Span::new(Arc::new(file.to_owned()), line, col)
}

// ── A. `key` rename ───────────────────────────────────────────────────────────

/// Known span is emitted under the renamed key `:call-span`, not `:span`.
#[test]
fn key_rename_span_known_emits_renamed_key() {
    let e = KeyRenameTest::WithCallSpan {
        span: known_span("f.wat", 3, 5),
        name: "foo".to_owned(),
    };
    let edn = wat_edn::write(&e.to_edn());
    // Stone B: span fields now emit #wat.core/Span tagged records, not bare maps.
    assert_eq!(
        edn,
        r#"#wat.kernel/WithCallSpan {:call-span #wat.core/Span {:file "f.wat" :line 3 :col 5 :end #wat.core.Option/None []} :name "foo"}"#,
    );
}

// ── F. STRUCT derive (296 closing strike, Stone 1) — `#[derive(ToEdn)]` on a struct emits
// one tagged record `#wat.<ns>/<Name> {fields}`, namespace via `#[to_edn(namespace = ...)]`,
// with `Option<record>` nesting honestly (#wat.core.Option/{Some,None}). The pattern the
// `#wat.core/Span` record (Stone 2) is built on: a struct → a typed record, data all the way down.
#[derive(wat_edn::ToEdn)]
#[to_edn(namespace = crate::error_ns::CORE)]
struct PosProbe296 {
    line: i64,
    col: i64,
}
#[derive(wat_edn::ToEdn)]
#[to_edn(namespace = crate::error_ns::CORE)]
struct SpanProbe296 {
    file: String,
    line: i64,
    col: i64,
    end: Option<PosProbe296>,
}

#[test]
fn struct_derive_emits_namespaced_tagged_record_with_optional_nested() {
    // Some(end): a real range — nested record inside an Option inside the record.
    let some = SpanProbe296 {
        file: "f.wat".to_owned(),
        line: 3,
        col: 8,
        end: Some(PosProbe296 { line: 3, col: 12 }),
    };
    assert_eq!(
        wat_edn::write(&some.to_edn()),
        r#"#wat.core/SpanProbe296 {:file "f.wat" :line 3 :col 8 :end #wat.core.Option/Some [#wat.core/PosProbe296 {:line 3 :col 12}]}"#,
    );
    // None(end): a point — absence spoken as #wat.core.Option/None, no end==start sentinel.
    let none = SpanProbe296 {
        file: "g.wat".to_owned(),
        line: 1,
        col: 0,
        end: None,
    };
    assert_eq!(
        wat_edn::write(&none.to_edn()),
        r#"#wat.core/SpanProbe296 {:file "g.wat" :line 1 :col 0 :end #wat.core.Option/None []}"#,
    );
}

/// Arc 298.2: `rust_caller_span!()` IS emitted under the renamed `:call-span` key.
/// The sentinel-elide-when-unknown discipline is retired; every span is real.
#[test]
fn key_rename_span_rust_caller_emits_renamed_key() {
    let e = KeyRenameTest::WithCallSpan {
        span: crate::rust_caller_span!(),
        name: "foo".to_owned(),
    };
    let edn = wat_edn::write(&e.to_edn());
    // Renamed key must appear.
    // rune:lint(loose-assert) — EDN embeds rust_caller_span!() (variable Rust file/line/col); full assert_eq! infeasible; key presence is the contract
    assert!(edn.contains(":call-span"), ":call-span must be emitted; got: {}", edn);
    // File must be a real Rust source path.
    // rune:lint(loose-assert) — variable Rust source path embedded in rust_caller_span!() span; path prefix presence is the contract
    // A real Rust path is one ENDING in `.rs` — wat sources are `.wat`. The old check keyed on a
    // `wat-rs/` prefix that `rust_caller_span!()` used to glue on; the suffix is the honest test.
    assert!(edn.contains(".rs\""), ":call-span file must be real Rust path; got: {}", edn);
    // The non-span field must still appear.
    // rune:lint(loose-assert) — EDN embeds rust_caller_span!() making full assert_eq! infeasible; non-span field presence is the contract
    assert!(edn.contains(r#":name "foo""#), ":name must be present; got: {}", edn);
}

// ── B. Field-level `via` ──────────────────────────────────────────────────────

/// `toy_transform` receives the field ref and returns a custom OwnedValue.
/// The non-ToEdn type compiles because `.to_edn()` is never called on `xs`.
#[test]
fn field_via_calls_helper_with_field_ref() {
    let e = FieldViaTest::Transform {
        xs: vec![(1usize, vec!["a".to_owned()])],
        n: 42,
    };
    let edn = wat_edn::write(&e.to_edn());
    assert_eq!(edn, r#"#wat.kernel/Transform {:xs "count=1" :n 42}"#);
}

/// Empty vec: helper still called, returns a valid OwnedValue.
#[test]
fn field_via_empty_vec() {
    let e = FieldViaTest::Transform {
        xs: vec![],
        n: 0,
    };
    let edn = wat_edn::write(&e.to_edn());
    assert_eq!(edn, r#"#wat.kernel/Transform {:xs "count=0" :n 0}"#);
}

// ── C. Variant-level `literal` ────────────────────────────────────────────────

/// Unit variant: only synthetic pairs, no real fields.
#[test]
fn literal_unit_variant_prepends_synthetic_pairs() {
    let e = LiteralTest::NilType;
    let edn = wat_edn::write(&e.to_edn());
    assert_eq!(
        edn,
        r#"#wat.kernel/NilType {:primitive ":()" :fqdn ":wat::core::nil"}"#,
    );
}

/// Struct variant: synthetic pair is PREPENDED before the real field.
/// `tag_name` (ident) → `"tag-name"` (snake→kebab EDN key).
#[test]
fn literal_struct_variant_prepended_before_fields() {
    let e = LiteralTest::Mixed { count: 7 };
    let edn = wat_edn::write(&e.to_edn());
    assert_eq!(edn, r#"#wat.kernel/Mixed {:tag-name "mixed" :count 7}"#);
}

// ── D. Variant-level computed `via` ──────────────────────────────────────────

/// `Some` path: `:hints` key is APPENDED after field pairs.
#[test]
fn variant_via_some_appends_key() {
    let e = ViaVariantTest::Pair {
        a: "x".to_owned(),
        b: "y".to_owned(),
    };
    let edn = wat_edn::write(&e.to_edn());
    assert_eq!(edn, r#"#wat.kernel/Pair {:a "x" :b "y" :hints ["x" "y"]}"#);
}

/// `None` path: `:hints` key is entirely elided from the output.
#[test]
fn variant_via_none_elides_key() {
    let e = ViaVariantTest::Pair {
        a: "".to_owned(),
        b: "y".to_owned(),
    };
    let edn = wat_edn::write(&e.to_edn());
    assert_eq!(edn, r#"#wat.kernel/Pair {:a "" :b "y"}"#);
}

// ── E. Secondary Span field ───────────────────────────────────────────────────

/// Both spans known: both keys appear; `:outer-span` is the override key
/// (not the default `:outer-define-span`).
#[test]
fn secondary_span_both_known_key_override_applied() {
    let e = MultiSpanTest::Def {
        span: known_span("a.wat", 1, 1),
        outer_define_span: known_span("b.wat", 2, 3),
        name: "def".to_owned(),
    };
    let edn = wat_edn::write(&e.to_edn());
    // Stone B: span fields now emit #wat.core/Span tagged records, not bare maps.
    assert_eq!(
        edn,
        r#"#wat.kernel/Def {:span #wat.core/Span {:file "a.wat" :line 1 :col 1 :end #wat.core.Option/None []} :outer-span #wat.core/Span {:file "b.wat" :line 2 :col 3 :end #wat.core.Option/None []} :name "def"}"#,
    );
}

/// Arc 298.2: both spans always emitted. Primary is `rust_caller_span!()` (real
/// Rust location); secondary is a known wat span. Both keys appear.
#[test]
fn secondary_span_primary_rust_caller_secondary_known() {
    let e = MultiSpanTest::Def {
        span: crate::rust_caller_span!(),
        outer_define_span: known_span("b.wat", 2, 3),
        name: "def".to_owned(),
    };
    let edn = wat_edn::write(&e.to_edn());
    // Primary span must appear with a real Rust path.
    // rune:lint(loose-assert) — EDN embeds rust_caller_span!() (variable Rust file/line/col); key presence is the contract
    assert!(edn.contains(":span"), "primary :span must be emitted; got: {}", edn);
    // rune:lint(loose-assert) — variable Rust source path from rust_caller_span!(); path prefix is the contract
    assert!(edn.contains(".rs\""), "primary :span must be real Rust path; got: {}", edn);
    // Secondary span must appear with the known wat location.
    // rune:lint(loose-assert) — EDN embeds variable primary rust_caller_span!(); secondary known-span substring presence is the contract
    // Stone B: span fields emit #wat.core/Span tagged records; substring match on tag + file.
    assert!(edn.contains(r#":outer-span #wat.core/Span {:file "b.wat" :line 2 :col 3"#), ":outer-span must emit known span; got: {}", edn);
    // Non-span field must appear.
    // rune:lint(loose-assert) — EDN embeds variable rust_caller_span!() making full assert_eq! infeasible; non-span field presence is the contract
    assert!(edn.contains(r#":name "def""#), ":name must be present; got: {}", edn);
}

/// Arc 298.2: both `rust_caller_span!()` values are real Rust locations —
/// both `:span` and `:outer-span` ARE emitted (sentinel-elide retired).
#[test]
fn secondary_span_both_rust_caller_both_emit() {
    let e = MultiSpanTest::Def {
        span: crate::rust_caller_span!(),
        outer_define_span: crate::rust_caller_span!(),
        name: "def".to_owned(),
    };
    let edn = wat_edn::write(&e.to_edn());
    // Both span keys must appear with real Rust file paths.
    // rune:lint(loose-assert) — EDN embeds two rust_caller_span!() values (variable file/line/col); key presence is the contract
    assert!(edn.contains(":span"), "primary :span must be emitted; got: {}", edn);
    // rune:lint(loose-assert) — EDN embeds two rust_caller_span!() values (variable file/line/col); key presence is the contract
    assert!(edn.contains(":outer-span"), ":outer-span must be emitted; got: {}", edn);
    assert!(edn.matches(".rs\"").count() >= 2, "both spans must have real Rust paths; got: {}", edn);
    // Non-span field must appear.
    // rune:lint(loose-assert) — EDN embeds variable rust_caller_span!() values making full assert_eq! infeasible; non-span field presence is the contract
    assert!(edn.contains(r#":name "def""#), ":name must be present; got: {}", edn);
}

// ── F. Single-field tuple variant (Strike 3b) ─────────────────────────────────

/// F1. Plain tuple: `Wrap(String)` with `#[to_edn(key = "cause")]` on the
/// variant derives to `#wat.kernel/Wrap {:cause "…"}`.
///
/// This is the minimal proof of the new tuple-variant capability: the variant
/// tag is emitted as usual; the single field gets the EDN key declared by
/// the variant-level annotation.
#[derive(wat_edn::ToEdn)]
enum TupleVariantTest {
    /// Keyed single-field tuple: variant-level key names the field's EDN key.
    #[to_edn(key = "cause")]
    Wrap(String),
    /// Mix in a struct variant to prove the two shapes coexist in one enum.
    Named { count: usize },
}

/// Via helper for F2: returns the string length as an integer OwnedValue.
fn toy_tuple_via(s: &str) -> OwnedValue {
    OwnedValue::Integer(s.len() as i64)
}

/// F2. Tuple variant with field-level `via` — field-level `#[to_edn(via = ...)]`
/// overrides how the single field's value is computed.
#[derive(wat_edn::ToEdn)]
enum TupleVariantViaTest {
    #[to_edn(key = "len")]
    WithVia(
        #[to_edn(via = toy_tuple_via)]
        String
    ),
}

/// F1a. Keyed tuple: inner string is serialized via `.to_edn()` (plain String).
#[test]
fn tuple_variant_keyed_emits_correct_map() {
    let e = TupleVariantTest::Wrap("hello".to_owned());
    let edn = wat_edn::write(&e.to_edn());
    assert_eq!(edn, r#"#wat.kernel/Wrap {:cause "hello"}"#);
}

/// F1b. Struct sibling variant still emits correctly in the same derive.
#[test]
fn tuple_variant_struct_sibling_unaffected() {
    let e = TupleVariantTest::Named { count: 3 };
    let edn = wat_edn::write(&e.to_edn());
    assert_eq!(edn, r#"#wat.kernel/Named {:count 3}"#);
}

/// F2. Field-level `via` on the tuple field: `toy_tuple_via` is called instead
/// of `.to_edn()`, returning the string length as an integer.
#[test]
fn tuple_variant_field_via_called() {
    let e = TupleVariantViaTest::WithVia("hello".to_owned());
    let edn = wat_edn::write(&e.to_edn());
    assert_eq!(edn, r#"#wat.kernel/WithVia {:len 5}"#);
}
