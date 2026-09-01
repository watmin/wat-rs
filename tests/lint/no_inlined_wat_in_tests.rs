//! THE ABSOLUTE LINT (test-infra annihilation, part C) — bans inlined-wat worlds/drivers in tests,
//! surface-agnostically (arc 278, `DESIGN-STONE-inline-wat-reader-gate.md`).
//!
//! Builder-directed ("hard bandaid pull… we witness the fire of our creation"): a test must get its
//! world from a co-located `.wat` fixture — `startup_beside(file!())` for real wat-under-test — or
//! from `startup_bare()` for an incidental world. Building the world (or a driver expression fired
//! against it) from an inlined string is the violation this lint annihilates.
//!
//! ## Detection: by the reader, not by a surface regex
//!
//! The prior gate matched the substring `startup_from_source(` — world-only (blind to inline
//! *drivers*, e.g. `let run = format!("(:wat…")`), and surface-specific (its implicit `(:` shape is
//! blind to arc 300's faithful-Clojure surface, `(wat.core/defn …)`, which carries no `(:` prefix).
//!
//! This gate instead feeds every string-literal's CONTENT to wat's own reader
//! (`wat::parser::parse_one_with_file`). If it parses to a list whose head is a `Keyword` or a
//! `Symbol`, the literal IS a wat form — surface-agnostic by construction: the one reader accepts
//! both `(:wat::core::…)` and `(wat.core/…)` during the dual-surface period, and once
//! `(:wat::core::…)` retires it becomes a parse *error*, so the gate follows the language with zero
//! maintenance.
//!
//! It scans every `tests/**/*.rs` and FAILS listing every offender — the campaign's progress meter.
//! Drive it to ZERO, chunk by chunk (group by group). Until then this is the ONE expected-red test;
//! nextest isolates it, so a SECOND red is a real regression.
//!
//! Escape hatch: a file with a genuine need for a dynamically-constructed world/driver carries a
//! `// rune:lint(no-inlined-wat) — <reason>` marker and is skipped (rare — the reason must earn it).
//! `rune:lint(<name>)` is the repo's project-custom-lint exemption form: owner `lint` = the project
//! lint suite, NOT a grimoire spell (precedent: `rune:coverage(unreachable)` in src/). excusare audits
//! the reason; a future build tool will validate `<name>` against the lint registry.

use std::path::{Path, PathBuf};

use wat::parser::parse_one_with_file;
use wat::WatAST;

fn collect_rs(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(dir) else { return };
    for e in entries.flatten() {
        let p = e.path();
        if p.is_dir() {
            collect_rs(&p, out);
        } else if p.extension().and_then(|x| x.to_str()) == Some("rs") {
            out.push(p);
        }
    }
}

/// `format!`-template placeholders (`{ns}`, `{}`, `{fire_fn}`, …) aren't wat syntax — substitute
/// each non-nested `{…}` span with a bare placeholder symbol so the template's *shape* still parses
/// as wat (a driver template built from keyword/symbol pieces reads the same whether or not its
/// interpolated slots are filled in).
fn replace_placeholders(s: &str) -> String {
    let chars: Vec<char> = s.chars().collect();
    let n = chars.len();
    let mut out = String::with_capacity(s.len());
    let mut i = 0;
    while i < n {
        if chars[i] == '{' {
            if let Some(rel_close) = chars[i..].iter().position(|&c| c == '}') {
                let close = i + rel_close;
                let inner_has_brace = chars[i + 1..close].contains(&'{');
                if !inner_has_brace {
                    out.push_str("__ph__");
                    i = close + 1;
                    continue;
                }
            }
        }
        out.push(chars[i]);
        i += 1;
    }
    out
}

/// The detector contract (arc 278 `DESIGN-STONE-inline-wat-reader-gate.md`): a string literal IS
/// inline wat iff wat's OWN reader reads it as a form whose head is a keyword or a symbol.
/// Surface-agnostic by construction — the one reader accepts `(:wat::core::…)` (rust-scheme) and
/// `(wat.core/…)` (faithful-Clojure, arc 300's ship target) identically.
fn is_inline_wat_form(literal_content: &str) -> bool {
    let src = replace_placeholders(literal_content);
    // Most literals in the corpus are ordinary Rust content (English prose, error-message
    // fixtures, …) — not wat at all. A handful of those, when force-fed to the reader, hit lexer
    // edge cases (e.g. a byte/char-boundary bug on certain non-ASCII sequences) that panic rather
    // than return a clean `ParseError`. That is a wat-reader robustness gap, not this gate's to
    // fix (arc 278 scopes this strike to `tests/lint/` only, no non-test source) — and either way a
    // string that makes the reader panic is definitely not a well-formed wat form, so treat a
    // panicking parse the same as a parse error: not a match.
    let result = std::panic::catch_unwind(|| parse_one_with_file(&src, "<inline-wat-lint>"));
    matches!(
        result,
        Ok(Ok(WatAST::List(items, _)))
            if matches!(items.first(), Some(WatAST::Keyword(..)) | Some(WatAST::Symbol(..)))
    )
}

