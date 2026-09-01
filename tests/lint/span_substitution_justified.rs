//! THE SPAN-SUBSTITUTION LINT — a wall against pointing a user at RUST.
//!
//! `unused_span_justified` polices span **omission**: a param named `_…span: &Span`, ignored, whose
//! location a fallible fn could have used. Its own header states the boundary is load-bearing —
//! *"`caller_span: &Span` (a USED param) must NOT match"*.
//!
//! That leaves the other half unwalled, and it is the half that bit us. A site can DISCARD nothing
//! and still lie: it has a real wat span in scope and stamps `rust_caller_span!()` anyway. Nothing
//! is underscore-prefixed, so the omission lint cannot see it — and the value that reaches the user
//! names a line in `src/*.rs` instead of the `.wat` they wrote.
//!
//! MEASURED, not theorised (arc 170 #6): `kwargs-lower` rewrites `(svc/start …)` into
//! `(svc/start$impl …)`, the emitted call carries the TEMPLATE's span, and every frame under a
//! kwargs fn names `wat/core.wat:649`. So `assertion-failed!` inside ANY kwargs fn reports the
//! author's failure as living in core.wat. The author's real line is not buried — it is ABSENT
//! from the stack (probes: `wat-scripts/scratch-pad/probe-{call-site-kwargs,kwargs-stack-shape}.wat`
//! rule out `-1`, name-search, and every reader-side selection policy). Substitution destroys the
//! location at the point of substitution; nothing downstream recovers it. So it must be walled
//! WHERE IT HAPPENS.
//!
//! ## What this lint flags
//!
//! A fn that (a) has a **USED** wat span in scope — a param `…span: &Span` with NO leading `_` —
//! and (b) calls `rust_caller_span!()` in its body. A real location was in hand and a Rust one was
//! minted instead.
//!
//! ## What it does NOT flag (and must not)
//!
//! A **leaf** with no wat span in scope. `rust_caller_span!()` exists for exactly that case — a
//! Rust-side helper genuinely holding no wat location, where a Rust line beats nothing. Those sites
//! have no span param, so they never match. This lint is about the CHOICE between a real location
//! and a Rust one, never about the absence of a choice.
//!
//! ## ⛔ THE PREDICATE IS A PROXY — and this is the population it protects
//!
//! "has no `…span: &Span` param" stands in for "no wat span is available". Those are not the
//! same claim. A fn can have no span param while the caller ONE FRAME UP holds the author's
//! span and already uses it — so the proxy admits a site the principle above would refuse.
//!
//! It was not hypothetical. `refuse_export_without_arm`'s two call sites were exactly that:
//! `fire_rules_on_session` and `fire_once_session` stamped `rust_caller_span!()` for a
//! USER-REACHABLE refusal and were invisible here, while all three real entries
//! (`eval_fire_rules_native`, `eval_fire_once_native`, `eval_fire_rules_explain`) held
//! `list_span` and already used it.
//!
//! **The cure was to make the proxy TRUE, not to widen it.** Both fns now take `span: &Span`,
//! so they sit inside this lint's view and any future substitution in either body reddens with
//! no new predicate and no caller analysis
//! (`docs/arc/2026/06/278-rules-engine/strike-refusal-span/DESIGN.md`).
//!
//! Widening was MEASURED before it was rejected. Under `src/`, **494** sites are a span-LESS fn
//! stamping `rust_caller_span!()`, **69** of them under `src/rete/` — and the visible majority
//! are test helpers and genuine leaves, precisely the population the exclusion above exists to
//! protect. The instrument, so those numbers stay recheckable rather than folklore: `violations_in`
//! with its `carries_span` test inverted to `!carries_span`, walked over the same file set
//! `span_substitutions_are_justified` walks. Separating real defects from that population needs
//! analysis ACROSS frames, and this project has a recorded failure mode for exactly that — a
//! static audit of a call graph is wrong in both directions and looks right each time. A lint
//! that guessed across frames would be a new source of false findings.
//!
//! ## The exemption, and what it may NOT say
//!
//! A `// rune:lint(span-substitution) — <reason>` on the `rust_caller_span!()` line, OR anywhere in
//! the contiguous `//` comment block directly above it. The reason must state why the in-scope span
//! is WRONG for this value (e.g. the value is a synthesised node that never existed in user source,
//! so no user location is truthful).
//!
//! The block form is deliberate: the sibling lint's SAME-LINE rule fits the short thing it guards,
//! but an earned reason here is a sentence or three, and a one-line cage would breed terse, weak
//! reasons — the opposite of the standard. Widened when the fleet's first real rune (a four-line
//! justification, sound) would otherwise have been rejected for its length.
//!
//! ⛔ `"located elsewhere via rust_caller_span!()"` does NOT earn standing here, and the sibling
//! lint must stop accepting it either: `unused_span_justified`'s scope paragraph names pointing at
//! *"a `rust_caller_span!()` Rust line instead of their `.wat`"* as the HARM, then lists
//! `rust_caller_span!()` among the earned reasons. A rule cannot both forbid a thing and accept it
//! as a justification for itself. That contradiction is why this half was never built (excusare —
//! the reason must earn its standing; R52 `QVOD LEX ACCENDIT` — a corrected law lights every
//! existing violator, and the burning IS the correction).

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

