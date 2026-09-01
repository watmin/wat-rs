//! A GATE THAT REACHES NOTHING PASSES — SO EVERY GATE THAT DISCOVERS ITS SUBJECT SAYS HOW IT
//! KNOWS IT REACHED SOMETHING.
//!
//! `tests/lint/` is where this arc's guarantees live. Most of those gates do not name their
//! subject — they *discover* it, by walking a directory or by reading a spawned process's output.
//! A discovering gate whose discovery comes back EMPTY asserts nothing over nothing and reports
//! PASS, and every verdict downstream of it inherits that silence. `no_ceiling_raise_in_rete.rs`
//! writes the reason verbatim: *"A typo'd path finding zero files would make this gate pass
//! forever while checking nothing."*
//!
//! The contract this gate keeps: **a gate that discovers its subject set states how it knows the
//! set was not empty — in code, or in a rune that names the shape it uses instead.**
//!
//! ## THE POPULATION THIS GATE RECOGNISES, AND WHAT IT LEAVES OUT
//!
//! In scope: a `tests/lint/*.rs` file whose text contains `read_dir` (a runtime directory
//! traversal) or `Command::new` (a spawned process whose output is the population). Those are the
//! two ways a gate in this directory acquires a subject it did not name, and they are the two ways
//! the subject can silently become empty — a moved root, a renamed directory, a reworded
//! diagnostic — WITHOUT anything failing.
//!
//! Deliberately OUT of scope, and this is a cut rather than an oversight:
//!
//! - **A gate that names its files.** `include_str!`, or `read_to_string` of a const path, cannot
//!   silently reach zero FILES: `include_str!` is a compile error and `.expect()` is a panic. Its
//!   *extractor* can still go blind over a file that is present — `gen_doc_surface_matches.rs`
//!   parses 27 verbs out of a named file and would pass on 0 — but that is a different population
//!   with a different cure (prove the parser, as `no_new_broken_doc_link.rs` does), and mixing the
//!   two would make this gate's rule unstatable. It is named here so it is tracked, not silent.
//! - **Anything outside `tests/lint/`.** The floor at large discovers subjects too; this gate is
//!   scoped to the directory whose whole job is judging, where a vacuous PASS is worst.
//!
//! Over-inclusion is the safe direction: a file that spawns a process for some reason other than
//! its population is asked for one declaration line, never for a change to what it asserts.
//!
//! ## WHAT COUNTS AS A DECLARATION — AND WHY IT IS NOT A GREP FOR `assert!(n > 0)`
//!
//! There is MORE THAN ONE legitimate guard shape, so a syntactic search for the usual assert would
//! flag correct gates. `no_new_broken_doc_link.rs` is the counter-example living in this tree: its
//! population is a diagnostic stream that is MEANT to reach zero, so "found at least N" would make
//! a clean tree RED. It proves its extractor against a fixed sample of rustdoc's format instead.
//!
//! So the declaration is what this gate reads, in one of two forms:
//!
//! 1. **In code** — a comment containing `NON-VACUITY`, with an assertion within
//!    [`ASSERT_WINDOW`] lines after it. The marker must INTRODUCE a guard; a marker with no
//!    assertion under it is a shrug and is refused.
//! 2. **A rune** — `rune:lint(vacuity-guard) — <what this gate does instead, and what would red
//!    first>`. A DECLARATION, NOT A SUPPRESSION: the reason must name a mechanism. "N/A" and its
//!    family are refused by name, and a reason shorter than [`MIN_REASON_CHARS`] cannot state one.
//!
//! Neither form can prove an assertion is MEANINGFUL — no static check can, and claiming otherwise
//! would be the decoration this codebase keeps removing. What both forms do is make the question
//! unskippable and put a human sentence next to the answer, which is what
//! `no_new_broken_doc_link.rs:236` already does by hand and what this gate makes universal.
//!
//! Shape and precedent: `tests/lint/no_unknown_sequi_rune.rs` — *"the table is the definition,
//! this is the gate."*
//!
//! ## THIS GATE IS ITSELF A WALKING GATE
//!
//! It walks `tests/lint/`, so it is in its own population and carries its own declaration below.
//! A vacuity lint that is itself vacuous is the joke this file exists to prevent — and a count
//! alone would not catch a recogniser that had stopped recognising, so the self-guard also carries
//! a POSITIVE CONTROL: a named file that must come back in-scope AND guarded.

