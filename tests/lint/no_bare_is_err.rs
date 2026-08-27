//! THE BARE-`is_err()` LINT — bans an `assert!` statement whose body mentions `.is_err()` and
//! checks nothing about WHICH error.
//!
//! Stone L's whole thesis (`git log -1 4b49f3c5c`): a bare `is_err()` is satisfied by a
//! retirement, a typo, a renamed fixture, and the intended defect ALIKE. It found six sites in
//! the corpus that were passing for a reason unrelated to what they claimed to pin — three where
//! an unrelated upstream gate fires first, and two still blocked on arc 255's unbuilt builtin
//! registry (`--check` exits 0, so there is genuinely no error to name yet). Phase 2 migrated 142
//! of 150; this lint is the permanent wall on the now near-zero population so the shape cannot
//! silently return.
//!
//! Port of the validated Python census
//! (`docs/arc/2026/06/296-diagnostics-fully-edn/PROBE-296-L-bare-is-err-census.py`), which itself
//! records a wrong-instrument lesson worth repeating here: a line-based grep undercounted by more
//! than half (70 vs. 150) because `assert!( ... )` spans lines: detection here is
//! **statement-scoped and paren-balanced**, matching every `assert!( ... )` regardless of how
//! many lines it wraps. Char and string literals are skipped while balancing parens — a stray
//! `'('` inside a char literal (e.g. `!w.starts_with('(')`) desyncs a naive counter and swallows
//! the rest of the file into one "statement"; that exact defect was found IN the Python
//! instrument, after it had already produced a number, and is fixed here from the start
//! (`[[feedback_state_what_the_instrument_can_see_before_quoting_it]]`).
//!
//! A statement is BARE if its body contains `.is_err()` and none of: `matches!`,
//! `assert_check_error_present`, `assert_startup_error`, `.kind`, `unwrap_err`, `err_kind`,
//! `StartupError::`. It scans `tests/` only (mirroring the census; this is a test-idiom lint, not
//! a `src/` one).
//!
//! `rune:lint(<name>)` is the repo's project-custom-lint exemption form (owner `lint` = the
//! project lint suite, NOT a grimoire spell); excusare audits the reason so "legitimate" stays
//! honest. Exemption form for a future site: `// rune:lint(bare-is-err) — <reason>`, per-offense
//! (on the offending line or the one above), mirroring `no_inlined_edn`'s per-offense marker
//! rather than `no_inlined_wat`'s file-wide skip.
//!
//! ⛔ TWO SITES ARE FROZEN IN AN EXPLICIT ALLOWLIST, NOT FIXED. Both live in
//! `tests/wat_lang/probe_undefined_builtin_resolves.rs`, both `#[ignore]`d, both waiting on arc
//! 255's unbuilt builtin registry — `--check` exits 0 on their fixtures today, so there is no
//! error at all to name a kind of, let alone a stable discriminant. The allowlist is keyed by
//! **file + test function name**, never by line number (which moves under any unrelated edit in
//! the file), and each entry carries its own blocker inline. A COUNT cannot distinguish "+1 new,
//! -1 fixed" from "nothing happened", and its failure text cannot name the offender — this lint
//! freezes NAMES `[[feedback_a_gate_freezes_names_never_a_count]]`.

use std::path::{Path, PathBuf};

