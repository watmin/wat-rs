//! Excursus 003 strike D, GD5 — no golden carries a synthesized or placeholder `Frame`.
//!
//! The `Frame` reshape (`AUDIT-the-shape-of-an-error.md` F6, RULING 2026-09-27 item 2;
//! RULING 2026-10-03 item 1) retired the OLD `{symbol span kind}` shape outright: `kind`
//! (derivable from `span.end`, D1) is gone with no replacement accessor, and the `<rust>`
//! placeholder `symbol` (D3) is gone too — every `Frame`'s `fn` is now mandatory and real
//! (`crate::value::frame::current_activation`, item 4).
//!
//! **Parse, never grep.** A text search for `:kind` would also catch unrelated records
//! that happen to have their OWN `:kind` field (`#wat.kernel/Remedy`, `MapDestructureKind`,
//! …) — measured the hard way: a first draft of this gate text-matched any `:kind` line
//! and found 39 false positives, none of them a `Frame`. This gate parses every tracked
//! `.edn` golden and walks the DATA, checking ONLY inside a `Tagged` value whose tag is
//! `wat.kernel/Frame` — the same `no_causes_key_in_golden_files.rs`/`no_double_quoted_edn_
//! in_golden_files.rs` discipline (parse, walk, never grep).
//!
//! Scope matches those gates' own: `git ls-files '*.edn'`, no exclusions.

use std::path::Path;
use wat_edn::{OwnedValue, Value};

fn git_ls_files(root: &str, glob: &str) -> Vec<String> {
    let out = std::process::Command::new("git")
        .args(["-C", root, "ls-files", "--", glob])
        .output()
        .expect("git ls-files");
    assert!(out.status.success(), "git ls-files must succeed");
    String::from_utf8_lossy(&out.stdout)
        .lines()
        .map(str::to_string)
        .collect()
}

fn is_frame_tag(tag: &wat_edn::Tag) -> bool {
    tag.namespace() == "wat.kernel" && tag.name() == "Frame"
}

/// Recursively walk `v`, checking every `Tagged(wat.kernel/Frame, body)` found at any
/// depth (nested in a map value, a list/vector/set element, or another Tagged's body).
/// Pushes one violation string per defect found.
fn find_frame_violations(v: &OwnedValue, out: &mut Vec<String>) {
    match v {
        Value::Tagged(tag, body) => {
            if is_frame_tag(tag) {
                if let Value::Map(entries) = &**body {
                    for (k, val) in entries {
                        let Value::Keyword(kw) = k else { continue };
                        if kw.namespace().is_some() {
                            continue;
                        }
                        if kw.name() == "kind" {
                            out.push("a Frame carries a :kind key — Frame's retired field".to_string());
                        }
                        if kw.name() == "fn" {
                            if let Value::String(s) = val {
                                if s.as_ref() == "<rust>" {
                                    out.push(
                                        "a Frame's :fn is the retired \"<rust>\" placeholder"
                                            .to_string(),
                                    );
                                }
                            }
                        }
                    }
                }
            }
            find_frame_violations(body, out);
        }
        Value::Map(entries) => {
            for (k, val) in entries {
                find_frame_violations(k, out);
                find_frame_violations(val, out);
            }
        }
        Value::List(xs) | Value::Vector(xs) | Value::Set(xs) => {
            for x in xs {
                find_frame_violations(x, out);
            }
        }
        _ => {}
    }
}