/// A single decoded escape from a Rust string literal, plus how many source chars it consumed
/// (so the caller can advance its cursor past the escape sequence).
fn decode_escape(chars: &[char], backslash_at: usize) -> (Option<char>, usize) {
    let n = chars.len();
    let Some(&kind) = chars.get(backslash_at + 1) else {
        return (None, 1);
    };
    match kind {
        'n' => (Some('\n'), 2),
        't' => (Some('\t'), 2),
        'r' => (Some('\r'), 2),
        '\\' => (Some('\\'), 2),
        '\'' => (Some('\''), 2),
        '"' => (Some('"'), 2),
        '0' => (Some('\0'), 2),
        'x' => {
            let start = backslash_at + 2;
            let mut end = start;
            while end < n && end < start + 2 && chars[end].is_ascii_hexdigit() {
                end += 1;
            }
            let hex: String = chars[start..end].iter().collect();
            let ch = u8::from_str_radix(&hex, 16).ok().map(|v| v as char);
            (ch, end - backslash_at)
        }
        'u' => {
            if chars.get(backslash_at + 2) != Some(&'{') {
                return (None, 2);
            }
            let start = backslash_at + 3;
            let mut end = start;
            while end < n && chars[end] != '}' {
                end += 1;
            }
            let hex: String = chars[start..end].iter().collect();
            let consumed = if end < n { end + 1 - backslash_at } else { end - backslash_at };
            let ch = u32::from_str_radix(&hex, 16).ok().and_then(char::from_u32);
            (ch, consumed)
        }
        // Line continuation: `\` immediately followed by a newline elides the newline and any
        // leading whitespace on the next line — no character is emitted.
        '\n' => {
            let mut end = backslash_at + 2;
            while end < n && (chars[end] == ' ' || chars[end] == '\t' || chars[end] == '\n' || chars[end] == '\r') {
                end += 1;
            }
            (None, end - backslash_at)
        }
        other => (Some(other), 2),
    }
}

/// Extract the CONTENT of every string literal (`"…"` with escapes/line-continuation, and
/// `r#"…"#` raw strings of any hash-count) in a chunk of Rust source, skipping `//` line comments
/// and `/* … */` block comments (Rust block comments nest — this walk tracks depth). Char literals
/// (`'x'`, `'"'`, `'\''`) are recognized and skipped whole so an embedded quote character inside one
/// can't be mistaken for the start of a string; bare lifetimes (`'a`) are left untouched.
fn extract_string_literals(src: &str) -> Vec<String> {
    let chars: Vec<char> = src.chars().collect();
    let n = chars.len();
    let mut out = Vec::new();
    let mut i = 0;

    while i < n {
        let c = chars[i];

        // `//` line comment.
        if c == '/' && chars.get(i + 1) == Some(&'/') {
            while i < n && chars[i] != '\n' {
                i += 1;
            }
            continue;
        }

        // `/* … */` block comment — nests.
        if c == '/' && chars.get(i + 1) == Some(&'*') {
            i += 2;
            let mut depth = 1usize;
            while i < n && depth > 0 {
                if chars[i] == '/' && chars.get(i + 1) == Some(&'*') {
                    depth += 1;
                    i += 2;
                } else if chars[i] == '*' && chars.get(i + 1) == Some(&'/') {
                    depth -= 1;
                    i += 2;
                } else {
                    i += 1;
                }
            }
            continue;
        }

        // Raw string: `r"…"`, `r#"…"#`, `r##"…"##`, … — the head-count of `#` must match on close.
        if c == 'r' || c == 'R' {
            let mut j = i + 1;
            let mut hashes = 0usize;
            while chars.get(j) == Some(&'#') {
                hashes += 1;
                j += 1;
            }
            if chars.get(j) == Some(&'"') {
                let content_start = j + 1;
                let mut k = content_start;
                let mut closed_at = None;
                while k < n {
                    if chars[k] == '"' {
                        let mut m = k + 1;
                        let mut h = 0usize;
                        while h < hashes && chars.get(m) == Some(&'#') {
                            h += 1;
                            m += 1;
                        }
                        if h == hashes {
                            closed_at = Some((k, m));
                            break;
                        }
                    }
                    k += 1;
                }
                match closed_at {
                    Some((close_start, resume)) => {
                        out.push(chars[content_start..close_start].iter().collect());
                        i = resume;
                    }
                    None => {
                        // Unterminated raw string — no more literals to find in this file.
                        i = n;
                    }
                }
                continue;
            }
            // `r`/`R` not followed by a raw-string opener (an identifier, `r#ident`, etc.) — fall
            // through and let the char be scanned normally below.
        }

        // Char literal: `'x'`, `'\n'`, `'"'`, `'\''`, `'\u{2764}'`, … Distinguished from a bare
        // lifetime (`'a`, `'static`) by actually closing with a matching `'`.
        if c == '\'' {
            if chars.get(i + 1) == Some(&'\\') {
                let (_, consumed) = decode_escape(&chars, i + 1);
                let after = i + 1 + consumed;
                if chars.get(after) == Some(&'\'') {
                    i = after + 1;
                    continue;
                }
                // Not actually a closed char literal (e.g. a lifetime that happens to precede a
                // backslash elsewhere) — treat the quote as ordinary and move on one char.
            } else if chars.get(i + 2) == Some(&'\'') {
                i += 3;
                continue;
            }
            // Bare lifetime (`'a`, `'de`, `'static`) — not a literal; leave the identifier for
            // normal scanning.
            i += 1;
            continue;
        }

        // Regular string literal.
        if c == '"' {
            i += 1;
            let mut content = String::new();
            while i < n {
                let cc = chars[i];
                if cc == '"' {
                    i += 1;
                    break;
                }
                if cc == '\\' {
                    let (decoded, consumed) = decode_escape(&chars, i);
                    if let Some(ch) = decoded {
                        content.push(ch);
                    }
                    i += consumed;
                    continue;
                }
                content.push(cc);
                i += 1;
            }
            out.push(content);
            continue;
        }

        i += 1;
    }

    out
}