use std::path::{Path, PathBuf};

/// How far below a `NON-VACUITY` marker an assertion may sit and still count as the guard it
/// introduces. Generous enough for a multi-line comment above the assert (the shape every existing
/// marker in this directory uses), tight enough that an unrelated assertion elsewhere in the
/// function cannot be borrowed as one.
const ASSERT_WINDOW: usize = 12;

/// Shortest reason that can name a mechanism. Every refused shrug below is under it; the shortest
/// real reason written in this directory is several times over it.
const MIN_REASON_CHARS: usize = 40;

/// The rune that declares a gate uses a shape other than "the walk found at least N".
const RUNE: &str = "rune:lint(vacuity-guard)";

/// The marker that introduces an in-code guard. Matched case-insensitively: the tree already
/// writes it both ways.
const MARKER: &str = "non-vacuity";

/// Reasons that describe the ABSENCE of an answer rather than an answer. A rune carrying one of
/// these is a suppression wearing a declaration's clothes.
const REFUSED_REASONS: &[&str] = &[
    "n/a",
    "not applicable",
    "none",
    "nothing",
    "no reason",
    "see above",
    "obvious",
    "by inspection",
    "not needed",
    "unnecessary",
];

/// The file that must always come back in-scope and guarded. It is the exemplar the design record
/// cites, and it is the positive control for BOTH halves of the recogniser: if the scope detector
/// or the declaration reader breaks, this is what says so.
const POSITIVE_CONTROL: &str = "no_ceiling_raise_in_rete.rs";

/// How a gate acquires a subject it did not name.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
enum Discovery {
    /// A runtime directory traversal.
    DirWalk,
    /// A spawned process whose output is the population.
    Subprocess,
    /// The gate names its subject; it cannot silently reach zero files.
    Named,
}

/// Which discovery mechanism this source uses, if any.
fn discovery(src: &str) -> Discovery {
    if src.contains("read_dir") {
        Discovery::DirWalk
    } else if src.contains("Command::new") {
        Discovery::Subprocess
    } else {
        Discovery::Named
    }
}

/// Assertion heads, as an ARRAY rather than as inline string-literal arguments.
///
/// Not a style choice: `tests/lint/no_loose_string_assert.rs` refuses a loose string match whose
/// argument is a literal on a line its opener-detector also reads as an assertion — and a line
/// that names an assertion macro inside a literal is exactly that shape. It reads comment lines
/// too, so this note is worded around the pattern rather than quoting it. Moving the needles into
/// a const means no line here is both halves at once.
const ASSERT_HEADS: &[&str] = &["assert!(", "assert_eq!(", "assert_ne!("];

/// A line-comment opener, as a const for the same reason.
const COMMENT_HEAD: &str = "//";

/// Doc-comment openers, excluded on the RUNE path only — see [`rune_declaration`].
///
/// ⚠ **The scope is the rune, not every declaration, and the asymmetry is deliberate.** An earlier
/// wording here read *"a doc comment describes; only a plain `//` comment declares"*, which states a
/// general rule this constant does not have: it is consulted at exactly one site. Driven
/// 2026-09-01 — turning a `NON-VACUITY` marker into `/// NON-VACUITY` leaves this gate GREEN.
///
/// That is correct, and the reason is the difference in what carries the evidence. A rune's
/// REASON TEXT *is* the evidence, so a description of the rune form reads as an answer — which is
/// how this gate came one run from vouching for itself with its own `Declaration::Rune` doc. A
/// marker's evidence is the ASSERTION UNDER IT, which a doc comment cannot fake: `is_assert`
/// rejects any commented line, so a `///` marker with nothing real beneath it is still `Missing`.
const DOC_HEADS: [&str; 2] = ["///", "//!"];

/// The separator between a rune and its reason, as a CHAR.
const EM_DASH: char = '\u{2014}';

/// Is this line an assertion opener?
fn is_assert(line: &str) -> bool {
    let t = line.trim_start();
    if t.starts_with(COMMENT_HEAD) {
        return false;
    }
    ASSERT_HEADS.iter().any(|h| t.contains(h))
}

