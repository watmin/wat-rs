//! Arc 278 — FactBag is the one owner of the rete fact base.
//!
//! `:wat::rete::FactBag/items` and `:wat::rete::Session/facts` have no form in
//! `wat/` outside `wat/rete/factbag.wat`. The string `"facts"` has no form in
//! `src/rete/` outside `session.rs`'s two doors (`session_facts`,
//! `session_with_facts`).
//!
//! ⛔ THIS IS A BUILD GATE, NOT THE TYPE SYSTEM. A record's `:restricted-to` is
//! parsed, stored, and never enforced
//! (`docs/arc/2026/04/109-kill-std/NOTE-a-records-restricted-to-is-stored-and-never-enforced.md`).
//! `defrecord` cannot even express the map. When rung 3 arrives, the record
//! gains one line and THIS GATE IS DELETED — that deletion is the proof rung 3
//! arrived. The exemption list is empty. A rune does not save a raw access.
//!
//! Gate A is the proof (structural, deterministic).

// rune:lint(no-inlined-wat) — detector specimens are the banned FactBag/Session.facts accessors this gate exists to catch; they are not a world under test

use std::path::{Path, PathBuf};

const FACTBAG: &str = "wat/rete/factbag.wat";
const SESSION_RS: &str = "src/rete/kernel/session.rs";
const BANNED_ITEMS: &str = "FactBag/items";
const BANNED_FACTS: &str = "Session/facts";

fn collect_wat(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(dir) else { return };
    for e in entries.flatten() {
        let p = e.path();
        if p.is_dir() {
            collect_wat(&p, out);
        } else if p.extension().and_then(|x| x.to_str()) == Some("wat") {
            out.push(p);
        }
    }
}

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

fn code_of(line: &str) -> &str {
    match line.find(";;") {
        Some(i) => &line[..i],
        None => line,
    }
}

fn rust_code_of(line: &str) -> &str {
    match line.find("//") {
        Some(i) => &line[..i],
        None => line,
    }
}

fn wat_violations(rel: &str, src: &str) -> Vec<String> {
    let mut out = Vec::new();
    for (i, line) in src.lines().enumerate() {
        let code = code_of(line);
        if code.contains(BANNED_ITEMS) {
            out.push(format!(
                "  {rel}:{}: raw `{BANNED_ITEMS}` (only {FACTBAG} may unwrap)",
                i + 1
            ));
        }
        if code.contains(BANNED_FACTS) {
            out.push(format!(
                "  {rel}:{}: raw `{BANNED_FACTS}` (only {FACTBAG} may read the field)",
                i + 1
            ));
        }
    }
    out
}

fn rust_facts_hits(rel: &str, src: &str) -> Vec<String> {
    let mut out = Vec::new();
    for (i, line) in src.lines().enumerate() {
        let code = rust_code_of(line);
        if code.contains("\"facts\"") {
            out.push(format!("  {rel}:{}", i + 1));
        }
    }
    out
}

mod detector {
    use super::*;

    #[test]
    fn factbag_items_outside_the_owner_is_a_hit() {
        let src = "(:wat::rete::FactBag/items b)\n";
        let v = wat_violations("insert.wat", src);
        assert_eq!(v.len(), 1, "raw FactBag/items must redden; got {v:?}");
    }

    #[test]
    fn session_facts_outside_the_owner_is_a_hit() {
        let src = "(:wat::rete::Session/facts session)\n";
        let v = wat_violations("insert.wat", src);
        assert_eq!(v.len(), 1, "raw Session/facts must redden; got {v:?}");
    }

    #[test]
    fn a_comment_is_not_a_hit() {
        let src = ";; (:wat::rete::Session/facts session)\n";
        let v = wat_violations("query.wat", src);
        assert!(v.is_empty(), "comments are prose; got {v:?}");
    }

    #[test]
    fn rust_facts_string_outside_session_rs_is_a_hit() {
        let hits = rust_facts_hits("src/rete/kernel/insert.rs", "session_named_field(s, \"facts\")\n");
        assert_eq!(hits.len(), 1, "raw \"facts\" in insert.rs must redden; got {hits:?}");
    }
}

#[test]
fn no_raw_factbag_access_outside_the_owner() {
    let manifest = Path::new(env!("CARGO_MANIFEST_DIR"));
    let wat_root = manifest.join("wat");
    let mut wat_files = Vec::new();
    collect_wat(&wat_root, &mut wat_files);
    wat_files.sort();

    // NON-VACUITY: the walk must actually find the wat/ tree. A typo'd path finding
    // zero files would make this gate pass forever while checking nothing. 8 is a
    // floor of what the walk finds today (driven 2026-09-06), not a count of the corpus.
    assert!(
        wat_files.len() >= 8,
        "factbag gate found only {} .wat files under wat/ — it is not looking at the tree it claims to guard",
        wat_files.len()
    );

    let mut wat_violations_all = Vec::new();
    let mut owner_has_items = false;
    let mut owner_has_facts = false;
    for f in &wat_files {
        let rel = f
            .strip_prefix(manifest)
            .unwrap_or(f)
            .to_string_lossy()
            .replace('\\', "/");
        let src = std::fs::read_to_string(f).unwrap_or_else(|e| panic!("read {rel}: {e}"));
        if rel == FACTBAG {
            owner_has_items = src.lines().any(|l| code_of(l).contains(BANNED_ITEMS));
            owner_has_facts = src.lines().any(|l| code_of(l).contains(BANNED_FACTS));
            continue;
        }
        wat_violations_all.extend(wat_violations(&rel, &src));
    }

    assert!(
        owner_has_items,
        "{FACTBAG} must contain the ONE `{BANNED_ITEMS}` unwrap; the extractor did not find it"
    );
    assert!(
        owner_has_facts,
        "{FACTBAG} must contain the ONE `{BANNED_FACTS}` read; the extractor did not find it"
    );
    assert!(
        wat_violations_all.is_empty(),
        "raw FactBag access outside {FACTBAG}:\n{}",
        wat_violations_all.join("\n")
    );

    let rete_root = manifest.join("src/rete");
    let mut rs_files = Vec::new();
    collect_rs(&rete_root, &mut rs_files);
    rs_files.sort();
    // NON-VACUITY: the walk must actually find src/rete/. A typo'd path finding
    // zero files would make the `"facts"` rule pass over nothing.
    assert!(
        rs_files.len() >= 4,
        "factbag gate found only {} .rs files under src/rete/ — it is not looking at the tree it claims to guard",
        rs_files.len()
    );

    let mut rust_violations = Vec::new();
    let mut door_hits = 0usize;
    for f in &rs_files {
        let rel = f
            .strip_prefix(manifest)
            .unwrap_or(f)
            .to_string_lossy()
            .replace('\\', "/");
        let src = std::fs::read_to_string(f).unwrap_or_else(|e| panic!("read {rel}: {e}"));
        let hits = rust_facts_hits(&rel, &src);
        if rel == SESSION_RS {
            door_hits = hits.len();
            continue;
        }
        rust_violations.extend(hits);
    }
    assert_eq!(
        door_hits, 2,
        "{SESSION_RS} must contain exactly two `\"facts\"` strings (session_facts + session_with_facts); found {door_hits}"
    );
    assert!(
        rust_violations.is_empty(),
        "raw `\"facts\"` outside {SESSION_RS}'s two doors:\n{}",
        rust_violations.join("\n")
    );
}
