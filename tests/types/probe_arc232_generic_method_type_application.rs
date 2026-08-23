//! Arc 232 follow-on (6b-ii-β) — generic-method TYPE-ARGUMENT APPLICATION, **RETIRED**.
//!
//! This probe once asserted that `(:user::Mk/mk<wat::core::i64,wat::core::i64> …)` — the
//! *turbofish*, explicit type-args at a generic-method call site — resolved and minted a typed
//! bound. Stone 6b-DEP (arc 272) made it green by stripping the `<…>` suffix in
//! `runtime::canonical_callable_name` to match the registered bare method name.
//!
//! **Arc 109 "the comma dies in the reader" retired the spelling itself.** A comma can never
//! appear in a keyword body, at any depth — and `:user::Mk/mk<i64,i64>` is a keyword whose body
//! carries one. The construct is a LEX error now, raised before any resolution or
//! type-application logic runs, so `canonical_callable_name`'s strip is unreachable from source.
//!
//! Why the spelling had to go, and it is not about the brackets: **`,` is whitespace in EDN and in
//! wat** — `(:wat::core::Vector :- [:i64] 1, 2, 3)` reads as `[1 2 3]`. `Head<K,V>` was the only
//! construct in the language that gave a comma meaning, and the substrate carried a bidirectional
//! wire-escape (`,`↔`_`) plus a language-wide reservation on `_` inside `<…>` purely to smuggle it
//! across a wire that cannot represent it. All three are deleted.
//!
//! The fixture is kept as `.wat.bad` — the negative control. A feature removed without a test that
//! proves it stays removed is a feature that comes back.

use wat::freeze::startup_from_file;

const FIXTURE: &str = "tests/types/probe_arc232_generic_method_type_application.wat.bad";

#[test]
fn the_callable_turbofish_is_refused_by_the_reader() {
    let err = startup_from_file(FIXTURE)
        .map(|_| ())
        .expect_err("the turbofish `:user::Mk/mk<i64,i64>` must be REFUSED — arc 109");
    let msg = format!("{err:?}");
    // rune:lint(loose-assert) — a targeted PRESENCE over a large structured diagnostic. The
    // assertion names the MECHANISM (the comma, not the brackets), which is the whole claim of
    // arc 109's kill strike. An exact-match golden would pin the lex error's byte offset and
    // re-break on every unrelated edit to the fixture.
    assert!(
        msg.contains("comma inside keyword body retired"),
        "must be refused for the COMMA specifically — that is the EDN violation the strike closed, \
         not the angle brackets. got: {msg}"
    );
}

/// The dual, and the row that makes the refusal above mean something: a comma between VALUES is
/// still ordinary EDN whitespace. A wall that refused commas everywhere would pass the test above
/// and break the language. The co-located `.wat` builds a 3-element Vector written `1, 2, 3`.
#[test]
fn a_comma_between_values_is_still_whitespace() {
    let got = wat::freeze::call_beside_value(file!(), ":user::compute")
        .expect("commas between values must remain EDN whitespace");
    assert_eq!(
        got,
        wat::runtime::Value::i64(3),
        "`(:wat::core::Vector :- [:i64] 1, 2, 3)` must read as THREE elements — the comma is \
         whitespace between values, and only comma-as-separator-inside-a-name died"
    );
}
