//! 255.6 — rete's clause grammar adopts `canonical_identity`.
//! A converted clause `(vrm/F (?k :- :k))` / `(?fact :- weather/Type)` parses;
//! `(?k <- :k)` stays `MalformedClause`.

use wat::freeze::{startup_beside, startup_from_file};

#[test]
fn converted_fact_pattern_and_fact_bind_check() {
    assert!(
        startup_beside(file!()).is_ok(),
        "(weather/Temperature …) and (?fact :- weather/ColdAndWindy) must freeze"
    );
}

#[test]
fn retired_bind_arrow_is_still_malformed() {
    let err = startup_from_file("tests/rete/probe_arc255_6_rete_adopts_the_door.wat.bad")
        .expect_err("(?k <- :k) must stay a freeze error");
    let msg = format!("{err}");
    // rune:lint(loose-assert) — the diagnostic carries a file:line span; pin the
    // two stable tokens of the refusal (MalformedClause + the retired arrow).
    assert!(
        msg.contains("MalformedClause") && msg.contains("<-"),
        "expected MalformedClause naming the retired `<-` bind, got:\n{msg}"
    );
}
