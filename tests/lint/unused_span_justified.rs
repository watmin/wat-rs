//! THE UNUSED-SPAN JUSTIFICATION LINT — a structural wall against dropped locations.
//!
//! An ignored span param (`_span`, `_list_span`, `_head_span`: `&Span`) silences rustc's
//! unused-param warning — but a `&Span` carries the **source location** for a located diagnostic.
//! Ignoring it *can* mean a fallible fn emits an **unlocated** error (the "burned us" class): the
//! source span was in hand and thrown away, so the user's crash points at nothing (or at a
//! `rust_caller_span!()` Rust line instead of their `.wat`). A one-time hand-audit is unreliable
//! (this migration mis-classified it 3×). So make it structural: every ignored span param must
//! carry a co-located justification rune, enforced here — the wrong form (a silently-ignored span)
//! becomes unrepresentable (R52 `QVOD LEX ACCENDIT` / R57 unrepresentable > flagged — the reason
//! must EARN its standing).
//!
//! ## Scope: span params ONLY
//!
//! The lint targets an identifier that **starts with `_` and ends in `span`**, typed `&Span`
//! (`_span`, `_list_span`, `_head_span`). Other ignored params (`_sym`, `_env`, …) carry NO
//! location — their ignore has no failure mode, so linting them is rune-noise. The boundary before
//! the leading `_` is load-bearing: `caller_span: &Span` (a USED param) must NOT match — only a
//! genuinely underscore-prefixed, therefore ignored, span param does.
//!
//! ## The earned exemption
//!
//! Each ignored-span site carries a co-located (same-line) `// rune:lint(unused-span) — <reason>`
//! stating why the drop is safe. Two honest reasons earn standing:
//!   • **infallible** — the fn has no error path (`infallible — no error path`).
//!   • **located elsewhere** — every error already uses a real WAT span (a per-arg `arg.span()`,
//!     a threaded inner span, or the error is a *value* located at the caller's match). The rune
//!     states WHERE.
//!
//! ⛔ `rust_caller_span!()` DOES NOT EARN STANDING, and offering it here was a contradiction this
//! lint carried from the start: the scope paragraph above names pointing at *"a
//! `rust_caller_span!()` Rust line instead of their `.wat`"* as the HARM, and the exemption list
//! then offered that same thing as the cure. A rule cannot forbid something and accept it as its
//! own justification. Two sites leaned on it — `eval_tuple_ctor` and `eval_bytes_to_hex` — and
//! both turned out to be FIXES, not exemptions: each had the span as a parameter and raised at a
//! Rust line anyway.
//!
//! That pairing is also this lint's blind spot, now named. A site that BOTH ignores its span param
//! AND substitutes a Rust one slips `span_substitution_justified` too, because that lint requires a
//! USED param. Neither wall saw the intersection — and this rune reason is what made it invisible.
//! A site whose error is genuinely **unlocated** (the `_span` was available and would improve it)
//! does NOT earn a rune — it gets FIXED (thread the span into the error, rename `_span`→`span`),
//! so it no longer matches this lint at all. A rune reason of "we drop the location but ignore it"
//! is a launder, not an exemption (excusare — the reason must earn it).
//!
//! `rune:lint(<name>)` is the repo's project-custom-lint exemption form (owner `lint` = the project
//! lint suite, NOT a grimoire spell). excusare audits the reason; a future build tool validates
//! `<name>` against the lint registry.

use std::path::{Path, PathBuf};
use std::sync::LazyLock;

use regex::Regex;

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

/// A param whose identifier starts with `_`, ends in `span`, typed `&Span`. The leading `\b`
/// word-boundary is what keeps a USED `caller_span: &Span` from matching: a leading `_` (after `(`,
/// `,`, or whitespace) sits on a word boundary, but the `_` inside `caller_span` (between two word
/// chars) does NOT — so the ignored param matches and the used one does not.
static SPAN_PARAM: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\b_[a-z_]*span: ?&Span\b").unwrap());

fn is_ignored_span_param(line: &str) -> bool {
    SPAN_PARAM.is_match(line)
}

