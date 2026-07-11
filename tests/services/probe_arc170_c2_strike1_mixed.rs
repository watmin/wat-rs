//! Arc 170 C2 — Strike 1 (RUNTIME) hand-wired MIXED proof, no `bracket/uses` macro (Strike 2).
//!
//! Proves the record-carrier N-service runtime in ONE shot for a 12-field kwargs work-fn:
//! **7 heterogeneous `Peer'` service kwargs** (dialed) **+ 5 `String` data kwargs** (copied).
//! This exercises both the removal of the old first/second/third=3 positional-accessor CAP
//! (N=7 services) AND the data-copy path (5 data fields), reconciled BY FIELD NAME off the
//! named `:probe::enrich::Coords` record (Strike 1a) through the generalized dial-runner
//! (Strike 1b) and `:wat::bracket::uses'`'s single `PoolMsg::Setup(coords)` (Strike 1c).
//!
//! This test FORKS processes (7 services + N pool workers, each a fork) — run --test-threads=1:
//! cargo nextest run -p wat -E 'test(/probe_arc170_c2_strike1_mixed/)' --test-threads=1
//!
//! Driven via a programmatic AST call (not an inline `parse_one!` string) so it trips no
//! `no_inlined_wat` lint — mirrors `tests/services/probe_arc170_c1_kwargs_bracket.rs`.

use wat::ast::WatAST;
use wat::freeze::{eval_in_frozen, startup_from_file};
use wat::runtime::{Environment, Value};

#[test]
fn c2_strike1_mixed_7_services_5_data_runs_end_to_end() {
    let world = startup_from_file("tests/services/probe_arc170_c2_strike1_mixed.wat")
        .expect("startup should succeed (arc 170 C2 Strike 1: 7 services + 5 data, record carrier)");
    let call = WatAST::List(
        vec![WatAST::Keyword(":probe::run".into(), wat::rust_caller_span!())],
        wat::rust_caller_span!(),
    );
    let got = eval_in_frozen(&call, &world, &Environment::new())
        .unwrap_or_else(|e| panic!("run raised: {e:?}"))
        .value_owned();
    match got {
        Value::Vec(ref v) => {
            let strs: Vec<String> = v
                .iter()
                .map(|tv| match tv {
                    Value::String(s) => (**s).clone(),
                    other => panic!("expected String elements, got {other:?}"),
                })
                .collect();
            assert_eq!(
                strs,
                vec![
                    "a|s1:as2:as3:as4:as5:as6:as7:aD1D2D3D4D5".to_string(),
                    "b|s1:bs2:bs3:bs4:bs5:bs6:bs7:bD1D2D3D4D5".to_string(),
                ],
                "arc 170 C2 Strike 1 mixed: all 7 services dialed (no cap) + all 5 data values \
                 copied, each item run through the record-reconciled ::Kwargs, in input order"
            );
        }
        other => panic!("expected Vector<String>, got {other:?}"),
    }
}
