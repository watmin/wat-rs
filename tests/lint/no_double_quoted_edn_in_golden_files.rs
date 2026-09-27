//! Excursus 003 step 3b, gate G1 — no tracked `.edn` golden holds a STRING whose own content
//! is itself tagged EDN.
//!
//! `LociDiedError`'s failure variants used to carry `to_wire_edn(inner_error)` as a plain
//! `String` inside a `:message` field — a fully serialized `#wat.xxx/Tag {…}` document, quoted
//! and escaped, sitting where a real record belonged. Every consumer that wanted the structured
//! cause had to re-parse that string; a consumer that didn't got a mask. The reshape
//! (`wat/kernel/diagnostics.wat`'s `Failure`/`LociDiedError`, `src/kernel/error.rs`,
//! `src/process/died.rs`, `src/process/verbs.rs`) makes every constructor emit a real record
//! instead — this gate is the standing proof that stays true.
//!
//! **Parse, never grep.** A text search for `"#wat.` only catches an EDN document that starts
//! with a tag at byte 0 of the string; nothing stops a future producer from prefixing whitespace,
//! wrapping in a vector, or picking a differently-shaped mask. This gate parses every string leaf,
//! at every depth, and asks the EDN reader itself whether that string's OWN content is a tagged
//! document — the actual shape of the defect, not one text pattern it happened to produce.
//!
//! Scope matches the brief's own measurement exactly: `git ls-files '*.edn'`, no exclusions.
//! Anchor (excursus 003 step 3b BRIEF, measured on the pre-reshape tree, commit `77c40bb4f`):
//! RED, naming exactly 21 files.

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

/// Does `s`'s own content parse as a whole TAGGED EDN document? Scoped to `Tagged` specifically
/// (not "parses as EDN at all") because the double-quoting defect always produces a
/// `#namespace/Name {…}` document — a plain string that happens to parse as e.g. an integer or a
/// bare keyword is not this defect and must not be flagged.
fn string_is_double_quoted_edn(s: &str) -> bool {
    matches!(wat_edn::parse_owned(s), Ok(Value::Tagged(..)))
}

/// Recursively collect every String leaf under `v`, at any depth (map keys and values, list/
/// vector/set elements, inside a Tagged body), whose own content is itself tagged EDN.
fn find_double_quoted(v: &OwnedValue, out: &mut Vec<String>) {
    match v {
        Value::String(s) => {
            if string_is_double_quoted_edn(s) {
                out.push(s.to_string());
            }
        }
        Value::List(xs) | Value::Vector(xs) | Value::Set(xs) => {
            for x in xs {
                find_double_quoted(x, out);
            }
        }
        Value::Map(entries) => {
            for (k, val) in entries {
                find_double_quoted(k, out);
                find_double_quoted(val, out);
            }
        }
        Value::Tagged(_, inner) => find_double_quoted(inner, out),
        _ => {}
    }
}

#[test]
fn no_edn_golden_holds_a_string_that_parses_as_tagged_edn() {
    let root = env!("CARGO_MANIFEST_DIR");
    let paths = git_ls_files(root, "*.edn");
    // NON-VACUITY: measured 458 tracked `*.edn` files 2026-09-26; a count this low means
    // `git ls-files` itself broke, or the corpus was deleted out from under this wall.
    assert!(
        paths.len() > 100,
        "expected the tracked .edn corpus; got {}",
        paths.len()
    );

    let mut violations: Vec<String> = Vec::new();
    let mut unparseable = 0usize;
    for rel in &paths {
        let full = Path::new(root).join(rel);
        let src = std::fs::read_to_string(&full).unwrap_or_else(|e| panic!("{rel}: read: {e}"));
        let trimmed = src.trim();
        if trimmed.is_empty() {
            continue;
        }
        // A handful of goldens are TEMPLATES: an unquoted `{PLACEHOLDER}` (a line number, a pid)
        // stands in for a value substituted at comparison time — e.g. `:line  {LINE}` reads as a
        // malformed map, not as this file lying about being EDN. This gate is about a STRING
        // that is itself tagged EDN, which requires the surrounding document to parse at all;
        // a template that cannot parse raw is simply out of this gate's reach (nothing here
        // claims every tracked `.edn` is literal, parseable-as-is EDN — `every_tracked_wat_parses`
        // is the analogous wall for `.wat`, and no `.edn` counterpart exists because templating
        // is a documented shape here — measured 2026-09-26: 5 of 458).
        let parsed = match wat_edn::parse_owned(trimmed) {
            Ok(v) => v,
            Err(_) => {
                unparseable += 1;
                continue;
            }
        };
        let mut hits = Vec::new();
        find_double_quoted(&parsed, &mut hits);
        if !hits.is_empty() {
            violations.push(rel.clone());
        }
    }

    eprintln!("{unparseable} of {} tracked .edn files skipped (unquoted template placeholder)", paths.len());
    // NON-VACUITY on the skip count: if parsing collapsed wholesale (a `wat_edn` regression, a
    // corpus-wide encoding change), this gate would silently check nothing and stay green.
    assert!(
        unparseable < paths.len() / 2,
        "{unparseable} of {} tracked .edn files did not parse at all — far more than the \
         template-placeholder shape this gate expects; `wat_edn::parse_owned` may be broken, \
         which would make every skip above silent",
        paths.len()
    );

    assert!(
        violations.is_empty(),
        "{} golden .edn file(s) hold a string whose own content is itself tagged EDN — the \
         double-quoting defect excursus 003 step 3b exists to kill. Each producer should emit \
         the inner value as a real record (see `RuntimeError::to_record`, `Failure`'s \
         constructors in `src/kernel/error.rs` / `src/process/died.rs` / \
         `src/process/verbs.rs`), never `to_wire_edn` a value into a String field:\n  {}",
        violations.len(),
        violations.join("\n  ")
    );
}