/// A USED wat span param: `…span: &Span` whose identifier does NOT start with `_`.
///
/// The leading `(?:^|[(,\s])` + `[a-z]` is the mirror of the sibling lint's boundary: an
/// underscore-prefixed `_list_span` must NOT match here (that is omission, the other lint's
/// subject), while `span`/`list_span`/`caller_span` must.
/// Matches only the TAIL (`span: &Span`); the identifier's leading char is checked in code by
/// walking back over the ident. Two reasons, both learned the hard way:
///   • an earlier one-regex form was `[a-z][a-z_]*span`, which requires at least one char BEFORE
///     `span` — so a bare `span: &Span`, the commonest form, silently never matched and the lint
///     read GREEN on the very sites it exists for (caught by this file's own detector tests);
///   • a leading `(?:^|[(,\s])` makes the literal open with `(`, which `no_inlined_edn` reads as
///     EDN-esque. Walking the ident in code sidesteps both — and says what it means.
static SPAN_PARAM_TAIL: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"span: ?&Span\b").unwrap());

static RUST_CALLER_SPAN: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"rust_caller_span!\s*\(").unwrap());

/// A `fn` header line. Signatures may wrap, so the caller accumulates until the body's `{`.
static FN_HEADER: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^\s*(?:pub(?:\([^)]*\))?\s+)?(?:async\s+)?(?:const\s+)?(?:unsafe\s+)?fn\s+\w+").unwrap());

/// A USED wat span param: `…span: &Span` whose identifier does NOT start with `_`. An
/// underscore-prefixed `_list_span` is the SIBLING lint's subject (omission) and must not match
/// here — this lint is about substitution, where nothing was discarded.
pub(crate) fn has_used_span_param(sig: &str) -> bool {
    SPAN_PARAM_TAIL.find_iter(sig).any(|m| {
        // Walk back over the identifier chars preceding the matched `span:` tail; the char just
        // before that run is the boundary. `_span`/`_list_span` start with `_` → not ours.
        let head = &sig[..m.start()];
        let ident_start = head
            .rfind(|c: char| !(c.is_alphanumeric() || c == '_'))
            .map(|i| i + 1)
            .unwrap_or(0);
        !head[ident_start..].starts_with('_')
    })
}

/// Is line `k` covered by a `rune:lint(span-substitution)` — on the line itself, or anywhere in
/// the contiguous `//` comment block directly above it?
///
/// The sibling `unused_span_justified` demands the rune be SAME-LINE, which is right for the short
/// thing it guards (a param declaration). It is wrong here. An earned reason must say why the
/// in-scope span is WRONG for this value — that is a sentence or three, not a trailing fragment.
/// Forcing it onto one line would push authors toward terse, weak reasons, which is the opposite of
/// "the reason must earn its standing" (excusare). So the block above counts. This widening was
/// forced by the first real rune the fleet produced: a four-line justification the same-line rule
/// would have rejected while the reason itself was sound.
fn runed(lines: &[&str], k: usize) -> bool {
    const MARKER: &str = "rune:lint(span-substitution)";
    if lines[k].contains(MARKER) {
        return true;
    }
    let mut i = k;
    while i > 0 {
        i -= 1;
        let t = lines[i].trim_start();
        if !t.starts_with("//") {
            return false;
        }
        if t.contains(MARKER) {
            return true;
        }
    }
    false
}

/// Walk one file, returning `(line_number, fn_name)` for every `rust_caller_span!()` call that sits
/// inside a fn whose signature carries a USED wat span, and that lacks a co-located rune.
fn violations_in(src: &str) -> Vec<(usize, String)> {
    let lines: Vec<&str> = src.lines().collect();
    let mut out = Vec::new();
    let mut i = 0usize;

    while i < lines.len() {
        if !FN_HEADER.is_match(lines[i]) || lines[i].trim_start().starts_with("//") {
            i += 1;
            continue;
        }

        // Accumulate the signature until the body's opening brace.
        let fn_name = lines[i]
            .split_whitespace()
            .skip_while(|t| *t != "fn")
            .nth(1)
            .unwrap_or("<anon>")
            .trim_end_matches(|c: char| !c.is_alphanumeric() && c != '_')
            .to_string();

        let mut sig = String::new();
        let mut j = i;
        while j < lines.len() {
            sig.push_str(lines[j]);
            sig.push(' ');
            if lines[j].contains('{') {
                break;
            }
            j += 1;
        }
        if j >= lines.len() {
            break;
        }

        // Body: from the signature's `{` until braces balance.
        let carries_span = has_used_span_param(&sig);
        let mut depth: i32 = 0;
        let mut k = j;
        loop {
            if k >= lines.len() {
                break;
            }
            for c in lines[k].chars() {
                if c == '{' {
                    depth += 1;
                } else if c == '}' {
                    depth -= 1;
                }
            }
            if carries_span
                && RUST_CALLER_SPAN.is_match(lines[k])
                && !lines[k].trim_start().starts_with("//")
                && !runed(&lines, k)
            {
                out.push((k + 1, fn_name.clone()));
            }
            if depth <= 0 && k > j {
                break;
            }
            if depth <= 0 && lines[j].contains('}') {
                break;
            }
            k += 1;
        }
        i = k + 1;
    }
    out
}

