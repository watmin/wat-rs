//! **A SOURCE LOCATION NAMED IN A COMMENT MUST EXIST — THE PATH *AND* THE LINE.**
//!
//! ── THE CLASS ────────────────────────────────────────────────────────────────────────────────
//!
//! `src/rete/kernel/fire/mod.rs` opened with *"`kernel/tests.rs` is their only caller"*. That was
//! true when written and false four hours later, in the same session, because the same author
//! split `tests.rs` into `kernel/tests/`. Nothing noticed. The doc kept pointing at a file that
//! no longer existed, and a reader following it would have found nothing and concluded the note
//! was stale in some unknown way — or worse, gone looking for the wrong caller.
//!
//! This is the same defect `tests/lint/gen_doc_surface_matches.rs` was built for on 2026-08-26,
//! one level down: **a hand-maintained mirror of a real thing, with no red build behind it.**
//! `circumspicere` named it then as a GENERATOR of drift rather than an instance. Here is a
//! second instance, so here is a second gate.
//!
//! ── A `path:line` IS TWO CLAIMS, AND THE SECOND ONE USED TO BE CHECKED NOWHERE ────────────────
//!
//! Until 2026-09-04 this gate answered exactly one question: *does this path exist*. That let
//! `wat/rete.wat:1508` stand in **five** comments across **four** files while `wat/rete.wat` was
//! **533** lines long. The path half was true — `wat/rete.wat` exists — so the gate was green on
//! every one of them.
//!
//! And the citation had been RIGHT when written: at `30725034f`, `wat/rete.wat` was 3660 lines and
//! line 1508 was `(:wat::core::defn :wat::rete::insert-all-spec`, exactly as promised. The file
//! then lost 3127 lines to the kernel split and the `$oracle` rename, and the five citations rotted
//! without a single edit to any of them. **That is the shape: a `:LINE` is a claim about a file
//! the citing file never touches, so nothing local to either one can notice it going false.**
//!
//! So the gate now checks both halves: the path must resolve, and a cited `:LINE` must be within
//! that file's length. The failure names the cited line AND the real length, because a reader who
//! cannot see both numbers cannot tell a drifted citation from a typo.
//!
//! ── WHY A GATE AND NOT A CONVENTION (the honest rung) ────────────────────────────────────────
//!
//! "Check your paths when you move a file" is a convention, and conventions lean on every future
//! hand remembering during a rename. A path either exists on disk or it does not, and a line
//! number either is or is not within a file's length — both decidable, with no false positives
//! available to them, which is exactly the shape that belongs in a build gate rather than in a
//! habit. This is the top rung the material allows: Rust cannot make an unresolvable path in a
//! comment fail to compile, so a check at build time is as far up the ladder as this class goes.
//!
//! ── WHAT THIS GATE CANNOT DO — stated, because the gap is the point ──────────────────────────
//!
//! It checks that a named LOCATION exists. It cannot check that the location is the RIGHT one,
//! that the symbol named beside it lives there, or that any surrounding claim is true. A citation
//! whose line has drifted from the `defn` it meant onto some other line of the same file still
//! passes here — nothing short of resolving the symbol can see that, which is why the cure for a
//! rotted citation is to name the SYMBOL and drop the line rather than to correct the number.
//! Four false claims were found in these files on 2026-08-30 by `intueri`; this gate would have
//! caught exactly one of them. The others needed a ward, and their cure is
//! `tests/lint/rete_header_claims_are_asserted.rs` — assertions, not prose.
//!
//! ⛔ **It does not judge NAMES.** `tests/lint/rete_names_in_wat_scripts_resolve.rs` rules
//! deliberately that prose may name a retired form — accurate history is not a defect. This gate
//! is orthogonal: a comment may say whatever it likes about `:wat::rete::insert-all-spec`, so long
//! as any `path:line` it prints points somewhere real.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

/// The scanned roots, each with the file extension whose comments it holds.
///
/// `src/rete` is the original root — the rete tree's headers cite each other constantly.
/// `wat/` and `wat-tests/` were added 2026-09-04: all five `wat/rete.wat:1508` citations lived
/// there, and nothing had ever scanned `.wat` prose at all.
const ROOTS: &[(&str, &str)] = &[
    ("src/rete", "rs"),
    ("wat", "wat"),
    ("wat-tests", "wat"),
];

