//! The codemod-reach engine — arc 255 stone 255.80 (R1: "the recorded codemods move the
//! 1,093 type-position sites embedded in Rust string literals by the SAME recorded codemod as
//! the `.wat` corpus, not by hand"; X2: "no new Rust parsing library").
//!
//! Given a wat-shaped literal's DECODED text (from [`crate::embedded_wat`]) and the SAME text
//! after a recorded `wat-scripts/fixes/*.wat` codemod has run over it, this module:
//!
//! 1. runs the codemod as a real subprocess over a temp `.wat` file — exactly
//!    `./target/release/wat ./wat-scripts/fixes/<fix>.wat` with `["<path>"]` on stdin, the
//!    documented invocation (`scripts/replay/delta.sh`) — never a Rust reimplementation of the
//!    codemod's rewrite rules;
//! 2. finds exactly what changed by parsing OLD and NEW decoded text with wat's OWN reader and
//!    walking both trees in lockstep (`WatAST::children()`), leaf by leaf. `fix-text-apply`
//!    (`wat/fix.wat`) only ever rewrites a LEAF's span text — it never inserts, removes or
//!    reorders a node — so the two trees are isomorphic by construction: same node kind, same
//!    child count, everywhere. A changed leaf's OLD span (converted to a decoded-text char
//!    offset the same way `wat::fix::fix-text-offset-of` does: line-start + col-1) names
//!    exactly the token the codemod touched, and the NEW tree's leaf at the same tree position
//!    gives the exact replacement text — no diffing library, no regex, no guessing which part
//!    of a long literal changed;
//! 3. maps each changed leaf's decoded-offset range through `LiteralSpan::char_map` back to raw
//!    source char offsets, and REFUSES the splice (rather than risk corrupting the file) unless
//!    the raw source there is CHAR-FOR-CHAR the old decoded text — the same guard
//!    `wat::fix::fix-text-apply` itself applies, run here in Rust because the subject is a raw
//!    Rust string literal's escaped source, which the codemod never sees.
//!
//! The driver (`src/bin/wat-fix-rust.rs`) and this module's own probe test both go through this
//! exact path — nothing here reimplements what a codemod decides to rewrite; it only locates
//! and verifies WHERE to splice the codemod's own output.

use std::io;
use std::path::Path;
use std::process::Command;

use crate::embedded_wat::LiteralSpan;
use crate::{parse_all_with_file, Span, WatAST};

/// One leaf-level change between an OLD and a NEW parse of the same (isomorphic) program:
/// `old_decoded[old_lo..old_hi]` (char indices into the OLD decoded text) must become
/// `new_text`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Edit {
    pub old_lo: usize,
    pub old_hi: usize,
    pub new_text: String,
}

/// A changed region the driver declined to splice, because the raw Rust source underneath it
/// is not char-for-char the old decoded text (an escape sequence, or elided line-continuation
/// content, sits inside the span) — applying the codemod's replacement there would silently
/// drop or corrupt raw source the decoder didn't reproduce 1:1.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RefusedSplice {
    pub old_lo: usize,
    pub old_hi: usize,
    pub old_decoded_text: String,
    pub raw_text: String,
    pub reason: String,
}

/// `line`/`col` (1-indexed, char-counted, matching `wat::fix::fix-text-offset-of`) → a flat
/// 0-indexed char offset into `text`, whose lines are `text.split('\n')`.
fn offset_of(line: i64, col: i64, lines: &[&str]) -> usize {
    let mut off = 0usize;
    for l in &lines[0..(line - 1) as usize] {
        off += l.chars().count() + 1; // +1 for the newline fix-text-line-start also counts.
    }
    off + (col - 1) as usize
}

fn span_offsets(span: &Span, lines: &[&str]) -> (usize, usize) {
    let start = offset_of(span.line, span.col, lines);
    let end = span.end.as_ref().expect("a real-source parse always has an end position");
    let end_off = offset_of(end.line, end.col, lines);
    (start, end_off)
}

