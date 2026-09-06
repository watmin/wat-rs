//! STONE-the-then-block-never-checked-its-operand — negative controls for both defects.
//!
//! Defect 1: a `:then` operand is never checked against the field it lands in.
//! Defect 2: `RhsUnresolvableOperand`'s accepted list omitted the call form, which
//! is the message that taught `Break.kind` to be a String.
//!
//! These probes are rules the fix must REJECT, so they live under `tests/rete/`
//! (STOP-6: `every_wat_scripts_file_loads` would go red on success if they sat
//! in `wat-scripts/`). Un-arm either check and these tests go green-when-they-
//! should-be-red.

use wat::freeze::{startup_from_file, StartupError};

fn validator_text(path: &str) -> String {
    let err = startup_from_file(path)
        .expect_err("this fixture must fail check");
    let StartupError::Validator(errs) = &err else {
        panic!("expected StartupError::Validator, got {err:?}");
    };
    format!("{errs}")
}

#[test]
fn i64_into_a_string_then_is_refused() {
    let rendered = validator_text(
        "tests/rete/probe_then_operand_fits_the_field_i64_into_string.wat.bad",
    );
    // rune:lint(loose-assert) — the error embeds an absolute path (Span :file).
    assert!(
        rendered.contains("RhsOperandTypeMismatch"),
        "wrong error kind:\n{rendered}"
    );
    // rune:lint(loose-assert) — same span/path reason. Names the rule.
    assert!(rendered.contains("fit::arm-a"), "must name the rule:\n{rendered}");
    // rune:lint(loose-assert) — names the field.
    assert!(rendered.contains("label"), "must name the field:\n{rendered}");
    // rune:lint(loose-assert) — names both declared types, not rete segments.
    assert!(
        rendered.contains("wat::core::String") && rendered.contains("wat::core::i64"),
        "must name declared String and actual i64:\n{rendered}"
    );
}

#[test]
fn enum_into_a_different_enum_is_refused() {
    let rendered = validator_text(
        "tests/rete/probe_then_operand_fits_the_field_alpha_into_beta.wat.bad",
    );
    // rune:lint(loose-assert) — Span :file is an absolute path.
    assert!(
        rendered.contains("RhsOperandTypeMismatch"),
        "wrong error kind:\n{rendered}"
    );
    // rune:lint(loose-assert) — a segment check would say both are "enum" and pass.
    assert!(
        rendered.contains("fit::Alpha") && rendered.contains("fit::Beta"),
        "must name Alpha and Beta, not the segment `enum`:\n{rendered}"
    );
}

#[test]
fn two_bad_operands_are_both_reported() {
    let rendered = validator_text(
        "tests/rete/probe_then_operand_fits_the_field_two_bad.wat.bad",
    );
    // rune:lint(loose-assert) — Span :file is an absolute path.
    let n = rendered.matches("RhsOperandTypeMismatch").count();
    assert!(
        n >= 2,
        "a rule with two bad operands must report both; got {n}:\n{rendered}"
    );
    // rune:lint(loose-assert) — both fields named.
    assert!(
        rendered.contains("\"a\"") || rendered.contains(":a") || rendered.contains(" a "),
        "must name field a:\n{rendered}"
    );
}

#[test]
fn unresolvable_accepted_list_names_the_call_form() {
    let rendered = validator_text(
        "tests/rete/probe_then_operand_fits_the_field_bare_keyword.wat.bad",
    );
    // rune:lint(loose-assert) — Span :file is an absolute path.
    assert!(
        rendered.contains("RhsUnresolvableOperand"),
        "wrong error kind:\n{rendered}"
    );
    // rune:lint(loose-assert) — the call form was the missing accepted entry.
    assert!(
        rendered.contains("call form") || rendered.contains("constructor"),
        "accepted list must name the call form / constructor:\n{rendered}"
    );
    // rune:lint(loose-assert) — a keyword is still a field reference, not a value.
    assert!(
        rendered.contains("field reference") || rendered.contains("Block"),
        "a bare keyword must still be refused, and the field-reference reason findable:\n{rendered}"
    );
}

#[test]
fn same_enum_then_still_compiles() {
    startup_from_file("tests/rete/probe_then_operand_fits_the_field_same_enum.wat")
        .expect("Alpha into Alpha must still compile");
}
