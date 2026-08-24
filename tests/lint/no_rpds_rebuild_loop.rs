//! THE NO-REBUILD-LOOP LINT — a persistent structure built in a loop is a TRANSIENT.
//!
//! `rpds` exposes two families for every mutation: a **copying** one (`push_back`, `insert`, …)
//! that returns a new structure, and a **`_mut`** one (`push_back_mut`, `insert_mut`, …) that
//! writes in place. They are not "slower vs faster" — they are categorically different, and the
//! reason is visible in rpds's own source (`rpds-1.2.1`, `src/vector/mod.rs`):
//!
//! ```ignore
//! pub fn push_back(&self, v: T) -> Vector<T, P> {
//!     let mut new_vector = self.clone();   // every trie node's refcount 1 -> 2
//!     new_vector.push_back_mut(v);
//!     new_vector
//! }
//! fn assoc(&mut self, …) { SharedPointer::make_mut(&mut self.root).assoc(…) }
//! ```
//!
//! `make_mut` copies a node **only when it is shared**. So the `clone()` inside the copying API is
//! itself what forces a full root→leaf **path copy**, on every call. Building a 40,000-element
//! vector that way allocates ~4 nodes per element and throws 39,999 intermediate versions away
//! unread.
//!
//! Measured on arc 278's fanout cell (`0416d1a5`): converting ONE function, `production_to_pm`, took
//! `out:production` from **28.53 ms to 4.47 ms** and the whole fire from 105.76 to 85.82 ms.
//!
//! ## What this lint forbids — a SHAPE, not a function
//!
//! It is **not** "prefer the `_mut` API". The copying form is correct and *required* whenever the
//! previous version is still live:
//!
//! ```ignore
//! let pv = match pm.get(&k) { Some(PV(existing)) => existing.clone(), _ => new_sync() };
//! pm.insert_mut(k, Value::wat__core__PersistentVector(pv.push_back(fact.clone())));
//! //                                                   ^^^^^^^^^ CORRECT — `pv` shares nodes
//! //                                                   with the map's live copy; it MUST copy.
//! ```
//!
//! What is banned is the **self-reassignment**:
//!
//! ```ignore
//! x = x.push_back(v)     // BANNED: assigning over `x` proves the old `x` is dead, so the path
//!                        //   copy this form forces can never be observed by anyone.
//! y = x.push_back(v)     // fine — different binding, the old version is still live
//! ```
//!
//! That is why the detector is *sound* rather than heuristic: the shape itself carries the proof
//! that the copy is unobservable. No aliasing analysis, no per-site judgement.
//!
//! ## The exemption — narrow, and currently unused
//!
//! The soundness argument above holds for a **borrowing** API (`fn push_back(&self) -> Self`),
//! which is what rpds gives us. It does **not** hold for a **consuming builder**
//! (`fn insert(mut self, …) -> Self`), where `x = x.insert(…)` is a *move*, allocates nothing, and
//! is the idiomatic form. No such type trips this lint today, but the case is real, so a
//! co-located `// rune:lint(no-rpds-rebuild-loop) — <reason>` is available. The reason must name
//! the method as **self-consuming** — "it looked fine" is not a reason (excusare: an exemption
//! earns its standing or it is struck).
//!
//! ## Armed at ZERO
//!
//! `no_rc_use`'s header states the principle this file obeys: *a lint raised at zero is a wall, a
//! lint raised at 1306 is a campaign* (`no_inlined_edn` is the campaign, still red). The sweep
//! (`efa9b5a2`) emptied the class across six files; this is the only cheap moment to close it.
//!
//! And the class has a demonstrated ability to regrow: `no_rc_use` exists because a rider copied
//! `Rc<ShadowNode>` verbatim out of a single-threaded reference implementation, where it was
//! correct. The sweep fixes the instances; only the wall stops the next sketch.

use std::path::{Path, PathBuf};
use std::sync::LazyLock;

use regex::Regex;

fn collect_rs(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(dir) else { return };
    for e in entries.flatten() {
        let p = e.path();
        if p.is_dir() {
            let name = p.file_name().and_then(|n| n.to_str()).unwrap_or("");
            if name == "target" || name == ".claude" {
                continue;
            }
            collect_rs(&p, out);
        } else if p.extension().and_then(|x| x.to_str()) == Some("rs") {
            out.push(p);
        }
    }
}