/// Parse `text` (one literal's decoded content, or placeholder-substituted equivalent) into its
/// top-level forms, under a stable `<embedded-wat>` file label (never used for anything but the
/// error message, since offsets are computed from `lines`, not the label).
fn parse_forms(text: &str) -> Result<Vec<WatAST>, String> {
    parse_all_with_file(text, "<embedded-wat>").map_err(|e| format!("{e:?}"))
}

/// Walk `old`/`new` (same tree position each step) collecting every leaf whose extracted
/// source text differs. `old_lines`/`new_lines` are `old_text.split('\n')` / `new_text.split('\n')`
/// — the SAME split `wat::fix::fix-text-offset-of` uses, computed once by the caller.
fn diff_node(
    old: &WatAST,
    new: &WatAST,
    old_lines: &[&str],
    new_lines: &[&str],
    old_text: &str,
    new_text: &str,
    out: &mut Vec<Edit>,
) -> Result<(), String> {
    let old_children = old.children();
    let new_children = new.children();

    if old_children.is_empty() && new_children.is_empty() {
        // A leaf (or a genuinely empty List/Vector/Map/Set — comparing span text is harmless
        // either way: an empty form never differs, so this path only ever fires for real leaves).
        let (old_lo, old_hi) = span_offsets(old.span(), old_lines);
        let (new_lo, new_hi) = span_offsets(new.span(), new_lines);
        let old_slice = &old_text[byte_range_for_chars(old_text, old_lo, old_hi)];
        let new_slice = &new_text[byte_range_for_chars(new_text, new_lo, new_hi)];
        if old_slice != new_slice {
            out.push(Edit { old_lo, old_hi, new_text: new_slice.to_string() });
        }
        return Ok(());
    }

    if old_children.len() != new_children.len() {
        return Err(format!(
            "codemod restructured the tree (was {} children, now {}) at old span {:?} — \
             fix-text-apply is supposed to rewrite leaf text only; refusing to guess an edit",
            old_children.len(),
            new_children.len(),
            old.span()
        ));
    }

    for (oc, nc) in old_children.iter().zip(new_children.iter()) {
        diff_node(oc, nc, old_lines, new_lines, old_text, new_text, out)?;
    }
    Ok(())
}

/// Char-index range → byte range, since Rust string slicing is byte-indexed but our offsets
/// (mirroring wat's own `fix-text-offset-of`) are char-indexed.
fn byte_range_for_chars(s: &str, lo: usize, hi: usize) -> std::ops::Range<usize> {
    let mut byte_lo = s.len();
    let mut byte_hi = s.len();
    for (seen, (bi, _)) in s.char_indices().chain(std::iter::once((s.len(), '\0'))).enumerate() {
        if seen == lo {
            byte_lo = bi;
        }
        if seen == hi {
            byte_hi = bi;
            break;
        }
    }
    byte_lo..byte_hi
}

/// Parse both OLD and NEW decoded text and return the leaf-level [`Edit`]s between them, in
/// OLD-text ascending-offset order. Both must parse (a codemod that leaves either side
/// unparseable is a driver-level error, not a splice question) and must be structurally
/// isomorphic (see [`diff_node`]).
pub fn diff_decoded(old_text: &str, new_text: &str) -> Result<Vec<Edit>, String> {
    let old_forms = parse_forms(old_text)?;
    let new_forms = parse_forms(new_text)?;
    if old_forms.len() != new_forms.len() {
        return Err(format!(
            "top-level form count changed: {} -> {}",
            old_forms.len(),
            new_forms.len()
        ));
    }
    let old_lines: Vec<&str> = old_text.split('\n').collect();
    let new_lines: Vec<&str> = new_text.split('\n').collect();
    let mut edits = Vec::new();
    for (of, nf) in old_forms.iter().zip(new_forms.iter()) {
        diff_node(of, nf, &old_lines, &new_lines, old_text, new_text, &mut edits)?;
    }
    edits.sort_by_key(|e| e.old_lo);
    Ok(edits)
}