/// The verdict on one file's declaration.
#[derive(Debug, PartialEq, Eq)]
enum Declaration {
    /// A `NON-VACUITY` marker with an assertion under it.
    InCode,
    /// A `rune:lint(vacuity-guard)` whose reason names a mechanism.
    Rune,
    /// Nothing was declared.
    Missing,
    /// Something was declared but does not hold up — the string says which.
    Hollow(String),
}

/// Read a file's declaration. The rune is read first: a gate that declares an alternative shape has
/// answered the question even if the word `NON-VACUITY` also appears somewhere in its prose.
fn declaration(src: &str) -> Declaration {
    if let Some(v) = rune_declaration(src) {
        return v;
    }
    marker_declaration(src)
}

/// The rune form, or `None` if this file writes no rune at all.
fn rune_declaration(src: &str) -> Option<Declaration> {
    // The rune must be DECLARED, in a PLAIN `//` line comment on the gate itself. A `///` or `//!`
    // doc comment DESCRIBES the gate — this file's own header explains the rune form, and its
    // `Declaration::Rune` variant is documented by name — and reading a description as an answer is
    // how a gate comes to vouch for itself. That is not hypothetical: the first driven run of this
    // gate went RED on exactly that line of this file.
    let line = src.lines().find(|l| {
        let t = l.trim_start();
        t.starts_with(COMMENT_HEAD)
            && !DOC_HEADS.iter().any(|d| t.starts_with(d))
            && t.contains(RUNE)
    })?;
    let at = line.find(RUNE).expect("the line was selected because it holds the rune");
    let tail = line[at + RUNE.len()..].trim();

    // The em-dash separator is what the repo's rune form uses; a rune with no separator has no
    // reason at all, which is the case this branch must not wave through.
    let Some(reason) = tail.strip_prefix(EM_DASH).or_else(|| tail.strip_prefix('-')) else {
        return Some(Declaration::Hollow(format!(
            "the rune carries no reason (expected `{RUNE} \u{2014} <reason>`, found `{tail}`)"
        )));
    };
    let reason = reason.trim();
    let folded = reason.to_ascii_lowercase();

    let is_shrug = |r: &str| {
        folded == r
            || folded.trim_end_matches('.') == r
            || folded
                .strip_prefix(r)
                .is_some_and(|rest| rest.starts_with(' ') || rest.starts_with(',') || rest.starts_with(';'))
    };
    if let Some(shrug) = REFUSED_REASONS.iter().find(|r| is_shrug(r)) {
        return Some(Declaration::Hollow(format!(
            "the rune's reason is `{reason}` — `{shrug}` names no mechanism. Say what this gate \
             does INSTEAD of a count, and what would go red FIRST if that shape stopped working"
        )));
    }
    if reason.chars().count() < MIN_REASON_CHARS {
        return Some(Declaration::Hollow(format!(
            "the rune's reason is {} chars (`{reason}`) — under {MIN_REASON_CHARS}, which is too \
             short to name both what this gate does instead and what would red first",
            reason.chars().count()
        )));
    }
    Some(Declaration::Rune)
}

/// The in-code form: a `NON-VACUITY` marker with an assertion within [`ASSERT_WINDOW`] lines.
fn marker_declaration(src: &str) -> Declaration {
    let lines: Vec<&str> = src.lines().collect();
    let mut marked = false;
    for (i, line) in lines.iter().enumerate() {
        if !line.to_ascii_lowercase().contains(MARKER) {
            continue;
        }
        marked = true;
        let end = (i + 1 + ASSERT_WINDOW).min(lines.len());
        if lines[i + 1..end].iter().any(|l| is_assert(l)) {
            return Declaration::InCode;
        }
    }
    if marked {
        return Declaration::Hollow(format!(
            "a `NON-VACUITY` marker appears but no assertion follows it within {ASSERT_WINDOW} \
             lines — the marker must INTRODUCE a guard, not stand in for one"
        ));
    }
    Declaration::Missing
}

fn lint_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/lint")
}

