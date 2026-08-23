//! Arc 170 C2 — Strike 2: the C2 gate THROUGH THE `bracket/uses` MACRO (mixed 7 services + 5 data).
//!
//! The complete `bracket/uses` surface, end to end, in one user form:
//!   `(:wat::bracket::uses (process) items :probe::enrich :name val …)` — RAW `:name val` pairs
//!   (handles for the 7 services, values for the 5 data), SCRAMBLED order. Exercises every part:
//!  - **Part A** — the checker's service params are `TypedCapability<S,R>` (arc 170 C2 Candidate
//!    D — raw handles type-check; the checker coords each service field internally into
//!    `::Coords` AND collects them into `::GrantHandles`).
//!  - **Part B** — the macro parses `(locus items work-fn :name val …)` and expands to
//!    `(let [coords (…kwargs-check :name val…)] (uses' locus [(Tuple :name val) …] items work-fn coords))`.
//!  - **Part C (Candidate D)** — `uses'`'s grant-boot calls the minted, TYPED
//!    `<fqdn>::grant-worker` (no dispatch, no erasure) over the 7 service handles held in
//!    `::GrantHandles`; the 5 data values never enter a grant carrier at all.
//!  - the Strike-1 dial runtime (unchanged): `::Coords → ::Kwargs` by field name, dial 7 + copy 5.
//!
//! 1. `mixed_via_macro_runs` (POSITIVE) — forks; runs; returns the correct per-item vector.
//! 2. `mixed_via_macro_swap_is_compile_error` (NEGATIVE — the soundness gate) — a swapped service
//!    handle is a located structural `TypeMismatch` (now naming `TypedCapability`/`Handle`), at
//!    check time.
//!
//! The positive FORKS processes (7 services + N pool workers) — run --test-threads=1:
//!   cargo nextest run -p wat -E 'test(/probe_arc170_c2_mixed_macro/)' --test-threads=1

use wat::ast::WatAST;
use wat::check::error::{CheckErrorKind, CheckErrors};
use wat::freeze::{eval_in_frozen, startup_from_file, StartupError};
use wat::runtime::{Environment, Value};

#[test]
fn mixed_via_macro_runs() {
    let world = startup_from_file("tests/services/probe_arc170_c2_mixed_macro.wat")
        .expect("startup should succeed (arc 170 C2 Strike 2: mixed 7 services + 5 data via macro)");
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
                "arc 170 C2 Strike 2: (bracket/uses …) with 7 raw service handles (all dialed, \
                 granted per-val) + 5 raw data values (copied), scrambled order, in input order"
            );
        }
        other => panic!("expected Vector<String>, got {other:?}"),
    }
}

#[test]
fn mixed_via_macro_swap_is_compile_error() {
    let err = startup_from_file("tests/services/probe_arc170_c2_mixed_macro_swap.wat.bad")
        .expect_err("a swapped service handle through bracket/uses must fail check");
    let StartupError::Check(CheckErrors(errs)) = &err else {
        panic!("expected a type-check error, got {err:?}");
    };
    // `:s1 h2` — S2's handle bound to the :s1 kwarg (TypedCapability<S1…>). An
    // s2'::Handle satisfies ONLY TypedCapability<S2…>, so the checker's
    // TypedCapability param rejects it.
    // rune:lint(no-inlined-wat) — the expected/got strings below are golden COMPARISON
    // text for a TypeMismatch's rendered fields, never a wat world/driver; they happen to be
    // reader-parseable now only because the checker's error renderer emits real `(Head :- [args])`
    // syntax instead of the retired unparseable `Head<a,b>` pseudo-syntax (that is the whole point
    // of this stone). Nothing here builds or runs a wat program from this string.
    // STONE-defservice-emits-the-binder (arc 109) — same call site, re-rendered: the
    // checker stopped minting `Head<a,b>` (a spelling the reader now refuses) and emits
    // the surviving `(Head :- [args])` form instead.
    wat::assert_check_error_present!(errs,
        CheckErrorKind::TypeMismatch { expected, got, .. }
            if expected == "(:wat::capability::TypedCapability :- [:probe::S1::Op :probe::S1::Reply])"
            && got == "(:probe::s2::Handle :- [:wat::kernel::Wire])");
}