/// Map one [`Edit`] (decoded-text char offsets) through `span.char_map` to a verified RAW
/// char-offset range, or refuse it. `raw_chars` is the WHOLE file's raw source as a `Vec<char>`
/// (offsets in `span.char_map` are indices into exactly this).
pub fn verify_and_map(
    span: &LiteralSpan,
    old_decoded: &str,
    raw_chars: &[char],
    edit: &Edit,
) -> Result<(usize, usize, String), RefusedSplice> {
    let old_decoded_chars: Vec<char> = old_decoded.chars().collect();
    let old_slice: String = old_decoded_chars[edit.old_lo..edit.old_hi].iter().collect();

    let Some((raw_lo, raw_hi)) = span.raw_range_for(edit.old_lo, edit.old_hi) else {
        return Err(RefusedSplice {
            old_lo: edit.old_lo,
            old_hi: edit.old_hi,
            old_decoded_text: old_slice,
            raw_text: String::new(),
            reason: "empty or out-of-range decoded span (offset map has no entry)".to_string(),
        });
    };
    let raw_slice: String = raw_chars[raw_lo..raw_hi].iter().collect();
    if raw_slice != old_slice {
        return Err(RefusedSplice {
            old_lo: edit.old_lo,
            old_hi: edit.old_hi,
            old_decoded_text: old_slice,
            raw_text: raw_slice,
            reason: "raw source under this span is not char-for-char the decoded old text \
                      (an escape or elided line-continuation sits inside the changed region)"
                .to_string(),
        });
    }
    Ok((raw_lo, raw_hi, edit.new_text.clone()))
}

/// Apply verified `(raw_lo, raw_hi, new_text)` splices to `raw_chars`, right-to-left (by
/// descending `raw_lo`) so earlier offsets stay valid as later ones are spliced — the same
/// right-to-left discipline `wat::fix::fix-text-apply` itself uses. Splices must be
/// non-overlapping; panics (a driver bug, never a data condition) if they are not.
pub fn splice_raw(raw_chars: &[char], mut splices: Vec<(usize, usize, String)>) -> String {
    splices.sort_by_key(|(lo, _, _)| *lo);
    for w in splices.windows(2) {
        assert!(w[0].1 <= w[1].0, "overlapping splices: {:?} and {:?}", w[0], w[1]);
    }
    let mut out = String::new();
    let mut cursor = 0usize;
    for (lo, hi, new_text) in splices {
        out.extend(&raw_chars[cursor..lo]);
        out.push_str(&new_text);
        cursor = hi;
    }
    out.extend(&raw_chars[cursor..]);
    out
}

/// Run a recorded codemod over `text` exactly as documented
/// (`printf '["pathA"]\n' | ./target/release/wat ./wat-scripts/fixes/<fix>.wat`): write `text`
/// to a fresh temp `.wat` file, invoke `wat_binary codemod_path` with that one path on stdin,
/// and return the file's content afterward. Never touches the codemod's rewrite rules.
pub fn run_codemod_over_text(wat_binary: &Path, codemod_path: &Path, text: &str) -> io::Result<String> {
    let dir = std::env::temp_dir().join(format!("wat-codemod-driver-{}", std::process::id()));
    std::fs::create_dir_all(&dir)?;
    let tmp_path = dir.join(format!("probe-{}.wat", uuid::Uuid::new_v4()));
    std::fs::write(&tmp_path, text)?;
    run_codemod_batch(wat_binary, codemod_path, std::slice::from_ref(&tmp_path))?;
    let new_text = std::fs::read_to_string(&tmp_path)?;
    let _ = std::fs::remove_file(&tmp_path);
    Ok(new_text)
}

