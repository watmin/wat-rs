//! THE `rune:perspicere` AND `rune:purgare` CATEGORIES ARE CLOSED SETS — an invented one is a red
//! build.
//!
//! Both wards exempt a site by rune. `perspicere` exempts a type expression too deeply nested to
//! read, declaring that no typealias can fix it; `purgare` exempts a defined thing with no visible
//! consumer, declaring that it is alive at an end the compiler cannot see. In each case the
//! category carries the WHY, and the vocabularies with their discriminating questions live in
//! `docs/CONVENTIONS.md` — "The `rune:perspicere` vocabulary" and "The `rune:purgare` vocabulary".
//! The tables are the definition; this is the gate.
//!
//! ## THE ORDER THIS FILE WAS WRITTEN IN IS LOAD-BEARING
//!
//! The definitions were written FIRST, from the ward spells and from what the runes' reasons
//! actually argue, and only then was this gate pointed at the tree. That order is not procedure,
//! it is the difference between a gate and a transcript: a vocabulary derived from the categories
//! already in use contains, by construction, every category already in use. Such a gate passes
//! every site on the day it lands — including any site that is wrong — and freezes the tree's
//! current labelling behind a green test. `sequi` was given a written table for exactly this
//! reason after `ARM_TABLE` and `EXEC_ARENA` were found carrying different categories for the same
//! shape; see `tests/lint/no_unknown_sequi_rune.rs`, whose header records that incident.
//!
//! ## WHAT THIS GATE CHECKS, AND WHAT IT DOES NOT — stated, because the gap is the point
//!
//! It closes the SETS. A category invented at a call site fails here, naming its file, its line
//! and the ward whose vocabulary it is outside of.
//!
//! It CANNOT tell you a rune picked the wrong member of a set it spells correctly. **Spelling is
//! machine-checkable; fit is not.** Nothing here reads a reason and rules that it argues for a
//! different category than the label above it — and that is not a hypothetical gap. Measured
//! 2026-09-01: the clause "alias would be a mumble" — which is `mumble-alias`'s argument — appears
//! verbatim at 18 `read-once` sites across four files, as a trailing flourish under reasons whose
//! first clause makes the `read-once` argument. This gate is green over every one of them and
//! always will be. Only the tables in `docs/CONVENTIONS.md` can judge fit, which is why each one
//! carries a discriminating question rather than a list of examples to pattern-match.
//!
//! ## `tests/lint/` IS OUT OF THE WALK, AND THAT IS A CUT RATHER THAN A SELF-EXEMPTION
//!
//! A gate directory's rune-shaped strings are SPECIMENS, not sites: a lint proves it can go red by
//! feeding its detector the exact shape a drifting call site would have, and that shape read as a
//! real site would red the tree for a negative control doing its job. The exclusion is of the whole
//! category of gate files, not of this file — `tests/lint/no_unknown_sequi_rune.rs` carries the
//! same kind of specimen and would be caught by the same cut. Measured 2026-09-01: zero real
//! `perspicere` or `purgare` runes live under `tests/lint/`, so the cut costs no coverage today.
//! Belt as well as braces, the specimens in this file's own `detector` module are built by
//! interpolating a bound ward name, so the source text here holds no literal marker at all.

use std::path::{Path, PathBuf};

/// The closed sets, one row per ward. Adding a member here is a deliberate act that must be
/// accompanied by a row in that ward's vocabulary table in `docs/CONVENTIONS.md` — the table is
/// the definition, this is the gate.
const WARD_VOCABULARIES: &[(&str, &[&str])] = &[
    (
        "perspicere",
        &["read-once", "mumble-alias", "intentional-structure"],
    ),
    (
        "purgare",
        &[
            "public-api",
            "trait-contract",
            "future-fixture",
            "safety-margin",
        ],
    ),
];

/// Every root that holds runed code. Wider than `no_unknown_sequi_rune.rs`'s `src`-only walk on
/// purpose: `purgare` runes live in `crates/wat-edn/`, and a `perspicere` rune lives in a `.wat`
/// file under `wat/` and in a probe under `tests/`. A root that holds none today still belongs
/// here — the cost is a directory walk and the alternative is a site added tomorrow to a root
/// nothing reads.
const SCAN_ROOTS: &[&str] = &["src", "crates", "tests", "wat", "wat-scripts", "wat-tests"];

/// The gate directory, excluded from the walk — see this file's header.
const SPECIMEN_DIR: &str = "tests/lint";

/// Extensions that can carry a rune: Rust `//` comments and wat `;;` comments.
const RUNED_EXTENSIONS: &[&str] = &["rs", "wat"];

/// The file that must come back reached and runed, or the walk and the extractor are unproven.
///
/// `src/comms/process.rs` is the positive control because it is the one file carrying runes from
/// BOTH wards — a `perspicere` pair (the `pair<T>()` return shape and `Sender::send`'s
/// `SendError`) and a `purgare` pair (the two manual `Debug` impls). A count alone cannot see an
/// extractor that has stopped extracting; this can.
const POSITIVE_CONTROL: &str = "src/comms/process.rs";

