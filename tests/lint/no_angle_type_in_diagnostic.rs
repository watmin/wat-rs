//! THE NO-ANGLE-TYPE-IN-DIAGNOSTIC LINT — channel 4 of "a type must reach a user in a
//! spelling the reader accepts" stays closed.
//!
//! STONE-close-the-last-two-channels (arc 109): 122 hard-coded string literals named a wat
//! type in the retired `Head<A,B>` spelling, in exactly the place a user is most looking for
//! guidance — a `TypeMismatch`/`MalformedForm` diagnostic. All were rewritten to the
//! surviving `(Head :- [A B])` reference form (or, for the retired bare `Vec` alias,
//! renamed to `Vector` in the same motion). This lint is the step that makes the rewrite
//! STAY rewritten: a fresh diagnostic string built the old way — copy-pasted from a sibling
//! message, or hand-rolled by someone who half-remembers the old spelling — cannot return
//! un-noticed.
//!
//! ## The shape banned
//!
//! One of a fixed, closed list of wat type-constructor head names — `Vector`, `HashMap`,
//! `HashSet`, `PersistentVector`, `PersistentMap`, `Option`, `Result`, `Tuple`, `Address`,
//! `Peer`, `Listener`, `Thread`, `Process`, `ThreadSelfPeer`, `WalkStep`, `NextOutcome`,
//! `Stream`, `Vec` (the retired bare alias for `Vector`), `Bytes`, `LociDiedError`, `Handle`,
//! `AST`, `List`, `Fn` — immediately followed by `<`, inside a string literal.
//!
//! This is deliberately a NAME allowlist, not "any capitalized identifier followed by `<`":
//! the sibling channel-3 stone's own warning applies here too — `Vec<T>`, `Arc<Function>`,
//! `Cow<'_, [WatAST]>` are Rust's own generics and appear throughout `src/` in ordinary,
//! correct Rust type position. A blanket scan would need exemptions so broad they would mean
//! nothing (STOP-3); a closed list of the SPECIFIC names this campaign's own census found
//! wearing the retired spelling does not.
//!
//! ## Scope
//!
//! `src/` and `crates/wat-reader`, `crates/wat-doc`, `crates/wat-source-derive`,
//! `crates/wat-to-edn-derive` — the crates that either implement the reader itself or emit
//! text a wat PROGRAM's author reads. **Deliberately excludes** `crates/wat-edn` (its
//! `Lexer`/`Parser` is a DIFFERENT grammar — the EDN wire-tag reader, where `#ns/Name<...>`
//! tag syntax is generic text with no parametric-type semantics tied to wat's own type
//! system; its own tests exist specifically to prove `<`/`_` are ordinary keyword-body
//! characters there, not a channel-4 violation) and `crates/wat-macros` (its diagnostics —
//! `wat_dispatch: Option<T> must have exactly one type argument` and siblings — describe
//! REQUIREMENTS ON RUST'S OWN generic syntax, read by a wat-rs contributor writing an
//! intrinsic in Rust at compile time, never by a wat program's author).
//!
//! Within scope, a file stops being scanned at its first `#[cfg(test)]` line: a test fixture
//! that feeds the OLD spelling to the parser to prove it is REFUSED (`parse_type_expr(":Vec
//! <:String>")` and siblings, `src/types.rs`'s `angle_bracket_parametric_head_is_illegal`)
//! is a positive control, not a diagnostic shown to a user — the same class-C exemption the
//! reader's own refusal messages earn, just made structurally exempt instead of
//! per-line-marked because an entire test module is that shape, not one line of it.
//!
//! ## The escape hatch
//!
//! A live (non-test) site that legitimately quotes the retired spelling to TEACH by naming
//! it — "here is the old form, here is the new" (`crates/wat-doc/src/lib.rs`'s `@arg`/`@ret`
//! refusal message, `crates/wat-reader/src/lexer.rs`'s `CommaInKeywordBody` /
//! `AngleTypeHeadInName` refusal text) — earns a co-located, same-line
//! `// rune:lint(no-angle-type-in-diagnostic) — <reason>`, exactly the shape
//! `no_angle_suffix_strip.rs` / `one_param_spec.rs` already use.
//!
//! `rune:lint(<name>)` is the repo's project-custom-lint exemption form (owner `lint` = the
//! project lint suite, NOT a grimoire spell).
//!
//! ## A known blind spot, stated rather than hidden
//!
//! The detector is line-scoped (like `no_angle_suffix_strip.rs`'s): it cannot see a match
//! split across a Rust `\`-continued multi-line string literal (`crates/wat-reader/src/
//! lexer.rs`'s `AngleTypeHeadInName` message is exactly this shape — its `Vector<wat::core::
//! i64>` quote spans a line break and so never reaches this lint at all, needing no
//! exemption marker only because the detector cannot see it, not because it was judged
//! clean). A regression written the same way — wrapped across a `\`-continuation — would
//! also go unseen. This is a real gap in this gate's reach, named rather than papered over
//! with a wider pattern that would then need exemptions so broad they mean nothing.

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

