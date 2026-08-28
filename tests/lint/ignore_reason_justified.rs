//! THE IGNORE-REASON-JUSTIFIED LINT — a structural wall against a promise wearing a condition's
//! clothes.
//!
//! Arc 255 Stone P3 (2026-08-28) re-diagnosed seven `#[ignore]`s that all carried the identical
//! copy-pasted string *"unlock when we circle back to arc 255"*. Every one of the seven had
//! already fired — the arc circled back weeks earlier — and nothing noticed, because a promise
//! ("we will return to this") reads exactly like a condition ("this is blocked on X") until
//! someone re-checks it by hand. `probe_arc255_reflection_parity.rs:94-121` records that TWO
//! SIBLING tests were deleted in August for exactly this staleness while the SURVIVORS kept the
//! identical string, unrechecked, for three weeks.
//!
//! **The class, not the instance:** a copy-pasted unlock condition outlives the thing it was
//! waiting for. This lint makes the shape structural: an `#[ignore = "…"]` reason may not contain
//! a bare *"circle back / come back to / when we get to arc N"* promise — at minimum, the ONE
//! thing a reason must never be is a promise to revisit rather than a checkable fact (a line, a
//! design ruling, a named stone, a measured count) a reader can verify without re-deriving it.
//!
//! ## Scope: real `#[ignore]` attributes only, `tests/` tree
//!
//! Every real `#[ignore]`/`#[ignore = "…"]` ATTRIBUTE in the corpus lives under `tests/` (verified
//! this session — none in `src/`, `benches/`, or `crates/`; every apparent hit there is prose in a
//! doc comment naming the attribute, e.g. `` `#[ignore]`d `` in a `///` line). A **naive substring
//! grep for `#[ignore` returns 68+** because it also matches that prose AND matches the phrase
//! embedded inside `tests/lint/no_bare_is_err.rs`'s own `assert!` message string (a multi-line
//! Rust string literal whose continuation lines, after trimming, also start with `#[ignore]d…`).
//! Neither is a real attribute. This lint tracks Rust `"…"` string-literal and `//`-comment state
//! across the whole file (mirroring `no_bare_is_err.rs`'s paren-balanced `assert!` scanner) so
//! only a genuine `#[ignore …]` attribute token — never a mention of one — is counted.
//!
//! ## THE FROZEN ALLOWLIST — out-of-scope residue, not an exemption from failing
//!
//! Stone P3's blast radius is the SEVEN arc-255 ignores; it does not touch, and must not widen
//! this lint to fix, the other real ignores in the tree. **Anchored count this session: 11 real
//! `#[ignore]` attributes total** — 5 re-pointed/surviving arc-255 ones (2 of the original 7 were
//! UN-IGNORED this stone: `metadata_of_answers_for_a_rust_builtin`,
//! `metadata_of_emits_plain_values_and_enums_not_holon_ast`) **and 6 belonging to other arcs**
//! (260, 259, 278, the 300 equality matrix). ⚠ The Stone P3 brief's own table claimed *"14 (7
//! arc-255 + 7 other)"* — re-measured this session and found to be **13 (7 + 6)** pre-edit; the
//! brief's "other seven" over-counted by one. None of the six other-arc reasons currently carries
//! the banned phrase (each names its own arc/stone), so the allowlist is not presently load-bearing
//! for any of them — it exists so the residue is **visible and countable**, exactly as
//! `tests/lint/no_bare_is_err.rs`'s allowlist does, and so a future widening of this lint's check
//! does not silently start failing sites this stone deliberately left untouched.
//!
//! **Identity = file + test fn name, NEVER a line number** (a line number moves under any
//! unrelated edit in the file).
//!
//! `rune:lint(<name>)` is the repo's project-custom-lint exemption form (owner `lint` = the
//! project lint suite, NOT a grimoire spell).

use std::path::{Path, PathBuf};

