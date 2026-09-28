//! Excursus 003 step 3c, Gate A — no tracked `.edn` golden holds a FLOORED map (one
//! carrying `:message` AND `:location` — the `:wat::core::Error` floor, excursus 003
//! strike B1's `{message location}`) whose
//! `:location` is anything OTHER than a `#wat.core/Span`.
//!
//! `WatError::location()` used to return `OwnedValue`, so an impl COULD return `nil` — and
//! seven sites did (`CheckErrors`, `ReteCheckErrors`, `ResolveError::UnresolvedReferences`,
//! `StartupError`'s `Validator`/`SigmaFn`/`MainSignature` arms, `FlatMessage`). The cure
//! (`src/edn/contract.rs`) changes the trait signature itself to return
//! `crate::span::Span`, so `nil` is unrepresentable through the trait — this lint is only the
//! WITNESS that stays true, not the mechanism.
//!
//! **Parse, never grep.** Scoped to a MAP that carries all three floor keys — a collection
//! sub-value that legitimately has no `:location` field at all (e.g. a bare `#wat.core/Pos`)
//! is not this gate's business; only a map that claims the floor and then fails to honour it
//! is a violation.
//!
//! Scope matches the brief's own measurement: `git ls-files '*.edn'`, no exclusions — the
//! same corpus `no_double_quoted_edn_in_golden_files.rs` (step 3b's Gate G1) walks.
//!
//! Anchor (excursus 003 step 3c BRIEF, measured on `65272de6f`'s goldens via a sibling
//! worktree checkout): RED, naming 155 files — an exact match to the brief's own tally
//! (`git ls-files '*.edn' | xargs grep -l ":location nil" | wc -l`).

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

/// Is `v` a `#wat.core/Span {…}` tagged value?
fn is_span(v: &OwnedValue) -> bool {
    matches!(v, Value::Tagged(tag, _) if tag.namespace() == wat_edn::CORE && tag.name() == "Span")
}

/// Does `entries` carry both floor keys (`:message`, `:location`)? If so, return the
/// `:location` value. Excursus 003 strike B1: the floor is `{message location}` now —
/// `:causes` left it (F3) — so a two-key check is the CURRENT floor test, not a
/// weakening: the same maps that used to carry all three now carry these two, and a map
/// that never had the floor still has neither.
fn floor_location<'a>(entries: &'a [(OwnedValue, OwnedValue)]) -> Option<&'a OwnedValue> {
    let msg_kw = OwnedValue::Keyword(wat_edn::Keyword::new("message"));
    let loc_kw = OwnedValue::Keyword(wat_edn::Keyword::new("location"));
    let has = |k: &OwnedValue| entries.iter().any(|(ek, _)| ek == k);
    if has(&msg_kw) && has(&loc_kw) {
        entries.iter().find(|(k, _)| k == &loc_kw).map(|(_, v)| v)
    } else {
        None
    }
}

/// Recursively collect every violation under `v`: a map (bare, or a `Tagged` value's body)
/// that carries the floor (`:message`/`:location`) whose `:location` is not a
/// `#wat.core/Span`. Returns `(tag-or-"<untagged>", location-rendered)` per violation. Each
/// map is checked EXACTLY ONCE — the `Tagged` arm checks its own body map directly rather than
/// re-dispatching into the generic `Map` arm, so a tagged floor map is never reported twice.
fn find_nil_locations(v: &OwnedValue, out: &mut Vec<(String, String)>) {
    match v {
        Value::Map(entries) => {
            if let Some(loc_val) = floor_location(entries) {
                if !is_span(loc_val) {
                    out.push(("<untagged floor map>".to_string(), wat_edn::write(loc_val)));
                }
            }
            for (k, val) in entries {
                find_nil_locations(k, out);
                find_nil_locations(val, out);
            }
        }
        Value::List(xs) | Value::Vector(xs) | Value::Set(xs) => {
            for x in xs {
                find_nil_locations(x, out);
            }
        }
        Value::Tagged(tag, inner) => {
            if let Value::Map(entries) = &**inner {
                if let Some(loc_val) = floor_location(entries) {
                    if !is_span(loc_val) {
                        // No leading `#` in this label: a `format!` literal opening with an
                        // EDN sigil trips `no_inlined_edn`'s heuristic even though this is a
                        // diagnostic label, not a compared EDN literal (see that lint's own
                        // "format-scaffold opener" section for why `{}/{}`  alone is fine but
                        // `#{}/{}` is not).
                        out.push((
                            format!("{}/{}", tag.namespace(), tag.name()),
                            wat_edn::write(loc_val),
                        ));
                    }
                }
                for (k, val) in entries {
                    find_nil_locations(k, out);
                    find_nil_locations(val, out);
                }
            } else {
                find_nil_locations(inner, out);
            }
        }
        _ => {}
    }
}

#[test]
fn no_edn_golden_holds_a_floored_map_with_a_non_span_location() {
    let root = env!("CARGO_MANIFEST_DIR");
    let paths = git_ls_files(root, "*.edn");
    // NON-VACUITY: mirrors `no_double_quoted_edn_in_golden_files.rs`'s own guard — a count
    // this low means `git ls-files` itself broke, or the corpus was deleted out from under
    // this wall.
    assert!(
        paths.len() > 100,
        "expected the tracked .edn corpus; got {}",
        paths.len()
    );

    let mut violations: Vec<(String, String, String)> = Vec::new();
    let mut unparseable = 0usize;
    for rel in &paths {
        let full = Path::new(root).join(rel);
        let src = std::fs::read_to_string(&full).unwrap_or_else(|e| panic!("{rel}: read: {e}"));
        let trimmed = src.trim();
        if trimmed.is_empty() {
            continue;
        }
        // Template goldens (an unquoted `{PLACEHOLDER}`) don't parse raw — see
        // `no_double_quoted_edn_in_golden_files.rs`'s identical skip for the same reason.
        let parsed = match wat_edn::parse_owned(trimmed) {
            Ok(v) => v,
            Err(_) => {
                unparseable += 1;
                continue;
            }
        };
        let mut hits = Vec::new();
        find_nil_locations(&parsed, &mut hits);
        for (tag, loc) in hits {
            violations.push((rel.clone(), tag, loc));
        }
    }

    eprintln!("{unparseable} of {} tracked .edn files skipped (unquoted template placeholder)", paths.len());
    assert!(
        unparseable < paths.len() / 2,
        "{unparseable} of {} tracked .edn files did not parse at all — far more than the \
         template-placeholder shape this gate expects; `wat_edn::parse_owned` may be broken, \
         which would make every skip above silent",
        paths.len()
    );

    assert!(
        violations.is_empty(),
        "{} golden .edn file(s) hold a FLOORED map (:message/:location) whose \
         :location is not a #wat.core/Span — the `nil`-location defect excursus 003 step 3c \
         exists to make unrepresentable. Each producer's `WatError::location()` must return a \
         real `crate::span::Span` (build one with `crate::edn::contract::location_from_span`, \
         an aggregate's first item's `.location()`, or `crate::rust_caller_span!()` for a \
         genuinely flat failure):\n  {}",
        violations.len(),
        violations
            .iter()
            .map(|(f, tag, loc)| format!("{f}  ({tag}  :location {loc})"))
            .collect::<Vec<_>>()
            .join("\n  ")
    );
}
