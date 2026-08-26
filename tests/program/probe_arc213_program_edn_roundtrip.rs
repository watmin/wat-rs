//! Gate probe — arc 213.S: WatAST ↔ plain-EDN round-trip.
//!
//! See `docs/arc/2026/05/213-libc-fork-mismanagement/BRIEF-213-SERIALIZER-BRIDGE.md`.
//!
//! Asserts:
//! 1. `program_to_edn` output has NO tagged-HolonAST substring (plain EDN only).
//! 2. The serialized frame visibly contains `:wat.core/defn` keywords and
//!    native `{ }` / `#{ }` syntax.
//! 3. `edn_to_program` → `startup_from_forms` freezes Ok for a round-tripped program.
//! 4. Keyword round-trip unit tests: `:wat::core::i64::+`, `:user::main` survive.
//!
//! ## STOP TRIGGER (reported, not papered)
//!
//! Keywords with `/` in their NAME segment (after the last `::`) cannot round-trip
//! through the existing codec. Specifically:
//! - `:wat::core::HashMap/length` → encodes to EDN `:wat.core/HashMap/length`
//!   (two `/` in the body) — the EDN parser rejects this as "more than one /".
//! - `:wat::core::HashSet/length` — same issue.
//!
//! ⚠ The witness was `:wat::core::char/of` until arc 255 renamed that verb to
//! `:wat::core::char`, which has no `/` and therefore round-trips fine. The PROPERTY
//! under test is unchanged and still real; only the exhibit moved, to a slash-in-name
//! verb that is still live. A test that keeps citing a retired name is a pin waiting
//! to mislead — the defect R9 names.
//!
//! This is the STOP trigger from BRIEF-213-SERIALIZER-BRIDGE.md §STOP-triggers:
//! "if a wat keyword cannot round-trip through the reused codec, STOP and report
//! the exact keyword — do not paper it with a String fallback."
//!
//! The test `stop_trigger_slash_in_name_keyword` documents this failure explicitly:
//! it asserts that `edn_to_program` ERRORS on the output of `program_to_edn` for
//! a form containing `:wat::core::HashMap/length`, rather than silently corrupting it.
//!
//! The sample program below is the one from the brief MINUS the `/`-in-name
//! call forms (`HashMap/length`, `HashSet/length`); those forms share the same
//! structural defect documented in `stop_trigger_slash_in_name_keyword`. The remainder
//! of the program exercises every collection type and every keyword shape that DOES
//! round-trip.
//!
//! Run: `cargo test --release --test probe_arc213_program_edn_roundtrip`

use std::sync::Arc;
use wat::freeze::startup_from_forms;
use wat::load::loader::InMemoryLoader;
use wat::parser::parse_all_with_file;
use wat::edn::bridge::{
    edn_to_program, edn_to_watast, program_to_edn, watast_to_edn, WatEdnBridgeError,
};
use wat::WatAST;

/// The sample program from BRIEF-213-SERIALIZER-BRIDGE.md, adjusted:
/// - Minus the `:wat::core::HashMap/length` and `:wat::core::HashSet/length` call forms
///   that trigger the `/`-in-name STOP condition (documented separately below).
/// - The brief's `:user::main` body uses `:wat::core::nil` as a VALUE — Doctrine 1
///   (arc 242) requires bare `nil` in value position; fixed here so the program freezes Ok.
///
/// Still exercises every collection shape: Map `{...}`, Set `#{...}`,
/// Vector `[...]`, List `(...)`, plus `:keys` destructure and multiple keyword namespaces.
///
/// Lives in the co-located fixture `probe_arc213_program_edn_roundtrip.wat`, read as raw
/// text (not startup_beside — this test feeds the text straight into `parse_all_with_file`
/// / the EDN bridge, not a `FrozenWorld`).
fn sample_program() -> String {
    std::fs::read_to_string("tests/program/probe_arc213_program_edn_roundtrip.wat")
        .expect("sample fixture must exist (run from crate root)")
}

/// T1 — program_to_edn output is PLAIN EDN (no tagged-HolonAST forms) and
/// contains the expected structural markers.
#[test]
fn t1_program_to_edn_is_plain_edn() {
    let src = sample_program();
    let forms = parse_all_with_file(&src, "probe_arc213.wat")
        .expect("sample program must parse");
    let frame = program_to_edn(&forms);

    // Exact golden covers all property checks: no holon tags, native set/map
    // syntax present, :wat.core/ keywords emitted — all verified implicitly.
    wat::assert_edn_matches_file!(frame.clone(), "probe_arc213_program_edn_roundtrip__program_frame.edn", "t1_frame: program_to_edn golden");

    eprintln!("T1 PASS — sample EDN frame:\n{}", &frame[..frame.len().min(800)]);
}

/// T2 — edn_to_program produces a Vec<WatAST> of the same length, and
/// startup_from_forms freezes Ok.
#[test]
fn t2_round_trip_and_freeze() {
    let src = sample_program();
    let forms = parse_all_with_file(&src, "probe_arc213.wat")
        .expect("sample program must parse");
    let form_count = forms.len();
    let frame = program_to_edn(&forms);

    let decoded = edn_to_program(&frame)
        .expect("T2 FAIL: edn_to_program should succeed for a valid frame");
    assert_eq!(
        decoded.len(),
        form_count,
        "T2 FAIL: round-tripped program has {} forms, expected {}",
        decoded.len(),
        form_count
    );

    // freeze the round-tripped forms — startup_from_forms must return Ok.
    let result = startup_from_forms(decoded, None, Arc::new(InMemoryLoader::new()));
    assert!(
        result.is_ok(),
        "T2 FAIL: startup_from_forms failed on round-tripped program: {:?}",
        result.err()
    );
    eprintln!("T2 PASS — {} forms round-tripped and froze Ok", form_count);
}

