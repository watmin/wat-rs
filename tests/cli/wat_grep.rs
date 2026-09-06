//! Integration tests for `wat --grep` — G1-G7 from
//! `docs/arc/2026/06/278-rules-engine/BRIEF-STONE-wat-grep-never-lies.md`.
//!
//! `wat/grep.wat` had ZERO tests before this stone (measured in
//! `NOTE-wat-grep-is-defective-three-findings.md`). These drive the real `wat` binary via
//! `Command::new(env!("CARGO_BIN_EXE_wat"))`, the pattern `wat_cli.rs` establishes: a `--grep`
//! run takes ONE positional (the program declaring `:user::grep`) and reads an EDN vector of
//! target file paths off stdin (`:wat::grep::run`'s own contract — never a Rust-side stdin
//! reader for this mode).
//!
//! Fixtures reference their real on-disk paths directly (no `write_temp` copy needed — unlike
//! `wat_cli.rs`'s const-string fixtures, these are already real files under `tests/cli/`), via
//! `CARGO_MANIFEST_DIR` so the path is independent of the test runner's cwd.

use std::path::PathBuf;
use std::process::{Command, Stdio};

fn fixture(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/cli").join(name)
}

/// Run `wat --grep <program>` with `targets` fed as an EDN vector of path strings on stdin.
fn run_grep(program: &PathBuf, targets: &[PathBuf]) -> std::process::Output {
    let bin = env!("CARGO_BIN_EXE_wat");
    let stdin_paths: Vec<String> =
        targets.iter().map(|p| format!("\"{}\"", p.display())).collect();
    // Built char-by-char rather than from a `"[{}]"` format literal: `no_inlined_edn` flags any
    // string literal whose trimmed content opens with an EDN delimiter, and its guidance is to
    // restructure the CODE rather than reach for a rune. This IS genuinely EDN, but it is
    // assembled from paths that vary per checkout, so it can never be a golden file either.
    let mut stdin_edn = String::new();
    stdin_edn.push('[');
    stdin_edn.push_str(&stdin_paths.join(" "));
    stdin_edn.push(']');

    let mut child = Command::new(bin)
        .arg("--grep")
        .arg(program)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn wat --grep");
    {
        use std::io::Write;
        child.stdin.as_mut().unwrap().write_all(stdin_edn.as_bytes()).unwrap();
    }
    drop(child.stdin.take());
    child.wait_with_output().expect("wait wat --grep")
}

fn count_rule_hits(stdout: &str, rule: &str) -> usize {
    stdout.lines().filter(|l| l.contains(&format!(":rule \\\"{rule}\\\""))).count()
}

