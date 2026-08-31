//! Arc 278 C1 — **a table that says `MINIMUM` may not compute a MEAN.**
//!
//! ── THE CLASS ────────────────────────────────────────────────────────────────────────────────
//!
//! Commit `89e8c3ed0` — *"the instrument measured the wrong estimator — 106 accumulators, mean ->
//! minimum"* — **moved the labels and left the arithmetic.** `render_phase_table`'s header read
//! `MINIMUM of {RUNS} runs` while its `stat()` returned `sum / xs.len()`, and 96 per-test
//! accumulators still did `x += sample` inside `for _ in 0..RUNS` and then `x /= r`.
//!
//! That is a worse failure than an unfixed bug: every number the tables printed for a month
//! carried a label asserting an estimator nothing computed. The estimator matters on its own
//! merits — `89e8c3ed0`'s own measurement showed a first arm paying **287.4 ms against 11.5 and
//! 11.4 for identical work**, which is exactly the shape a mean cannot survive and a minimum
//! shrugs off.
//!
//! ── THE RULE ─────────────────────────────────────────────────────────────────────────────────
//!
//! **A file that prints a `MINIMUM of` header may not average across runs.** Averaging is spelled
//! two ways here and both are caught:
//!
//! | shape | example |
//! |---|---|
//! | divide by the run count, or by anything bound from it | `x /= r;` · `let (a, b) = (a / r, b / r);` · `*x /= r;` · `/ RUNS as f64` |
//! | divide by the LENGTH of a sample bag | `sum as f64 / xs.len() as f64` |
//!
//! The rule is **FILE-scoped, deliberately.** A partially converted file is precisely the defect
//! this gate exists to remove: the label then lies for the unconverted half while the converted
//! half makes the file look swept. A file must be fully converted to pass.
//!
//! ── WHAT IS NOT A VIOLATION ──────────────────────────────────────────────────────────────────
//!
//! Dividing by a **UNIT COUNT** — facts, pairs, elements, iterations — is normalisation, not
//! averaging, and it stays. `src/rete/kernel/tests/mod.rs::calibrate_mark_ns` is the worked
//! example and the shape to copy: it takes `min` across BATCHES and divides by `PER_BATCH`,
//! because "ns per mark pair" is a rate and the batch size is its denominator. The discriminator
//! is exact — **divide by a unit count stays; divide by the number of REPEATED MEASUREMENTS is
//! the defect.**
//!
//! ── THE ONE WAY TO CHEAT, NAMED HERE SO IT IS NOT AVAILABLE QUIETLY ──────────────────────────
//!
//! This gate can be made green by editing 43 headers from `MINIMUM` to `MEAN`. **That is
//! `89e8c3ed0` performed in the opposite direction** — a label moved to match the code instead of
//! the code moved to match the label — and it is a failure, not a fix. If some single figure
//! genuinely wants a mean, that figure's header says MEAN *and the reason is written beside it*;
//! it is a finding to surface, never a default.

use std::path::{Path, PathBuf};

/// The promise. A file containing this in a printed header is claiming an estimator.
const MINIMUM_HEADER: &str = "MINIMUM of";

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

/// Blank every comment and every string/char literal, preserving line structure.
///
/// The scan below must read CODE. The headers themselves live in format strings full of `/` and
/// `{}`, and the prose in these files explains this very rule — a reader that did not strip both
/// would flag its own rationale. Line count and column count are preserved (each stripped byte
/// becomes a space, newlines survive) so reported line numbers stay true.
fn code_only(src: &str) -> String {
    let b = src.as_bytes();
    let mut out = vec![b' '; b.len()];
    let mut i = 0usize;
    while i < b.len() {
        let c = b[i];
        match c {
            b'\n' => {
                out[i] = b'\n';
                i += 1;
            }
            b'/' if i + 1 < b.len() && b[i + 1] == b'/' => {
                while i < b.len() && b[i] != b'\n' {
                    i += 1;
                }
            }
            b'/' if i + 1 < b.len() && b[i + 1] == b'*' => {
                // Rust block comments nest.
                let mut depth = 1usize;
                i += 2;
                while i < b.len() && depth > 0 {
                    if b[i] == b'\n' {
                        out[i] = b'\n';
                        i += 1;
                    } else if i + 1 < b.len() && b[i] == b'/' && b[i + 1] == b'*' {
                        depth += 1;
                        i += 2;
                    } else if i + 1 < b.len() && b[i] == b'*' && b[i + 1] == b'/' {
                        depth -= 1;
                        i += 2;
                    } else {
                        i += 1;
                    }
                }
            }
            b'r' | b'b' if raw_string_hashes(b, i).is_some() => {
                let (open_end, hashes) = raw_string_hashes(b, i).expect("just checked");
                i = skip_raw_string(b, &mut out, open_end, hashes);
            }
            b'"' => {
                i += 1;
                while i < b.len() {
                    if b[i] == b'\\' {
                        if b[i + 1..].first() == Some(&b'\n') {
                            out[i + 1] = b'\n';
                        }
                        i += 2;
                    } else if b[i] == b'"' {
                        i += 1;
                        break;
                    } else {
                        if b[i] == b'\n' {
                            out[i] = b'\n';
                        }
                        i += 1;
                    }
                }
            }
            b'\'' if is_char_literal(b, i) => {
                i += 1;
                while i < b.len() && b[i] != b'\'' {
                    i += if b[i] == b'\\' { 2 } else { 1 };
                }
                i += 1;
            }
            _ => {
                out[i] = c;
                i += 1;
            }
        }
    }
    String::from_utf8_lossy(&out).into_owned()
}

