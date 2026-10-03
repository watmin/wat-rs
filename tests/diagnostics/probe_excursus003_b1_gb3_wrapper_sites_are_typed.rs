//! Excursus 003 strike B1, GB3 — the four wrapper sites are typed.
//!
//! Item 5's four sites (`check_failed_cause`; the second `CheckFailed` producer;
//! `read_outcome_malformed`; `tagged_read_outcome_malformed`) used to wrap their decoded
//! diagnostic in a `:wat::core::Fault` with a fabricated `<runtime>:0:0` location. All
//! four now hand back a real, declared record class: the first three hand back the
//! diagnostic's OWN class directly; the fourth (`tagged_read_outcome_malformed`, behind
//! `:wat::edn::read-json`/`read-foreign`) closed in strike B3 item 2 — a JSON/EDN text
//! that fails to PARSE at all has no declared type of its own, so it routes through
//! `EdnReadErrorKind::Other` into the EXISTING declared `:wat::edn::ReadError` catch-all
//! (`wat/edn.wat` — "an unparseable frame" is literally what that record's doc names),
//! not a `:wat::core::Fault` stand-in.
//!
//! Mutation for sites 1-3 (recorded in the strike report, not re-encoded here): restore
//! the `fault_with_cause` wrap at each site — this test's `assert_eq!` on `.class` goes
//! RED, naming `wat::core::Fault` where the diagnostic's own class was expected. Site 4's
//! own mutation is strike B3 item 2's gate (recorded in that strike's report).

use wat::freeze::call_beside_value;
use wat::runtime::Value;

fn aggregate<'a>(v: &'a Value, site: &str) -> &'a wat::runtime::AggregateValue {
    match v {
        Value::Aggregate(a) => a,
        other => panic!("{site}: expected a typed Aggregate record; got {other:?}"),
    }
}

/// Site 1 — `check_failed_cause`, via `:wat::eval-with-defs!`'s per-line REPL check.
#[test]
fn site1_check_failed_cause_is_the_diagnostics_own_class() {
    let v = call_beside_value(file!(), ":user::check-failed-repl")
        .expect(":user::check-failed-repl must run and return the cause, never raise");
    let cause = aggregate(&v, "site1");
    assert_eq!(
        cause.class.as_ref(),
        "wat::check::CheckErrors",
        "site1: the cause must be the real CheckErrors aggregate, not a Fault wrapper \
         (got class {:?})",
        cause.class
    );
}

/// Site 2 — the second `CheckFailed` producer (`eval_form_against_defs`'s baseline-freeze
/// arm).
#[test]
fn site2_second_check_failed_producer_is_the_diagnostics_own_class() {
    let v = call_beside_value(file!(), ":user::check-failed-second-producer")
        .expect(":user::check-failed-second-producer must run and return the cause, never raise");
    let cause = aggregate(&v, "site2");
    assert_eq!(
        cause.class.as_ref(),
        "wat::resolve::UnresolvedReferences",
        "site2: the cause must be the real diagnostic, not a Fault framing WHICH freeze \
         failed (got class {:?})",
        cause.class
    );
}

/// Site 3 — `read_outcome_malformed`, via `:wat::core::read-string` on malformed source.
#[test]
fn site3_read_string_malformed_is_the_diagnostics_own_class() {
    let v = call_beside_value(file!(), ":user::read-string-malformed")
        .expect(":user::read-string-malformed must run and return the cause, never raise");
    let cause = aggregate(&v, "site3");
    assert_eq!(
        cause.class.as_ref(),
        "wat::parse::UnclosedParen",
        "site3: the cause must be the real ParseErrorKind record, not a Fault wrapper \
         (got class {:?})",
        cause.class
    );
}

/// Site 4 — `tagged_read_outcome_malformed`, via `:wat::edn::read-json` on malformed JSON.
/// Strike B3 item 2 closed this site's gap: a JSON parse failure has no declared type of
/// its OWN (the text never named a tag at all), but it is NOT a bare `:wat::core::Fault`
/// any more — it routes through `EdnReadErrorKind::Other` into the declared
/// `:wat::edn::ReadError` catch-all (`wat/edn.wat`), the SAME wrap `read_edn_caps` already
/// used for its own unparseable-frame case.
#[test]
fn site4_read_json_malformed_is_the_declared_edn_read_error() {
    let v = call_beside_value(file!(), ":user::read-json-malformed")
        .expect(":user::read-json-malformed must run and return the cause, never raise");
    match &v {
        Value::Aggregate(a) => assert_eq!(
            a.class.as_ref(),
            "wat::edn::ReadError",
            "site4: a JSON parse failure routes through the declared :wat::edn::ReadError \
             catch-all now, not a Fault stand-in (got class {:?})",
            a.class
        ),
        Value::wat__edn__ForeignRecord(_) => {
            panic!("site4: the cause is a foreign, dynamic ForeignRecord — the exact mask \
                    excursus 003 exists to kill")
        }
        other => panic!("site4: expected a typed Aggregate :wat::edn::ReadError; got {other:?}"),
    }
}