/// The two sites that cannot be migrated today, identified by file + enclosing test fn — NEVER
/// by line number. Each blocker must still hold for the entry to stay exempt; if arc 255's
/// registry lands and one of these fixtures starts raising a real, stable error, migrate it to
/// `assert_startup_error!`/`matches!` and remove it from this list rather than leaving it stale.
const FROZEN_ALLOWLIST: &[(&str, &str, &str)] = &[
    (
        "tests/wat_lang/probe_undefined_builtin_resolves.rs",
        "wrong_operator_leaf_is_a_check_error",
        "BLOCKED on arc 255's unbuilt builtin registry — `--check` exits 0 on \
         probe_undefined_builtin_resolves_wrong_leaf.wat.bad today (verified), so there is no \
         error at all to name a kind of. #[ignore]d; unlock when arc 255's registry lands.",
    ),
    (
        "tests/wat_lang/probe_undefined_builtin_resolves.rs",
        "bogus_leaf_under_known_namespace_is_a_check_error",
        "BLOCKED on arc 255's unbuilt builtin registry — `--check` exits 0 on \
         probe_undefined_builtin_resolves_bogus.wat.bad today (verified), so there is no error at \
         all to name a kind of. #[ignore]d; unlock when arc 255's registry lands.",
    ),
    (
        "tests/rete/probe_arc278_seq1b_list_hofs.rs",
        "list_map_is_not_vector",
        "BLOCKED on a RULING, not on work. The test's name and docstring claim CONTAINER \
         PRESERVATION — that `map` over a List must not satisfy a Vector return. Arc 118.2a \
         flipped map/filter/take/drop LAZY, so they now return a Stream and preserve NO container \
         at all; the sibling test `list_hofs_preserve_container` in the same file documents the \
         flip. Repairing the retired angle-bracket spelling makes it raise \
         ReturnTypeMismatch{expected: Vector, got: Stream} — factually what fires, but asserting \
         it would present a Stream/Vector mismatch as proof of List preservation, which is \
         precisely the drift this lint exists to catch. Grounded 2026-08-27 by the Phase 3 rider \
         and NOT migrated, deliberately. The honest dispositions are to RETIRE the probe or \
         RE-POINT it at a genuinely list-preserving op (reverse/concat) — and re-pointing changes \
         what a test named `list_map_is_not_vector` tests, so it is the builder's call.",
    ),
];

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

/// Yield (line_no, statement_text) for each paren-balanced `assert!( ... )` in `src`.
/// Skips char literals (`'('`, `'\''`, `'x'`) and string literals (`"…"`, with `\`-escapes)
/// while balancing so an embedded `(` or `)` inside either never desyncs the depth counter.
fn assert_statements(src: &str) -> Vec<(usize, String)> {
    let bytes = src.as_bytes();
    let mut out = Vec::new();
    let mut search_from = 0usize;
    while let Some(rel) = src[search_from..].find("assert!") {
        let call_start = search_from + rel;
        // Find the opening paren after `assert!` (allow whitespace, e.g. `assert! (`).
        let mut i = call_start + "assert!".len();
        while i < bytes.len() && (bytes[i] as char).is_whitespace() {
            i += 1;
        }
        if i >= bytes.len() || bytes[i] != b'(' {
            // Not actually a call (e.g. `assert!` inside a doc comment word) — skip past it.
            search_from = call_start + "assert!".len();
            continue;
        }
        let open = i;
        let mut depth = 0i32;
        let mut j = open;
        while j < bytes.len() {
            let c = bytes[j] as char;
            // Char literal: 'x' or '\'' or '\n' etc. — a lightweight scan: `'`, then either an
            // escape (`\` + one char) or a single char, then a closing `'`.
            if c == '\'' {
                if j + 1 < bytes.len() && bytes[j + 1] == b'\\' && j + 3 < bytes.len() && bytes[j + 3] == b'\'' {
                    j += 4;
                    continue;
                } else if j + 2 < bytes.len() && bytes[j + 2] == b'\'' {
                    j += 3;
                    continue;
                }
                // Not a recognizable char literal (e.g. a lifetime `'a`) — treat as ordinary char.
                j += 1;
                continue;
            }
            if c == '"' {
                j += 1;
                while j < bytes.len() && bytes[j] != b'"' {
                    j += if bytes[j] == b'\\' { 2 } else { 1 };
                }
                j += 1; // consume closing quote
                continue;
            }
            if c == '(' {
                depth += 1;
            } else if c == ')' {
                depth -= 1;
                if depth == 0 {
                    j += 1;
                    break;
                }
            }
            j += 1;
        }
        let stmt = &src[call_start..j.min(bytes.len())];
        let line_no = src[..call_start].matches('\n').count() + 1;
        out.push((line_no, stmt.to_string()));
        search_from = j.max(call_start + 1);
    }
    out
}

const KIND_MARKERS: [&str; 7] = [
    "matches!",
    "assert_check_error_present",
    "assert_startup_error",
    ".kind",
    "unwrap_err",
    "err_kind",
    "StartupError::",
];