/// The non-arc-255 real `#[ignore]` sites, frozen by file + enclosing test fn name — out of
/// Stone P3's scope (arc 255's own seven only). If a future stone re-diagnoses one of these and
/// its reason no longer needs freezing, remove it from this list rather than leaving it stale.
const FROZEN_ALLOWLIST: &[(&str, &str)] = &[
    (
        "tests/macros/probe_arc260_keyword_args.rs",
        "user_fn_keyword_args_reorder_to_positional",
    ),
    (
        "tests/kernel/probe_arc259_started_at_boot.rs",
        "started_at_is_the_primed_boot_not_the_seam",
    ),
    (
        "tests/kernel/probe_arc259_started_at_boot.rs",
        "peer_started_at_is_after_started_at",
    ),
    (
        "tests/services/probe_arc278_self_scheduling.rs",
        "self_tick_fires_rearms_and_reactor_serves_thread",
    ),
    (
        "tests/services/probe_arc278_self_scheduling.rs",
        "self_tick_fires_rearms_and_reactor_serves_process",
    ),
    (
        "tests/value/clj_expr_parity.rs",
        "wat_expr_matches_clj_oracle",
    ),
];

fn collect_rs(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(dir) else { return };
    for e in entries.flatten() {
        let p = e.path();
        if p.is_dir() {
            let name = p.file_name().and_then(|n| n.to_str()).unwrap_or("");
            if name == "target" || name == ".claude" {
                continue;
            }
            collect_rs(&p, out);
        } else if p.extension().and_then(|x| x.to_str()) == Some("rs") {
            out.push(p);
        }
    }
}

/// One real `#[ignore]` attribute occurrence: its 1-indexed line and its reason text (empty for a
/// bare `#[ignore]`).
struct IgnoreHit {
    line_no: usize,
    reason: String,
}

/// Walk `src` character-by-character, tracking `"…"` string-literal and `//…`/`///…`/`//!…`
/// line-comment state, and return every `#[ignore` token found OUTSIDE both — i.e. every real
/// attribute, never a mention of one in prose or in an assembled message string.
fn find_ignore_attrs(src: &str) -> Vec<IgnoreHit> {
    // Char-indexed (byte offset, char) pairs, plus a sentinel EOF marker — every slice below
    // uses a byte offset that came directly from `char_indices`, so it always lands on a real
    // char boundary (UTF-8 content, e.g. em-dashes in the reason prose, is common in this tree).
    let cis: Vec<(usize, char)> = src.char_indices().collect();
    let eof = src.len();
    let byte_at = |k: usize| -> usize { cis.get(k).map(|&(b, _)| b).unwrap_or(eof) };

    let mut out = Vec::new();
    let mut k = 0usize;
    let mut in_line_comment = false;
    while k < cis.len() {
        let (bi, c) = cis[k];
        if c == '\n' {
            in_line_comment = false;
            k += 1;
            continue;
        }
        if in_line_comment {
            k += 1;
            continue;
        }
        if c == '/' && cis.get(k + 1).map(|&(_, n)| n) == Some('/') {
            in_line_comment = true;
            k += 2;
            continue;
        }
        if c == '"' {
            // Skip the whole string literal, honoring `\`-escapes (including a
            // `\`-newline continuation, which does not end the literal).
            k += 1;
            while k < cis.len() && cis[k].1 != '"' {
                k += if cis[k].1 == '\\' { 2 } else { 1 };
            }
            k += 1; // consume closing quote
            continue;
        }
        if src[bi..].starts_with("#[ignore") {
            let line_no = src[..bi].matches('\n').count() + 1;
            // Find the matching `]` for this attribute (balance `[`/`]`; the reason
            // string itself is skipped via the same string-literal logic, so a `]`
            // inside the reason text never closes the attribute early).
            let mut m = k;
            let mut depth = 0i32;
            let mut in_str = false;
            while m < cis.len() {
                let cm = cis[m].1;
                if in_str {
                    if cm == '\\' {
                        m += 2;
                        continue;
                    }
                    if cm == '"' {
                        in_str = false;
                    }
                    m += 1;
                    continue;
                }
                match cm {
                    '"' => in_str = true,
                    '[' => depth += 1,
                    ']' => {
                        depth -= 1;
                        if depth == 0 {
                            m += 1;
                            break;
                        }
                    }
                    _ => {}
                }
                m += 1;
            }
            let attr_text = &src[bi..byte_at(m)];
            let reason = extract_reason(attr_text);
            out.push(IgnoreHit { line_no, reason });
            k = m.max(k + 1);
            continue;
        }
        k += 1;
    }
    out
}

