//! Integration coverage for `:wat::core::string::*` + `:wat::core::regex::*`.
//!
//! Each test calls a named fn in the co-located fixture via eval_in_frozen.
//! Bool fns return Value::bool; String fns return Value::String.
//! Error fns assert eval_in_frozen returns Err.

use wat::freeze::call_beside_value;
use wat::runtime::Value;

fn run_fn(fn_name: &str) -> Value {
    call_beside_value(file!(), fn_name).expect("eval should succeed")
}

fn run_expecting_eval_err(fn_name: &str) -> String {
    let err = call_beside_value(file!(), fn_name).expect_err("expected runtime error");
    format!("{:?}", err)
}

fn assert_str(val: Value, expected: &str) {
    match val {
        Value::String(s) => assert_eq!(
            &*s, expected,
            "expected String({expected:?}); got String({s:?})"
        ),
        other => panic!("expected String({expected:?}); got {:?}", other),
    }
}

// ─── :wat::core::string::contains? / starts-with? / ends-with? ──────────

#[test]
fn contains_hit() {
    assert!(matches!(run_fn(":my::compute-contains-hit"), Value::bool(true)));
}

#[test]
fn contains_miss() {
    assert!(matches!(run_fn(":my::compute-contains-miss"), Value::bool(false)));
}

#[test]
fn starts_with_hit_and_miss() {
    assert!(matches!(run_fn(":my::compute-starts-with-hit"), Value::bool(true)));
    assert!(matches!(run_fn(":my::compute-starts-with-miss"), Value::bool(false)));
}

#[test]
fn ends_with_hit_and_miss() {
    assert!(matches!(run_fn(":my::compute-ends-with-hit"), Value::bool(true)));
    assert!(matches!(run_fn(":my::compute-ends-with-miss"), Value::bool(false)));
}

// ─── :wat::core::string::length ─────────────────────────────────────────

#[test]
fn length_counts_chars_not_bytes() {
    // The fixture fn returns bool(true) when length == 5 (chars, not bytes).
    assert!(matches!(run_fn(":my::compute-length-chars"), Value::bool(true)));
}

// ─── :wat::core::string::trim ───────────────────────────────────────────

#[test]
fn trim_strips_whitespace() {
    assert_str(run_fn(":my::compute-trim"), "hello");
}

// ─── :wat::core::string::split / join ───────────────────────────────────

#[test]
fn split_produces_vec() {
    assert_str(run_fn(":my::compute-split-join"), "a|b|c");
}

#[test]
fn join_renders_non_string_elements() {
    assert_str(run_fn(":my::compute-join-non-string"), "1,2,3");
}

#[test]
fn join_renders_string_elements_bare() {
    assert_str(run_fn(":my::compute-join-string-bare"), "a-b");
}

#[test]
fn split_empty_separator_rejected() {
    let msg = run_expecting_eval_err(":my::compute-split-empty-sep");
    assert!(
        msg.contains("separator must not be empty"), // rune:lint(loose-assert) — error embeds machine-specific absolute path from startup_beside/file!()
        "expected empty-separator error; got {}",
        msg
    );
}

// ─── :wat::core::regex::matches? ────────────────────────────────────────

#[test]
fn regex_matches_unanchored() {
    assert!(matches!(run_fn(":my::compute-regex-match"), Value::bool(true)));
}

#[test]
fn regex_matches_no_match() {
    assert!(matches!(run_fn(":my::compute-regex-no-match"), Value::bool(false)));
}

#[test]
fn regex_invalid_pattern_errors() {
    let msg = run_expecting_eval_err(":my::compute-regex-invalid");
    assert!(
        msg.contains("invalid regex"), // rune:lint(loose-assert) — error embeds machine-specific absolute path from startup_beside/file!()
        "expected invalid regex error; got {}",
        msg
    );
}