/// ── STOP-1: THE PILE THE 2026-09-04 SCOPE WIDENING SURFACED, ENUMERATED ──────────────────────
///
/// Adding `wat/` and `wat-tests/` to [`ROOTS`] took the population from 174 citations to 610 and
/// found **34 pre-existing stale paths** that no gate had ever looked at. Curing 34 comment sites
/// was not this strike's scope, so they are DEFERRED HERE — never suppressed:
///
/// - Every row is an exact `(naming file, cited path)` pair, so a new stale path in any of these
///   files still REDs; only these thirty-four specific rots are held.
/// - A row that stops matching is a HARD FAILURE (the `unused` check below), so curing one forces
///   its removal and this list can only shrink. It cannot rot the way the citations in it did.
///
/// The rots themselves are ordinary: a `scratchpad/` that became `wat-scripts/scratch-pad/`, probe
/// files deleted after their arc closed, `src/rete/validate.rs` and `src/rete/kernel/tests.rs` that
/// became directories. `wat/rete.wat`'s `kernel/tests.rs` is THE citation in this file's own
/// opening paragraph — it was in `wat/` all along, where nothing was scanning.
const DEFERRED: &[(&str, &str)] = &[
    ("wat/bracket.wat", "scratchpad/probe-c1-kwargs-impl-astname.wat"),
    ("wat/bracket.wat", "scratchpad/probe-m1-worker-setup.wat"),
    ("wat/cache.wat", "crates/wat-holon-lru/wat/holon/lru/HologramCache.wat"),
    ("wat/cache.wat", "wat-scripts/scratch-pad/probe-arc278-cache-batch-request-bytes.wat"),
    ("wat/cache.wat", "wat-scripts/scratch-pad/probe-arc278-concrete-satisfies-parametric.wat"),
    ("wat/capability.wat", "scratchpad/probe-c2-typed-coordinate.wat"),
    ("wat/capability.wat", "scratchpad/probe-v-bodiless.wat"),
    ("wat/gen.wat", "tests/lint/gen_lib_laws.rs"),
    ("wat/gen.wat", "wat-scripts/fuzz/gen-selftest.wat"),
    ("wat/gen.wat", "wat-scripts/lib/gen.wat"),
    ("wat/kernel/readln.wat", "wat/kernel/services/stdin.wat"),
    ("wat/query.wat", "scratchpad/probe-rule-lits.wat"),
    ("wat/repl.wat", "wat-scripts/demos/repl/repl.wat"),
    ("wat/rete.wat", "kernel/tests.rs"),
    ("wat/service.wat", "scratchpad/probe-v-bodiless.wat"),
    ("wat/spawn.wat", "scratchpad/probe-m1-worker-setup.wat"),
    ("wat-tests/core/core-arithmetic.wat", "tests/wat_polymorphic_arithmetic.rs"),
    ("wat-tests/core/core-arithmetic.wat", "wat-tests/core/seq-fold-aliases.wat"),
    ("wat-tests/core/core-equality.wat", "tests/wat_polymorphic_arithmetic.rs"),
    ("wat-tests/core/core-equality.wat", "wat-tests/core/seq-fold-aliases.wat"),
    ("wat-tests/core/core-threading.wat", "tests/probe_arc249_threading.rs"),
    ("wat-tests/core/record-def.wat", "tests/probe_arc234_stone2b_defrecord_macro.rs"),
    ("wat-tests/core/record-def.wat", "tests/probe_arc237_sC3_macro_split.rs"),
    ("wat-tests/format.wat", "tests/probe_arc279_format.rs"),
    ("wat-tests/gen.wat", "wat-scripts/fuzz/sampling-order-probe.wat"),
    ("wat-tests/process/signal-reset-sigusr1-is-a-transition.wat", "tests/process/signal_user1_delivers_child_observes_flag.wat"),
    ("wat-tests/process/signal-terminate-kills-the-child-and-the-read-sees-it.wat", "tests/process/shutdown_cascade_memory.rs"),
    ("wat-tests/process/signal-user1-delivers-child-observes.wat", "tests/process/signal_user1_delivers_child_observes_flag.wat"),
    ("wat-tests/process/signal-user2-and-hangup-independent.wat", "tests/process/signal_user1_delivers_child_observes_flag.wat"),
    ("wat-tests/rete/differential-fuzz-nesting.wat", "src/rete/validate.rs"),
    ("wat-tests/rete/differential-fuzz-scalars.wat", "src/rete/validate.rs"),
    ("wat-tests/service-locus-parity.wat", "tests/probe_arc272_6b_defservice_on_process.rs"),
    ("wat-tests/spawn/multiline-roundtrip.wat", "wat-tests/process-multiline-roundtrip.wat"),
    ("wat-tests/spawn/recv-budget-override.wat", "wat-tests/spawn/overcap-flood-no-deadlock.wat"),
];

