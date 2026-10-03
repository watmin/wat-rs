//! Arc 278 F1 — a raw `map::keys network` walk (grok's own spelling was `PersistentMap/keys network`, retired on this tree) in `wat/rete/oracle/**`
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

// rune:lint(no-inlined-wat) — detector specimens are the banned map::keys network call this gate exists to catch; they are not a world under test

use std::path::{Path, PathBuf};

const ORACLE: &str = "wat/rete/oracle";
const VERB: &str = ":wat::rete::topological-node-ids";
const KEYS: &str = ":wat::core::keys";
// ⛔ RE-SPELLED at replay #398 (finding 33's class) — grok's own literal was
// `"PersistentMap/keys network"`, the accessor's spelling on grok's tree. This tree's own
// (pre-existing, earlier) rename retired `:wat::core::PersistentMap/keys` in favor of
// `:wat::map::keys` (confirmed: `wat --check` on the old spelling raises a retirement error
// with the remedy `:wat::map::keys`), so the banned phrase is re-spelled to match the LIVE
// accessor this tree's own `:wat::rete::topological-node-ids` now calls.


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

struct KeysWalk {
    in_verb: bool,
    line: i64,
    defn: String,
}

/// `(:wat::core::keys network)` / `(wat.map/keys network)`, by identity of the head.
fn keys_walks(src: &str) -> Vec<KeysWalk> {
    let forms = wat::parse_all_with_file(src, "<oracle>")
        .unwrap_or_else(|e| panic!("parse <oracle>: {e:?}"));
    let mut out = Vec::new();
    fn walk(node: &wat::WatAST, in_verb: bool, defn: &str, out: &mut Vec<KeysWalk>) {
        let wat::WatAST::List(items, _) = node else {
            for child in node.children().iter() {
                walk(child, in_verb, defn, out);
            }
            return;
        };
        let head = items.first().and_then(crate::decl_identity::canon);
        let mut here = in_verb;
        let mut here_defn = defn.to_string();
        if head.as_deref() == Some(":wat::core::defn") {
            if let Some(name) = items.get(1).and_then(crate::decl_identity::canon) {
                here = name == VERB;
                here_defn = name;
            }
        }
        if head.as_deref() == Some(KEYS) {
            let names_network = items.iter().skip(1).any(|n| {
                matches!(n, wat::WatAST::Symbol(id, _) if id.as_str() == "network")
            });
            if names_network {
                out.push(KeysWalk {
                    in_verb: here,
                    line: node.span().line,
                    defn: here_defn.clone(),
                });
            }
        }
        for child in items.iter().skip(1) {
            walk(child, here, &here_defn, out);
        }
    }
    for form in &forms {
        walk(form, false, "<preamble>", &mut out);
    }
    out
}

/// Violations in one source: a keys-walk of `network` outside the verb.
/// A rune does not exempt — the exemption list is empty.
fn violations_in(rel: &str, src: &str) -> Vec<String> {
    let walks = keys_walks(src);
    walks
        .into_iter()
        .filter(|w| !w.in_verb)
        .map(|w| format!("  {rel}:{} in `{}`: raw `{KEYS}` of network (not {VERB})", w.line, w.defn))
        .collect()
}

mod detector {
    use super::*;

    fn specimen(rest: &str) -> String {
        format!("{}{}", "(", rest)
    }

    #[test]
    fn a_raw_walk_outside_the_verb_is_a_hit() {
        let src = specimen(":wat::core::defn :wat::rete::harvest-support\n  [n <- :wat::core::PersistentMap]\n  (:wat::core::keys network))\n");
        let v = violations_in("explain.wat", &src);
        assert_eq!(v.len(), 1, "raw walk must redden; got {v:?}");
    }

    #[test]
    fn the_verb_body_is_not_a_hit() {
        let src = specimen(":wat::core::defn :wat::rete::topological-node-ids\n  [network <- :wat::core::PersistentMap]\n  (:wat::core::keys network))\n");
        let v = violations_in("pass.wat", &src);
        assert!(v.is_empty(), "the verb is the one allowed walk; got {v:?}");
    }

    #[test]
    fn a_runed_walk_is_still_a_hit() {
        let src = specimen(":wat::core::defn :wat::rete::node-parents\n  [c <- :wat::core::i64 network <- :wat::core::PersistentMap]\n    (:wat::core::keys network))  ;; rune:lint(oracle-keys-order-insensitive) — fold builds a set\n");
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
        let walks = keys_walks(&src);
        for walk in &walks {
            banned_in_code += 1;
            if walk.in_verb {
                verb_has_keys = true;
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
        "exactly one `{KEYS}` of `network` in oracle code (the verb); found {banned_in_code} — the exemption list must stay empty"
    );

    assert!(
        violations.is_empty(),
        "raw map::keys network walk outside {VERB}:\n{}",
        violations.join("\n")
    );
}
