//! THE NO-`Rc` LINT — a structural wall keeping shared ownership thread-crossable.
//!
//! `std::rc::Rc` is a non-atomic refcount: it is **not `Send`**, so anything holding one cannot
//! cross a thread boundary. This substrate crosses thread boundaries everywhere — `run-thread`
//! loci, `defservice` actors, the spawn tiers — and it chose atomicity deliberately long ago:
//! `HashTrieMapSync` / `VectorSync` (the *Sync* rpds variants) appear 56× in `src/rete/kernel.rs`
//! alone, and `Arc` appears 61× in `runtime.rs`, 24× in `rete/kernel.rs`, 20× in `io.rs`.
//!
//! Until 2026-07-31 the tree held **zero** `Rc`. Then the alpha-discrimination-tree stone landed
//! nine of them in one file, because its design sketch copied `Rc<ShadowNode>` verbatim out of
//! `holon-lab-ddos/veth-lab/filter/src/tree.rs` — a reference implementation that is a
//! **single-threaded userspace compiler**, where `Rc` is correct and `Arc` would be waste. The
//! constraint did not transfer; the type did.
//!
//! That is the failure this wall exists for. The code compiled and was sound *at that moment*
//! (built and consumed inside one `fire_fixpoint_delta` call, one thread) — `Rc` only stops
//! compiling once the structure is hoisted to compile-time and rides a `Session` across a locus,
//! which is the very next queued stone. A latent blocker that type-checks today is exactly the
//! kind a convention does not catch, and exactly the kind a rider inherits from a sketch.
//!
//! Builder's ruling (2026-07-31): *"i do not trust using Rc — we have use Arc /everywhere/ for
//! anything that needs it."* Armed at **zero offenders**, which is the only cheap moment to arm a
//! wall: a lint raised at zero is a wall, a lint raised at 1306 is a campaign (see
//! `no_inlined_edn`).
//!
//! ## Scope
//!
//! `src/` and `crates/*/src/` — all shipped Rust. A line whose match sits in a Rust
//! **string-continuation** (`\n\`) is message text describing shapes to a user, not code holding
//! one; the sole such line today is `crates/wat-macros/src/wat_value.rs:121`, a derive diagnostic
//! naming `Box<Self> / Arc<Self> / Rc<Self>` as accepted user field shapes. Real code never
//! declares an `Rc` on a string-continuation line.
//!
//! ## The exemption
//!
//! A genuinely single-threaded structure that provably never crosses a locus may carry a
//! co-located `// rune:lint(no-rc) — <reason>`. The reason must name **why it cannot cross**, not
//! that it happens not to today — `Rc` compiling is not evidence it is safe to keep (excusare: the
//! reason must earn its standing; the alpha tree compiled fine and was still wrong).

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

/// `Rc` in type position (`Rc<…>`), path position (`Rc::new`), or an import of the module.
///
/// The leading `\b` is load-bearing in the other direction from what it looks like: `Arc` cannot
/// match `Rc` at all (`A`-`r`-`c` vs `R`-`c` — different case, different letters), so the boundary
/// is guarding against an identifier like `MyRc<T>`, not against `Arc`.
static RC_USE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\bRc<|\bRc::|use std::rc::").unwrap());

fn uses_rc(line: &str) -> bool {
    let t = line.trim_start();
    // A comment names a type; it does not hold one.
    if t.starts_with("//") {
        return false;
    }
    // A Rust string-continuation line is message text, not code (see § Scope).
    if line.trim_end().ends_with("\\n\\") {
        return false;
    }
    RC_USE.is_match(line)
}

#[test]
fn shared_ownership_is_atomic() {
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

    // NON-VACUITY: the guard below is this gate's answer to it. 236 .rs files are found today
    // (driven 2026-09-01); the floor of 50 catches a walk that stopped reaching the tree.
    // A discovering walk must prove it discovered something, or an empty sweep reads as clean.
    assert!(
        files.len() > 50,
        "the no-Rc walk found only {} .rs files — it is not reaching the tree, so its green means \
         nothing (a gate that discovers its inputs must floor-assert the count).",
        files.len()
    );

    let mut violations = Vec::new();
    for f in &files {
        let Ok(src) = std::fs::read_to_string(f) else { continue };
        let rel = f.strip_prefix(manifest).unwrap_or(f).display().to_string();
        for (idx, line) in src.lines().enumerate() {
            if uses_rc(line) && !line.contains("// rune:lint(no-rc)") {
                violations.push(format!("{}:{}  {}", rel, idx + 1, line.trim()));
            }
        }
    }

    assert!(
        violations.is_empty(),
        "\n\n🔥🔥🔥 NON-ATOMIC SHARED OWNERSHIP — {} use(s) of `std::rc::Rc`.\n\
         \n\
         `Rc` is not `Send`. Anything holding one cannot cross a thread boundary, and this\n\
         substrate crosses them everywhere (run-thread loci, defservice actors, the spawn tiers).\n\
         The tree is `Arc`-everywhere by long-standing practice and by ruling.\n\
         \n\
         ⚠ `Rc` COMPILING IS NOT EVIDENCE IT IS SAFE. It compiles right up until the structure\n\
         holding it is shared across a locus — the failure is latent, and the compiler will not\n\
         find it for you today.\n\
         \n\
         THE FIX — `use std::sync::Arc;` and `Rc` → `Arc`. The walk-side cost is zero: a hot path\n\
         that borrows (`&Node`) never touches the refcount; only construction does.\n\
         \n\
         If a structure PROVABLY cannot cross a locus, add a co-located\n\
         `// rune:lint(no-rc) — <reason>` naming why it cannot — not that it currently does not.\n\
         \n\
         Offenders:\n\n{}\n",
        violations.len(),
        violations.join("\n"),
    );
}

#[cfg(test)]
mod detector_tests {
    use super::uses_rc;

    #[test]
    fn matches_real_rc_usage() {
        assert!(uses_rc("use std::rc::Rc;"));
        assert!(uses_rc("    children: HashMap<Value, Rc<Node>>,"));
        assert!(uses_rc("    wildcard: Option<Rc<Node>>,"));
        assert!(uses_rc(") -> Rc<Node> {"));
        assert!(uses_rc("    Rc::new(Node { dim, children })"));
        assert!(uses_rc("use std::rc::Weak;"));
    }

    #[test]
    fn does_not_match_arc() {
        // The whole point: `Arc` must never trip a lint hunting `Rc`.
        assert!(!uses_rc("use std::sync::Arc;"));
        assert!(!uses_rc("    children: HashMap<Value, Arc<Node>>,"));
        assert!(!uses_rc("    Arc::new(Node { dim, children })"));
        assert!(!uses_rc("    bindings: Arc<[(Value, Value)]>,"));
    }

    #[test]
    fn does_not_match_comments_or_message_text() {
        assert!(!uses_rc("// Rc<Node> would not be Send here"));
        assert!(!uses_rc("    /// `Rc::new` is forbidden; see the no-rc lint"));
        // A Rust string-continuation line is a diagnostic's text, not a declaration.
        assert!(!uses_rc("        (single Box<Self> / Arc<Self> / Rc<Self> / Self field)\\n\\"));
    }

    #[test]
    fn does_not_match_identifiers_merely_ending_in_rc() {
        assert!(!uses_rc("    let my_src: MySrc<T> = build();"));
        assert!(!uses_rc("    let s = SomeRc_thing;"));
    }
}
