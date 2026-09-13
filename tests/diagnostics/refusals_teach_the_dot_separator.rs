//! Seven refusal texts taught the retired `::` variant separator. Each test
//! is pinned to ONE site in BRIEF-2b D1: the refusal names `.`, and the
//! spelling that site's predicate accepts starts up.
//!
//! Sites 187 / 6955 / 7253 cannot be fired from wat: 187's fallback is only
//! reached with a path not in the five retired bares (every wat caller passes
//! one of the five); 6955 and 7253 sit behind `is_namespaced_variant`, the
//! same `decompose_variant` they then re-ask. Those three pin the format
//! string itself (so E2 can revert that site alone) and still run the
//! remedied input.

use wat::freeze::startup_from_file;

fn refusal(path: &str) -> String {
    match startup_from_file(path) {
        Ok(_) => panic!("{path} started up — the refusal did not fire"),
        Err(e) => format!("{e}"),
    }
}

fn accepted(path: &str, site: &str) {
    if let Err(e) = startup_from_file(path) {
        panic!("{site}: remedied input {path} refused: {e}");
    }
}

fn source_contains(rel: &str, needle: &str, site: &str) {
    let text = std::fs::read_to_string(rel).unwrap_or_else(|e| panic!("read {rel}: {e}"));
    assert!(
        text.contains(needle),
        "{site}: {rel} no longer contains {needle:?}"
    );
}

#[test]
fn match_arm_not_namespaced_teaches_dot() {
    let msg = refusal(
        "tests/diagnostics/refusals_teach_the_dot_separator_s1_not_namespaced.wat.bad",
    );
    let named_dot = msg.contains("is not namespaced; write `<enum>.<Variant>`");
    assert!(
        named_dot,
        "match_arm.rs:108 must name the dot form; got {msg}"
    );
    accepted(
        "tests/diagnostics/refusals_teach_the_dot_separator_s1_not_namespaced.wat",
        "match_arm.rs:108",
    );
}

#[test]
fn bare_variant_retired_reason_fallback_teaches_dot() {
    source_contains(
        "src/match_arm.rs",
        "write a qualified Type.Variant FQDN",
        "match_arm.rs:187",
    );
    accepted(
        "tests/diagnostics/refusals_teach_the_dot_separator_s2_option_some.wat",
        "match_arm.rs:187",
    );
}

#[test]
fn variant_parent_of_example_teaches_dot() {
    let msg = refusal(
        "tests/diagnostics/refusals_teach_the_dot_separator_s3_variant_parent_of.wat.bad",
    );
    let named_dot = msg.contains(":Ns::Enum.Variant");
    assert!(
        named_dot,
        "check.rs:2877 must name the dot form; got {msg}"
    );
    accepted(
        "tests/diagnostics/refusals_teach_the_dot_separator_s3_variant_parent_of.wat",
        "check.rs:2877",
    );
}

#[test]
fn cover_variant_arm_unqualified_teaches_dot() {
    source_contains(
        "src/check.rs",
        "variant `{path}` must be `<enum>.<Variant>`",
        "check.rs:6955",
    );
    accepted(
        "tests/diagnostics/refusals_teach_the_dot_separator_s1_not_namespaced.wat",
        "check.rs:6955",
    );
}

#[test]
fn nested_variant_map_unqualified_teaches_dot() {
    source_contains(
        "src/check.rs",
        "variant constructor pattern {path} must be `<enum>.<Variant>`",
        "check.rs:7253",
    );
    accepted(
        "tests/diagnostics/refusals_teach_the_dot_separator_s5_nested_map.wat",
        "check.rs:7253",
    );
}

#[test]
fn keyword_subpattern_teaches_dot() {
    let msg = refusal(
        "tests/diagnostics/refusals_teach_the_dot_separator_s6_keyword_subpattern.wat.bad",
    );
    let named_dot = msg.contains("keyword sub-pattern :B must be `<enum>.<Variant>`");
    assert!(
        named_dot,
        "check.rs:7508 must name the dot form and not offer retired :None; got {msg}"
    );
    accepted(
        "tests/diagnostics/refusals_teach_the_dot_separator_s6_keyword_subpattern.wat",
        "check.rs:7508",
    );
}

#[test]
fn list_constructor_pattern_teaches_dot() {
    let msg = refusal(
        "tests/diagnostics/refusals_teach_the_dot_separator_s7_list_ctor.wat.bad",
    );
    let named_dot = msg.contains("variant constructor pattern :u::E::A must be `<enum>.<Variant>`");
    assert!(
        named_dot,
        "check.rs:7755 must name the dot form; got {msg}"
    );
    accepted(
        "tests/diagnostics/refusals_teach_the_dot_separator_s7_list_ctor.wat",
        "check.rs:7755",
    );
}