/// Find the `fn <name>` that textually encloses `line_no` (1-indexed) in `src` — a simple
/// nearest-preceding-`fn`-declaration scan, sufficient for identifying the allowlist entries
/// (which are stated by file + fn name, not line).
fn enclosing_fn_name(src: &str, line_no: usize) -> Option<String> {
    let mut best: Option<String> = None;
    for (idx, line) in src.lines().enumerate() {
        if idx + 1 > line_no {
            break;
        }
        if let Some(rest) = line.trim_start().strip_prefix("fn ") {
            let name: String = rest
                .chars()
                .take_while(|c| c.is_alphanumeric() || *c == '_')
                .collect();
            if !name.is_empty() {
                best = Some(name);
            }
        } else if let Some(rest) = line.trim_start().strip_prefix("pub fn ") {
            let name: String = rest
                .chars()
                .take_while(|c| c.is_alphanumeric() || *c == '_')
                .collect();
            if !name.is_empty() {
                best = Some(name);
            }
        }
    }
    best
}

#[test]
fn tests_carry_no_bare_is_err() {
    let manifest = env!("CARGO_MANIFEST_DIR");
    let mut files = Vec::new();
    collect_rs(&Path::new(manifest).join("tests"), &mut files);
    files.sort();

    let mut violations = Vec::new();
    for f in &files {
        if f.file_name().and_then(|n| n.to_str()) == Some("no_bare_is_err.rs") {
            continue;
        }
        let Ok(src) = std::fs::read_to_string(f) else { continue };
        let rel = f.strip_prefix(manifest).unwrap_or(f).display().to_string();

        for (line_no, stmt) in assert_statements(&src) {
            if !stmt.contains(".is_err()") {
                continue;
            }
            if KIND_MARKERS.iter().any(|k| stmt.contains(k)) {
                continue;
            }
            // Per-offense exemption: the marker on the offending line or the one above.
            let lines: Vec<&str> = src.lines().collect();
            let has_rune = (line_no.saturating_sub(2)..line_no)
                .filter_map(|i| lines.get(i))
                .any(|l| l.contains("// rune:lint(bare-is-err)"));
            if has_rune {
                continue;
            }
            let fn_name = enclosing_fn_name(&src, line_no).unwrap_or_default();
            if FROZEN_ALLOWLIST
                .iter()
                .any(|(af, afn, _)| *af == rel && *afn == fn_name)
            {
                continue;
            }
            violations.push(format!("{rel}:{line_no}  (in fn {fn_name})"));
        }
    }

    let allowlist_block = FROZEN_ALLOWLIST
        .iter()
        .map(|(f, n, why)| format!("  {f}  fn {n}\n    BLOCKER: {why}"))
        .collect::<Vec<_>>()
        .join("\n");

    assert!(
        violations.is_empty(),
        "\n\n🔥🔥🔥 BARE `is_err()` ASSERTIONS — {} site(s) assert only that startup/check FAILED,\n\
         never WHICH error. A retirement, a typo, a renamed fixture, and the intended defect all\n\
         satisfy `assert!(result.is_err())` identically — this is arc 296 Stone L's whole thesis\n\
         (git log -1 4b49f3c5c).\n\
         \n\
         THE FIX: name the discriminant. Prefer `assert_startup_error!(result, StartupError::<Kind>(..))`\n\
         or a `matches!(err, StartupError::<Kind>(..))` guard; for a CheckErrors payload use\n\
         `assert_check_error_present!`. Ground the real error first with\n\
         `./target/release/wat --check <fixture>` — do not guess the kind, capture it.\n\
         \n\
         EXEMPT a site whose error genuinely has no stable discriminant yet (e.g. blocked on an\n\
         unbuilt registry) with a per-site `// rune:lint(bare-is-err) — <reason>`, on the offending\n\
         line or the one above.\n\
         \n\
         FROZEN ALLOWLIST (identity = file + test fn name, NEVER line number — two sites, both\n\
         #[ignore]d, both blocked on arc 255's unbuilt builtin registry):\n{}\n\
         \n\
         Drive it to ZERO. Offenders:\n\n{}\n",
        violations.len(),
        allowlist_block,
        violations.join("\n"),
    );
}
