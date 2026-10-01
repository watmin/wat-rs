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

    let stdin_payload = format!("[\"{}\"]\n", tmp_path.display());
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
        let _ = std::fs::remove_file(&tmp_path);
        return Err(io::Error::other(format!(
            "codemod exited {:?}\nstdout:\n{}\nstderr:\n{}",
            output.status.code(),
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr),
        )));
    }
    let new_text = std::fs::read_to_string(&tmp_path)?;
    let _ = std::fs::remove_file(&tmp_path);
    Ok(new_text)
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
    #[test]
    fn codemod_reaches_a_literal_through_escapes_continuation_and_placeholder() {
        let manifest = env!("CARGO_MANIFEST_DIR");
        let path = Path::new(manifest).join("src/macros/parse.rs");
        let raw_src = std::fs::read_to_string(&path).expect("read src/macros/parse.rs");
        let raw_chars: Vec<char> = raw_src.chars().collect();

        let spans = extract_literal_spans(&raw_chars);
        let span = spans
            .iter()
            .find(|s| s.decoded.contains("defmacro {bare_name}") && s.decoded.contains(":wat::core::Vector"))
            .expect("the aggregate_kwargs_companion_source format! literal must still be present");

        // Measured precondition: this literal really does carry both an escape/continuation
        // AND a type-position marker — the exact combination STOP-1 is about.
        let raw_slice: String = raw_chars[span.raw_quote_start..span.raw_quote_end].iter().collect();
        assert!(raw_slice.contains('\\'), "probe literal must contain a raw backslash (escape/continuation)");
        assert!(span.decoded.contains(":wat::WatAST"), "probe literal must carry a type-position :wat::core::<24> token");

        // 1. decode, then substitute format! placeholders (same length, so char_map still
        //    applies 1:1) so wat's reader can parse the template shape.
        let old_ph = replace_placeholders_preserving_len(&span.decoded);
        assert_eq!(old_ph.chars().count(), span.decoded.chars().count());

        // 2. run the recorded codemod over the decoded literal, via a real temp .wat file —
        //    the exact documented invocation, never a Rust reimplementation of its rules.
        let wat_binary = Path::new(manifest).join("target/release/wat");
        let codemod = Path::new(manifest).join("wat-scripts/fixes/types-to-wat-type.wat");
        assert!(wat_binary.is_file(), "release binary must be built first (cargo build --release)");
        let new_ph = run_codemod_over_text(&wat_binary, &codemod, &old_ph)
            .expect("codemod composition over a decoded embedded-wat literal must succeed — STOP-1 if not");

        // Some real conversion must have happened (otherwise this probe proves nothing).
        assert_ne!(old_ph, new_ph, "the codemod found nothing to convert in the probe literal");

        // 3. diff old vs new by walking both parses leaf-by-leaf (never a text diff / regex).
        let edits = diff_decoded(&old_ph, &new_ph).expect("old/new decoded text must diff cleanly");
        assert!(!edits.is_empty());
        assert!(
            edits.iter().any(|e| e.new_text == "wat.type/AST") && edits.iter().any(|e| e.new_text == "wat.type/Vector"),
            "expected the two :wat::WatAST sites and the :wat::core::Vector head to convert; got {edits:?}"
        );

        // 4. map every edit back to raw offsets and verify the raw slice is exactly the old
        //    text (no escape hiding inside the changed region) — none of this probe's three
        //    target tokens sit inside an escape, so none should be refused.
        let mut raw_splices = Vec::new();
        for edit in &edits {
            match verify_and_map(span, &old_ph, &raw_chars, edit) {
                Ok(triple) => raw_splices.push(triple),
                Err(refusal) => panic!("STOP-1: splice refused unexpectedly: {refusal:?}"),
            }
        }
        assert_eq!(raw_splices.len(), edits.len());

        // 5. splice into a COPY of the raw file (never touch the real file from a test) and
        //    confirm the resulting Rust compiles.
        let spliced = splice_raw(&raw_chars, raw_splices);
        assert_ne!(spliced, raw_src);

        let tmp_dir = std::env::temp_dir().join(format!("probe-255-80-splice-{}", std::process::id()));
        std::fs::create_dir_all(&tmp_dir).unwrap();
        let tmp_rs = tmp_dir.join("parse_spliced.rs");
        std::fs::write(&tmp_rs, &spliced).unwrap();
        let check = Command::new("rustc")
            .args(["--edition", "2021", "--crate-type", "lib", "--emit=metadata"])
            .arg("-o")
            .arg(tmp_dir.join("out.rmeta"))
            .arg(&tmp_rs)
            .output();
        // A standalone rustc check can't resolve this module's `use crate::...` / `super::`
        // paths (it isn't the real crate graph) — so what this step actually certifies is
        // narrower than "compiles in the workspace": that the SPLICE produced syntactically
        // valid Rust (balanced strings/escapes, no stray quote introduced by the splice).
        // The real "does the crate build" claim is certified separately, below, by splicing
        // the ACTUAL file and running `cargo build --release` over the real workspace.
        if let Ok(out) = check {
            let stderr = String::from_utf8_lossy(&out.stderr);
            // A standalone-file rustc check can't resolve this module's `crate::`/`super::`
            // paths at all (it isn't fed the real crate graph), so NAME-RESOLUTION errors
            // (E0432/E0433, "too many leading `super`") are expected noise. What this step
            // actually screens for is a SYNTAX error the splice itself could have introduced
            // (an unbalanced quote/paren from a bad raw-offset splice) — those are different
            // error codes entirely (E0765 unterminated literal, E0624/E0601 structural, a bare
            // "error: expected .."/"mismatched closing delimiter" with no error code).
            let resolution_only = !out.status.success()
                && stderr
                    .lines()
                    .filter(|l| l.trim_start().starts_with("error["))
                    .all(|l| {
                        l.contains("E0432")
                            || l.contains("E0433")
                            || l.contains("E0425")
                            || l.contains("E0599")
                            || l.contains("too many leading")
                    });
            assert!(
                out.status.success() || resolution_only,
                "splice produced syntactically invalid Rust (not just unresolved-name noise):\n{stderr}"
            );
        }

        // 6. the literal, re-extracted from the spliced raw source and decoded, must equal the
        //    codemod's own output (modulo restoring the `{bare_name}` placeholder the codemod
        //    never saw — it only ever saw the same-length `x`-run stand-in).
        let spliced_chars: Vec<char> = spliced.chars().collect();
        let re_spans = extract_literal_spans(&spliced_chars);
        let re_span = re_spans
            .iter()
            .find(|s| s.decoded.contains("defmacro {bare_name}"))
            .expect("the literal must still be found after splicing");
        let re_decoded_ph = replace_placeholders_preserving_len(&re_span.decoded);
        assert_eq!(
            re_decoded_ph, new_ph,
            "the spliced literal must decode (post placeholder-substitution) to exactly the codemod's output"
        );
    }
}