/// The closed list of wat type-constructor head names this campaign's own census found
/// wearing the retired `Head<...>` spelling in a live diagnostic. See the module doc for
/// why this is a name allowlist rather than "any capitalized identifier."
const HEADS: &[&str] = &[
    "Vector", "HashMap", "HashSet", "PersistentVector", "PersistentMap", "Option", "Result",
    "Tuple", "Address", "Peer", "Listener", "Thread", "Process", "ThreadSelfPeer", "WalkStep",
    "NextOutcome", "Stream", "Vec", "Bytes", "LociDiedError", "Handle", "AST", "List", "Fn",
];

/// Does `code` (already comment-stripped) contain a string literal naming one of `HEADS`
/// immediately followed by `<`? A hand-rolled scan, not a `Regex` dependency — mirrors
/// `no_angle_suffix_strip.rs`'s own `code_hit` in spirit: look inside quoted text only.
fn angle_head_hit(code: &str) -> Option<String> {
    let mut in_str = false;
    let bytes = code.as_bytes();
    let mut i = 0;
    let mut str_start = 0;
    while i < bytes.len() {
        let c = bytes[i];
        if c == b'"' && (i == 0 || bytes[i - 1] != b'\\') {
            if !in_str {
                in_str = true;
                str_start = i + 1;
            } else {
                in_str = false;
                let s = &code[str_start..i];
                for head in HEADS {
                    if let Some(rest_start) = find_word_then_lt(s, head) {
                        let end = (rest_start + 24).min(s.len());
                        return Some(s[rest_start.saturating_sub(head.len())..end].to_string());
                    }
                }
            }
        }
        i += 1;
    }
    None
}

/// Find `head` immediately followed by `<`, as a WHOLE word (not a substring of a longer
/// identifier — so `HeadOption<T>` does not false-positive on `Option`, but `(Option<T>`,
/// `: Option<T>,`, `a Option<T>` all do). Returns the byte offset just past the match (the
/// position of the `<`), or `None`.
fn find_word_then_lt(s: &str, head: &str) -> Option<usize> {
    let bytes = s.as_bytes();
    let hb = head.as_bytes();
    let mut start = 0;
    while let Some(rel) = s[start..].find(head) {
        let idx = start + rel;
        let before_ok = idx == 0 || !bytes[idx - 1].is_ascii_alphanumeric();
        let after_idx = idx + hb.len();
        let after_ok = after_idx < bytes.len() && bytes[after_idx] == b'<';
        if before_ok && after_ok {
            return Some(after_idx);
        }
        start = idx + 1;
        if start >= s.len() {
            break;
        }
    }
    None
}

/// Scan one file's already-read source, returning `(1-based line number, matched text)` for
/// each live-code violation. Shared by the main sweep and this file's own unit tests, so the
/// unit tests exercise the REAL cutoff/exemption logic, not a re-description of it.
fn scan_source(src: &str) -> Vec<(usize, String)> {
    let mut hits = Vec::new();
    let mut in_cfg_test = false;
    let mut depth: i32 = 0;
    let mut opened = false;
    for (idx, line) in src.lines().enumerate() {
        // A `#[cfg(test)]` marks ONE item, not the rest of the file. Skipping to EOF was a
        // 45%-of-the-tree blind spot: `src/runtime.rs`'s first one is at line 106 of 40,673,
        // so 99.7% of the largest file in the substrate went unscanned while the gate
        // reported green — and `src/check.rs`, holding the biggest concentration of the
        // sites this gate exists to catch, was blind from 12,285 of 22,345.
        //
        // Skip the ATTRIBUTED BLOCK instead: from the `{` that opens it to its matching `}`.
        // Depth counting is line-based and does not understand braces inside string literals;
        // that is adequate for `mod tests { … }` and is stated rather than assumed.
        if line.contains("#[cfg(test)]") {
            in_cfg_test = true;
            depth = 0;
            opened = false;
            continue;
        }
        if in_cfg_test {
            depth += line.matches('{').count() as i32;
            depth -= line.matches('}').count() as i32;
            if line.contains('{') {
                opened = true;
            }
            if opened && depth <= 0 {
                in_cfg_test = false;
            }
            continue;
        }
        if line.contains("rune:lint(no-angle-type-in-diagnostic)") {
            continue;
        }
        let comment_at = line.find("//");
        let code = match comment_at {
            Some(c) => &line[..c],
            None => line,
        };
        if let Some(shown) = angle_head_hit(code) {
            hits.push((idx + 1, shown));
        }
    }
    hits
}

