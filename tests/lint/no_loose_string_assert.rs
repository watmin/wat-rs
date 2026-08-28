//! THE LOOSE-ASSERT LINT — bans `contains`/`starts_with`/`ends_with` string checks inside
//! assertions where an exact `assert_eq!` belongs.
//!
//! Builder-directed ("we just found the pattern — remove all examples of bad behavior"): a test that
//! asserts a DETERMINISTIC value with `assert!(s.contains("…"))` / `.starts_with` / `.ends_with`
//! (including the negated `assert!(!s.contains("…"))`) is a LOOSE check — it passes on reordered
//! fields, malformed maps, and appended garbage (296 R5's own critique). The exact form is
//! `assert_eq!(s, "<the whole thing>")`, captured not guessed. This lint makes every offender scream.
//!
//! It scans `src/`, `tests/`, and `crates/` and FAILS listing every offending `file:line` — the
//! campaign's progress meter. Drive it to ZERO: TIGHTEN the real offenders (deterministic value →
//! byte-identical `assert_eq!`), or EXEMPT the legitimately-loose ones (a value that varies per run —
//! a path/pid/hash/timestamp — a property over a variable set, or a targeted absence on a large
//! output) with a per-site `// rune:lint(loose-assert) — <reason>` marker. Until then this is an
//! expected-red test; nextest isolates it, so a SECOND red is a real regression.
//!
//! Detection is assertion-scoped: a loose string-match with a string-literal argument, inside a
//! statement that also contains an `assert` macro. Control-flow (`if x.starts_with(…)`) and
//! collection membership (`vec.contains(&item)` — arg is not a string literal) never match.
//!
//! `rune:lint(<name>)` is the repo's project-custom-lint exemption form (owner `lint` = the project
//! lint suite, NOT a grimoire spell); excusare audits the reason so "legitimate" stays honest.

use std::path::{Path, PathBuf};

fn collect_rs(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(dir) else { return };
    for e in entries.flatten() {
        let p = e.path();
        if p.is_dir() {
            // skip build artifacts / harness state
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

/// Does this line carry a loose string-match `.<method>(<string-literal>)` — where the argument is a
/// string literal (`"…"`, `r"…"`, `r#"…"#`, `format!(…)`), optionally behind `&`? A collection
/// membership check `.contains(&item)` (arg is a value ref, not a literal) does NOT match.
fn has_loose_string_match(line: &str) -> bool {
    for method in [".contains(", ".starts_with(", ".ends_with("] {
        let mut from = 0;
        while let Some(i) = line[from..].find(method) {
            let after = &line[from + i + method.len()..];
            let arg = after.trim_start().trim_start_matches('&').trim_start();
            if arg.starts_with('"')
                || arg.starts_with("r\"")
                || arg.starts_with("r#")
                || arg.starts_with("format!")
            {
                return true;
            }
            from += i + method.len();
        }
    }
    false
}

fn is_assert_opener(line: &str) -> bool {
    line.contains("assert!")
        || line.contains("assert_eq!")
        || line.contains("assert_ne!")
        || line.contains("debug_assert")
}

#[test]
fn tests_carry_no_loose_string_assert() {
    let manifest = env!("CARGO_MANIFEST_DIR");
    let mut files = Vec::new();
    for sub in ["src", "tests", "crates"] {
        collect_rs(&Path::new(manifest).join(sub), &mut files);
    }
    files.sort();

    let mut violations = Vec::new();
    for f in &files {
        // This file names the forbidden patterns in its own detector — skip self.
        if f.file_name().and_then(|n| n.to_str()) == Some("no_loose_string_assert.rs") {
            continue;
        }
        let Ok(src) = std::fs::read_to_string(f) else { continue };
        let rel = f.strip_prefix(manifest).unwrap_or(f).display().to_string();

        // Statement-scoped scan: a loose string-match counts only inside a statement that also
        // holds an `assert` macro. Reset at statement/block boundaries (`;` `{` `}`).
        let mut stmt_has_assert = false;
        let mut stmt_has_rune = false;
        for (idx, line) in src.lines().enumerate() {
            if is_assert_opener(line) {
                stmt_has_assert = true;
            }
            if line.contains("// rune:lint(loose-assert)") {
                stmt_has_rune = true;
            }
            if stmt_has_assert && !stmt_has_rune && has_loose_string_match(line) {
                violations.push(format!("{}:{}", rel, idx + 1));
            }
            // ⛔ A COMMENT IS NOT A STATEMENT BOUNDARY. Without this guard a rune whose prose
            // happened to end a line with `;` — e.g. "…embedding a Span path;" — silently reset
            // `stmt_has_rune` before reaching its own assert, and the site was reported as an
            // offender while carrying a perfectly good exemption two lines above it. Found
            // 2026-08-28 by writing exactly that comment. The failure direction was safe (a false
            // POSITIVE, never a missed violation), which is why it survived: it costs a confusing
            // red rather than a silent pass.
            let t = line.trim_end();
            if t.trim_start().starts_with("//") {
                continue;
            }
            if t.ends_with(';') || t.ends_with('{') || t.ends_with('}') {
                stmt_has_assert = false;
                stmt_has_rune = false;
            }
        }
    }

    assert!(
        violations.is_empty(),
        "\n\n🔥🔥🔥 LOOSE STRING ASSERTIONS — {} site(s) assert a value with contains/starts_with/\n\
         ends_with where an exact `assert_eq!` belongs. A loose check passes on reordered fields,\n\
         malformed maps, and appended garbage.\n\
         \n\
         THE FIX (RUBRIC: docs/CONVENTIONS.md § 'Test idioms' -> 'The .edn golden'): a deterministic\n\
         STRUCTURED value goes in a co-located `<probe>__<label>.edn` golden, compared via\n\
         `wat::assert_edn_eq!(actual, include_str!(\"...edn\"))` (parses both sides, structure-exact) —\n\
         capture the whole value, never guess. A scalar -> byte-identical `assert_eq!`. EXEMPT a\n\
         legitimately-loose one (a value that varies per run: path/pid/hash/timestamp, or a targeted\n\
         absence over a large output) with a per-site `// rune:lint(loose-assert) — <reason>`.\n\
         \n\
         Drive it to ZERO. Offenders:\n\n{}\n",
        violations.len(),
        violations.join("\n"),
    );
}
