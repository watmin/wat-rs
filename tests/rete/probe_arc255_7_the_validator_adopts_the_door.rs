//! 255.7 — rete validator `:then` heads adopt `canonical_identity`.
//! Converted `(weather/ColdAndWindy :location ?loc)` validates; a bad field
//! and a non-list :then stay diagnosed; `(?k <- :k)` stays MalformedClause.

use wat::freeze::{startup_beside, startup_from_file};

#[test]
fn converted_then_head_validates() {
    assert!(
        startup_beside(file!()).is_ok(),
        "(weather/ColdAndWindy :location ?loc) as a :then insert must freeze"
    );
}

#[test]
fn unknown_then_field_is_still_diagnosed() {
    let err = startup_from_file(
        "tests/rete/probe_arc255_7_the_validator_adopts_the_door_unknown_field.wat.bad",
    )
    .expect_err("a bad :then field must stay a freeze error");
    let msg = format!("{err}");
    // rune:lint(loose-assert) — span varies; pin the diagnostic kind and the bad field.
    assert!(
        msg.contains("UnknownField") && msg.contains("nope"),
        "expected UnknownField naming :nope (diagnosed, not skipped), got:\n{msg}"
    );
}

#[test]
fn non_list_then_item_is_still_malformed() {
    let err = startup_from_file(
        "tests/rete/probe_arc255_7_the_validator_adopts_the_door_non_list.wat.bad",
    )
    .expect_err("a non-list :then item must stay a freeze error");
    let msg = format!("{err}");
    // rune:lint(loose-assert) — span varies; pin MalformedClause so a skip cannot hide.
    assert!(
        msg.contains("MalformedClause"),
        "expected MalformedClause for :then [42], got:\n{msg}"
    );
}

#[test]
fn retired_bind_arrow_is_still_malformed() {
    let err = startup_from_file("tests/rete/probe_arc255_6_rete_adopts_the_door.wat.bad")
        .expect_err("(?k <- :k) must stay a freeze error");
    let msg = format!("{err}");
    // rune:lint(loose-assert) — pin MalformedClause + the retired arrow.
    assert!(
        msg.contains("MalformedClause") && msg.contains("<-"),
        "expected MalformedClause naming the retired `<-` bind, got:\n{msg}"
    );
}