#[test]
fn ignored_spans_are_justified() {
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
        "the unused-span walk found only {} .rs file(s) — it is not \
         reaching the tree it claims to guard, so its green means nothing",
        files.len()
    );

    let mut violations = Vec::new();
    for f in &files {
        let Ok(src) = std::fs::read_to_string(f) else { continue };
        let rel = f.strip_prefix(manifest).unwrap_or(f).display().to_string();

        for (idx, line) in src.lines().enumerate() {
            // A line-comment / doc-comment that merely names the param type is not a param
            // declaration — the rune itself is an end-of-line comment on the param's OWN line
            // (which starts with the identifier, not `//`), so skipping `//`-opening lines never
            // skips a real site.
            if line.trim_start().starts_with("//") {
                continue;
            }
            if is_ignored_span_param(line) && !line.contains("// rune:lint(unused-span)") {
                violations.push(format!("{}:{}", rel, idx + 1));
            }
        }
    }

    assert!(
        violations.is_empty(),
        "\n\n🔥🔥🔥 UNJUSTIFIED IGNORED SPAN — {} param(s) named `_…span: &Span` silence rustc's\n\
         unused warning while carrying a source location that a fallible fn could have used. An\n\
         ignored `&Span` can mean an UNLOCATED error (the span was in hand and dropped).\n\
         \n\
         THE FIX — read the fn's error paths and do ONE of:\n\
           • FIX (unlocated): the fn emits an error while this `_span` was available and would\n\
             improve it → thread the span into that error's `RuntimeError {{ span: … }}`, rename\n\
             `_span`→`span`, NO rune (it no longer matches this lint).\n\
           • RUNE (earned): the drop is safe — add a co-located, same-line\n\
             `// rune:lint(unused-span) — <reason>`. Honest reasons:\n\
               - `infallible — no error path`\n\
               - `located elsewhere` — every error already uses a real WAT span (`arg.span()`,\n\
                 a threaded inner span, or the error is a VALUE located at the caller's\n\
                 match). State WHERE.\n\
         `rust_caller_span!()` does NOT earn standing: a Rust line is the HARM this lint names,\n\
         not a location. A site that ignores its span AND raises at a Rust line is a FIX.\n\
         A rune of \"drops the location but we ignore it\" does NOT earn its standing — that site is\n\
         a FIX, not a rune (excusare — the reason must earn it). For a single-line signature, break\n\
         the `_…span: &Span` param onto its own line to carry the rune.\n\
         \n\
         Offenders:\n\n{}\n",
        violations.len(),
        violations.join("\n"),
    );
}

#[cfg(test)]
mod detector_tests {
    use super::is_ignored_span_param;

    #[test]
    fn matches_underscore_span_variants() {
        assert!(is_ignored_span_param("    _span: &Span,"));
        assert!(is_ignored_span_param("    _list_span: &Span,"));
        assert!(is_ignored_span_param("    _head_span: &Span,"));
        assert!(is_ignored_span_param(
            "fn f(args: &[WatAST], _list_span: &Span) -> Result<Value, EvalBreak> {"
        ));
        // No space after the colon is still a match.
        assert!(is_ignored_span_param("    _span:&Span,"));
    }

    #[test]
    fn does_not_match_used_span_params() {
        // No leading underscore — a USED param whose name merely ends in `_span`.
        assert!(!is_ignored_span_param("    caller_span: &Span,"));
        assert!(!is_ignored_span_param("    list_span: &Span,"));
        assert!(!is_ignored_span_param("fn f(args: &[WatAST], list_span: &Span) {"));
    }

    #[test]
    fn does_not_match_non_span_ignored_params() {
        // `_sym`/`_env` carry no location — out of scope.
        assert!(!is_ignored_span_param("    _sym: &SymbolTable,"));
        assert!(!is_ignored_span_param("    _env: &Environment,"));
        // `_call_site` ends in `site`, not `span`.
        assert!(!is_ignored_span_param("    _call_site: &Span,"));
    }
}