fn gate_files() -> Vec<PathBuf> {
    let mut out = Vec::new();
    let Ok(entries) = std::fs::read_dir(lint_dir()) else {
        return out;
    };
    for e in entries.flatten() {
        let p = e.path();
        if p.extension().and_then(|x| x.to_str()) != Some("rs") {
            continue;
        }
        // `mod.rs` is a two-line `include!` stub with no test in it.
        if p.file_name().and_then(|n| n.to_str()) == Some("mod.rs") {
            continue;
        }
        out.push(p);
    }
    out.sort();
    out
}

#[test]
fn every_discovering_gate_declares_how_it_knows_it_reached_something() {
    let files = gate_files();

    // NON-VACUITY, first half: this gate walks a directory, so it is in its own population and
    // must answer its own question. Measured 2026-09-01: 30 gate files under tests/lint/.
    assert!(
        files.len() >= 25,
        "the vacuity-guard walk found only {} .rs file(s) under tests/lint/ — a vacuity lint that \
         is itself vacuous is the joke this gate exists to prevent",
        files.len()
    );

    let mut in_scope: Vec<String> = Vec::new();
    let mut undeclared: Vec<String> = Vec::new();
    let mut hollow: Vec<String> = Vec::new();
    let mut control_seen = false;

    for f in &files {
        let name = f
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or_default()
            .to_string();
        let Ok(src) = std::fs::read_to_string(f) else {
            continue;
        };
        if discovery(&src) == Discovery::Named {
            continue;
        }
        in_scope.push(name.clone());
        match declaration(&src) {
            Declaration::InCode | Declaration::Rune => {
                if name == POSITIVE_CONTROL {
                    control_seen = true;
                }
            }
            Declaration::Missing => undeclared.push(name),
            Declaration::Hollow(why) => hollow.push(format!("{name}: {why}")),
        }
    }

    // NON-VACUITY, second half: a count cannot see a recogniser that stopped recognising. The
    // exemplar the design record cites must come back in-scope AND guarded, or both halves of this
    // gate's reader are unproven and its green means nothing.
    assert!(
        control_seen,
        "the positive control `{POSITIVE_CONTROL}` did not come back in-scope AND guarded — so \
         either the discovery detector or the declaration reader has stopped working, and every \
         green below is unearned. In scope this run: {in_scope:?}"
    );
    // Measured 2026-09-01: 24 of 30 gate files discover their subject.
    assert!(
        in_scope.len() >= 18,
        "only {} gate(s) were recognised as discovering their subject — the scope detector went \
         blind, so this gate is judging almost nothing: {in_scope:?}",
        in_scope.len()
    );

    assert!(
        hollow.is_empty(),
        "\n\n{} gate(s) DECLARE a vacuity guard that does not hold up. A rune with a hollow reason \
         is worse than no rune: it reads, forever, as though someone answered.\n\n{}\n",
        hollow.len(),
        hollow.join("\n")
    );

    assert!(
        undeclared.is_empty(),
        "\n\n🔥 {} gate(s) in tests/lint/ DISCOVER their subject set and never say how they know \
         it was not empty. A walk that comes back empty asserts nothing over nothing and reports \
         PASS — and every verdict downstream inherits that silence.\n\
         \n\
         THE FIX, one of two:\n\
         \n\
         1. Add the guard, introduced by a `NON-VACUITY:` comment — assert the walk found at least \
         a floor of what it actually finds today. Drive the gate and read the real number first; a \
         floor picked by symmetry with a sibling is not a measurement. Shape: \
         `tests/lint/no_ceiling_raise_in_rete.rs`.\n\
         \n\
         2. If \"found at least N\" is genuinely unwritable — the population is a stream that is \
         MEANT to reach zero, so a floor would red a clean tree — declare the shape you use \
         instead with `// {RUNE} \u{2014} <what this gate does instead, and what would red first>`. \
         Shape: `tests/lint/no_new_broken_doc_link.rs`, which proves its extractor against a fixed \
         sample of the format it parses. A reason that names no mechanism is refused.\n\
         \n\
         Undeclared:\n\n{}\n",
        undeclared.len(),
        undeclared.join("\n")
    );
}

#[cfg(test)]
mod detector {
    use super::*;

    #[test]
    fn a_dir_walk_is_in_scope() {
        assert_eq!(
            discovery("fn f() {\n    let x = std::fs::read_dir(p);\n}\n"),
            Discovery::DirWalk
        );
    }

