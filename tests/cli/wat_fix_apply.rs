//! Arc 282 — the wall under `:wat::fix::fix-text-apply`.
//!
//! Before this stone an edit carried `(offset, old-len, new-text)`: the applier knew HOW MANY
//! characters to overwrite and never learned WHAT IT BELIEVED WAS THERE, so it could not tell a
//! correct edit from a catastrophic one and spliced either. The corruption that forced the stone
//! (`255/NOTE-a-name-the-reader-manufactured-has-no-text-to-rewrite.md`) had a PERFECTLY CORRECT
//! `old-len` — it was the span width — and a rule that believed it was replacing something else.
//!
//! ⛔ Which is why these two fixtures are a PAIR, and why the negative one uses a claim of the
//! SAME LENGTH as the source text. A bounds check catches an overrunning claim; only comparing
//! the claim against the source catches a same-length lie. A test that only exercised the
//! overrun case would pass against a wall that guards almost nothing.
//!
//! The fixtures are byte-identical but for the claim: `"xxxxx"` (a lie) versus `"world"` (true).

use std::process::{Command, Stdio};

fn fixture(name: &str) -> std::path::PathBuf {
    std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/cli").join(name)
}

fn run(name: &str) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_wat"))
        .arg(fixture(name))
        .stdin(Stdio::null())
        .output()
        .expect("spawn wat")
}

/// NEGATIVE — a same-length wrong claim must be refused, and the message must name all three
/// things a codemod author needs: where, what was claimed, what is actually there.
#[test]
fn a_same_length_wrong_claim_is_refused() {
    let output = run("wat_fix_apply__liar.wat");
    assert_ne!(output.status.code(), Some(0), "a lying edit must not exit clean");
    let stderr = String::from_utf8_lossy(&output.stderr);
    // rune:lint(loose-assert) — the raise arrives wrapped in a panic envelope carrying a
    // per-run thread id and a src/*.rs frame line that drifts with unrelated edits; pinning the
    // whole payload would be a span-pinned golden with none of a golden's value. The three
    // substrings below ARE the claim under test: offset, belief, and reality.
    for needle in ["offset 6", "claims old-text", "xxxxx", "world"] {
        assert!(stderr.contains(needle), "the refusal must name {needle:?}; got: {stderr}");
    }
    assert!(
        String::from_utf8_lossy(&output.stdout).trim().is_empty(),
        "a refused edit must produce no rewritten source"
    );
}

/// POSITIVE — the identical edit with a TRUE claim still splices. Without this the negative
/// control is satisfied by a `fix-text-apply` that refuses everything.
#[test]
fn a_truthful_claim_still_splices() {
    let output = run("wat_fix_apply__truthful.wat");
    assert_eq!(output.status.code(), Some(0), "stderr: {}", String::from_utf8_lossy(&output.stderr));
    assert_eq!(String::from_utf8_lossy(&output.stdout).trim(), "\"hello there\"");
}