/// `<lhs> = <recv>.<method>(` — the two identifiers captured SEPARATELY, because the `regex`
/// crate has no backreferences; the equality test lives in `rebuild_loop` below.
///
/// The alternation is ordered longest-first so `push_back` wins over `push`. A `_mut` call can
/// never match: after `push_back` the next character is `_`, not `(`, and every shorter
/// alternative fails the same way.
static REBUILD: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"^\s*(?:let\s+(?:mut\s+)?)?([A-Za-z_][A-Za-z0-9_]*)\s*=\s*([A-Za-z_][A-Za-z0-9_]*)\s*\.\s*(push_back|push_front|drop_last|drop_first|insert|remove|push|set)\s*\(",
    )
    .unwrap()
});

/// True iff `line` rebuilds a binding from itself through a copying persistent-collection call.
fn rebuild_loop(line: &str) -> bool {
    // A comment describes the shape; it does not perform it.
    if line.trim_start().starts_with("//") {
        return false;
    }
    match REBUILD.captures(line) {
        // The whole rule: the assigned binding and the receiver must be the SAME identifier.
        Some(c) => c.get(1).map(|m| m.as_str()) == c.get(2).map(|m| m.as_str()),
        None => false,
    }
}

/// Join a line whose code ends at a bare `=` with the one after it, so a rustfmt-wrapped
/// `x =\n    x.push_back(v);` is not invisible to a line-based detector. A line-based grep that
/// cannot see a wrapped form is a gate that cannot reach — the exact false-pass the arc-249
/// INSCRIPTION audit hit.
fn logical_lines(src: &str) -> Vec<(usize, String)> {
    let raw: Vec<&str> = src.lines().collect();
    let mut out = Vec::with_capacity(raw.len());
    for (i, line) in raw.iter().enumerate() {
        if line.trim_end().ends_with('=') {
            if let Some(next) = raw.get(i + 1) {
                out.push((i + 1, format!("{} {}", line.trim_end(), next.trim_start())));
                continue;
            }
        }
        out.push((i + 1, (*line).to_string()));
    }
    out
}

#[test]
fn a_persistent_structure_built_in_a_loop_is_a_transient() {
    let manifest = env!("CARGO_MANIFEST_DIR");
    let root = Path::new(manifest);

    let mut files = Vec::new();
    collect_rs(&root.join("src"), &mut files);
    if let Ok(entries) = std::fs::read_dir(root.join("crates")) {
        for e in entries.flatten() {
            collect_rs(&e.path().join("src"), &mut files);
        }
    }
    files.sort();

    // A discovering walk must prove it discovered something, or an empty sweep reads as clean.
    assert!(
        files.len() > 50,
        "the no-rebuild-loop walk found only {} .rs files — it is not reaching the tree, so its \
         green means nothing (a gate that discovers its inputs must floor-assert the count).",
        files.len()
    );

    let mut violations = Vec::new();
    for f in &files {
        let Ok(src) = std::fs::read_to_string(f) else { continue };
        let rel = f.strip_prefix(manifest).unwrap_or(f).display().to_string();
        for (lineno, line) in logical_lines(&src) {
            if rebuild_loop(&line) && !line.contains("// rune:lint(no-rpds-rebuild-loop)") {
                violations.push(format!("{}:{}  {}", rel, lineno, line.trim()));
            }
        }
    }

    assert!(
        violations.is_empty(),
        "\n\n🔥🔥🔥 COPY-PER-ELEMENT REBUILD — {} site(s) of `x = x.<copying-call>(…)`.\n\
         \n\
         `Vector::push_back(&self)` starts with `self.clone()`, which raises every trie node's\n\
         refcount to 2, so the `make_mut` inside `assoc` is FORCED to copy the whole root->leaf\n\
         path — and you then throw that copy away by assigning over the binding. Building N\n\
         elements this way allocates ~4 nodes per element for nothing.\n\
         \n\
         THE FIX — the `_mut` twin, which leaves the refcount at 1 so the write lands in place:\n\
         \n\
             x = x.push_back(v);   ->   x.push_back_mut(v);\n\
             x = x.insert(k, v);   ->   x.insert_mut(k, v);\n\
         \n\
         The binding is already `let mut` (it had to be, to be reassigned), so nothing else moves.\n\
         \n\
         ⚠ THIS IS A RULE ABOUT A SHAPE, NOT ABOUT WHICH FUNCTION IS NICER. The copying form is\n\
         CORRECT wherever the previous version is still live — `y = x.push_back(v)` and a nested\n\
         `pm.insert_mut(k, PV(pv.push_back(f)))` are both fine and are not flagged. Only the\n\
         SELF-reassignment is banned, because assigning over the binding proves the old version is\n\
         dead and therefore that the copy can never be observed.\n\
         \n\
         If the method is a SELF-CONSUMING builder (`fn m(mut self, …) -> Self`), the\n\
         reassignment is a move and allocates nothing — add a co-located\n\
         `// rune:lint(no-rpds-rebuild-loop) — <reason naming the method as self-consuming>`.\n\
         \n\
         Offenders:\n\n{}\n",
        violations.len(),
        violations.join("\n"),
    );
}

