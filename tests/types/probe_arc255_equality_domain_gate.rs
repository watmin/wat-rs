//! Arc 255 Stone 1c-b-iii — the equality domain gate fires, and this test is its proof.
//!
//! `:wat::core::<` is `@Totality Total` because `infer_ordering` gates on `is_type_orderable`,
//! narrowing its declared domain to what the runtime can actually compare. `:wat::core::=` had no
//! such gate: it declared `∀T`, `Fn` was inside that domain, and `values_equal` returns `None`
//! there — which `eval_eq` raised. The fixture beside this file compares two functions and,
//! before the gate, `--check` exited 0 on it while RUNNING it raised.
//!
//! ⛔ THIS TEST EXISTS BECAUSE A GATE THAT HAS ONLY EVER BEEN SEEN GREEN HAS NOT BEEN SHOWN TO
//! WORK. The fixture is the negative witness; without a harness asserting the refusal, the gate
//! could be deleted tomorrow and nothing would notice.
//!
//! ⚠ The golden is the WHOLE rejection, not a substring. A first cut asserted
//! `rendered.contains("equatable")` and `contains("fn")` — the second was simply wrong (the type
//! renders `[:wat::core::i64 :-> :wat::core::i64]`, with no `fn` in it), and the first is the
//! loose-string-assert the repo's own lint bans: it would keep passing on reordered fields,
//! appended garbage, or a rejection for an entirely unrelated reason. The `.edn` golden pins the
//! callee, the parameter, the full declared domain, the offending type AND the span.
//!
//! Wat source: tests/types/probe_arc255_equality_domain_gate.wat.bad

use wat::freeze::startup_from_file;

const FIXTURE: &str = "tests/types/probe_arc255_equality_domain_gate.wat.bad";

/// Comparing two `fn` values is refused at CHECK time, naming the equatable domain.
#[test]
fn equality_refuses_a_non_equatable_declared_type() {
    let err = startup_from_file(FIXTURE)
        .expect_err("the equality domain gate must REFUSE a comparison of two `fn` values");
    wat::assert_edn_matches_file!(
        format!("{err:?}"),
        "probe_arc255_equality_domain_gate__fn_operand_refused.edn",
        "the equality domain gate must refuse a `fn` operand, naming :wat::core::= , parameter #1, \
         the equatable domain, and the offending type"
    );
}
