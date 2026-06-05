//! CALLER-CONTRACT GATE — `macro_eval_pre_validated` sanctioned call site.
//!
//! # What this gate makes impossible
//!
//! `macro_eval_pre_validated` (src/macros/eval.rs) is a short-circuit evaluator that
//! skips the `validate_pure_total` purity walk because its caller guarantees the form
//! was already validated at definition time (the hoist — arc 249 stone O). Its contract
//! is documented at src/macros/eval.rs ~97-99:
//!
//!   "INVARIANT: callers must guarantee the form was validated by `validate_pure_total`
//!    before calling this function. The only sanctioned call site is `expand_program_body`."
//!
//! A future caller that invokes `macro_eval_pre_validated` from an unsanctioned site
//! (outside `src/macros/expand.rs`) would bypass the purity gate — silently admitting
//! effectful forms into the macro evaluator. This gate FAILS LOUD the moment such a
//! caller is added, forcing the author to either route through `macro_eval` (the safe
//! path that runs `validate_pure_total`) or justify-and-allowlist the new site here.
//!
//! # The allowlist
//!
//! Each entry is a `src/`-relative file path permitted to call (or define)
//! `macro_eval_pre_validated(`. Adding an entry is a deliberate, reviewed act.
//!
//! # Precedent
//!
//! Mirrors the mechanics of `tests/probe_hygiene_scopes_reader_gate.rs`
//! (Stone 249.5g) — same pattern: token + allowlist + anti-rot guard.

use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

/// Files permitted to contain `macro_eval_pre_validated(` (with the opening paren).
///
/// - `macros/eval.rs` — the definition site and the one internal call inside
///   `macro_eval` itself (which IS the purity-validating wrapper).
/// - `macros/expand.rs` — `expand_program_body`, the only sanctioned external call
///   site; it calls the pre-validated path because the body was already validated
///   by `validate_macro_definition` at definition time (arc 249 stone O / hoist).
const SANCTIONED_CALLERS: &[(&str, &str)] = &[
    (
        "macros/eval.rs",
        "definition site (pub(super) fn macro_eval_pre_validated) + internal call \
         inside macro_eval (the purity-validating public wrapper); both are within \
         the same module as the gate.",
    ),
    (
        "macros/expand.rs",
        "expand_program_body — the ONLY sanctioned external call site; the form \
         passed here was validated ONCE at definition time by validate_macro_definition \
         (arc 249 stone O hoist); re-running validate_pure_total on every invocation \
         would be redundant. Any OTHER site in expand.rs outside expand_program_body \
         must be investigated.",
    ),
];

fn collect_rs(dir: &Path, out: &mut Vec<PathBuf>) {
    for entry in fs::read_dir(dir).expect("src/ must be readable") {
        let p = entry.expect("dir entry").path();
        if p.is_dir() {
            collect_rs(&p, out);
        } else if p.extension().map_or(false, |e| e == "rs") {
            out.push(p);
        }
    }
}

/// Scanning token: the function name with the opening paren distinguishes calls
/// and the definition from bare doc-comment mentions (which omit the paren).
const TOKEN: &str = "macro_eval_pre_validated(";

#[test]
fn only_sanctioned_files_call_macro_eval_pre_validated() {
    let src = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut files = Vec::new();
    collect_rs(&src, &mut files);

    let allowed: HashSet<&str> = SANCTIONED_CALLERS.iter().map(|(f, _)| *f).collect();

    let mut violations = Vec::new();
    for f in &files {
        let rel = f
            .strip_prefix(&src)
            .expect("under src/")
            .to_string_lossy()
            .replace('\\', "/");
        let body = fs::read_to_string(f).expect("readable .rs");
        if body.contains(TOKEN) && !allowed.contains(rel.as_str()) {
            violations.push(rel);
        }
    }

    assert!(
        violations.is_empty(),
        "CALLER-CONTRACT VIOLATION — these src/ files call or define \
         `macro_eval_pre_validated(` but are NOT sanctioned callers:\n  {:?}\n\n\
         `macro_eval_pre_validated` skips `validate_pure_total` under the invariant \
         that the form was already validated at definition time (arc 249 stone O). \
         An unsanctioned caller breaks the purity gate — effectful forms could reach \
         the macro evaluator silently.\n\n\
         FIX: route the call through `macro_eval` (the purity-validating public wrapper). \
         If this is a GENUINELY new sanctioned consumer, add it to SANCTIONED_CALLERS in \
         this file WITH its reason — and verify that the form IS validated before the call.",
        violations
    );
}

/// Guards the gate itself against rot: every allowlisted path must still exist and
/// still actually contain the token. A stale allowlist entry (file moved, or the
/// call renamed) is a silent weakening of the gate — fail loud so it stays honest.
#[test]
fn allowlist_has_no_stale_entries() {
    let src = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut stale = Vec::new();
    for (rel, _reason) in SANCTIONED_CALLERS {
        let p = src.join(rel);
        let contains = fs::read_to_string(&p)
            .map(|b| b.contains(TOKEN))
            .unwrap_or(false);
        if !contains {
            stale.push(*rel);
        }
    }
    assert!(
        stale.is_empty(),
        "stale SANCTIONED_CALLERS entries (file missing, or no longer contains \
         `macro_eval_pre_validated(`): {:?}. Remove them — an allowlist that \
         over-permits is a weakened gate.",
        stale
    );
}