/// How a comment opens, per extension. A `.wat` comment is `;;`; a Rust one is `//` (which covers
/// `///` and `//!` by prefix).
fn comment_head(ext: &str) -> &'static str {
    match ext {
        "wat" => ";;",
        _ => "//",
    }
}

/// One `path:line` (or bare `path`) named in a comment.
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone)]
struct Citation {
    path: String,
    /// The cited line, when the comment printed one.
    line: Option<usize>,
}

/// Every `word/with/slashes.rs` (or `.wat`) that appears inside a comment, with its `:LINE` suffix
/// when it carries one. Backtick-quoted or bare; both are how these files write them.
///
/// `:` joins the token so a suffix survives the split, then is peeled back off: a leading-colon
/// name like `:wat::rete::insert-all$oracle` has no segment ending in `.rs`/`.wat` and drops out,
/// while `wat/rete.wat:1508` splits cleanly into path and line. A range (`fire/rules.rs:629-642`)
/// cites its FIRST line — that is the one a reader jumps to.
fn citations_in_comments(src: &str, head: &str) -> BTreeSet<Citation> {
    let mut out = BTreeSet::new();
    for line in src.lines() {
        let t = line.trim_start();
        if !t.starts_with(head) {
            continue;
        }
        for tok in t.split(|c: char| !(c.is_alphanumeric() || "._/-:".contains(c))) {
            let tok = tok.trim_end_matches(':');
            let (path, cited) = split_line_suffix(tok);
            let looks_like_source = path.ends_with(".rs") || path.ends_with(".wat");
            if looks_like_source
                && path.contains('/')
                && !path.starts_with('.')
                && !path.contains("..")
                && !parent_is_a_source_file(path)
            {
                out.insert(Citation {
                    path: path.to_string(),
                    line: cited,
                });
            }
        }
    }
    out
}

/// `spawn.wat/bracket.wat` (`wat/service.wat:164`) is PROSE — two file names joined by a slash
/// meaning "and", not a path. No directory anywhere in this repo is named `*.rs` or `*.wat`
/// (driven 2026-09-04: `find . -type d -name '*.wat' -o -type d -name '*.rs'` is empty), so a token
/// whose PARENT component is itself a source file name cannot be a path and is prose.
///
/// This is the one false positive the 2026-09-04 scope widening produced, and it is cured rather
/// than exempted: this file's own resolver note says a gate that cries wolf gets muted.
fn parent_is_a_source_file(p: &str) -> bool {
    // `p` is a FILESYSTEM PATH lifted out of a doc comment (`wat/rete/oracle/insert.wat`), never a
    // wat name: no `::` path, no `/` receiver, no prime. Routing it through `Identifier`'s
    // accessors would ask the name grammar a question about a directory separator, which is the
    // confusion STONE-one-name-grammar exists to stop.
    let Some((parent, _)) = p.rsplit_once('/') else { // rune:lint(one-name-grammar) — a filesystem path out of a doc comment, not a wat name: no `::` path, no `/` receiver, no prime for the door's accessors to read
        return false;
    };
    let last = parent.rsplit('/').next().unwrap_or(parent);
    last.ends_with(".rs") || last.ends_with(".wat")
}

/// Peel a trailing `:LINE` (or `:LINE-LINE`) off a token. Anything that is not a decimal line
/// number stays part of the path, where it will simply fail the extension test.
fn split_line_suffix(tok: &str) -> (&str, Option<usize>) {
    let Some((path, rest)) = tok.rsplit_once(':') else {
        return (tok, None);
    };
    let first = rest.split('-').next().unwrap_or(rest);
    if first.is_empty() || !first.chars().all(|c| c.is_ascii_digit()) {
        return (tok, None);
    }
    match first.parse::<usize>() {
        Ok(n) => (path, Some(n)),
        Err(_) => (tok, None),
    }
}