/// `r"`, `r#"`, `br##"` … → (offset just past the opening quote, hash count). `None` otherwise.
fn raw_string_hashes(b: &[u8], i: usize) -> Option<(usize, usize)> {
    // Must not be the tail of an identifier (`for_r"` is not a raw string; `let r = …` is not one
    // either, and this is the check that keeps a bare `r` binding from being read as a literal).
    if i > 0 && (b[i - 1].is_ascii_alphanumeric() || b[i - 1] == b'_') {
        return None;
    }
    let mut j = i;
    if b[j] == b'b' {
        j += 1;
        if b.get(j) != Some(&b'r') {
            return None;
        }
    }
    if b.get(j) != Some(&b'r') {
        return None;
    }
    j += 1;
    let mut hashes = 0usize;
    while b.get(j) == Some(&b'#') {
        hashes += 1;
        j += 1;
    }
    if b.get(j) == Some(&b'"') {
        Some((j + 1, hashes))
    } else {
        None
    }
}

fn skip_raw_string(b: &[u8], out: &mut [u8], mut i: usize, hashes: usize) -> usize {
    while i < b.len() {
        if b[i] == b'\n' {
            out[i] = b'\n';
            i += 1;
        } else if b[i] == b'"' && b[i + 1..].iter().take(hashes).all(|h| *h == b'#') {
            return i + 1 + hashes;
        } else {
            i += 1;
        }
    }
    i
}

/// `'a'` / `'\n'` is a literal; `'static` is a lifetime and must not desync the scanner.
fn is_char_literal(b: &[u8], i: usize) -> bool {
    match b.get(i + 1) {
        Some(b'\\') => true,
        Some(_) => b.get(i + 2) == Some(&b'\''),
        None => false,
    }
}

fn ident_at(b: &[u8], mut i: usize) -> (String, usize) {
    let start = i;
    while i < b.len() && (b[i].is_ascii_alphanumeric() || b[i] == b'_') {
        i += 1;
    }
    (String::from_utf8_lossy(&b[start..i]).into_owned(), i)
}

/// Every identifier bound from the run count — `let r = RUNS as f64;` yields `r`.
///
/// This is what makes the gate immune to the scoping error that produced the strike's first,
/// WRONG population count. That count was a `/= r` grep: it read `fire /= r;` and generalised
/// from it, so it was blind to `let (a, b) = (a / r, b / r);` and reported `rank_and_instrument`
/// as ZERO where the file has 21. The gate does not guess a spelling — it learns the divisor's
/// NAME from its binding and then looks for that name after any `/`.
fn runs_aliases(code: &str) -> Vec<String> {
    let mut out = vec!["RUNS".to_string()];
    for line in code.lines() {
        let t = line.trim_start();
        let Some(rest) = t.strip_prefix("let ") else { continue };
        let Some((lhs, rhs)) = rest.split_once('=') else { continue };
        if !rhs.split(|c: char| !(c.is_alphanumeric() || c == '_')).any(|w| w == "RUNS") {
            continue;
        }
        for w in lhs.split(|c: char| !(c.is_alphanumeric() || c == '_')) {
            if !w.is_empty() && w != "mut" && !out.contains(&w.to_string()) {
                out.push(w.to_string());
            }
        }
    }
    out
}