/// Pull the `"…"` reason payload out of a `#[ignore = "…"]` attribute's raw text; a bare
/// `#[ignore]` (no `=`) yields an empty reason.
fn extract_reason(attr_text: &str) -> String {
    let Some(start) = attr_text.find('"') else { return String::new() };
    let rest = &attr_text[start + 1..];
    let mut out = String::new();
    let mut chars = rest.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '\\' {
            // A `\`-newline continuation collapses to nothing in the assembled Rust
            // string (and any leading whitespace on the next line with it); every
            // other escape is passed through as its escaped char for search purposes.
            if let Some(&next) = chars.peek() {
                if next == '\n' {
                    chars.next();
                    while matches!(chars.peek(), Some(c) if c.is_whitespace() && *c != '\n') {
                        chars.next();
                    }
                    continue;
                }
            }
            if let Some(next) = chars.next() {
                out.push(next);
            }
            continue;
        }
        if c == '"' {
            break;
        }
        out.push(c);
    }
    out
}

/// The banned promise-shaped phrases (case-insensitive substring match) — a reason may name a
/// checkable fact, never a promise to revisit. `[circle back N]` matches "arc 255" or any digits
/// after "when we get to arc"/"come back to arc" so the ban is not 255-specific.
fn banned_phrase(reason_lower: &str) -> Option<&'static str> {
    const PHRASES: &[&str] = &["circle back", "come back to arc", "when we get to arc"];
    PHRASES.iter().copied().find(|p| reason_lower.contains(p))
}

/// Find the `fn <name>` an `#[ignore]` attribute at `line_no` (1-indexed) NAMES — the NEXT `fn`
/// declaration at or after `line_no`. Unlike `no_bare_is_err.rs`'s identical-looking helper
/// (which locates the fn enclosing a statement INSIDE a body, so it scans backward), an attribute
/// always precedes the item it decorates — possibly past other attributes (`#[test]`, doc
/// comments) — so this scans FORWARD for the next declaration instead.
fn enclosing_fn_name(src: &str, line_no: usize) -> Option<String> {
    for (idx, line) in src.lines().enumerate() {
        if idx + 1 < line_no {
            continue;
        }
        let trimmed = line.trim_start();
        let rest = trimmed.strip_prefix("fn ").or_else(|| trimmed.strip_prefix("pub fn "));
        if let Some(rest) = rest {
            let name: String = rest.chars().take_while(|c| c.is_alphanumeric() || *c == '_').collect();
            if !name.is_empty() {
                return Some(name);
            }
        }
    }
    None
}

