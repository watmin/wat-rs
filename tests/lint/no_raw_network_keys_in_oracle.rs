//! Arc 278 F1 — a raw `PersistentMap/keys network` walk in `wat/rete/oracle/**`
//! has no form outside `:wat::rete::topological-node-ids`.
//!
//! HAMT key order is not topological. `fire-once$oracle` learned this once
//! (`fire.wat`'s WHY-sort); `harvest-support` did not, and the referee for
//! explain attributed a derived fact to a different rule on different runs.
//! The sort lives in one verb. A raw `keys network` outside that verb has
//! no form — not even behind a rune. `node-parents` walks the verb too:
//! its parent-id VECTOR order is the inner first-wins over tokens, which
//! is observable in `Support/token` / the derivation tree.
//!
//! Gate A is the proof (structural, deterministic). Gate B (the differential)
//! is behavioural and was only probabilistically red at HEAD.

// rune:lint(no-inlined-wat) — detector specimens are the banned PersistentMap/keys network call this gate exists to catch; they are not a world under test

use std::path::{Path, PathBuf};

const ORACLE: &str = "wat/rete/oracle";
const VERB: &str = ":wat::rete::topological-node-ids";
const BANNED: &str = "PersistentMap/keys network";

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

/// Code (before `;;`) contains the banned call.
fn code_has_banned_keys(line: &str) -> bool {
    let code = match line.find(";;") {
        Some(i) => &line[..i],
        None => line,
    };
    code.contains(BANNED)
}

fn defn_name(line: &str) -> Option<&str> {
    let t = line.trim_start();
    t.strip_prefix('(')?
        .strip_prefix(":wat::core::defn ")?
        .split_whitespace()
        .next()
}

/// Violations in one source: banned `keys network` outside the verb.
/// A rune does not exempt — the exemption list is empty.
fn violations_in(rel: &str, src: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut current = "<preamble>";
    let mut in_verb = false;
    for (i, line) in src.lines().enumerate() {
        if let Some(name) = defn_name(line) {
            current = name;
            in_verb = name == VERB;
        }
        if !code_has_banned_keys(line) {
            continue;
        }
        if in_verb {
            continue;
        }
        out.push(format!(
            "  {rel}:{} in `{current}`: raw `{BANNED}` (not {VERB})",
            i + 1
        ));
    }
    out
}

mod detector {
    use super::*;

    fn specimen(rest: &str) -> String {
        format!("{}{}", "(", rest)
    }

    #[test]
    fn a_raw_walk_outside_the_verb_is_a_hit() {
        let src = specimen(":wat::core::defn :wat::rete::harvest-support\n  [n <- :wat::core::PersistentMap]\n  (:wat::core::PersistentMap/keys network)))\n");
        let v = violations_in("explain.wat", &src);
        assert_eq!(v.len(), 1, "raw walk must redden; got {v:?}");
    }

    #[test]
    fn the_verb_body_is_not_a_hit() {
        let src = specimen(":wat::core::defn :wat::rete::topological-node-ids\n  [network <- :wat::core::PersistentMap]\n  (:wat::core::PersistentMap/keys network)))\n");
        let v = violations_in("pass.wat", &src);
        assert!(v.is_empty(), "the verb is the one allowed walk; got {v:?}");
    }

    #[test]
    fn a_runed_walk_is_still_a_hit() {
        let src = specimen(":wat::core::defn :wat::rete::node-parents\n  [c <- :wat::core::i64 network <- :wat::core::PersistentMap]\n    (:wat::core::PersistentMap/keys network)))  ;; rune:lint(oracle-keys-order-insensitive) — fold builds a set\n");
        let v = violations_in("pass.wat", &src);
        assert_eq!(v.len(), 1, "the exemption list is empty; a rune must not save a raw walk; got {v:?}");
    }
}

#[test]
fn no_raw_network_keys_walk_outside_topological_node_ids() {
    let manifest = Path::new(env!("CARGO_MANIFEST_DIR"));
    let oracle = manifest.join(ORACLE);
    let mut files = Vec::new();
    collect_wat(&oracle, &mut files);
    files.sort();

    // NON-VACUITY: the walk must actually find the oracle tree. A typo'd path
    // finding zero files would make this gate pass forever while checking nothing.
    assert!(
        files.len() >= 4,
        "oracle keys-walk found only {} .wat files under {ORACLE} — it is not looking at the tree it claims to guard",
        files.len()
    );

    let mut violations = Vec::new();
    let mut verb_has_keys = false;
    let mut banned_in_code = 0usize;
    for f in &files {
        let rel = f
            .strip_prefix(manifest)
            .unwrap_or(f)
            .to_string_lossy()
            .replace('\\', "/");
        let src = std::fs::read_to_string(f).unwrap_or_else(|e| panic!("read {rel}: {e}"));
        let mut in_verb = false;
        for line in src.lines() {
            if let Some(name) = defn_name(line) {
                in_verb = name == VERB;
            }
            if code_has_banned_keys(line) {
                banned_in_code += 1;
                if in_verb {
                    verb_has_keys = true;
                }
            }
        }
        violations.extend(violations_in(&rel, &src));
    }

    assert!(
        verb_has_keys,
        "{VERB} must exist and be the one keys-walk; the extractor did not find it"
    );
    assert_eq!(
        banned_in_code, 1,
        "exactly one `{BANNED}` in oracle code (the verb); found {banned_in_code} — the exemption list must stay empty"
    );

    assert!(
        violations.is_empty(),
        "raw PersistentMap/keys network walk outside {VERB}:\n{}",
        violations.join("\n")
    );
}
