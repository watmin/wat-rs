//! Arc 279 follow-on — `str` must be TOTAL (the disconfirming probe).
//!
//! THE GAP: 279 minted `:wat::core::str` and its DESIGN.md:67 specifies it as rendering **"ANY value
//! unquoted (String→itself, i64→digits, bool→true/false, …)"**. What shipped is a five-arm match —
//! `String | i64 | f64 | bool | u8` — that RAISES `TypeMismatch` on everything else. The `…` was
//! never filled in. This is not a ruling being overturned; it is 279's own unfinished intent.
//!
//! WHY IT MATTERS NOW: `wat.string/join` should render its elements (`(join "," [1 2 3])` → `"1,2,3"`),
//! which is only sound if `str` is total. A partial `str` forces either a type-variable BOUND — a form
//! wat does not have — or a `join` that cannot join numbers.
//!
//! THE TARGET RENDERING IS NOT INVENTED. It is what the EDN encoder already emits for the same values
//! (measured through `println`, 2026-08-14): `:a-keyword` · `nil` · `[1 2 3]` · `["a" "b"]` · `{:a 1 :b 2}`.
//! A total, correct renderer already exists; `show` (`value/observe.rs::render_value`) is a THIRD
//! implementation that duplicates it in Rust `Debug` shape — `()` for nil, `[1, 2, 3]`, `{:a: 1}`.
//!
//! CONTROLS ARE ROWS 1-3 AND THEY ARE LOAD-BEARING (R59 `NISI FRANGAS, NIHIL PROBAS`). They pass at
//! HEAD. Without them, a red below could mean "the harness is broken" rather than "`str` is partial",
//! and the probe would prove nothing.
//!
//! RED at HEAD (rows 4-8). GREEN when the stone lands.

use wat::freeze::call_beside_value;
use wat::runtime::Value;

fn rendered(target: &str) -> String {
    match call_beside_value(file!(), target)
        .unwrap_or_else(|e| panic!("({target}) must return a String; it raised: {e:?}"))
    {
        Value::String(s) => (*s).clone(),
        other => panic!("({target}) must return a String; got {other:?}"),
    }
}

// ─── CONTROLS — green at HEAD, and they must stay green ─────────────────────

#[test]
fn control_str_renders_a_top_level_string_bare() {
    assert_eq!(rendered(":t::control-str-string-is-bare"), "abc");
}

#[test]
fn control_show_renders_a_top_level_string_quoted() {
    assert_eq!(rendered(":t::control-show-string-is-quoted"), "\"abc\"");
}

#[test]
fn control_str_renders_an_i64() {
    assert_eq!(rendered(":t::control-str-i64"), "42");
}

// ─── THE REDS — each raises `TypeMismatch` at HEAD ──────────────────────────

#[test]
fn str_renders_a_keyword() {
    assert_eq!(rendered(":t::probe-keyword"), ":a-keyword");
}

#[test]
fn str_renders_nil_as_nil_not_unit() {
    // `show` currently answers `()` here — Rust's unit type leaking through a wat verb.
    assert_eq!(rendered(":t::probe-nil"), "nil");
}

#[test]
fn str_renders_a_vector_in_wat_form_not_rust_debug() {
    // The distinguishing byte is the comma: `show` answers `[1, 2, 3]`.
    assert_eq!(rendered(":t::probe-vector"), "[1 2 3]");  // rune:lint(no-inlined-edn) — this asserts the SERIALIZATION, not the data; an .edn golden via assert_edn_eq! normalizes `[1 2 3]` and `[1, 2, 3]` to the same value, erasing the exact difference under test.
}

#[test]
fn str_renders_a_map_in_wat_form() {
    // ONE key, so this asserts SHAPE without asserting ORDER — maps are unordered and
    // pinning their order would be string equality standing in for data equality.
    // The distinguishing bytes are the doubled colon: `show` answers `{:a: 1}`.
    assert_eq!(rendered(":t::probe-map"), "{:a 1}");  // rune:lint(no-inlined-edn) — this asserts the SERIALIZATION, not the data; an .edn golden via assert_edn_eq! normalizes `[1 2 3]` and `[1, 2, 3]` to the same value, erasing the exact difference under test.
}

#[test]
fn str_keeps_nested_strings_quoted() {
    // The row that proves `str` is not "show with the quotes stripped": the OUTER string
    // of a bare `(str "abc")` is unquoted, but a string INSIDE a collection stays quoted.
    assert_eq!(rendered(":t::probe-nested-string-stays-quoted"), "[\"a\"]");  // rune:lint(no-inlined-edn) — this asserts the SERIALIZATION, not the data; an .edn golden via assert_edn_eq! normalizes `[1 2 3]` and `[1, 2, 3]` to the same value, erasing the exact difference under test.
}

/// A RECORD renders by NAME, not positionally — the row the first draft of this probe missed.
///
/// It sampled a map, a float, a keyword, nil and a nested string: every shape EXCEPT the one
/// that consults the type registry. So `str` was certified total while `(str <record>)`
/// answered `{:field-0 1 :field-1 2}` — the names discarded — and `println` of the same value
/// answered `{:x 1 :y 2}`. The cause was a door that hardcoded `None` for the registry
/// (`value_to_edn_string`, now DELETED). Compared through an `.edn` golden because the claim is
/// STRUCTURAL and a byte-exact compare would pin the key order of a two-key map.
#[test]
fn str_renders_a_record_by_name_not_positionally() {
    wat::assert_edn_matches_file!(rendered(":t::probe-record-named-fields"), "probe_arc279_str_totality__record_named_fields.edn");
}