/// Run a recorded codemod ONCE over every path in `paths` — the literal brief usage
/// (`printf '["pathA" "pathB" …]\n' | ./target/release/wat ./wat-scripts/fixes/<fix>.wat`)
/// extended to many paths in a single process invocation ("one batch"), so converting a
/// file's several embedded-wat literals costs one subprocess, not one per literal. Each
/// path's file is read, converted, and written back IN PLACE by the codemod itself
/// (`:user::apply-each`); this function only drives the process and surfaces a failure.
pub fn run_codemod_batch(wat_binary: &Path, codemod_path: &Path, paths: &[std::path::PathBuf]) -> io::Result<()> {
    let mut stdin_payload = String::from("[");
    for (i, p) in paths.iter().enumerate() {
        if i > 0 {
            stdin_payload.push(' ');
        }
        stdin_payload.push('"');
        stdin_payload.push_str(&p.display().to_string());
        stdin_payload.push('"');
    }
    stdin_payload.push_str("]\n");

    let mut child = Command::new(wat_binary)
        .arg(codemod_path)
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()?;
    {
        use std::io::Write;
        child.stdin.take().unwrap().write_all(stdin_payload.as_bytes())?;
    }
    let output = child.wait_with_output()?;
    if !output.status.success() {
        return Err(io::Error::other(format!(
            "codemod batch ({} path(s)) exited {:?}\nstdout:\n{}\nstderr:\n{}",
            paths.len(),
            output.status.code(),
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr),
        )));
    }
    Ok(())
}

/// Is `text` (already placeholder-substituted, same-length) wat-shaped — the SAME
/// surface-agnostic contract the inline-wat lint uses (`is_inline_wat_form`,
/// `tests/lint/no_inlined_wat_in_tests.rs:80`): wat's own reader parses it to a List whose
/// head is a Keyword or a Symbol. A `catch_unwind` guards the same rare lexer-panic-on-
/// pathological-input gap the lint's version documents — most literals in a Rust source
/// file are ordinary prose/fixtures, not wat, and forcing them through the reader must
/// never crash the driver.
pub fn is_candidate_wat(placeholder_substituted_text: &str) -> bool {
    let result = std::panic::catch_unwind(|| crate::parser::parse_one_with_file(placeholder_substituted_text, "<embedded-wat-candidate>"));
    matches!(
        result,
        Ok(Ok(WatAST::List(items, _)))
            if matches!(items.first(), Some(WatAST::Keyword(..)) | Some(WatAST::Symbol(..)))
    )
}

/// One literal's outcome inside a file-level apply.
#[derive(Debug)]
pub enum LiteralOutcome {
    NotCandidate,
    Unchanged,
    Edited { changes: Vec<(String, String)> },
    Refused(Vec<RefusedSplice>),
    /// The codemod's own diff/parse failed for a reason unrelated to splicing (e.g. the
    /// codemod restructured the tree, or NEW text fails to parse) — reported, never applied.
    DiffFailed(String),
}

/// The result of running a codemod over one whole `.rs` file's embedded wat literals.
pub struct FileApplyResult {
    pub new_src: String,
    pub changed: bool,
    pub total_edits: usize,
    pub total_refused: usize,
    pub per_literal: Vec<LiteralOutcome>,
}