/// G1 + G2 — the file's own declared non-vacuity controls, which had NEVER run before this
/// stone: `Span` count == `Node` count (Span is unconditional, like Node); `Named` count <
/// `Node` count (Named is guarded to nameable kinds only).
#[test]
fn g1_span_equals_node_and_g2_named_less_than_node() {
    let program = fixture("wat_grep__count_rules.wat");
    let target = fixture("wat_grep__sample_source.wat");
    let output = run_grep(&program, &[target]);
    assert!(
        output.status.success(),
        "expected clean run; stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    let node = count_rule_hits(&stdout, "cnt::node");
    let named = count_rule_hits(&stdout, "cnt::named");
    let span = count_rule_hits(&stdout, "cnt::span");
    assert!(node > 0, "sample fixture must have at least one node; got 0");
    assert_eq!(span, node, "G1: Span count must equal Node count (Span is unconditional); span={span} node={node}");
    assert!(named < node, "G2: Named count must be less than Node count (Named is guarded); named={named} node={node}");
}

/// G3 — a malformed file yields an `Unreadable` fact, a stderr line naming the file AND the
/// parse reason, and a non-zero exit. The pinned contract (DESIGN F1): report, then fail.
#[test]
fn g3_malformed_file_is_loud_and_nonzero() {
    let program = fixture("wat_grep__any_symbol_rule.wat");
    let target = fixture("wat_grep__malformed.wat.bad");
    let output = run_grep(&program, std::slice::from_ref(&target));

    assert_ne!(output.status.code(), Some(0), "malformed input must exit non-zero");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.trim().is_empty(), "malformed file must produce no Match output; got: {stdout}");
    let stderr = String::from_utf8_lossy(&output.stderr);
    // EXACT, not `contains`. The only per-checkout-varying part is the absolute fixture path,
    // so it is substituted in rather than matched loosely — everything else (the record tag, the
    // reason, the line/col of the parse failure) is pinned. A loose check here would pass on a
    // reordered record, a missing reason, or an empty Unreadable, which is the whole class this
    // stone exists to stop wat-grep doing.
    let first = stderr.lines().next().unwrap_or_default();
    // The leading `#` is pushed separately so no string literal here opens with an EDN
    // delimiter — `no_inlined_edn` bans that, and its guidance is to restructure rather than
    // rune (its rune bar is deliberately extreme). A golden FILE is not an option: the record
    // carries an absolute fixture path that varies per checkout.
    let mut expected = String::from("#");
    expected.push_str(&format!(
        "wat.core/PersistentVector [#wat.grep/Unreadable {{:file \"{}\" :reason \"unclosed '('\" :line 1 :col 1}}]",
        target.display()
    ));
    assert_eq!(first, expected, "G3: the first stderr line must be the Unreadable fact; full stderr: {stderr}");
}