fn collect(dir: &Path, specimens: &Path, out: &mut Vec<PathBuf>) {
    if dir == specimens {
        return;
    }
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for e in entries.flatten() {
        let p = e.path();
        if p.is_dir() {
            if p.file_name().and_then(|n| n.to_str()) == Some("target") {
                continue;
            }
            collect(&p, specimens, out);
        } else {
            let ext = p.extension().and_then(|x| x.to_str()).unwrap_or_default();
            if RUNED_EXTENSIONS.contains(&ext) {
                out.push(p);
            }
        }
    }
}

/// Every category named for `ward` on one line, in source order.
///
/// Split on the needle rather than regex: the marker is a fixed string and the category is
/// everything up to the closing paren. A line bearing two runes of the same ward yields two
/// categories, which is what keeps a valid rune from shielding an invented one beside it.
fn categories_on<'a>(line: &'a str, ward: &str) -> Vec<&'a str> {
    let needle = format!("rune:{ward}(");
    let mut out = Vec::new();
    let mut rest = line;
    while let Some(at) = rest.find(&needle) {
        let after = &rest[at + needle.len()..];
        match after.find(')') {
            Some(close) => {
                out.push(&after[..close]);
                rest = &after[close..];
            }
            // An unclosed marker is itself malformed; surface it as a category so the caller
            // reports it rather than silently walking past a broken rune.
            None => {
                out.push(after);
                break;
            }
        }
    }
    out
}

#[test]
fn every_ward_rune_names_a_known_category() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let specimens = root.join(SPECIMEN_DIR);

    let mut files = Vec::new();
    for r in SCAN_ROOTS {
        let dir = root.join(r);
        assert!(
            dir.is_dir(),
            "scan root `{r}` does not exist under {} — a root that has moved makes this gate walk \
             a subset of the tree and pass over the rest in silence",
            root.display()
        );
        collect(&dir, &specimens, &mut files);
    }
    files.sort();

    // NON-VACUITY, first half: a walk that comes back empty asserts nothing over nothing. Driven
    // 2026-09-01 — the six roots yield 2,589 `.rs`/`.wat` files with `tests/lint/` cut out. The
    // floor is set well under that so ordinary churn does not red it, and far enough over zero
    // that a broken root or a typo'd extension cannot slip past.
    assert!(
        files.len() >= 2000,
        "the walk found only {} .rs/.wat file(s) across {SCAN_ROOTS:?} — the roots moved or the \
         extension filter broke, and this gate is judging almost nothing",
        files.len()
    );

    let mut seen: Vec<(&str, usize)> = WARD_VOCABULARIES.iter().map(|(w, _)| (*w, 0)).collect();
    let mut control: Vec<(&str, usize)> = seen.clone();
    let mut violations: Vec<String> = Vec::new();

    for f in &files {
        let Ok(text) = std::fs::read_to_string(f) else {
            continue;
        };
        let is_control = f.strip_prefix(root).is_ok_and(|r| r == Path::new(POSITIVE_CONTROL));
        for (line_no, line) in text.lines().enumerate() {
            for (wi, (ward, known)) in WARD_VOCABULARIES.iter().enumerate() {
                for cat in categories_on(line, ward) {
                    seen[wi].1 += 1;
                    if is_control {
                        control[wi].1 += 1;
                    }
                    if !known.contains(&cat) {
                        violations.push(format!(
                            "  {}:{} — rune:{ward}({cat}) is not one of {known:?}",
                            f.display(),
                            line_no + 1
                        ));
                    }
                }
            }
        }
    }

    // NON-VACUITY, second half: per-ward floors, so retiring one vocabulary cannot leave the other
    // vouching for it. Driven 2026-09-01 — 46 `perspicere` runes and 11 `purgare` runes across the
    // six roots. A ward that falls to zero is a convention that was retired without its gate.
    let blind: Vec<String> = seen
        .iter()
        .zip([34usize, 8])
        .filter(|((_, n), floor)| n < floor)
        .map(|((ward, n), floor)| format!("  rune:{ward} — found {n}, floor {floor}"))
        .collect();
    assert!(
        blind.is_empty(),
        "a ward's rune population fell under its floor. Either the convention was retired (then \
         retire its row in WARD_VOCABULARIES and its table in docs/CONVENTIONS.md too) or the \
         scan broke and this gate is passing over runes it can no longer see:\n{}",
        blind.join("\n")
    );

    // NON-VACUITY, third half — the recogniser, which no count can see. `POSITIVE_CONTROL` carries
    // runes from both wards; if either comes back zero, the walk never reached the file or the
    // extractor stopped extracting, and every green above is unearned.
    let unreached: Vec<&str> = control
        .iter()
        .filter(|(_, n)| *n == 0)
        .map(|(ward, _)| *ward)
        .collect();
    assert!(
        unreached.is_empty(),
        "the positive control {POSITIVE_CONTROL} came back with ZERO rune(s) for {unreached:?} — \
         so either the walk never reached that file or the extractor has stopped extracting, and \
         this gate's green means nothing"
    );

    assert!(
        violations.is_empty(),
        "ward rune category outside its closed set ({} site(s)). Either the rune should name a \
         member of its ward's vocabulary, or a NEW category is genuinely warranted — in which \
         case add a row to that ward's table in docs/CONVENTIONS.md FIRST, then extend \
         WARD_VOCABULARIES here. The table is the definition; this is only the gate:\n{}",
        violations.len(),
        violations.join("\n")
    );
}