#[test]
fn ignore_reasons_are_justified() {
    let manifest = env!("CARGO_MANIFEST_DIR");
    let mut files = Vec::new();
    collect_rs(&Path::new(manifest).join("tests"), &mut files);
    files.sort();

    let mut violations = Vec::new();
    let mut total_real_ignores = 0usize;
    for f in &files {
        let Ok(src) = std::fs::read_to_string(f) else { continue };
        let rel = f.strip_prefix(manifest).unwrap_or(f).display().to_string();

        for hit in find_ignore_attrs(&src) {
            total_real_ignores += 1;
            let reason_lower = hit.reason.to_lowercase();
            let Some(phrase) = banned_phrase(&reason_lower) else { continue };
            let fn_name = enclosing_fn_name(&src, hit.line_no).unwrap_or_default();
            if FROZEN_ALLOWLIST.iter().any(|(af, afn)| *af == rel && *afn == fn_name) {
                continue;
            }
            violations.push(format!(
                "{rel}:{}  (in fn {fn_name}) — reason contains {phrase:?}",
                hit.line_no
            ));
        }
    }

    // Sanity floor: the lint must actually be seeing real attributes, not silently
    // scanning nothing (NISI FRANGAS, NIHIL PROBAS — a wall that finds zero of
    // everything proves nothing about the population it claims to police).
    assert!(
        total_real_ignores >= FROZEN_ALLOWLIST.len(),
        "ignore_reason_justified scanned {total_real_ignores} real #[ignore] attributes, fewer \
         than the {} frozen allowlist entries alone — the scanner is broken, not the corpus.",
        FROZEN_ALLOWLIST.len()
    );

    let allowlist_block = FROZEN_ALLOWLIST
        .iter()
        .map(|(f, n)| format!("  {f}  fn {n}"))
        .collect::<Vec<_>>()
        .join("\n");

    assert!(
        violations.is_empty(),
        "\n\n🔥🔥🔥 UNJUSTIFIED IGNORE REASON — {} site(s) carry an `#[ignore = \"…\"]` reason\n\
         that is a PROMISE to revisit, not a checkable CONDITION. Arc 255 Stone P3 found seven\n\
         `#[ignore]`s that all carried the identical copy-pasted \"unlock when we circle back to\n\
         arc 255\" — every one had already fired weeks earlier, and nothing noticed because a\n\
         promise reads exactly like a condition until someone re-checks it by hand.\n\
         \n\
         THE FIX: name something a reader can CHECK without re-deriving it — a source line, a\n\
         design-doc ruling, a named stone, a measured test count. \"We will get back to this\" is\n\
         never that; \"blocked on <file>:<line>, sized at <N> tests\" is.\n\
         \n\
         FROZEN ALLOWLIST (identity = file + test fn name, NEVER line number — {} site(s), out of\n\
         Stone P3's scope, belonging to other arcs; none currently trips this check, frozen so the\n\
         residue stays visible and countable):\n{}\n\
         \n\
         Offenders:\n\n{}\n",
        violations.len(),
        FROZEN_ALLOWLIST.len(),
        allowlist_block,
        violations.join("\n"),
    );
}

#[cfg(test)]
mod detector_tests {
    use super::*;

    #[test]
    fn finds_a_real_reasoned_attribute() {
        let src = "// fixture\n#[test]\n#[ignore = \"blocked on X\"]\nfn f() {}\n";
        let hits = find_ignore_attrs(src);
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].reason, "blocked on X");
    }

    #[test]
    fn finds_a_bare_attribute() {
        let src = "// fixture\n#[test]\n#[ignore]\nfn f() {}\n";
        let hits = find_ignore_attrs(src);
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].reason, "");
    }

    #[test]
    fn ignores_prose_mentioning_the_attribute() {
        let src = "/// Marked `#[ignore]` for a reason.\n//! Also `#[ignore]`'d elsewhere.\nfn f() {}\n";
        assert!(find_ignore_attrs(src).is_empty());
    }

    #[test]
    fn ignores_the_phrase_inside_a_string_literal() {
        let src = "assert!(x, \"blocked #[ignore]d here\");\n";
        assert!(find_ignore_attrs(src).is_empty());
    }

    #[test]
    fn handles_backslash_newline_continuation_in_the_reason() {
        let src = "// fixture\n#[ignore = \"first half \\\n            second half\"]\nfn f() {}\n";
        let hits = find_ignore_attrs(src);
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].reason, "first half second half");
    }

    #[test]
    fn detects_the_banned_promise_phrases() {
        assert_eq!(banned_phrase("unlock when we circle back to arc 255"), Some("circle back"));
        assert_eq!(banned_phrase("come back to arc 118 later"), Some("come back to arc"));
        assert_eq!(banned_phrase("when we get to arc 300"), Some("when we get to arc"));
        assert_eq!(banned_phrase("blocked on walk.rs:268, sized at 2539 tests"), None);
    }
}