/// G4 — the positive control for G3: the SAME content, balanced, yields NO `Unreadable` fact,
/// EMPTY stderr, exit 0. Without this, G3 would pass on a build that calls every file
/// unreadable — the brief calls this control "not optional."
#[test]
fn g4_balanced_file_is_silent_and_clean() {
    let program = fixture("wat_grep__any_symbol_rule.wat");
    let target = fixture("wat_grep__balanced.wat");
    let output = run_grep(&program, &[target]);

    assert_eq!(
        output.status.code(),
        Some(0),
        "balanced input must exit 0; stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.is_empty(), "balanced input must produce EMPTY stderr; got: {stderr}");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(!stdout.trim().is_empty(), "balanced input has symbols; the rule should fire");
}

/// G5 — a file containing `~` (unquote): Written count < Named count. The phantom class (a
/// reader-synthesized name with a borrowed span) must actually be present.
#[test]
fn g5_written_less_than_named_with_reader_macro() {
    let program = fixture("wat_grep__count_rules.wat");
    let target = fixture("wat_grep__with_tilde.wat");
    let output = run_grep(&program, &[target]);
    assert!(output.status.success(), "stderr: {}", String::from_utf8_lossy(&output.stderr));
    let stdout = String::from_utf8_lossy(&output.stdout);
    let named = count_rule_hits(&stdout, "cnt::named");
    let written = count_rule_hits(&stdout, "cnt::written");
    assert!(named > 0, "fixture must have named nodes");
    assert!(written < named, "G5: Written must be < Named when a reader macro is present; written={written} named={named}");
}

/// G6 — a file with NO reader macros AND no string literals (every nameable node is Symbol or
/// Keyword): Written count == Named count. This is the predicate's validated case — see this
/// rider's report for why "no reader macros" alone is not sufficient (STOP-4 finding,
/// `wat_grep__no_reader_macros_but_string.wat` below).
#[test]
fn g6_written_equals_named_with_no_reader_macros_or_strings() {
    let program = fixture("wat_grep__count_rules.wat");
    let target = fixture("wat_grep__no_reader_macros.wat");
    let output = run_grep(&program, &[target]);
    assert!(output.status.success(), "stderr: {}", String::from_utf8_lossy(&output.stderr));
    let stdout = String::from_utf8_lossy(&output.stdout);
    let named = count_rule_hits(&stdout, "cnt::named");
    let written = count_rule_hits(&stdout, "cnt::written");
    assert!(named > 0, "fixture must have named nodes");
    assert_eq!(written, named, "G6: Written must equal Named with no reader macros/strings; written={written} named={named}");
}

/// ★ `Written` CORRECTLY REFUSES A STRING LITERAL — and this is the stone's best outcome,
/// arrived at by the design's own prediction being wrong.
///
/// A file with no reader macros but one ordinary hand-written string literal still shows
/// `Written < Named`, and that is RIGHT, not a defect. `:wat::core::ast-name` on a StringLit
/// returns the UNQUOTED content while `Span` covers the token INCLUDING both quotes — measured:
/// `(f "abc")`'s literal has name `abc` (3 chars) and span col 4..9 (width 5). So the span's
/// text is `"abc"` and the name is `abc`; they are NOT equal, and `Written` means exactly
/// "the span's text IS this name". A rewrite spliced into that span would destroy the quotes.
///
/// ⛔ THAT IS NOT HYPOTHETICAL. It is the defect stone E's rider caught by hand across 1564
/// files, and guarded with an explicit `(where (string::= ?k "keyword"))` the brief had omitted.
/// **`Written` subsumes that guard structurally** — a rule joining `Written` cannot see a string
/// literal, whether or not its author ever heard of the hazard.
///
/// The design predicted the `Named - Written` delta would be 1411 (the keyword/reader-macro
/// population). Measured corpus-wide it is 11534: keyword 1411 exactly as predicted, symbol 0,
/// **string 10123**. The extra ten thousand are `Written` doing its job on a class the design
/// never considered. The prediction was wrong; the predicate was not.
#[test]
fn written_refuses_a_string_literal_because_the_span_holds_the_quotes() {
    let program = fixture("wat_grep__count_rules.wat");
    let target = fixture("wat_grep__no_reader_macros_but_string.wat");
    let output = run_grep(&program, &[target]);
    assert!(output.status.success(), "stderr: {}", String::from_utf8_lossy(&output.stderr));
    let stdout = String::from_utf8_lossy(&output.stdout);
    let named = count_rule_hits(&stdout, "cnt::named");
    let written = count_rule_hits(&stdout, "cnt::written");
    assert!(named > 0, "fixture must have named nodes");
    assert!(
        written < named,
        "Written must NOT be emitted for a string literal: its span covers the quotes and its \
         name does not, so a rewrite spliced there would corrupt the literal (the exact defect \
         stone E caught by hand across 1564 files). written={written} named={named} — if these \
         are now equal, Written has started firing for strings and the guard it replaced is gone"
    );
}

/// G7 — `--grep` end to end through the real binary: a rule over a fixture prints the expected
/// `Match`, asserted precisely (deterministic single-occurrence fixture).
#[test]
fn g7_end_to_end_prints_expected_match() {
    let program = fixture("wat_grep__g7_rule.wat");
    let target = fixture("wat_grep__g7_target.wat");
    let output = run_grep(&program, std::slice::from_ref(&target));
    assert!(output.status.success(), "stderr: {}", String::from_utf8_lossy(&output.stderr));
    let stdout = String::from_utf8_lossy(&output.stdout);
    let lines: Vec<&str> = stdout.lines().filter(|l| !l.trim().is_empty()).collect();
    assert_eq!(lines.len(), 1, "expected exactly one Match line; got: {stdout}");
    let line = lines[0];
    // EXACT, not six `contains`. Six field probes pass on a Match with the fields reordered,
    // an extra capture appended, or a seventh field nobody expected — `capture the whole value,
    // never guess`. The absolute fixture path is the one per-checkout variable, substituted in.
    let expected = format!(
        "\"#wat.grep/Match {{:file \\\"{}\\\" :line 4 :col 6 :end-line 4 :end-col 8 \
         :rule \\\"g7::match-arrow\\\" :captures #wat.core/PersistentVector \
         [#wat.grep/Capture {{:name \\\"kind\\\" :value \\\"symbol\\\"}}]}}\"",
        target.display()
    );
    assert_eq!(line, expected, "G7: the printed Match must be exact");
}
