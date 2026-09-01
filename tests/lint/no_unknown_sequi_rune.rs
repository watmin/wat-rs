//! THE `rune:sequi` CATEGORY IS A CLOSED SET — a fifth category is a red build.
//!
//! `sequi` asks whether state follows visibly through the types. State reached *around* the
//! signature (a `thread_local!`, a process global, a link-time registry, a discarded error
//! detail) earns a `// rune:sequi(<category>) — <reason>` rune declaring it conscious. The
//! category vocabulary and its discriminating question live in `docs/CONVENTIONS.md`,
//! "The `rune:sequi` vocabulary".
//!
//! ## What this gate can and cannot do — stated, because the gap is the whole point
//!
//! It closes the SET: a category invented at a call site fails here, naming itself. It does
//! **not** — and cannot — tell you a rune picked the WRONG one of the four.
//!
//! That distinction is not academic; it is the incident that produced this file. On 2026-08-25
//! `sequi` found `ARM_TABLE` (`src/rete/kernel/arm.rs`) categorised `host-idiom` while
//! `EXEC_ARENA` (`src/rete/expr_ir.rs`), two files over, carried `ambient-context` — the same
//! `thread_local!` shape, the same invisibility to the signature, holding the same kind of
//! thing. Both runes were valid names. Both were thoughtfully written. Nothing could catch the
//! disagreement, because the four categories had no written definition to be checked against.
//!
//! So the fix is two-part and only one part is mechanical: the TABLE in `CONVENTIONS.md`
//! carries the discriminating question (*what does a reader lose by not seeing this in the
//! signature?*), and this lint keeps the set from growing a fifth member behind the table's
//! back. Claiming the lint prevents miscategorisation would be the decoration this codebase
//! keeps finding and removing.

use std::path::{Path, PathBuf};

/// The closed set. Adding a member here is a deliberate act that must be accompanied by a row
/// in `docs/CONVENTIONS.md`'s vocabulary table — the table is the definition, this is the gate.
const SEQUI_CATEGORIES: &[&str] = &[
    "ambient-context",
    "performance-counter",
    "host-idiom",
    "reclassified-by-caller",
];

fn collect_rs(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for e in entries.flatten() {
        let p = e.path();
        if p.is_dir() {
            let name = p.file_name().and_then(|n| n.to_str()).unwrap_or("");
            if name == "target" {
                continue;
            }
            collect_rs(&p, out);
        } else if p.extension().and_then(|x| x.to_str()) == Some("rs") {
            out.push(p);
        }
    }
}

/// Every `rune:sequi(...)` category named on one line, in source order.
///
/// Split on the needle rather than regex: the marker is a fixed string, and the category is
/// everything up to the closing paren. A line bearing two runes yields two categories.
fn sequi_categories_on(line: &str) -> Vec<&str> {
    let mut out = Vec::new();
    let mut rest = line;
    while let Some(at) = rest.find("rune:sequi(") {
        let after = &rest[at + "rune:sequi(".len()..];
        match after.find(')') {
            Some(close) => {
                out.push(&after[..close]);
                rest = &after[close..];
            }
            // An unclosed `rune:sequi(` is itself malformed; surface it as an empty category so
            // the caller reports it rather than silently walking past a broken rune.
            None => {
                out.push(after);
                break;
            }
        }
    }
    out
}

#[test]
fn every_sequi_rune_names_a_known_category() {
    let mut files = Vec::new();
    collect_rs(Path::new("src"), &mut files);
    // NON-VACUITY: the guard below is this gate's answer. Note the RELATIVE path — this walk
    // depends on cargo's working directory, which is exactly the kind of assumption that silently
    // stops holding. 213 .rs files are found today (driven 2026-09-01).
    assert!(
        !files.is_empty(),
        "no .rs files found under src/ — the lint is scanning nothing and would pass vacuously"
    );

    let mut seen = 0usize;
    let mut violations: Vec<String> = Vec::new();
    for f in &files {
        let Ok(text) = std::fs::read_to_string(f) else {
            continue;
        };
        for (i, line) in text.lines().enumerate() {
            for cat in sequi_categories_on(line) {
                seen += 1;
                if !SEQUI_CATEGORIES.contains(&cat) {
                    violations.push(format!(
                        "  {}:{} — rune:sequi({cat}) is not one of the four categories {SEQUI_CATEGORIES:?}",
                        f.display(),
                        i + 1
                    ));
                }
            }
        }
    }

    assert!(
        seen > 0,
        "scanned {} files under src/ and found ZERO rune:sequi( markers — either the rune \
         convention was retired (then retire this lint too) or the scan broke",
        files.len()
    );

    assert!(
        violations.is_empty(),
        "rune:sequi category outside the closed set ({} site(s)). Either the rune should name one \
         of the four, or a FIFTH category is genuinely warranted — in which case add a row to the \
         vocabulary table in docs/CONVENTIONS.md FIRST, then extend SEQUI_CATEGORIES here:\n{}",
        violations.len(),
        violations.join("\n")
    );
}

#[test]
fn the_detector_fires_on_an_invented_category() {
    // The red-build proof, without mutating the tree: feed the detector the exact shape a
    // drifting call site would have. A gate that cannot go red is decoration.
    let invented = "worker-memo";
    let bogus = "// rune:sequi(worker-memo) — a category nobody defined";
    assert_eq!(sequi_categories_on(bogus), vec![invented]);
    // `invented` is bound rather than written inline, and that is load-bearing against two
    // gates that want OPPOSITE shapes on this one line. `no_loose_string_assert` bans
    // `.contains(&"literal")` — it cannot distinguish exact set membership from a loose
    // `str::contains`, and relaxing it would leave a real hole. Clippy's `manual_contains` bans
    // the `iter().any(|c| *c == "literal")` rewrite. Neither is wrong, and an `#[allow]` on
    // either would be silencing a gate to win an argument between gates. A binding satisfies
    // both: clippy gets its `contains`, and the loose-assert lint's documented carve-out
    // (`.contains(&item)` — the arg is a value, not a literal) applies exactly as written.
    assert!(!SEQUI_CATEGORIES.contains(&invented));

    // And it does NOT fire on the four that are real.
    for cat in SEQUI_CATEGORIES {
        let line = format!("// rune:sequi({cat}) — a documented reason");
        assert_eq!(sequi_categories_on(&line), vec![*cat]);
    }
}

#[test]
fn two_runes_on_one_line_are_both_read() {
    // The `find`-and-advance loop must not stop at the first marker; a line carrying two runes
    // that differ (one valid, one invented) has to surface the invented one.
    let line = "// rune:sequi(host-idiom) then rune:sequi(made-up) on the same line";
    assert_eq!(sequi_categories_on(line), vec!["host-idiom", "made-up"]);
}

#[test]
fn a_line_with_no_rune_yields_nothing() {
    assert!(sequi_categories_on("let x = 1; // ordinary comment").is_empty());
    // The word appearing in prose without the marker shape is not a rune.
    assert!(sequi_categories_on("// sequi found this last week").is_empty());
}