/// Apply `codemod_path` to every wat-shaped literal in `raw_src` (one Rust source file's
/// content), batched through ONE subprocess invocation covering every candidate literal in
/// THIS file (bounding the blast radius of a single malformed literal to one file, not the
/// whole corpus run). Never writes `raw_src`'s own file — the caller decides whether/where
/// to persist `FileApplyResult::new_src`.
pub fn apply_codemod_to_rust_source(wat_binary: &Path, codemod_path: &Path, raw_src: &str) -> io::Result<FileApplyResult> {
    let raw_chars: Vec<char> = raw_src.chars().collect();
    let spans = crate::embedded_wat::extract_literal_spans(&raw_chars);

    let mut candidates: Vec<(usize, String)> = Vec::new(); // (span index, placeholder-substituted decoded text)
    let mut per_literal: Vec<LiteralOutcome> = spans.iter().map(|_| LiteralOutcome::NotCandidate).collect();
    for (i, span) in spans.iter().enumerate() {
        let ph = crate::embedded_wat::replace_placeholders_preserving_len(&span.decoded);
        if is_candidate_wat(&ph) {
            candidates.push((i, ph));
        }
    }

    if candidates.is_empty() {
        return Ok(FileApplyResult { new_src: raw_src.to_string(), changed: false, total_edits: 0, total_refused: 0, per_literal });
    }

    let dir = std::env::temp_dir().join(format!("wat-codemod-driver-{}-{}", std::process::id(), uuid::Uuid::new_v4()));
    std::fs::create_dir_all(&dir)?;
    let mut temp_paths = Vec::with_capacity(candidates.len());
    for (i, ph) in &candidates {
        let p = dir.join(format!("lit-{i}.wat"));
        std::fs::write(&p, ph)?;
        temp_paths.push(p);
    }

    run_codemod_batch(wat_binary, codemod_path, &temp_paths)?;

    let mut all_splices: Vec<(usize, usize, String)> = Vec::new();
    let mut total_edits = 0usize;
    let mut total_refused = 0usize;

    for ((span_i, old_ph), temp_path) in candidates.iter().zip(temp_paths.iter()) {
        let new_ph = std::fs::read_to_string(temp_path)?;
        let span = &spans[*span_i];
        if new_ph == *old_ph {
            per_literal[*span_i] = LiteralOutcome::Unchanged;
            continue;
        }
        match diff_decoded(old_ph, &new_ph) {
            Ok(edits) if edits.is_empty() => {
                per_literal[*span_i] = LiteralOutcome::Unchanged;
            }
            Ok(edits) => {
                let mut refusals = Vec::new();
                let mut changes = Vec::new();
                for edit in &edits {
                    match verify_and_map(span, old_ph, &raw_chars, edit) {
                        Ok(triple) => {
                            let old_decoded_chars: Vec<char> = old_ph.chars().collect();
                            let old_text: String = old_decoded_chars[edit.old_lo..edit.old_hi].iter().collect();
                            changes.push((old_text, edit.new_text.clone()));
                            all_splices.push(triple);
                        }
                        Err(refusal) => refusals.push(refusal),
                    }
                }
                total_edits += changes.len();
                total_refused += refusals.len();
                per_literal[*span_i] = if refusals.is_empty() {
                    LiteralOutcome::Edited { changes }
                } else {
                    LiteralOutcome::Refused(refusals)
                };
            }
            Err(e) => {
                per_literal[*span_i] = LiteralOutcome::DiffFailed(e);
            }
        }
    }
    let _ = std::fs::remove_dir_all(&dir);

    let changed = !all_splices.is_empty();
    let new_src = if changed { splice_raw(&raw_chars, all_splices) } else { raw_src.to_string() };

    Ok(FileApplyResult { new_src, changed, total_edits, total_refused, per_literal })
}

#[cfg(test)]
mod probe_255_80 {
    use super::*;
    use crate::embedded_wat::{extract_literal_spans, replace_placeholders_preserving_len};