/// Averaging divides on one line of stripped code, as `(column, snippet)`.
fn averaging_divides(line: &str, aliases: &[String]) -> Vec<(usize, String)> {
    let b = line.as_bytes();
    let mut hits = Vec::new();
    let mut i = 0usize;
    while i < b.len() {
        if b[i] != b'/' {
            i += 1;
            continue;
        }
        let mut j = i + 1;
        if b.get(j) == Some(&b'=') {
            j += 1;
        }
        while b.get(j) == Some(&b' ') {
            j += 1;
        }
        // (a) divide by the run count, or by any name bound from it.
        let (word, after) = ident_at(b, j);
        if aliases.contains(&word) {
            hits.push((i + 1, line[i..after.min(line.len())].trim().to_string()));
            i = after;
            continue;
        }
        // (b) divide by the LENGTH of a sample bag — `sum as f64 / xs.len() as f64`. A `.len()`
        // denominator under a MINIMUM header is a mean over the samples by another spelling; it
        // is how `render_phase_table::stat` spelled it, and a reader that only knew (a) would
        // have gone green on the strike's own flagship instance.
        let mut k = j;
        while k < b.len() && (b[k].is_ascii_alphanumeric() || b[k] == b'_' || b[k] == b'.') {
            k += 1;
        }
        let path = &line[j..k];
        if path.ends_with(".len") && line[k..].starts_with("()") {
            hits.push((i + 1, line[i..k + 2].trim().to_string()));
            i = k;
            continue;
        }
        i += 1;
    }
    hits
}

#[test]
fn a_minimum_header_may_not_average() {
    let manifest = env!("CARGO_MANIFEST_DIR");
    let src_root = Path::new(manifest).join("src");
    let mut files = Vec::new();
    collect_rs(&src_root, &mut files);
    files.sort();

    // NON-VACUITY, first half: the walk must find the tree. A typo'd path finding zero files
    // would make this gate pass forever while reading nothing.
    assert!(
        files.len() > 100,
        "the estimator-label walk found only {} .rs files under src/ — it is not looking at the \
         tree it claims to guard",
        files.len()
    );

    let mut headered = 0usize;
    let mut header_occurrences = 0usize;
    let mut violations: Vec<String> = Vec::new();
    for f in &files {
        let Ok(src) = std::fs::read_to_string(f) else { continue };
        if !src.contains(MINIMUM_HEADER) {
            continue;
        }
        headered += 1;
        header_occurrences += src.matches(MINIMUM_HEADER).count();

        let rel = f
            .strip_prefix(manifest)
            .unwrap_or(f)
            .to_string_lossy()
            .replace('\\', "/");
        let code = code_only(&src);
        let aliases = runs_aliases(&code);
        let mut in_file: Vec<String> = Vec::new();
        for (n, line) in code.lines().enumerate() {
            for (col, snippet) in averaging_divides(line, &aliases) {
                in_file.push(format!("  {rel}:{}:{col}  `{snippet}`", n + 1));
            }
        }
        if !in_file.is_empty() {
            violations.push(format!(
                "{rel} — {} averaging divide(s), divisor name(s) {:?}:\n{}",
                in_file.len(),
                aliases,
                in_file.join("\n")
            ));
        }
    }

    // NON-VACUITY, second half: the headers must STILL be there. If they were renamed away — the
    // one way to cheat this gate, named in the module header — the file set would empty out and
    // this control would go quietly green while guarding nothing. That is the exact failure mode
    // of a check that outlives its subject, so the subject is asserted, not assumed.
    assert!(
        headered >= 8 && header_occurrences >= 35,
        "only {headered} file(s) and {header_occurrences} occurrence(s) of `{MINIMUM_HEADER}` \
         remain under src/ (expected at least 8 / 35). Either the cost census shrank, or headers \
         were relabelled to MEAN to silence this gate — which is arc 278 C1 performed in reverse. \
         If a figure genuinely wants a mean, say so beside it and lower this floor deliberately."
    );

    assert!(
        violations.is_empty(),
        "⛔ A TABLE LABELLED `{MINIMUM_HEADER}` IS COMPUTING A MEAN — arc 278 C1.\n\n\
         {}\n\n\
         Each site above averages across repeated measurements while its file's header promises \
         the minimum.\n\
         The fix is the ARITHMETIC, never the label:\n\n    \
         let mut x = f64::INFINITY;\n    \
         for _ in 0..RUNS {{ /* … */ x = x.min(sample); }}\n    \
         // and DELETE the `x /= r`\n\n\
         `src/rete/kernel/tests/mod.rs::calibrate_mark_ns` is the model, and it also shows what \
         is NOT a violation: dividing by a UNIT COUNT (facts, pairs, elements, iterations) is \
         normalisation and stays. Only dividing by the number of REPEATED MEASUREMENTS is this \
         defect.\n\n\
         ⛔ Relabelling the header to MEAN also makes this green and is NOT the fix — it is \
         commit 89e8c3ed0 run backwards. If one figure genuinely wants a mean, surface it as a \
         finding and write the reason beside it.",
        violations.join("\n\n")
    );
}