#[cfg(test)]
mod detector {
    use super::*;

    /// The ward names, BOUND rather than written into the specimens below. Every specimen in this
    /// module interpolates one of these, so no line of this file's source is itself a rune marker
    /// — see the header's note on specimens versus sites.
    const PERSPICERE: &str = "perspicere";
    const PURGARE: &str = "purgare";

    #[test]
    fn the_detector_fires_on_an_invented_category() {
        // The red-build proof, without mutating the tree: feed the detector the exact shape a
        // drifting call site would have.
        let invented = "zzz-not-a-category";
        let bogus = format!("// rune:{PERSPICERE}({invented}) — a category nobody defined");
        assert_eq!(categories_on(&bogus, PERSPICERE), vec![invented]);
        let (_, known) = WARD_VOCABULARIES[0];
        // `invented` is bound rather than written inline, and that is load-bearing against two
        // gates wanting OPPOSITE shapes on this line: `no_loose_string_assert` bans
        // `.contains(&"literal")` and clippy's `manual_contains` bans the `iter().any()` rewrite.
        // A binding satisfies both, exactly as `no_unknown_sequi_rune.rs` records.
        assert!(!known.contains(&invented));
    }

    #[test]
    fn every_documented_category_is_accepted_by_its_own_ward() {
        for (ward, known) in WARD_VOCABULARIES {
            for cat in *known {
                let line = format!("// rune:{ward}({cat}) — a documented reason");
                assert_eq!(categories_on(&line, ward), vec![*cat]);
            }
        }
    }

    #[test]
    fn a_category_valid_for_the_other_ward_is_still_a_violation() {
        // The failure a single shared vocabulary would have hidden: `safety-margin` is a real
        // category, and it is not one of `perspicere`'s.
        let borrowed = "safety-margin";
        let line = format!("// rune:{PERSPICERE}({borrowed}) — borrowed from the sibling ward");
        assert_eq!(categories_on(&line, PERSPICERE), vec![borrowed]);
        let (_, perspicere_known) = WARD_VOCABULARIES[0];
        let (_, purgare_known) = WARD_VOCABULARIES[1];
        assert!(!perspicere_known.contains(&borrowed));
        assert!(purgare_known.contains(&borrowed));
    }

    #[test]
    fn one_ward_does_not_read_the_others_runes() {
        let line = format!("// rune:{PURGARE}(public-api) — exported for downstream consumers");
        assert_eq!(categories_on(&line, PURGARE), vec!["public-api"]);
        assert!(categories_on(&line, PERSPICERE).is_empty());
    }

    #[test]
    fn two_runes_on_one_line_are_both_read() {
        // The `find`-and-advance loop must not stop at the first marker; a line carrying a valid
        // rune beside an invented one has to surface the invented one.
        let line = format!("// rune:{PERSPICERE}(read-once) then rune:{PERSPICERE}(made-up) here");
        assert_eq!(
            categories_on(&line, PERSPICERE),
            vec!["read-once", "made-up"]
        );
    }

    #[test]
    fn a_wat_comment_carries_a_rune_too() {
        // The one `perspicere` rune outside Rust lives in a `.wat` file, where the comment opener
        // is `;;`. The extractor keys on the marker, not on the comment syntax.
        let line = format!(";; rune:{PERSPICERE}(intentional-structure) — the no-alpha door");
        assert_eq!(
            categories_on(&line, PERSPICERE),
            vec!["intentional-structure"]
        );
    }

    #[test]
    fn an_unclosed_rune_is_surfaced_rather_than_skipped() {
        let line = format!("// rune:{PERSPICERE}(read-once — someone dropped the paren");
        let (_, known) = WARD_VOCABULARIES[0];
        let found = categories_on(&line, PERSPICERE);
        assert_eq!(found.len(), 1);
        assert!(!known.contains(&found[0]));
    }

    #[test]
    fn a_line_with_no_rune_yields_nothing() {
        assert!(categories_on("let x = 1; // ordinary comment", PERSPICERE).is_empty());
        // The ward's name in prose without the marker shape is not a rune.
        assert!(categories_on("// purgare flagged this last week", PURGARE).is_empty());
    }
}
