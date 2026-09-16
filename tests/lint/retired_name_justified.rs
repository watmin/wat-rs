//! THE RETIRED-NAME LINT — a wat name in a Rust string must be a name a user can type.
//!
//! Arc 278 0z (`70fe856d`) dropped the `'` from 24 IPC names (`send'`→`send`, `recv'`→`recv`,
//! `accept'`→`accept`, `connect'`→`connect`, `poll'`→`poll`, `select'`→`select`, `Thread'`→`Thread`,
//! …the full list is `wat-scripts/fixes/reclaim-ipc-prime-names.wat`) across 302 files. It reached
//! `.wat` keywords and Rust *symbol paths* — it did **not** reach names embedded in Rust **message
//! strings**. So the substrate still tells users about verbs that do not exist: a `CheckError` or
//! `RuntimeError` naming `send'`/`recv'`/`Thread'`/… when the user can only ever type the unprimed
//! form. R29 says the ruin educates; an un-caught site here educates toward a retired vocabulary.
//!
//! ## Scope: `'` inside a Rust STRING LITERAL only
//!
//! Rust identifiers cannot contain `'` (only a *leading* `'` as a lifetime, e.g. `'a`), so any
//! `word'` shape in a `.rs` file is either inside a string literal, or a comment. This lint scans
//! only inside actual `"…"` string literals — comments (full-line, via a leading-`//` skip, and
//! trailing, since a `//` comment is not inside the string) are structurally out of scope, and a
//! code-side lifetime (`'a`, `&'static`) never sits inside a string literal so it never enters the
//! scan.
//!
//! The scan is **stateful across the whole file**, not per physical line: a `\`-continued
//! multi-line message string (common in this codebase's diagnostics — `"foo \` on one line,
//! `bar".into()` on the next) is a SINGLE string literal, and a hit can land on any of its
//! continuation lines, not just the one bearing the opening `"`. A naive per-line reset of the
//! quote-tracking state would silently miss those (proven: `check.rs:8474`'s `(readln' <cap>)` sits
//! on a continuation line with no opening `"` of its own — a per-line scanner never sees it). Each
//! hit also records which physical line its ENCLOSING string literal closes on, since a `//
//! rune:lint(...)` can only be placed outside the string — on the hit's own line if the literal is
//! single-line, or anywhere from the hit's line through the string's closing line otherwise
//! (`multi_line_continuation_hit_is_found_and_exemptible_on_the_closing_line`).
//!
//! ## The predicate — three false-positive classes killed
//!
//! Base shape: **a kebab/alpha identifier, then `'`, where `'` is NOT followed by an ASCII
//! letter.**
//!
//! 1. **English possessive / contraction** (`don't`, `doesn't`, `Token's`, `the wall's`) — the `'`
//!    IS followed by a letter → excluded by the base shape's lookahead.
//! 2. **Comment-only reference** (`"connect' OUTCOME WALL"` inside a `//`-comment naming an arc by
//!    its historical name) — excluded because comments are never inside a string literal (full-line
//!    comments are additionally skipped up front, matching `unused_span_justified.rs`).
//! 3. **Single-quoted prose** (`expects 'edn' or 'json'`, `is ':wat::core::nil' (arc 153)`) — the
//!    dominant residue once (1) and (2) are excluded: a *closing* prose quote, not a prime suffix.
//!    Killed with a real predicate, not runes: scan each string literal left-to-right pairing
//!    `'`s. An apostrophe immediately preceded by a word/kebab char (`[A-Za-z0-9-]`) is a
//!    **candidate** (TRAILING) — unless a still-open **LEADING** opener (an apostrophe immediately
//!    preceded by a non-word char, e.g. a space or `:`) is already pending on this literal, in
//!    which case this apostrophe is that opener's CLOSER (a quote pair, e.g. `'edn'`) and is
//!    excluded, not a hit. Two genuine unpaired primes on the same string (`"recv' … select' set"`)
//!    are each TRAILING with no pending LEADING opener at the time they're seen, so both are
//!    correctly flagged (the naive "toggle on every apostrophe" scheme would wrongly pair them as
//!    a quote and lose the second — proven by `two_unpaired_primes_both_flagged_not_paired`).
//!
//! ## The earned exemption
//!
//! Each surviving hit carries a co-located (same-line) `// rune:lint(retired-name) — <reason>`.
//! Two honest reasons earn standing (24t's taxonomy):
//!   • **live dual-impl / macro pair** — `readln'` (the `readln` defmacro expands TO `readln'`,
//!     same name two forms). Rete no longer uses `'` for the kernel: public names are native,
//!     the wat reference is `$oracle`.
//!   • **positional constructor idiom** — `Frame'` (the record is `Frame`, `Frame'` builds one).
//! A rune of "it's just a message, nobody will notice" does NOT earn standing — that site is a
//! FIX (drop the `'`), not a rune (excusare — the reason must earn it).
//!
//! `rune:lint(<name>)` is the repo's project-custom-lint exemption form (owner `lint` = the project
//! lint suite, NOT a grimoire spell).

