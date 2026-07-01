//! Arc 296 Strike 2a — behavioral toy tests for `#[to_edn(...)]` attribute DSL.
//!
//! This file is declared as `#[cfg(test)] mod to_edn_derive_tests;` in
//! `src/lib.rs`, so it is only compiled during test builds. All toy enums are
//! defined here rather than in an integration test because `#[derive(ToEdn)]`
//! generates `impl crate::to_edn::ToEdn for <Enum>`, which only resolves inside
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

use crate::to_edn::ToEdn;
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
fn toy_transform(xs: &Vec<(usize, Vec<String>)>) -> OwnedValue {
    OwnedValue::String(Cow::Owned(format!("count={}", xs.len())))
}

/// Variant-level via helper: returns `Some([a, b])` when `a` is non-empty,
/// `None` otherwise. The `None` branch causes the `:hints` key to be elided
/// entirely from the EDN output.
fn toy_hints(a: &String, b: &String) -> Option<Vec<String>> {
    if a.is_empty() {
        None
    } else {
        Some(vec![a.clone(), b.clone()])
    }
}

// ── Toy enums ─────────────────────────────────────────────────────────────────

/// A. `key` rename on a Span field.
///
/// The `span` field's default key would be `:span`; the annotation overrides
/// it to `:call-span`. Span elide-when-unknown discipline still applies.
#[derive(wat_macros::ToEdn)]
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
#[derive(wat_macros::ToEdn)]
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
#[derive(wat_macros::ToEdn)]
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
#[derive(wat_macros::ToEdn)]
enum ViaVariantTest {
    #[to_edn(via(key = "hints", fn = toy_hints, args(a, b)))]
    Pair { a: String, b: String },
}

/// E. Primary + secondary Span fields with independent elide.
///
/// `span` uses the default key `:span`. `outer_define_span` has a key override
/// `#[to_edn(key = "outer-span")]` so the EDN key is `:outer-span` instead of
/// the snake→kebab default `:outer-define-span`. Both fields obey the
/// elide-when-unknown discipline independently.
#[derive(wat_macros::ToEdn)]
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
    assert_eq!(
        edn,
        r#"#wat.kernel/WithCallSpan {:call-span {:file "f.wat" :line 3 :col 5} :name "foo"}"#,
    );
}

/// Unknown span is elided entirely (`:call-span` key does not appear).
#[test]
fn key_rename_span_unknown_elides() {
    let e = KeyRenameTest::WithCallSpan {
        span: Span::unknown(),
        name: "foo".to_owned(),
    };
    let edn = wat_edn::write(&e.to_edn());
    assert_eq!(edn, r#"#wat.kernel/WithCallSpan {:name "foo"}"#);
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
    assert_eq!(
        edn,
        r#"#wat.kernel/Def {:span {:file "a.wat" :line 1 :col 1} :outer-span {:file "b.wat" :line 2 :col 3} :name "def"}"#,
    );
}

/// Primary span unknown: `:span` elided; secondary `:outer-span` still emits.
#[test]
fn secondary_span_primary_unknown_secondary_known() {
    let e = MultiSpanTest::Def {
        span: Span::unknown(),
        outer_define_span: known_span("b.wat", 2, 3),
        name: "def".to_owned(),
    };
    let edn = wat_edn::write(&e.to_edn());
    assert_eq!(
        edn,
        r#"#wat.kernel/Def {:outer-span {:file "b.wat" :line 2 :col 3} :name "def"}"#,
    );
}

/// Both spans unknown: both keys elided; only `:name` remains.
#[test]
fn secondary_span_both_unknown_both_elide() {
    let e = MultiSpanTest::Def {
        span: Span::unknown(),
        outer_define_span: Span::unknown(),
        name: "def".to_owned(),
    };
    let edn = wat_edn::write(&e.to_edn());
    assert_eq!(edn, r#"#wat.kernel/Def {:name "def"}"#);
}
