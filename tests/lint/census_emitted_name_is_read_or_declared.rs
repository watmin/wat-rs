//! **A CENSUS COUNTER THE ENGINE EMITS MUST BE READ BY A COST TEST — OR SAY WHY NOT.**
//!
//! Mirror of [`census_name_read_by_a_cost_test_is_emitted`]. That gate is READ ⇒ EMITTED;
//! this is EMITTED ⇒ READ, and it covers `census_count` / `census_count_n` ONLY —
//! `phase_end` names a region whose quantity is always "time spent there", so its name
//! cannot drift from its measurement, and its consumer is the census table, not an assertion.
//!
//! READ ⇒ EMITTED covers both emitters; EMITTED ⇒ READ covers counters only. The asymmetry
//! is deliberate.
//!
//! **The two sets are the sibling's.** This file does not re-scan the engine or the cost
//! tests. It filters the sibling's EMITTED set to the count emitters and uses the sibling's
//! READ set unchanged — those are the definitions with the measured counter-examples
//! (`ebucket`/`tbucket`, two scope cuts) behind them.
//!
//! ## THE EARNED EXEMPTION — `rune:lint(census-emitted-unread)`
//!
//! A counter nobody asserts on is not always waste. It may be a true count on a path a
//! cost test does not currently drive. Then the emit site carries
//! `// rune:lint(census-emitted-unread) <name> — <what it counts, why nothing asserts>`.
//! A rune is a DECLARATION, not a suppression. A name that IS read must not keep one.
//!
//! Shape: the sibling's `rune:lint(census-name-retired)`. Precedent for working a fence
//! down and deleting it rather than leaving it empty: `no_stale_path_in_doc.rs`'s
//! retired `DEFERRED` allowlist.

use super::census_name_read_by_a_cost_test_is_emitted as sibling;
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

const RUNE: &str = "rune:lint(census-emitted-unread)";
const MIN_REASON_CHARS: usize = 40;
const NOT_EMIT: &str = "src/rete/kernel/tests";

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn collect(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for e in entries.flatten() {
        let p = e.path();
        if p.is_dir() {
            collect(&p, out);
        } else if p.extension().and_then(|x| x.to_str()) == Some("rs") {
            out.push(p);
        }
    }
}

/// Co-located rune at an emit site: the `census_count("name")` line, or the line above it.
fn emit_rune(name: &str) -> Option<(String, usize, String)> {
    let skip = root().join(NOT_EMIT);
    let needle = format!("\"{name}\"");
    let mut files = Vec::new();
    collect(&root().join("src"), &mut files);
    for p in files {
        if p.starts_with(&skip) {
            continue;
        }
        let Ok(src) = std::fs::read_to_string(&p) else {
            continue;
        };
        let lines: Vec<&str> = src.lines().collect();
        for (i, line) in lines.iter().enumerate() {
            if !line.contains("census_count") && !line.contains("census_count_n") {
                continue;
            }
            if !line.contains(&needle) {
                continue;
            }
            let prev = if i > 0 { lines[i - 1] } else { "" };
            let blob = format!("{prev}\n{line}");
            if let Some(at) = blob.find(RUNE) {
                let after = &blob[at + RUNE.len()..];
                let tail = after.split('\n').next().unwrap_or("").trim();
                let tail = tail
                    .strip_prefix(name)
                    .map(str::trim)
                    .unwrap_or(tail);
                let tail = tail
                    .strip_prefix('—')
                    .or_else(|| tail.strip_prefix("--"))
                    .or_else(|| tail.strip_prefix('-'))
                    .unwrap_or(tail)
                    .trim();
                let rel = p
                    .strip_prefix(root())
                    .unwrap_or(&p)
                    .display()
                    .to_string();
                return Some((rel, i + 1, tail.to_string()));
            }
        }
    }
    None
}

#[test]
fn every_emitted_census_count_is_read_or_declared() {
    let em = sibling::emitted();
    let rd = sibling::reads();

    // NON-VACUITY. An empty count set would make this gate assert nothing.
    assert!(
        em.count_names.len() > 20,
        "only {} census_count names in the sibling EMITTED set — the filter is not seeing counters",
        em.count_names.len()
    );
    assert!(
        rd.len() > 40,
        "only {} READ names from the sibling — this gate would pass over nothing",
        rd.len()
    );
    for anchor in ["match:calls", "bindkey:alloc", "compiled:exec"] {
        assert!(
            em.count_names.contains(anchor),
            "count EMITTED is missing `{anchor}` — the sibling filter has gone blind"
        );
        assert!(
            rd.iter().any(|r| r.name == *anchor),
            "READ is missing `{anchor}` — a known-read counter vanished from the sibling set"
        );
    }

    let read_names: BTreeSet<&str> = rd.iter().map(|r| r.name.as_str()).collect();
    let mut unread: Vec<&String> = em
        .count_names
        .iter()
        .filter(|n| !read_names.contains(n.as_str()))
        .collect();
    unread.sort();

    let mut stale_runes = Vec::new();
    let mut short_runes = Vec::new();
    let mut undeclared = Vec::new();
    for name in &em.count_names {
        let read = read_names.contains(name.as_str());
        match (read, emit_rune(name)) {
            (true, Some((file, line, _))) => stale_runes.push(format!("  {file}:{line}  {name:?}")),
            (false, Some((_, _, reason))) if reason.chars().count() < MIN_REASON_CHARS => {
                short_runes.push(format!(
                    "  {name:?} reason is {} chars (`{reason}`)",
                    reason.chars().count()
                ));
            }
            (false, Some(_)) => {}
            (false, None) => undeclared.push(name.clone()),
            (true, None) => {}
        }
    }

    assert!(
        stale_runes.is_empty(),
        "\n{} {RUNE} rune(s) sit on a name a cost test DOES read:\n{}\n\nThe reader landed. \
         Drop the rune — an exemption that outlives its reason is how a gate stops gating.\n",
        stale_runes.len(),
        stale_runes.join("\n")
    );
    assert!(
        short_runes.is_empty(),
        "\n{} {RUNE} rune(s) have a reason under {MIN_REASON_CHARS} chars:\n{}\n",
        short_runes.len(),
        short_runes.join("\n")
    );
    assert!(
        undeclared.is_empty(),
        "\n{} census_count name(s) are EMITTED and neither READ by a cost test nor declared.\n\
         An unread counter is where a name rots: the A–M census was \"the counter's name says X, \
         the quantity is Y\".\n\n  {}\n\n\
         FIX — exactly one of three:\n\
         (1) Wire a reader that asserts a NONZERO value (zero cannot tell absent from measured 0).\n\
         (2) Delete the counter — it costs a branch on the hot path.\n\
         (3) `// rune:lint(census-emitted-unread) <name> — <what it counts, why nothing asserts>` \
         on the emit site.\n",
        undeclared.len(),
        undeclared.join("\n  ")
    );
}