#[test]
fn no_golden_frame_carries_kind_or_the_rust_placeholder() {
    let root = env!("CARGO_MANIFEST_DIR");
    let paths = git_ls_files(root, "*.edn");
    // NON-VACUITY: the walk must actually find the tracked .edn corpus — a typo'd glob or a
    // broken `git ls-files` would silently check zero files and report PASS.
    assert!(paths.len() > 100, "expected the tracked .edn corpus; got {}", paths.len());

    let mut violations: Vec<(String, Vec<String>)> = Vec::new();
    let mut unparseable = 0usize;
    for rel in &paths {
        let full = Path::new(root).join(rel);
        let src = std::fs::read_to_string(&full).unwrap_or_else(|e| panic!("{rel}: read: {e}"));
        let trimmed = src.trim();
        if trimmed.is_empty() {
            continue;
        }
        let parsed = match wat_edn::parse_owned(trimmed) {
            Ok(v) => v,
            Err(_) => {
                // A handful of goldens are TEMPLATES (an unquoted `{PLACEHOLDER}`) and do
                // not parse raw — same carve-out the sibling gates document.
                unparseable += 1;
                continue;
            }
        };
        let mut found = Vec::new();
        find_frame_violations(&parsed, &mut found);
        if !found.is_empty() {
            violations.push((rel.clone(), found));
        }
    }
    eprintln!("{unparseable} of {} tracked .edn files skipped (unquoted template placeholder)", paths.len());
    assert!(
        unparseable < paths.len() / 2,
        "{unparseable} of {} tracked .edn files did not parse at all — far more than the \
         template-placeholder shape this gate expects; `wat_edn::parse_owned` may be broken",
        paths.len()
    );

    let total: usize = violations.iter().map(|(_, v)| v.len()).sum();
    assert!(
        violations.is_empty(),
        "GD5: {} golden file(s), {} Frame violation(s) total — a synthesized/placeholder \
         Frame survived the reshape:\n{}",
        violations.len(),
        total,
        violations
            .iter()
            .map(|(f, vs)| format!("{f}: {}", vs.join("; ")))
            .collect::<Vec<_>>()
            .join("\n")
    );
}