#[cfg(test)]
mod detector_tests {
    use super::*;

    #[test]
    fn rust_scheme_keyword_head_is_wat() {
        assert!(is_inline_wat_form("(:wat::core::+ 2 3)"));
    }

    #[test]
    fn faithful_clojure_symbol_head_is_wat() {
        // The load-bearing new capability: arc 300's ship surface has no `(:` prefix at all, so
        // the old substring gate was structurally blind to it.
        assert!(is_inline_wat_form("(wat.core/if true 1 2)"));
    }

    #[test]
    fn format_template_with_placeholders_is_wat() {
        assert!(is_inline_wat_form("(:{ns}::run-counts :wat::rete::{fire_fn})"));
    }

    #[test]
    fn bare_type_string_is_not_wat() {
        assert!(!is_inline_wat_form("n::Bad"));
    }

    #[test]
    fn ordinary_rust_strings_are_not_wat() {
        assert!(!is_inline_wat_form("hello world"));
        assert!(!is_inline_wat_form("expected i32, got String"));
    }

    #[test]
    fn extractor_skips_comments() {
        let src = r#"
            // (:wat::core::char) — a line comment, not a literal
            /* (:wat::core::also-not-a-literal) */
            let x = "(:wat::core::+ 1 2)";
        "#;
        let lits = extract_string_literals(src);
        assert_eq!(lits, vec!["(:wat::core::+ 1 2)".to_string()]);
    }

    #[test]
    fn extractor_handles_raw_strings_and_hash_counts() {
        let src = r###"let a = r#"(:wat::core::+ 1 2)"#; let b = r##"has a "quote" inside"##;"###;
        let lits = extract_string_literals(src);
        assert_eq!(lits, vec![
            "(:wat::core::+ 1 2)".to_string(),
            "has a \"quote\" inside".to_string(),
        ]);
    }

    #[test]
    fn extractor_handles_escapes_and_line_continuation() {
        let src = "let x = \"line one \\\n     line two \\\"quoted\\\"\";";
        let lits = extract_string_literals(src);
        assert_eq!(lits, vec!["line one line two \"quoted\"".to_string()]);
    }

    #[test]
    fn extractor_does_not_choke_on_quote_char_literals() {
        let src = r#"if c == '"' { let s = "(:wat::core::+ 1 2)"; }"#;
        let lits = extract_string_literals(src);
        assert_eq!(lits, vec!["(:wat::core::+ 1 2)".to_string()]);
    }
}

/// Which shape an offending literal took, for the report's rough breakdown.
enum OffenseShape {
    /// A `format!`-built driver (the literal still contained `{…}` placeholders pre-substitution).
    FormatDriver,
    /// The faithful-Clojure surface arc 300 ships — a `Symbol` head, e.g. `(wat.core/defn …)`.
    FaithfulSurface,
    /// Anything else that parses whole as a wat form (typically a `parse_one!`/`parse_all!` body,
    /// or a `Keyword`-headed rust-scheme literal).
    ParseBody,
}