#[test]
fn no_diagnostic_string_names_a_wat_type_in_the_retired_angle_spelling() {
    let manifest = env!("CARGO_MANIFEST_DIR");
    let mut files = Vec::new();
    for sub in [
        "src",
        "crates/wat-reader",
        "crates/wat-doc",
        "crates/wat-source-derive",
        "crates/wat-to-edn-derive",
    ] {
        collect_rs(&Path::new(manifest).join(sub), &mut files);
    }
    files.sort();

    // NON-VACUITY: a walk that comes back empty asserts nothing over nothing and reports PASS, and
    // every verdict downstream inherits that silence. The floor sits well under the
    // 230 .rs file(s) this walk finds today — driven 2026-09-01, and the count comes
    // from `tests/lint/every_walking_gate_declares_non_vacuity.rs`, never from prose — so it
    // catches a walk gone blind — a moved root, a renamed directory — without rotting as the
    // tree grows.
    assert!(
        files.len() > 100,
        "the no-angle-type-in-diagnostic walk found only {} .rs file(s) — it is not \
         reaching the tree it claims to guard, so its green means nothing",
        files.len()
    );

    let mut violations = Vec::new();
    for f in &files {
        if f.file_name().and_then(|n| n.to_str()) == Some("no_angle_type_in_diagnostic.rs") {
            continue; // this file names the pattern in its own detector — skip self.
        }
        let rel = f.strip_prefix(manifest).unwrap_or(f).display().to_string();
        let Ok(src) = std::fs::read_to_string(f) else { continue };

        for (line_no, shown) in scan_source(&src) {
            violations.push(format!("{}:{}   ...{}...", rel, line_no, shown));
        }
    }

    assert!(
        violations.is_empty(),
        "\n\n🔥🔥🔥 A DIAGNOSTIC STRING NAMES A WAT TYPE IN THE RETIRED ANGLE SPELLING —\n\
         {} site(s). `Head<A,B>` is unexpressible in wat source; the reader refuses it\n\
         (arc 109 \"annihilate the angle bracket\"). A diagnostic that prints this spelling\n\
         hands a user text they cannot paste back into a program — the exact defect\n\
         STONE-close-the-last-two-channels closed across 122 sites.\n\
         \n\
         THE FIX — rewrite the head's mention to the surviving reference form,\n\
         `(Head :- [args])` (drop the retired bare `Vec` alias for `Vector` while you're at\n\
         it, if that's the one that fired). Read the message first: `Vec<T>`, `Arc<Function>`,\n\
         `Cow<'_, [WatAST]>` are Rust's OWN generics and live in the same files — a rewritten\n\
         Rust generic is worse than an unrewritten wat type, so this lint is a closed NAME\n\
         list, not a blanket scan; if it fired, the name really is one of wat's own container\n\
         heads.\n\
         \n\
         Genuinely teaching by naming the dead form on purpose (a refusal message quoting\n\
         what it refuses, a rename note showing before/after) — earn a co-located, same-line\n\
         `// rune:lint(no-angle-type-in-diagnostic) — <reason>`.\n\
         \n\
         Offenders:\n\n{}\n",
        violations.len(),
        violations.join("\n"),
    );
}

#[cfg(test)]
mod tests {
    use super::{angle_head_hit, find_word_then_lt, scan_source};

    #[test]
    fn flags_a_known_head_immediately_followed_by_angle() {
        assert!(angle_head_hit(r#"expected: "Vector<T>","#).is_some());
        assert!(angle_head_hit(r#"expected: "HashMap<K,V>","#).is_some());
        assert!(angle_head_hit(r#"expected: "(Option :- [T]) or Vec<T>","#).is_some());
    }

    #[test]
    fn ignores_the_binder_form_it_was_rewritten_to() {
        assert!(angle_head_hit(r#"expected: "(Vector :- [T])","#).is_none());
        assert!(angle_head_hit(r#"expected: "(HashMap :- [K V])","#).is_none());
    }

    #[test]
    fn ignores_rust_generics_outside_string_literals() {
        // A real Rust signature — never inside quotes — must not fire.
        assert!(angle_head_hit("fn f(x: Vec<T>) -> Option<T> {").is_none());
    }

    #[test]
    fn ignores_a_prefix_that_merely_ends_in_the_head_name() {
        // `HeadOption<T>` must not false-positive on the `Option` substring.
        assert!(find_word_then_lt("HeadOption<T>", "Option").is_none());
        assert!(find_word_then_lt("a Option<T> b", "Option").is_some());
    }

    #[test]
    fn a_marked_line_is_exempt() {
        let src = "                    expected: \"Option<T>\", // rune:lint(no-angle-type-in-diagnostic) — teaches by naming\n";
        assert!(scan_source(src).is_empty());
    }

    #[test]
    fn an_unmarked_live_site_is_flagged() {
        let src = "                    expected: \"Option<T>\",\n";
        assert_eq!(scan_source(src).len(), 1);
    }

    #[test]
    fn everything_past_cfg_test_is_exempt() {
        let src = "                expected: \"before Vector<T> is flagged\",\n\
                   #[cfg(test)]\n\
                   mod tests {\n\
                       const X: &str = \"Vector<T>\";\n\
                   }\n";
        let hits = scan_source(src);
        assert_eq!(hits.len(), 1, "only the pre-cfg(test) line should fire: {:?}", hits);
        assert_eq!(hits[0].0, 1);
    }
}