use std::path::{Path, PathBuf};

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

fn is_word_char(c: char) -> bool {
    c.is_ascii_alphanumeric() || c == '-'
}

/// Walk backward from `end` (byte index of the `'`, exclusive) over `[A-Za-z0-9-]` to find the
/// start of the identifier, then trim so the returned name starts with an ASCII letter (never a
/// digit or `-`) — matches the predicate's "kebab/alpha identifier" shape.
fn identifier_ending_at(line: &str, end: usize) -> Option<(usize, &str)> {
    let bytes = line.as_bytes();
    let mut start = end;
    while start > 0 && is_word_char(bytes[start - 1] as char) {
        start -= 1;
    }
    // Trim leading digits/hyphens — an identifier starts with a letter.
    while start < end && !(bytes[start] as char).is_ascii_alphabetic() {
        start += 1;
    }
    if start >= end {
        return None;
    }
    Some((start, &line[start..end]))
}

/// A retired-name-shaped hit inside a `.rs` string literal.
#[derive(Debug, PartialEq, Eq, Clone)]
struct Hit {
    /// 1-based line the `'` itself sits on.
    line: usize,
    name: String,
    /// 1-based line the ENCLOSING string literal's closing `"` sits on — same as `line` for a
    /// single-line literal, greater for a `\`-continued multi-line one. A `// rune:lint(...)` is
    /// only ever legal outside the string, so the exemption search window is `line..=close_line`.
    close_line: usize,
}

/// Find every retired-name-shaped hit inside `.rs` STRING LITERALS across the whole file. State
/// (inside-a-string, the pending single-quote-prose opener) is threaded across physical lines so a
/// `\`-continued multi-line literal is scanned as the one string it is — see the module doc for why
/// a per-line reset misses continuation-line hits.
fn retired_name_hits(src: &str) -> Vec<Hit> {
    let mut hits: Vec<Hit> = Vec::new();
    let mut in_str = false;
    let mut escape = false;
    let mut pending_leading = false;
    let mut line_no: usize = 1;
    // The current physical line's text so far, in and out of strings — used both for the
    // backward identifier walk and to spot a line-comment `//` opener outside a string.
    let mut cur_line = String::new();
    // Indices into `hits` opened by the string literal currently in progress (if any); resolved
    // to their `close_line` when that literal's closing `"` is reached.
    let mut open_hit_indices: Vec<usize> = Vec::new();

    let chars: Vec<char> = src.chars().collect();
    let mut i = 0;
    while i < chars.len() {
        let c = chars[i];
        if c == '\n' {
            line_no += 1;
            cur_line.clear();
            escape = false; // a `\<newline>` escape is fully consumed by the newline itself.
            i += 1;
            continue;
        }
        if escape {
            escape = false;
            cur_line.push(c);
            i += 1;
            continue;
        }
        if in_str {
            match c {
                '\\' => {
                    escape = true;
                    cur_line.push(c);
                }
                '"' => {
                    in_str = false;
                    pending_leading = false;
                    for idx in open_hit_indices.drain(..) {
                        hits[idx].close_line = line_no;
                    }
                    cur_line.push(c);
                }
                '\'' => {
                    let prev = cur_line.chars().last();
                    let next = chars.get(i + 1).copied();
                    let followed_by_letter = next.is_some_and(|n| n.is_ascii_alphabetic());
                    let preceded_by_word = prev.is_some_and(is_word_char);

                    if preceded_by_word {
                        if followed_by_letter {
                            // Contraction/possessive ("don't", "Token's") — the apostrophe sits
                            // BETWEEN two letters. Neither a prime nor a prose delimiter, and
                            // (unlike a LEADING opener) it cannot be mistaken for one since a
                            // prose opener is never preceded by a word char. No state change.
                        } else if pending_leading {
                            // TRAILING, closing a pending prose quote (`'edn'`) — not a hit.
                            pending_leading = false;
                        } else if let Some((_start, name)) = identifier_ending_at(&cur_line, cur_line.len()) {
                            // TRAILING, no opener pending — a genuine prime suffix.
                            let idx = hits.len();
                            hits.push(Hit { line: line_no, name: name.to_string(), close_line: line_no });
                            open_hit_indices.push(idx);
                        }
                    } else {
                        // LEADING: preceded by a non-word char (space, `:`, `(`, start-of-literal)
                        // — opens a candidate prose quote, whatever follows it (`'edn'`, `':wat…`).
                        pending_leading = true;
                    }
                    cur_line.push(c);
                }
                _ => {
                    cur_line.push(c);
                }
            }
        } else if c == '"' {
            in_str = true;
            pending_leading = false;
            cur_line.push(c);
        } else if c == '/' && chars.get(i + 1) == Some(&'/') {
            // A `//` outside any string starts a line comment — nothing to its right (on this
            // physical line) is ever inside a string literal, so stop scanning this line's chars.
            while i < chars.len() && chars[i] != '\n' {
                i += 1;
            }
            continue;
        } else {
            cur_line.push(c);
        }
        i += 1;
    }
    hits
}