#[test]
fn span_substitutions_are_justified() {
    let manifest = env!("CARGO_MANIFEST_DIR");
    let mut files = Vec::new();
    collect_rs(&Path::new(manifest).join("src"), &mut files);
    files.sort();

    let mut violations = Vec::new();
    for f in &files {
        let Ok(src) = std::fs::read_to_string(f) else { continue };
        let rel = f.strip_prefix(manifest).unwrap_or(f).display().to_string();
        for (line, fn_name) in violations_in(&src) {
            violations.push(format!("{rel}:{line}  (fn {fn_name})"));
        }
    }

    assert!(
        violations.is_empty(),
        "\n\n🔥🔥🔥 SPAN SUBSTITUTION — {} site(s) mint a RUST location while a real wat span was\n\
         in scope. The value that reaches the user names a line in src/*.rs instead of the .wat\n\
         they wrote. This is the half `unused_span_justified` cannot see: nothing is DISCARDED\n\
         here, so no `_span` param matches — the wrong span is USED.\n\
         \n\
         THE FIX — do ONE of:\n\
           • FIX: pass the in-scope wat span (`span.clone()` / the arg's own `.span()`) instead of\n\
             `rust_caller_span!()`. NO rune — it no longer matches.\n\
           • RUNE (earned): the in-scope span is WRONG for this value — e.g. it is a SYNTHESISED\n\
             node that never existed in user source, so no user location would be truthful. Add a\n\
             co-located `// rune:lint(span-substitution) — <reason>` stating that.\n\
         \n\
         ⛔ \"located elsewhere via rust_caller_span!()\" does NOT earn standing: a Rust line IS\n\
         the harm, not a location. If the user cannot be pointed at their own source, say WHY.\n\
         \n\
         Offenders:\n\n{}\n",
        violations.len(),
        violations.join("\n"),
    );
}

#[cfg(test)]
mod detector_tests {
    use super::{has_used_span_param, violations_in};

    #[test]
    fn used_span_params_match_and_ignored_ones_do_not() {
        assert!(has_used_span_param("fn f(span: &Span) {"));
        assert!(has_used_span_param("fn f(a: i32, list_span: &Span) {"));
        assert!(has_used_span_param("    caller_span: &Span,"));
        // the sibling lint's subject — omission, not substitution
        assert!(!has_used_span_param("fn f(_span: &Span) {"));
        assert!(!has_used_span_param("    _list_span: &Span,"));
        // no span at all
        assert!(!has_used_span_param("fn f(env: &Environment) {"));
    }

    #[test]
    fn flags_substitution_only_when_a_real_span_is_in_scope() {
        // NOT flagged: a genuine leaf with no wat span to choose from.
        let leaf = "fn leaf(v: &Value) -> Span {\n    rust_caller_span!()\n}\n";
        assert!(violations_in(leaf).is_empty());

        // FLAGGED: a real span was in hand and a Rust one was minted instead.
        let abuse = "fn abuse(span: &Span, v: &Value) -> Span {\n    rust_caller_span!()\n}\n";
        assert_eq!(violations_in(abuse).len(), 1);

        // NOT flagged: same site, rune earned.
        let runed = "fn ok(span: &Span) -> Span {\n    rust_caller_span!() // rune:lint(span-substitution) — synthesised node, no user source\n}\n";
        assert!(violations_in(runed).is_empty());

        // A rune in the contiguous comment BLOCK above also counts — an earned reason is often
        // several lines, and the same-line rule would reject a sound justification for its length.
        let block = "fn ok(span: &Span) -> Span {\n    // rune:lint(span-substitution) — a Native builtin has no\n    // wat definition site, so the call span would misattribute.\n    rust_caller_span!()\n}\n";
        assert!(violations_in(block).is_empty());

        // …but an unrelated comment above does NOT launder it.
        let bare_comment = "fn bad(span: &Span) -> Span {\n    // just a note\n    rust_caller_span!()\n}\n";
        assert_eq!(violations_in(bare_comment).len(), 1);
    }
}