/// T3 — keyword round-trip unit test.
/// `:wat::core::i64::+` and `:user::main` survive `watast_to_edn`→write→parse→`edn_to_watast`.
/// (`:wat::core::HashMap/length` is covered by `stop_trigger_slash_in_name_keyword` below.)
#[test]
fn t3_keyword_round_trip_no_slash() {
    for kw in &[":wat::core::i64::+", ":user::main", ":wat::config::set-capacity-mode!"] {
        let ast = WatAST::keyword(*kw);
        let edn_val = watast_to_edn(&ast);
        let edn_str = wat_edn::write(&edn_val);

        // Parse back and decode.
        let parsed = wat_edn::parse_owned(&edn_str)
            .unwrap_or_else(|e| panic!("keyword {} round-trip parse failed: {}", kw, e));
        let decoded = edn_to_watast(&parsed)
            .unwrap_or_else(|e| panic!("keyword {} edn_to_watast failed: {:?}", kw, e));

        match &decoded {
            WatAST::Keyword(k, _) => {
                assert_eq!(
                    k.as_str(),
                    *kw,
                    "T3 FAIL: keyword {} → {} → {} (round-trip changed the path)",
                    kw, edn_str, k
                );
            }
            other => panic!(
                "T3 FAIL: keyword {} decoded to {:?} (expected Keyword)",
                kw, other
            ),
        }
        eprintln!("T3 PASS: {} ↔ {}", kw, edn_str);
    }
}

/// STOP TRIGGER — documented, not papered.
///
/// `:wat::core::HashMap/length` cannot round-trip because the `/` in the name segment
/// (`HashMap/length` after the last `::`) makes the EDN writer emit `:wat.core/HashMap/length`
/// — a keyword body with TWO `/` separators, which the EDN parser rejects
/// ("more than one / in wat.core/HashMap/length").
///
/// Root cause: `keyword_from_wat_path` splits on the LAST `::` to get
/// (ns="wat.core", name="HashMap/length"), then calls `Keyword::try_ns(…)`.
/// `try_ns` only validates the FIRST CHARACTER, so it ACCEPTS `HashMap/length` as a name.
/// But the EDN writer emits `:ns/name` → `:wat.core/HashMap/length`, which the parser's
/// `parse_namespaced` rejects because `splitn(3, '/')` finds a third part.
///
/// The fix (NOT in scope for this strike): before the last `::`, check whether the
/// name segment contains a `/`. If it does, the current codec cannot represent this
/// keyword. Options: (a) encode the full path differently (escaping `/` or using a
/// different separator), (b) reject at encode time with a clear error, or (c) use
/// a `Tagged` wrapper (which the brief explicitly prohibits). This is a structural
/// gap that must be resolved at the codec level, not a String fallback.
///
/// This test asserts the FAILURE is detectable and honest (the EDN string produced
/// for `:wat::core::HashMap/length` fails to parse back), rather than silently returning
/// the wrong keyword.
#[test]
fn stop_trigger_slash_in_name_keyword() {
    let kw = ":wat::core::HashMap/length";
    let ast = WatAST::keyword(kw);
    let edn_val = watast_to_edn(&ast);
    let edn_str = wat_edn::write(&edn_val);

    eprintln!(
        "STOP TRIGGER: keyword {} encodes to EDN '{}' — two '/' in body",
        kw, edn_str
    );

    // The EDN parser MUST reject this (two `/` in keyword body).
    // If it somehow accepts it, the round-trip test below would catch it.
    let parse_result = wat_edn::parse_owned(&edn_str);
    if let Ok(parsed) = parse_result {
        // If parsing succeeded, check whether the round-trip is correct.
        match edn_to_watast(&parsed) {
            Ok(WatAST::Keyword(k, _)) if k.as_str() == kw => {
                // Unexpectedly correct — this would invalidate the STOP trigger report.
                eprintln!(
                    "NOTE: {} round-tripped correctly (EDN str: {}). STOP trigger may be resolved.",
                    kw, edn_str
                );
            }
            Ok(WatAST::Keyword(k, _)) => {
                panic!(
                    "STOP TRIGGER CONFIRMED: {} encoded to '{}', decoded to '{}' (CORRUPTED — wrong keyword)",
                    kw, edn_str, k
                );
            }
            Ok(other) => {
                panic!(
                    "STOP TRIGGER CONFIRMED: {} encoded to '{}', decoded to wrong variant {:?}",
                    kw, edn_str, other.variant_name()
                );
            }
            Err(WatEdnBridgeError::UnsupportedEdnForm { shape }) => {
                // Tagged or other unsupported form — also a failure.
                panic!(
                    "STOP TRIGGER CONFIRMED: {} encoded to '{}', edn_to_watast returned UnsupportedEdnForm({})",
                    kw, edn_str, shape
                );
            }
            Err(e) => {
                panic!(
                    "STOP TRIGGER CONFIRMED: {} encoded to '{}', edn_to_watast failed: {:?}",
                    kw, edn_str, e
                );
            }
        }
    } else {
        // EDN parse rejected it — confirm it's the two-slash error.
        let err_msg = parse_result.unwrap_err().to_string();
        assert_eq!(
            err_msg,
            "EDN parse error at byte 24: invalid keyword: more than one / in wat.core/HashMap/length",
            "stop_errmsg: EDN parse rejection golden"
        );
        eprintln!(
            "STOP TRIGGER CONFIRMED: '{}' rejected by EDN parser: {}",
            edn_str, err_msg
        );
        // This test passes — the failure is honest (parse error), not silent corruption.
        // The STOP is reported: the orchestrator must decide how to fix the codec.
    }
}