#[test]
fn retired_names_are_justified() {
    let manifest = env!("CARGO_MANIFEST_DIR");
    let mut files = Vec::new();
    collect_rs(&Path::new(manifest).join("src"), &mut files);
    files.sort();

    // NON-VACUITY: a walk that comes back empty asserts nothing over nothing and reports PASS, and
    // every verdict downstream inherits that silence. The floor sits well under the
    // 213 .rs file(s) this walk finds today — driven 2026-09-01, and the count comes
    // from `tests/lint/every_walking_gate_declares_non_vacuity.rs`, never from prose — so it
    // catches a walk gone blind — a moved root, a renamed directory — without rotting as the
    // tree grows.
    assert!(
        files.len() > 100,
        "the retired-name walk found only {} .rs file(s) — it is not \
         reaching the tree it claims to guard, so its green means nothing",
        files.len()
    );

    let mut violations = Vec::new();
    for f in &files {
        let Ok(src) = std::fs::read_to_string(f) else { continue };
        let rel = f.strip_prefix(manifest).unwrap_or(f).display().to_string();
        let lines: Vec<&str> = src.lines().collect();

        for hit in retired_name_hits(&src) {
            // The exemption may sit anywhere from the hit's own line through the line its
            // enclosing string literal closes on — a `\`-continued literal has no legal spot for
            // a `//` comment until the string ends (see module doc).
            let exempted = (hit.line..=hit.close_line)
                .filter_map(|n| lines.get(n - 1))
                .any(|l| l.contains("// rune:lint(retired-name)"));
            if exempted {
                continue;
            }
            violations.push(format!("{}:{}   {}'", rel, hit.line, hit.name));
        }
    }

    assert!(
        violations.is_empty(),
        "\n\n🔥🔥🔥 RETIRED NAME IN A RUST STRING — {} site(s) name a `'`-suffixed wat verb/type in\n\
         a Rust message string. Arc 278 0z dropped the `'` from 24 IPC names (the full list is\n\
         `wat-scripts/fixes/reclaim-ipc-prime-names.wat`) — a user can only ever type the UNPRIMED\n\
         form, so a message naming the primed form points at a verb that does not exist.\n\
         \n\
         THE FIX — classify each with the fast arbiter (`target/release/wat --check <fixture>`,\n\
         ~0.2s: a one-line fixture using the PLAIN name RESOLVES ⇒ retired, FIX; UnknownFunction ⇒\n\
         still live, RUNE):\n\
           • FIX (retired): drop the `'` in the message string. No rune (it no longer matches).\n\
           • RUNE (earned, live prime): add a co-located, same-line\n\
             `// rune:lint(retired-name) — <reason>`. Honest reasons — 24t's taxonomy:\n\
               - `readln' — the readln defmacro expands to it; same name, two forms`\n\
               - `Frame' — positional constructor idiom (Frame is the record, Frame' builds one)`\n\
         A rune reason of \"it's just a message\" does NOT earn its standing — that site is a FIX,\n\
         not a rune (excusare — the reason must earn it).\n\
         \n\
         Offenders:\n\n{}\n",
        violations.len(),
        violations.join("\n"),
    );
}