    /// FM 2-bis PROBE (STOP-1 gate) — can the recorded codemod reach a wat-shaped literal
    /// embedded in a Rust string, through escapes AND a `\` line continuation AND a
    /// `format!` placeholder, and splice back faithfully?
    ///
    /// The brief asks for this literal in `src/macros/tests.rs`. MEASURED (not assumed):
    /// that file has exactly 3 backslash characters in total (one `\` line continuation at
    /// line 403, one `\"` pair at line 1360) and NEITHER sits anywhere near a
    /// `:wat::core::<24>` type-position token — confirmed by a full character-by-character
    /// scan, not a sample. So no literal in that exact file satisfies "escape AND type
    /// marker" at once; the brief's premise does not hold for that file (doctrine: the code
    /// wins, say so). The nearest real, committed, non-hypothetical analog — same crate,
    /// same `macros` module, genuine production source (not a test fixture) — is
    /// `aggregate_kwargs_companion_source` in `src/macros/parse.rs`, whose `format!` template
    /// carries THREE `\` line continuations, a `\"`-escaped pair wrapping a `{bare_name}`
    /// placeholder, AND two type-position `:wat::WatAST` tokens plus a head-of-bracket
    /// `:wat::core::Vector` — strictly harder than the brief's ask, not easier.
    ///
    /// This test's body ran BEFORE item 4's corpus apply (when the literal still spelled
    /// `:wat::core::Vector`/`:wat::WatAST`) and proved the full composition: decode → run the
    /// recorded codemod on a temp `.wat` → diff leaf-by-leaf → map to raw offsets → verify →
    /// splice → recompiled syntactically → re-decoded to exactly the codemod's output (that
    /// run is what the commit message and SCORE report). Item 4's real driver run then
    /// converted this SAME literal for real, in place, in the actual file — so what this test
    /// asserts now is the POST-conversion invariant: running the codemod over the already-
    /// converted literal is a true no-op (idempotence), on the exact literal the probe used,
    /// through the exact same escape/continuation/placeholder machinery.
    #[test]
    fn codemod_reaches_a_literal_through_escapes_continuation_and_placeholder() {
        let manifest = env!("CARGO_MANIFEST_DIR");
        let path = Path::new(manifest).join("src/macros/parse.rs");
        let raw_src = std::fs::read_to_string(&path).expect("read src/macros/parse.rs");
        let raw_chars: Vec<char> = raw_src.chars().collect();

        let spans = extract_literal_spans(&raw_chars);
        let span = spans
            .iter()
            .find(|s| s.decoded.contains("defmacro {bare_name}") && s.decoded.contains("wat.type/Vector"))
            .expect("the aggregate_kwargs_companion_source format! literal must still be present, now converted");

        // Measured precondition: this literal still carries the escape/continuation shape
        // (255.80's apply only ever rewrites a leaf's span text, never the surrounding
        // escapes), and is now post-conversion (wat.type/AST, not :wat::WatAST).
        let raw_slice: String = raw_chars[span.raw_quote_start..span.raw_quote_end].iter().collect();
        assert!(raw_slice.contains('\\'), "probe literal must still contain a raw backslash (escape/continuation)");
        assert!(span.decoded.contains("wat.type/AST"), "probe literal must carry the now-converted type spelling");
        assert!(span.decoded.contains("wat.type/Vector"));
        assert!(!span.decoded.contains(":wat::WatAST"), "the old type-position spelling must be gone");
        assert!(!span.decoded.contains(":wat::core::Vector"), "the old type-position spelling must be gone");

        let old_ph = replace_placeholders_preserving_len(&span.decoded);
        assert_eq!(old_ph.chars().count(), span.decoded.chars().count());

        let wat_binary = Path::new(manifest).join("target/release/wat");
        let codemod = Path::new(manifest).join("wat-scripts/fixes/types-to-wat-type.wat");
        assert!(wat_binary.is_file(), "release binary must be built first (cargo build --release)");
        let new_ph = run_codemod_over_text(&wat_binary, &codemod, &old_ph)
            .expect("codemod composition over a decoded embedded-wat literal must succeed");

        // Idempotence: nothing left to convert on the already-converted literal.
        assert_eq!(old_ph, new_ph, "a second run over the already-converted literal must be a no-op");
        let edits = diff_decoded(&old_ph, &new_ph).expect("old/new decoded text must diff cleanly");
        assert!(edits.is_empty(), "expected no further edits post-conversion; got {edits:?}");
    }
}