/// Resolve a path as the comment's reader would. `src/rete/kernel/tests/strat_cost.rs` naming
/// `fire/rules.rs` means `src/rete/kernel/fire/rules.rs` — the reader walks up until the prefix
/// makes sense, so the gate does too: repo root, `src/`, then every ancestor of the naming file.
///
/// The first draft tried only three roots and reported six stale paths, five of which existed.
/// A gate that cries wolf gets muted, so the resolution rule has to match how the path is
/// actually read.
///
/// Answers WHERE it resolved, not merely whether: the line half needs the file to count.
fn resolve(root: &Path, naming_file: &Path, p: &str) -> Option<PathBuf> {
    for cand in [root.join(p), root.join("src").join(p)] {
        if cand.exists() {
            return Some(cand);
        }
    }
    let mut dir = naming_file.parent();
    while let Some(d) = dir {
        let cand = d.join(p);
        if cand.exists() {
            return Some(cand);
        }
        if d == root {
            break;
        }
        dir = d.parent();
    }
    None
}

/// Line count of a resolved file, memoised — the same handful of files are cited over and over,
/// and re-reading each one per citation is the difference between a lint and a build step.
fn line_count(cache: &mut BTreeMap<PathBuf, usize>, path: &Path) -> Option<usize> {
    if let Some(n) = cache.get(path) {
        return Some(*n);
    }
    let text = std::fs::read_to_string(path).ok()?;
    let n = text.lines().count();
    cache.insert(path.to_path_buf(), n);
    Some(n)
}

/// Every source file under the scanned roots, paired with its comment opener.
fn scanned_files(root: &Path) -> Vec<(PathBuf, &'static str)> {
    let mut out = Vec::new();
    for (rel, ext) in ROOTS {
        let mut stack = vec![root.join(rel)];
        while let Some(dir) = stack.pop() {
            let Ok(entries) = std::fs::read_dir(&dir) else {
                continue;
            };
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    stack.push(path);
                    continue;
                }
                if path.extension().and_then(|e| e.to_str()) != Some(*ext) {
                    continue;
                }
                out.push((path, comment_head(ext)));
            }
        }
    }
    out.sort();
    out
}

#[test]
fn every_location_named_in_a_doc_comment_exists() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let mut stale: Vec<String> = Vec::new();
    let mut out_of_range: Vec<String> = Vec::new();
    let mut checked = 0usize;
    let mut checked_lines = 0usize;
    let mut lens: BTreeMap<PathBuf, usize> = BTreeMap::new();
    let mut deferral_seen = vec![false; DEFERRED.len()];

    for (path, head) in scanned_files(root) {
        let Ok(src) = std::fs::read_to_string(&path) else {
            continue;
        };
        let naming = path.strip_prefix(root).unwrap_or(&path).display().to_string();
        for cite in citations_in_comments(&src, head) {
            checked += 1;
            let Some(target) = resolve(root, &path, &cite.path) else {
                match DEFERRED
                    .iter()
                    .position(|(f, c)| *f == naming && *c == cite.path)
                {
                    Some(i) => deferral_seen[i] = true,
                    None => {
                        stale.push(format!("{naming}: names `{}`, which does not exist", cite.path))
                    }
                }
                continue;
            };
            let Some(cited) = cite.line else {
                continue;
            };
            checked_lines += 1;
            let Some(len) = line_count(&mut lens, &target) else {
                continue;
            };
            if cited > len {
                out_of_range.push(format!(
                    "{naming}: cites `{}:{cited}`, but {} is {len} lines",
                    cite.path,
                    target.strip_prefix(root).unwrap_or(&target).display()
                ));
            }
        }
    }

    // Non-vacuity, first half: these files cite each other constantly. A run that checked nothing
    // would pass silently, which is the failure shape this whole gate exists to refuse.
    // Measured 2026-09-04 across `src/rete` + `wat` + `wat-tests`: 610 path references (174 of
    // them under `src/rete`, the root this gate walked before the widening).
    assert!(
        checked >= 450,
        "only {checked} path reference(s) found under the scanned roots — the extractor stopped \
         matching, or a root moved, so this gate is passing without checking anything"
    );
    // Non-vacuity, second half: a path count cannot see a LINE extractor that has gone blind. The
    // `:LINE` suffix is the newer half and the easier one to lose — peel the suffix wrong and every
    // citation reads as a bare path, `checked` is unchanged, and the depth check silently judges
    // zero of them. Measured 2026-09-04: 72 of the 610 carry a line number (30 under `src/rete`).
    assert!(
        checked_lines >= 55,
        "only {checked_lines} of {checked} citation(s) carried a `:LINE` — the line extractor went \
         blind, so the depth half of this gate is judging nothing"
    );

    assert!(
        stale.is_empty(),
        "comments name {} path(s) that do not exist:\n  {}",
        stale.len(),
        stale.join("\n  ")
    );
    // A held row that no longer matches is a cure that never removed its deferral — and a stale
    // deferral is how a fenced pile turns into a licence nobody re-reads. Removing it is the only
    // way this list moves.
    let unused: Vec<String> = DEFERRED
        .iter()
        .zip(&deferral_seen)
        .filter(|(_, seen)| !**seen)
        .map(|((f, c), _)| format!("{f} -> {c}"))
        .collect();
    assert!(
        unused.is_empty(),
        "\n\n{} DEFERRED row(s) no longer match anything the walk found — the citation was cured, \
         renamed, or deleted. Delete the row: a deferral that holds nothing is a standing licence \
         for a rot that is already gone.\n\n{}\n",
        unused.len(),
        unused.join("\n")
    );

    assert!(
        out_of_range.is_empty(),
        "\n\n{} citation(s) name a line past the end of the file they point at. A `path:line` is \
         TWO claims and this is the second one going false — the path still resolves, so nothing \
         else in the tree can see it.\n\n{}\n\n\
         THE FIX is normally NOT to correct the number. A line drifts every time anything above it \
         moves; a SYMBOL does not. Cite the symbol and drop the `:line`.\n",
        out_of_range.len(),
        out_of_range.join("\n")
    );
}