#[cfg(test)]
mod tests {
    use super::retired_name_hits;

    fn names(src: &str) -> Vec<String> {
        retired_name_hits(src).into_iter().map(|h| h.name).collect()
    }

    #[test]
    fn flags_a_bare_prime_in_a_string() {
        assert_eq!(names(r#"    Err(recv_outcome_lost("send': peer disconnected".into())),"#), vec!["send"]);
        assert_eq!(names(r#"reason: "recv' EDN decode failed: {}".into(),"#), vec!["recv"]);
    }

    #[test]
    fn ignores_english_possessive_and_contraction() {
        assert!(names(r#"    let s = "don't do that, it's the wall's job".to_string();"#).is_empty());
        assert!(names(r#"    let s = "Token's span was already consumed".to_string();"#).is_empty());
    }

    #[test]
    fn ignores_a_comment_only_reference() {
        assert!(names("// see the connect' OUTCOME WALL (arc 170 #24)").is_empty());
        assert!(names("    // recv' used to raise; now it returns an outcome").is_empty());
    }

    #[test]
    fn ignores_a_trailing_comment_after_code() {
        // The `'`-shaped text sits after `//`, outside any string literal on this line.
        assert!(names(r#"    do_thing(x); // historical: was called connect' pre-0z"#).is_empty());
    }

    #[test]
    fn ignores_single_quoted_prose_closing_quotes() {
        assert!(names(r#""wat: --check-output expects 'edn' or 'json'""#).is_empty());
        assert!(names(r#""canonical FQDN form is ':wat::core::nil' (arc 153)""#).is_empty());
        assert!(names(r#""must read 'expects at least 2 arguments'""#).is_empty());
    }

    #[test]
    fn two_unpaired_primes_both_flagged_not_paired() {
        // The naive "toggle on every apostrophe" scheme would treat the first `'` as an opener
        // and the second as its closer, silently losing the second violation. Both are genuine
        // TRAILING primes (no LEADING opener pending at either), so both must survive.
        assert_eq!(
            names(r#"reason: "recv' on a timer peer is not supported; place it in a select' set".into(),"#),
            vec!["recv", "select"]
        );
    }

    #[test]
    fn ignores_lifetimes_outside_string_literals() {
        // Lifetimes live in CODE, never inside a string literal, so they never enter the scan.
        assert!(names("    fn f<'a>(x: &'a Span, peer: &'a str) -> &'a str {").is_empty());
    }

    #[test]
    fn a_rune_reason_is_orthogonal_to_the_predicate() {
        // The predicate itself does not know about runes — that's the caller's job (the `#[test]`
        // skips lines containing the marker). Confirm the predicate still finds the hit; the
        // exemption is applied one layer up.
        assert_eq!(
            names(r#"    ("readln'", "Datum/Eof/Stopped") // rune:lint(retired-name) — macro pair"#),
            vec!["readln"]
        );
    }

    #[test]
    fn multi_line_continuation_hit_is_found_and_exemptible_on_the_closing_line() {
        // Mirrors the real miss at check.rs:8473-8474: a `\`-continued string literal where the
        // SECOND physical line (no opening `"` of its own) carries the hit. A per-line-reset
        // scanner never sees this; the whole-file scan must.
        let src = "\
                \"readln' takes exactly one argument (cap-i64); got {}. \\\n\
                 Use (readln' <cap>) with no ascription.\",\n";
        let hits = retired_name_hits(src);
        assert_eq!(hits.iter().map(|h| h.name.as_str()).collect::<Vec<_>>(), vec!["readln", "readln"]);
        // The first hit sits on line 1 (where the string opens); the second sits on line 2, which
        // has no opening `"` of its own. Both close on line 2, where the literal's `"` closes —
        // that is the only line either can legally carry a `// rune:lint(...)` comment.
        assert_eq!(hits[0].line, 1);
        assert_eq!(hits[0].close_line, 2);
        assert_eq!(hits[1].line, 2);
        assert_eq!(hits[1].close_line, 2);
    }
}