#[cfg(test)]
mod detector_tests {
    use super::{logical_lines, rebuild_loop};

    #[test]
    fn matches_the_self_reassignment_shape() {
        assert!(rebuild_loop("        pv = pv.push_back(v);"));
        assert!(rebuild_loop("    pm = pm.insert(k_val, v_val);"));
        assert!(rebuild_loop("                out = out.push_back(elem.clone());"));
        assert!(rebuild_loop("    m = m.remove(&k);"));
        assert!(rebuild_loop("    st = st.push(frame);"));
        assert!(rebuild_loop("    v = v.set(0, x);"));
        // A shadowing rebind discards the old version just as an assignment does.
        assert!(rebuild_loop("    let pv = pv.push_back(v);"));
        assert!(rebuild_loop("    let mut pv = pv.push_back(v);"));
    }

    #[test]
    fn does_not_match_a_different_binding() {
        // The old version is still live — this is the legitimate copying use.
        assert!(!rebuild_loop("    let pv2 = pv.push_back(v);"));
        assert!(!rebuild_loop("    out = other.push_back(v);"));
        assert!(!rebuild_loop("    acc = base.insert(k, v);"));
    }

    #[test]
    fn does_not_match_the_nested_inner_call() {
        // kernel.rs's one nested site: outer converted, inner MUST stay copying because `pv`
        // shares nodes with the map's live value. Flagging this line would be a false positive
        // that tells the reader to break correct code.
        assert!(!rebuild_loop(
            "                pm.insert_mut(k, Value::wat__core__PersistentVector(pv.push_back(fact.clone())));"
        ));
    }

    #[test]
    fn does_not_match_the_mut_family_it_is_asking_for() {
        assert!(!rebuild_loop("        pv.push_back_mut(v);"));
        assert!(!rebuild_loop("        pm.insert_mut(k, v);"));
        // Even written as an assignment, `_mut` returns () and is not the banned shape.
        assert!(!rebuild_loop("        x = x.push_back_mut(v);"));
    }

    #[test]
    fn does_not_match_comments() {
        assert!(!rebuild_loop("// pv = pv.push_back(v);"));
        assert!(!rebuild_loop("    /// `pm = pm.insert(k, v)` is the banned shape"));
    }

    #[test]
    fn sees_through_a_wrapped_assignment() {
        // A line-based detector that cannot see the rustfmt-wrapped form is a gate that cannot
        // reach; `logical_lines` is what stops that, so it gets its own proof.
        let src = "fn f() {\n    some_long_binding =\n        some_long_binding.push_back(v);\n}\n";
        let joined = logical_lines(src);
        assert!(
            joined.iter().any(|(_, l)| rebuild_loop(l)),
            "the wrapped form was invisible: {joined:?}"
        );
    }

    #[test]
    fn wrapping_does_not_manufacture_a_false_positive() {
        let src = "fn f() {\n    a =\n        b.push_back(v);\n}\n";
        let joined = logical_lines(src);
        assert!(!joined.iter().any(|(_, l)| rebuild_loop(l)));
    }
}