/// GD5's own anchor: this lint must be RED against the shape every golden ACTUALLY had
/// before this strike's reshape, or it proves nothing (R59 `NISI FRANGAS, NIHIL PROBAS`).
/// Driven, not hand-typed: `git show <pre-strike commit>:<path>` for every golden this
/// strike's OWN recapture touched (the 70-file list below is this test's own record of
/// `git diff --stat` at recapture time) — the REAL pre-reshape bytes, not a fixture
/// reconstructed from memory. `265820aec` is the brief's own commit (this strike's
/// starting point, before any Frame-shape edit) — pinned, not "HEAD~N", so this anchor
/// keeps meaning the same thing regardless of how many commits land on top later.
#[test]
fn anchor_is_red_on_the_pre_strike_shape() {
    const PRE_STRIKE_COMMIT: &str = "265820aec";
    const TOUCHED_GOLDENS: &[&str] = &[
        "tests/diagnostics/probe_arc237_stone4_rich_errors__attempt_list_count.edn",
        "tests/diagnostics/probe_arc237_stone4_rich_errors__no_matching_clause_round_trip.edn",
        "tests/diagnostics/probe_arc237_stone4_rich_errors__no_matching_clause_tag_clean.edn",
        "tests/diagnostics/probe_arc237_stone4_rich_errors__postcondition_ensure_and_returned.edn",
        "tests/diagnostics/probe_arc237_stone4_rich_errors__postcondition_failed_tag_clean.edn",
        "tests/diagnostics/probe_arc296_n3_per_phase_namespaces__runtime_unbound_symbol.edn",
        "tests/diagnostics/probe_arc296_typed_causes__macro_expansion_arity_mismatch.edn",
        "tests/diagnostics/probe_arc296_typed_causes__macro_expansion_depth_exceeded.edn",
        "tests/diagnostics/probe_arc298_3_runtime_derive_identical__arity_mismatch.edn",
        "tests/diagnostics/probe_arc298_3_runtime_derive_identical__assertion_failed_both_some.edn",
        "tests/diagnostics/probe_arc298_3_runtime_derive_identical__assertion_failed_expected_none.edn",
        "tests/diagnostics/probe_arc298_3_runtime_derive_identical__bad_condition.edn",
        "tests/diagnostics/probe_arc298_3_runtime_derive_identical__channel_disconnected.edn",
        "tests/diagnostics/probe_arc298_3_runtime_derive_identical__declaration_in_expression_position.edn",
        "tests/diagnostics/probe_arc298_3_runtime_derive_identical__division_by_zero.edn",
        "tests/diagnostics/probe_arc298_3_runtime_derive_identical__duplicate_define.edn",
        "tests/diagnostics/probe_arc298_3_runtime_derive_identical__edn_coerce_mismatch.edn",
        "tests/diagnostics/probe_arc298_3_runtime_derive_identical__edn_coerce_mismatch_empty_path.edn",
        "tests/diagnostics/probe_arc298_3_runtime_derive_identical__effectful_in_step.edn",
        "tests/diagnostics/probe_arc298_3_runtime_derive_identical__eval_forbids_mutation_form.edn",
        "tests/diagnostics/probe_arc298_3_runtime_derive_identical__eval_verification_failed.edn",
        "tests/diagnostics/probe_arc298_3_runtime_derive_identical__macro_abort.edn",
        "tests/diagnostics/probe_arc298_3_runtime_derive_identical__macro_expansion_failed.edn",
        "tests/diagnostics/probe_arc298_3_runtime_derive_identical__malformed_form.edn",
        "tests/diagnostics/probe_arc298_3_runtime_derive_identical__no_encoding_ctx.edn",
        "tests/diagnostics/probe_arc298_3_runtime_derive_identical__no_macro_registry.edn",
        "tests/diagnostics/probe_arc298_3_runtime_derive_identical__no_matching_clause.edn",
        "tests/diagnostics/probe_arc298_3_runtime_derive_identical__no_source_loader.edn",
        "tests/diagnostics/probe_arc298_3_runtime_derive_identical__no_step_rule.edn",
        "tests/diagnostics/probe_arc298_3_runtime_derive_identical__not_callable.edn",
        "tests/diagnostics/probe_arc298_3_runtime_derive_identical__param_shadows_builtin.edn",
        "tests/diagnostics/probe_arc298_3_runtime_derive_identical__pattern_match_failed.edn",
        "tests/diagnostics/probe_arc298_3_runtime_derive_identical__postcondition_failed.edn",
        "tests/diagnostics/probe_arc298_3_runtime_derive_identical__reserved_prefix.edn",
        "tests/diagnostics/probe_arc298_3_runtime_derive_identical__rete_defn_recursive.edn",
        "tests/diagnostics/probe_arc298_3_runtime_derive_identical__sandbox_scope_leak.edn",
        "tests/diagnostics/probe_arc298_3_runtime_derive_identical__service_not_running.edn",
        "tests/diagnostics/probe_arc298_3_runtime_derive_identical__type_mismatch.edn",
        "tests/diagnostics/probe_arc298_3_runtime_derive_identical__unbound_symbol.edn",
        "tests/diagnostics/probe_arc298_3_runtime_derive_identical__unknown_field.edn",
        "tests/diagnostics/probe_arc298_3_runtime_derive_identical__unknown_function.edn",
        "tests/diagnostics/probe_arc298_3_runtime_derive_identical__user_main_missing.edn",
        "tests/diagnostics/probe_ex003_hashability_looks_inside__enum_hashmap_stderr.edn",
        "tests/diagnostics/probe_ex003_hashability_looks_inside__enum_hashset_stderr.edn",
        "tests/diagnostics/probe_ex003_hashability_looks_inside__hashmap_value_hashmap_stderr.edn",
        "tests/diagnostics/probe_ex003_hashability_looks_inside__hashmap_value_hashset_stderr.edn",
        "tests/diagnostics/probe_ex003_hashability_looks_inside__list_hashmap_stderr.edn",
        "tests/diagnostics/probe_ex003_hashability_looks_inside__list_hashset_stderr.edn",
        "tests/diagnostics/probe_ex003_hashability_looks_inside__option_hashmap_stderr.edn",
        "tests/diagnostics/probe_ex003_hashability_looks_inside__option_hashset_stderr.edn",
        "tests/diagnostics/probe_ex003_hashability_looks_inside__persistent_map_hashset_stderr.edn",
        "tests/diagnostics/probe_ex003_hashability_looks_inside__persistent_vector_hashset_stderr.edn",
        "tests/diagnostics/probe_ex003_hashability_looks_inside__record_hashmap_stderr.edn",
        "tests/diagnostics/probe_ex003_hashability_looks_inside__record_hashset_stderr.edn",
        "tests/diagnostics/probe_ex003_hashability_looks_inside__result_hashset_stderr.edn",
        "tests/diagnostics/probe_ex003_hashability_looks_inside__stream_hashset_stderr.edn",
        "tests/diagnostics/probe_ex003_hashability_looks_inside__tuple_hashmap_stderr.edn",
        "tests/diagnostics/probe_ex003_hashability_looks_inside__tuple_hashset_stderr.edn",
        "tests/diagnostics/probe_ex003_hashability_looks_inside__vector_hashmap_stderr.edn",
        "tests/diagnostics/probe_ex003_hashability_looks_inside__vector_hashset_stderr.edn",
        "tests/diagnostics/probe_ex003_lru_new_refuses_as_a_value__holographic_lru_new_zero_unhandled_stderr.edn",
        "tests/diagnostics/probe_ex003_lru_new_refuses_as_a_value__lru_new_zero_unhandled_stderr.edn",
        "tests/diagnostics/probe_ex003_silent_failure_pair__f031_stderr.edn",
        "tests/diagnostics/probe_excursus003_step4_g1_c114_retired__overflow.edn",
        "tests/diagnostics/probe_excursus003_step4_g5_thread_locus__died.edn",
        "tests/diagnostics/probe_stone_233_3_runtime_error_edn__assertion_failed.edn",
        "tests/diagnostics/probe_stone_233_3_runtime_error_edn__not_callable.edn",
        "tests/diagnostics/probe_stone_233_3_runtime_error_edn__param_shadows_builtin.edn",
        "tests/diagnostics/probe_stone_233_3_runtime_error_edn__type_mismatch.edn",
        "tests/process/probe_supervisor_select_lost__process_panics.edn",
    ];
    let root = env!("CARGO_MANIFEST_DIR");
    let mut total = 0usize;
    let mut clean: Vec<&str> = Vec::new();
    for path in TOUCHED_GOLDENS {
        let out = std::process::Command::new("git")
            .args(["-C", root, "show", &format!("{PRE_STRIKE_COMMIT}:{path}")])
            .output()
            .unwrap_or_else(|e| panic!("git show {PRE_STRIKE_COMMIT}:{path}: {e}"));
        assert!(
            out.status.success(),
            "git show {PRE_STRIKE_COMMIT}:{path} failed: {}",
            String::from_utf8_lossy(&out.stderr)
        );
        let text = String::from_utf8_lossy(&out.stdout);
        let parsed = wat_edn::parse_owned(text.trim())
            .unwrap_or_else(|e| panic!("{path} at {PRE_STRIKE_COMMIT} failed to parse: {e}"));
        let mut found = Vec::new();
        find_frame_violations(&parsed, &mut found);
        total += found.len();
        if found.is_empty() {
            clean.push(path);
        }
    }
    assert!(
        total > 0,
        "anchor is vacuous: the lint found ZERO violations across all {} pre-strike goldens \
         at {PRE_STRIKE_COMMIT} — it is not looking at the right thing",
        TOUCHED_GOLDENS.len()
    );
    assert!(
        clean.is_empty(),
        "anchor is incomplete: {} of {} pre-strike goldens had NO violation (expected every \
         one to, since all were recaptured by this strike): {:?}",
        clean.len(),
        TOUCHED_GOLDENS.len(),
        clean
    );
    eprintln!(
        "GD5 anchor: {total} violation(s) across {} pre-strike goldens at {PRE_STRIKE_COMMIT} \
         — the lint is RED on the old shape, as required.",
        TOUCHED_GOLDENS.len()
    );
}