#[cfg(test)]
mod driver_tests {
    use super::*;

    fn wat_binary() -> std::path::PathBuf {
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("target/release/wat")
    }
    fn types_to_wat_type() -> std::path::PathBuf {
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("wat-scripts/fixes/types-to-wat-type.wat")
    }

    /// Item 5 — "the driver's idempotence on a fixture .rs": a file with a genuine
    /// type-position embedded literal converts once, then a SECOND run over the already-
    /// converted source finds nothing left to edit.
    #[test]
    fn driver_is_idempotent_on_a_fixture_rs_file() {
        let fixture = r#"
fn make_form() -> &'static str {
    "(:wat::core::defn :my::inc [x <- :wat::core::i64] -> :wat::core::i64 (:wat::i64::+ x 1))"
}
"#;
        let first = apply_codemod_to_rust_source(&wat_binary(), &types_to_wat_type(), fixture)
            .expect("first apply must succeed");
        assert!(first.changed, "the fixture's two type-position i64 sites must convert");
        assert_eq!(first.total_refused, 0);
        assert!(first.new_src.contains("wat.type/i64"));
        assert!(!first.new_src.contains(":wat::core::i64"));

        let second = apply_codemod_to_rust_source(&wat_binary(), &types_to_wat_type(), &first.new_src)
            .expect("second apply must succeed");
        assert!(!second.changed, "a re-run over already-converted source must find nothing to edit");
        assert_eq!(second.total_edits, 0);
        assert_eq!(second.new_src, first.new_src);
    }

    /// Item 5 — "a splice that must be refused": a type-position keyword whose colon is
    /// written as a `\x3a` hex escape. It decodes to a genuine `:wat::core::i64` type-
    /// position token (the codemod converts it), but the RAW source under that leaf's span
    /// is `\x3awat::core::i64` (18 raw chars for a 15-char decoded token) — not char-for-char
    /// the decoded text — so the splice must be refused, never silently applied.
    #[test]
    fn driver_refuses_a_splice_when_an_escape_hides_inside_the_changed_span() {
        let fixture = "fn f() -> &'static str { \"(:my::f [x <- \\x3awat::core::i64])\" }\n";
        let result = apply_codemod_to_rust_source(&wat_binary(), &types_to_wat_type(), fixture)
            .expect("apply must run (refusal is a reported outcome, not an error)");
        assert_eq!(result.total_edits, 0, "the one edit here must be refused, not applied");
        assert_eq!(result.total_refused, 1);
        assert!(!result.changed, "a file with only a refused splice must come back unchanged");
        assert_eq!(result.new_src, fixture);

        let refusals: Vec<_> = result
            .per_literal
            .iter()
            .filter_map(|o| if let LiteralOutcome::Refused(r) = o { Some(r) } else { None })
            .collect();
        assert_eq!(refusals.len(), 1);
        assert_eq!(refusals[0].len(), 1);
        assert_eq!(refusals[0][0].old_decoded_text, ":wat::core::i64");
        assert_ne!(refusals[0][0].raw_text, refusals[0][0].old_decoded_text);
    }

    /// A file with no wat-shaped literals at all is left alone — the common case across most
    /// of the corpus, and the non-vacuity floor: this must not misfire as "candidate" noise.
    #[test]
    fn driver_leaves_ordinary_rust_source_untouched() {
        let fixture = "fn f(x: i32) -> i32 { x + 1 }\n// a comment mentioning :wat::core::i64\n";
        let result = apply_codemod_to_rust_source(&wat_binary(), &types_to_wat_type(), fixture)
            .expect("apply must succeed");
        assert!(!result.changed);
        assert_eq!(result.total_edits, 0);
        assert_eq!(result.total_refused, 0);
        assert_eq!(result.new_src, fixture);
    }
}