    #[test]
    fn a_spawned_process_is_in_scope() {
        assert_eq!(
            discovery("fn f() {\n    let o = Command::new(bin).output();\n}\n"),
            Discovery::Subprocess
        );
    }

    #[test]
    fn a_gate_that_names_its_files_is_out_of_scope() {
        assert_eq!(
            discovery("const S: &str = include_str!(\"../../src/types.rs\");\n"),
            Discovery::Named
        );
    }

    #[test]
    fn a_marker_over_an_assertion_is_a_declaration() {
        let src = "fn f() {\n    // NON-VACUITY: the walk must find the tree.\n    assert!(files.len() > 20, \"blind\");\n}\n";
        assert_eq!(declaration(src), Declaration::InCode);
    }

    #[test]
    fn a_lowercase_marker_is_read_too() {
        let src = "fn f() {\n    // Non-vacuity: these files cite each other constantly.\n    assert!(checked >= 40, \"blind\");\n}\n";
        assert_eq!(declaration(src), Declaration::InCode);
    }

    #[test]
    fn a_marker_with_no_assertion_under_it_is_hollow() {
        let src = "fn f() {\n    // NON-VACUITY: trust me.\n    let x = 1;\n}\n";
        match declaration(src) {
            Declaration::Hollow(_) => {}
            other => panic!("a marker with no assertion must be hollow, got {other:?}"),
        }
    }

    #[test]
    fn an_assertion_past_the_window_does_not_count_as_the_guard() {
        let mut src = String::from("fn f() {\n    // NON-VACUITY: trust me.\n");
        for _ in 0..ASSERT_WINDOW + 2 {
            src.push_str("    let x = 1;\n");
        }
        src.push_str("    assert!(unrelated, \"far away\");\n}\n");
        match declaration(&src) {
            Declaration::Hollow(_) => {}
            other => panic!("an assertion past the window must not count, got {other:?}"),
        }
    }

    #[test]
    fn a_bare_walk_declares_nothing() {
        let src = "fn f() {\n    let x = std::fs::read_dir(p);\n    assert!(v.is_empty(), \"clean\");\n}\n";
        assert_eq!(declaration(src), Declaration::Missing);
    }

    #[test]
    fn a_rune_naming_a_mechanism_is_a_declaration() {
        let src = "// rune:lint(vacuity-guard) \u{2014} the population is a diagnostic stream meant \
                   to reach zero; the extractor is proven against a fixed sample instead, and that \
                   self-check reds first if the format moves.\n";
        assert_eq!(declaration(src), Declaration::Rune);
    }

    #[test]
    fn a_rune_reading_n_slash_a_is_refused() {
        let src = "// rune:lint(vacuity-guard) \u{2014} N/A\n";
        match declaration(src) {
            Declaration::Hollow(_) => {}
            other => panic!("`N/A` must be refused, got {other:?}"),
        }
    }

    #[test]
    fn a_rune_reading_not_applicable_is_refused() {
        let src = "// rune:lint(vacuity-guard) \u{2014} not applicable, this gate is fine really\n";
        match declaration(src) {
            Declaration::Hollow(_) => {}
            other => panic!("`not applicable` must be refused, got {other:?}"),
        }
    }

    #[test]
    fn a_rune_too_short_to_name_a_mechanism_is_refused() {
        let src = "// rune:lint(vacuity-guard) \u{2014} the stream can be empty\n";
        match declaration(src) {
            Declaration::Hollow(_) => {}
            other => panic!("a reason under the floor must be refused, got {other:?}"),
        }
    }

    #[test]
    fn a_rune_with_no_separator_carries_no_reason() {
        let src = "// rune:lint(vacuity-guard)\n";
        match declaration(src) {
            Declaration::Hollow(_) => {}
            other => panic!("a rune with no reason must be hollow, got {other:?}"),
        }
    }

    #[test]
    fn a_commented_out_assertion_is_not_an_assertion() {
        let src = "fn f() {\n    // NON-VACUITY: the walk must find the tree.\n    // assert!(files.len() > 20, \"blind\");\n}\n";
        match declaration(src) {
            Declaration::Hollow(_) => {}
            other => panic!("a commented assertion must not count, got {other:?}"),
        }
    }
}