fn classify_offense(literal_content: &str) -> OffenseShape {
    if literal_content.contains('{') && literal_content.contains('}') {
        return OffenseShape::FormatDriver;
    }
    let src = replace_placeholders(literal_content);
    if let Ok(WatAST::List(items, _)) = parse_one_with_file(&src, "<inline-wat-lint>") {
        if matches!(items.first(), Some(WatAST::Symbol(..))) {
            return OffenseShape::FaithfulSurface;
        }
    }
    OffenseShape::ParseBody
}

#[test]
fn tests_carry_no_inlined_wat() {
    let manifest = env!("CARGO_MANIFEST_DIR");
    let root = Path::new(manifest).join("tests");
    let mut files = Vec::new();
    collect_rs(&root, &mut files);
    files.sort();

    // NON-VACUITY: a walk that comes back empty asserts nothing over nothing and reports PASS, and
    // every verdict downstream inherits that silence. The floor sits well under the
    // 727 .rs file(s) this walk finds today — driven 2026-09-01, and the count comes
    // from `tests/lint/every_walking_gate_declares_non_vacuity.rs`, never from prose — so it
    // catches a walk gone blind — a moved root, a renamed directory — without rotting as the
    // tree grows.
    assert!(
        files.len() > 400,
        "the no-inlined-wat walk found only {} .rs file(s) — it is not \
         reaching the tree it claims to guard, so its green means nothing",
        files.len()
    );

    let mut violations = Vec::new();
    let mut format_driver_hits = 0usize;
    let mut faithful_surface_hits = 0usize;
    let mut parse_body_hits = 0usize;

    // Scanning force-feeds thousands of non-wat literals (English prose, error fixtures, …) to
    // wat's reader; `is_inline_wat_form` catch_unwinds the rare lexer panic on pathological
    // garbage input, but the default hook still prints each one — silence it for the scan so the
    // real assert failure (if any) isn't buried in unwind noise, then restore it.
    let prev_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(|_| {}));

    for f in &files {
        // This file names the detector in its own doc/tests — skip self.
        if f.file_name().and_then(|n| n.to_str()) == Some("no_inlined_wat_in_tests.rs") {
            continue;
        }
        let src = std::fs::read_to_string(f).expect("read test source");
        if src.contains("// rune:lint(no-inlined-wat)") {
            continue;
        }

        let mut file_is_offender = false;
        for lit in extract_string_literals(&src) {
            if is_inline_wat_form(&lit) {
                file_is_offender = true;
                match classify_offense(&lit) {
                    OffenseShape::FormatDriver => format_driver_hits += 1,
                    OffenseShape::FaithfulSurface => faithful_surface_hits += 1,
                    OffenseShape::ParseBody => parse_body_hits += 1,
                }
            }
        }

        if file_is_offender {
            let rel = f.strip_prefix(manifest).unwrap_or(f);
            violations.push(rel.display().to_string());
        }
    }

    std::panic::set_hook(prev_hook);

    assert!(
        violations.is_empty(),
        "\n\n🔥🔥🔥 INLINED-WAT IN TESTS — {} file(s) still carry a string literal that wat's own\n\
         reader parses as a form (surface-agnostic: rust-scheme `(:wat::core::…)` AND faithful\n\
         Clojure `(wat.core/…)` both count).\n\
         \n\
         THE FIX — move the wat into a co-located `.wat` fixture and drive it lint-clean via ONE of\n\
         two idioms. RUBRIC (which to reach for): docs/CONVENTIONS.md § 'Test idioms — EDN-over-stdio\n\
         vs just-eval'. In short:\n\
           • just-eval      — `call_beside_value(file!(), \":user::compute\")`: run a fixture's named entry\n\
                              fn in-process, inspect its typed Result<Value, RuntimeError>. For a\n\
                              VALUE/TYPE claim (a fn's return; a compile-time/freeze property, which\n\
                              often needs only `startup_beside(file!())`, no call).\n\
           • EDN-over-stdio — `run-hermetic` runs `:user::main` as a real process; it `println`s its\n\
                              result as EDN and the test `edn::read`s it back (lossless round-trip).\n\
                              For a PROGRAM claim (a crash/exit + reason, stdio effects, IPC fidelity,\n\
                              cross-loci behavior).\n\
         One-line: 'the PROGRAM does X' -> EDN-over-stdio ; 'this VALUE/TYPE is X' -> just-eval.\n\
         A legitimately-inline case (e.g. a parser/reader test) earns a per-site\n\
         `// rune:lint(no-inlined-wat) — <reason>` (the reason must earn it).\n\
         \n\
         Drive it to ZERO. Literal-hit breakdown so far: {} format!-driver, {} faithful-surface,\n\
         {} other parse-body. Offenders:\n\n{}\n",
        violations.len(),
        format_driver_hits,
        faithful_surface_hits,
        parse_body_hits,
        violations.join("\n"),
    );
}