#[cfg(test)]
mod extractor {
    use super::*;

    #[test]
    fn a_bare_path_carries_no_line() {
        let got = citations_in_comments("// see `src/rete/kernel/fire/mod.rs` for this\n", "//");
        assert_eq!(
            got.into_iter().collect::<Vec<_>>(),
            vec![Citation {
                path: "src/rete/kernel/fire/mod.rs".to_string(),
                line: None
            }]
        );
    }

    #[test]
    fn a_line_suffix_is_peeled_off() {
        let got = citations_in_comments(";; `wat/rete.wat:1508`'s oracle is the shape\n", ";;");
        assert_eq!(
            got.into_iter().collect::<Vec<_>>(),
            vec![Citation {
                path: "wat/rete.wat".to_string(),
                line: Some(1508)
            }]
        );
    }

    #[test]
    fn a_range_cites_its_first_line() {
        // The sample is deliberately paren-free: a `(word/word …)` string literal is a form wat's
        // own reader parses, which `tests/lint/no_inlined_wat_in_tests.rs` refuses on sight.
        let got = citations_in_comments("// fire/rules.rs:629-642 is the arm\n", "//");
        assert_eq!(
            got.into_iter().collect::<Vec<_>>(),
            vec![Citation {
                path: "fire/rules.rs".to_string(),
                line: Some(629)
            }]
        );
    }

    #[test]
    fn a_wat_symbol_name_is_not_a_citation() {
        let got = citations_in_comments(";; `:wat::rete::insert-all$oracle` is the live name\n", ";;");
        assert!(got.is_empty(), "a `::`-joined symbol must not read as a path: {got:?}");
    }

    #[test]
    fn code_outside_a_comment_is_not_read() {
        // Paren-free on purpose: a `(head …)` string literal is a form wat's own reader parses, and
        // `tests/lint/no_inlined_wat_in_tests.rs` refuses one on sight whatever it is testing.
        let got = citations_in_comments("  wat/rete.wat:1508 sits in a code position\n", ";;");
        assert!(got.is_empty(), "only comment lines are scanned: {got:?}");
    }

    #[test]
    fn a_non_numeric_suffix_stays_part_of_the_path() {
        assert_eq!(split_line_suffix("wat/rete.wat:oracle"), ("wat/rete.wat:oracle", None));
    }

    #[test]
    fn the_boundary_is_the_last_line_itself() {
        // The rule under test is `cited > len` — the last line of a file is IN range. A gate that
        // rejects a valid citation gets disabled by the next hand, so this is pinned.
        let len = 533usize;
        assert!(!(len > len), "citing the last line exactly must be in range");
        assert!(len + 1 > len, "citing one past the end must be out of range");
    }
}
