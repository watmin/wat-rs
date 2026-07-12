//! Arc 278 stone T1b.1 — the `:wat::telemetry'::Journal` surface acceptance gate.
//!
//! A throwaway toy `:probe::toy-journal'` (`:satisfies :wat::telemetry'::Journal`, mirroring
//! `mem-store'`'s satisfaction of `Store`) is spawned on a `:wat::spawn::thread`, dialed, and sent
//! a `write-metrics` call carrying a 1-element `Metric` batch. Proves the surface freezes, is
//! satisfiable, and replies through the wire with `Journal::WriteMetricsResponse::Success`.
//!
//! Driven via a programmatic AST call (not an inline `parse_one!` string) so it trips no
//! `no_inlined_wat` lint — mirrors `tests/services/probe_arc170_gapj_each_kwargs.rs`.
//!
//! The sibling negative `probe_arc278_journal_surface_swap.wat.bad` swaps `write-metrics`'s reply
//! to a `:wat::query::Store::PutResponse::Success` (a structurally-similar but DIFFERENT enum) —
//! `wrong_response_type_at_reply_site_is_compile_error` below proves that is a located
//! `TypeMismatch`, never a runtime mismatch.
//!
//! FORKS a thread (the toy service) — run --test-threads=1:
//! cargo nextest run --release -E 'test(/probe_arc278_journal_surface/)' --test-threads=1

use wat::ast::WatAST;
use wat::check::error::{CheckErrorKind, CheckErrors};
use wat::freeze::{eval_in_frozen, startup_from_file, StartupError};
use wat::runtime::{Environment, Value};

#[test]
fn toy_journal_satisfies_surface_and_replies_success() {
    let world = startup_from_file("tests/services/probe_arc278_journal_surface.wat").expect(
        "startup should succeed (arc 278 T1b.1: the Journal surface must freeze and the toy \
         satisfier must be well-typed against it)",
    );
    let call = WatAST::List(
        vec![WatAST::Keyword(":probe::run".into(), wat::rust_caller_span!())],
        wat::rust_caller_span!(),
    );
    let got = eval_in_frozen(&call, &world, &Environment::new())
        .unwrap_or_else(|e| panic!("run raised: {e:?}"))
        .value_owned();
    match got {
        Value::Enum(ev) => {
            assert_eq!(
                ev.type_path, ":wat::telemetry'::Journal::WriteMetricsResponse",
                "expected a WriteMetricsResponse enum; got type_path {:?}",
                ev.type_path
            );
            assert_eq!(
                ev.variant_name, "Success",
                "toy-journal' write-metrics must reply Success through the dialed Journal peer; \
                 got variant {:?}",
                ev.variant_name
            );
        }
        other => panic!("expected Value::Enum(Journal::WriteMetricsResponse::Success), got {other:?}"),
    }
}

#[test]
fn wrong_response_type_at_reply_site_is_compile_error() {
    let err = startup_from_file("tests/services/probe_arc278_journal_surface_swap.wat.bad")
        .expect_err("write-metrics replying a Store::PutResponse instead of a Journal::WriteMetricsResponse must fail check");
    let StartupError::Check(CheckErrors(errs)) = &err else {
        panic!("expected a type-check error, got {err:?}");
    };
    match &errs[0].kind {
        CheckErrorKind::TypeMismatch { expected, got, .. } => {
            assert_eq!(expected, ":wat::telemetry'::Journal::WriteMetricsResponse");
            assert_eq!(got, ":wat::query::Store::PutResponse");
        }
        other => panic!("expected TypeMismatch, got {other:?}"),
    }
}