/// The gate's reader, proven against all three spellings of the defect — and against the wrong
/// regex that produced the strike's first population count.
///
/// This is a counter-proof, not a demonstration. The scoping pass that preceded this gate used
/// `^\s*[a-z_]+ */= r;`, shaped from the first site it read. It could not see the destructured
/// form, and it therefore reported a file holding 21 sites as holding zero. **A gate carrying
/// that same blind spot would have shipped a half-swept corpus under a full-sweep green.** So the
/// reader is asserted against every spelling directly: cripple it to `/= r` and this test reddens
/// before any corpus file is even read.
#[test]
fn the_reader_sees_every_spelling_of_an_averaging_divide() {
    let sample = r#"
fn f() {
    const RUNS: usize = 3;
    let r = RUNS as f64;
    let mut fire = 0.0;
    fire /= r;                            // spelling 1 — the one the wrong regex saw
    let (a, b) = (a / r, b / r);          // spelling 2 — invisible to a `/= r` grep
    for x in &mut xs { *x /= r; }         // spelling 3 — a deref inside a loop
    let m = sum as f64 / xs.len() as f64; // spelling 4 — a mean by sample-bag length
    let per = total / RUNS as f64;        // spelling 5 — the bare const, no alias
}
"#;
    let code = code_only(sample);
    let aliases = runs_aliases(&code);
    // EXACT, not a membership probe. The alias set of this sample is fully determined: the bare
    // const, the `let r = RUNS as f64;` binding, and the `let per = total / RUNS as f64;` binding
    // — a reader that invented a fourth name, or dropped `per`, would be a different reader.
    assert_eq!(
        aliases,
        vec!["RUNS".to_string(), "r".to_string(), "per".to_string()],
        "the reader learned the wrong set of divisor names from the sample. Learning the NAME \
         from its binding is the whole reason this gate is not a `/= r` grep."
    );

    let per_line: Vec<usize> = code
        .lines()
        .map(|l| averaging_divides(l, &aliases).len())
        .collect();
    let total: usize = per_line.iter().sum();
    assert_eq!(
        total, 6,
        "the reader saw {total} averaging divide(s) in the five-spelling sample, expected 6 \
         (spelling 2 carries two). Per line: {per_line:?}. If this dropped to 1, the reader has \
         been narrowed back to `/= r` — the exact blind spot that made the first population count \
         report 37 sites where there are 96, and `rank_and_instrument.rs` as zero where it has 21."
    );

    // And it must NOT flag legitimate normalisation, or the gate would forbid `calibrate_mark_ns`.
    let normalisation = r#"
fn g() {
    const PER_BATCH: u64 = 200_000;
    let ns = t0.elapsed().as_nanos() as f64 / PER_BATCH as f64;
    let per_element = delta / elements as f64;
    let pct = 100.0 * net / total_net;
    let half = span / 2.0;
}
"#;
    let ncode = code_only(normalisation);
    let naliases = runs_aliases(&ncode);
    let nhits: Vec<(usize, String)> = ncode
        .lines()
        .flat_map(|l| averaging_divides(l, &naliases))
        .collect();
    assert!(
        nhits.is_empty(),
        "the reader flagged legitimate normalisation {nhits:?} — dividing by a UNIT COUNT is a \
         rate, not a mean, and `calibrate_mark_ns` depends on it being allowed."
    );

    // The stripper must blank prose and format strings, or this gate would flag the very
    // paragraphs that explain it — and every `MINIMUM of` header contains a `/` or two.
    let prose = r#"
fn h() {
    // the old shape was `x /= r;` and it lied
    let s = "raw / r  MINIMUM of {RUNS} runs, sum / xs.len()";
}
"#;
    let pcode = code_only(prose);
    let paliases = vec!["r".to_string(), "RUNS".to_string()];
    let phits: Vec<(usize, String)> = pcode
        .lines()
        .flat_map(|l| averaging_divides(l, &paliases))
        .collect();
    assert!(
        phits.is_empty(),
        "the reader flagged a comment or a string literal {phits:?} — it must read CODE. The \
         headers themselves live in format strings, and this file's own prose names the defect."
    );
}
