//! **A SOURCE PATH NAMED IN A DOC COMMENT MUST EXIST.**
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
//! ── WHY A GATE AND NOT A CONVENTION (the honest rung) ────────────────────────────────────────
//!
//! "Check your paths when you move a file" is a convention, and conventions lean on every future
//! hand remembering during a rename. A path either exists on disk or it does not — that is
//! decidable, with no false positives available to it, which is exactly the shape that belongs in
//! a build gate rather than in a habit. This is the top rung the material allows: Rust cannot
//! make an unresolvable path in a comment fail to compile, so a check at build time is as far up
//! the ladder as this class goes.
//!
//! ── WHAT THIS GATE CANNOT DO — stated, because the gap is the point ──────────────────────────
//!
//! It checks that a named path EXISTS. It cannot check that the path is the RIGHT one, that the
//! symbol named beside it lives there, or that any surrounding claim is true. Four false claims
//! were found in these files on 2026-08-30 by `intueri`; this gate would have caught exactly one
//! of them. The others needed a ward, and their cure is
//! `tests/lint/rete_header_claims_are_asserted.rs` — assertions, not prose.

use std::collections::BTreeSet;
use std::path::Path;

/// Every `word/with/slashes.rs` (or `.wat`) that appears inside a `//`-comment under the scanned
/// roots. Backtick-quoted or bare; both are how these files write them.
fn paths_named_in_comments(src: &str) -> BTreeSet<String> {
    let mut out = BTreeSet::new();
    for line in src.lines() {
        let t = line.trim_start();
        if !(t.starts_with("//") || t.starts_with("///") || t.starts_with("//!")) {
            continue;
        }
        for tok in t.split(|c: char| !(c.is_alphanumeric() || "._/-".contains(c))) {
            if (tok.ends_with(".rs") || tok.ends_with(".wat"))
                && tok.contains('/')
                && !tok.starts_with('.')
                && !tok.contains("..")
            {
                out.insert(tok.to_string());
            }
        }
    }
    out
}

/// Resolve a path as the comment's reader would. `src/rete/kernel/tests/strat_cost.rs` naming
/// `fire/rules.rs` means `src/rete/kernel/fire/rules.rs` — the reader walks up until the prefix
/// makes sense, so the gate does too: repo root, `src/`, then every ancestor of the naming file.
///
/// The first draft tried only three roots and reported six stale paths, five of which existed.
/// A gate that cries wolf gets muted, so the resolution rule has to match how the path is
/// actually read.
fn resolves(root: &Path, naming_file: &Path, p: &str) -> bool {
    if root.join(p).exists() || root.join("src").join(p).exists() {
        return true;
    }
    let mut dir = naming_file.parent();
    while let Some(d) = dir {
        if d.join(p).exists() {
            return true;
        }
        if d == root {
            break;
        }
        dir = d.parent();
    }
    false
}

#[test]
fn every_path_named_in_a_rete_doc_comment_exists() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let mut stale: Vec<String> = Vec::new();
    let mut checked = 0usize;

    let mut stack = vec![root.join("src/rete")];
    while let Some(dir) = stack.pop() {
        for entry in std::fs::read_dir(&dir).expect("read_dir under src/rete") {
            let path = entry.expect("dir entry").path();
            if path.is_dir() {
                stack.push(path);
                continue;
            }
            if path.extension().is_none_or(|e| e != "rs") {
                continue;
            }
            let src = std::fs::read_to_string(&path).expect("read a rete source");
            for named in paths_named_in_comments(&src) {
                checked += 1;
                if !resolves(root, &path, &named) {
                    stale.push(format!(
                        "{}: names `{named}`, which does not exist",
                        path.strip_prefix(root).unwrap_or(&path).display()
                    ));
                }
            }
        }
    }

    // Non-vacuity: these files cite each other constantly. A run that checked nothing would pass
    // silently, which is the failure shape this whole gate exists to refuse.
    assert!(
        checked >= 40,
        "only {checked} path references found under src/rete — the extractor stopped matching, so \
         this gate is passing without checking anything"
    );
    assert!(
        stale.is_empty(),
        "doc comments name {} path(s) that do not exist:\n  {}",
        stale.len(),
        stale.join("\n  ")
    );
}
