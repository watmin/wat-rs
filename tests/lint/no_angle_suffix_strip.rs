//! THE NO-ANGLE-SUFFIX-STRIP LINT — a name has no `<...>` suffix to strip, ever.
//!
//! STONE reap-the-angle-machinery (arc 109): `<K,V>` became unexpressible in `0811c3009` —
//! written, minted, or rendered. Four functions existed solely to strip a `<...>` suffix off a
//! NAME before using it (`canonical_callable_name`, `split_name_and_type_params`,
//! `split_type_params`/`split_type_params_pub`, `split_method_name_type_params`) — instrumented
//! at 16.2 million calls in one floor run, finding a type-head **zero** times. This stone deleted
//! all four and, at every call site, used the name directly instead. This lint is the step that
//! makes the deletion STAY deleted: a hand-rolled REINCARNATION of the same idiom — usually
//! written by someone who hits a `NoMatchingClause`/`UnknownFunction` on a generic call, half-
//! remembers "there used to be a strip for this," and re-derives it — cannot return un-noticed.
//!
//! ## The one shape banned, and why THIS one
//!
//! `s.ends_with('>')` is the fingerprint of the BALANCED-SUFFIX rule every one of the four
//! deleted functions used — *strip only when the name also ends in `>`* — which exists
//! specifically to protect the comparison operators (`:wat::core::<`, `:wat::core::>`,
//! `:wat::core::>=`, `<-`, `->`) from having a real character sliced off them. Measured: this
//! exact literal has **zero** occurrences left in `src/` or `tests/` after this stone's
//! deletions (a plain `grep -rn "ends_with('>')" src/ tests/` — the positive control below
//! reproduces it). Reintroducing it anywhere in this crate is reintroducing exactly the
//! mechanism whose sole reason to exist — a `<...>` name suffix — no longer exists: no keyword
//! or symbol can carry one (`crates/wat-reader`'s lexer refuses `<` opening a type head, in
//! source AND in a minted name — `angle_type_head_in_name`, `runtime.rs`).
//!
//! A bare `.find('<')` alone is NOT banned here — deliberately. It is far more common (plain
//! "does this string contain a `<`" tests exist for reasons unrelated to a name's own suffix),
//! and a STONE reap-the-angle-machinery follow-on (see below) already found a further ~11 sites
//! in `src/runtime.rs` / `src/check.rs` / `src/types.rs` still hand-rolling a bare `.find('<')`
//! strip on a DECLARATION name or aggregate type keyword — the same now-dead pattern, but never
//! instrumented by this stone's census and out of its boundary to fix inline. Banning bare
//! `.find('<')` today would indict that whole population before it has its own dedicated stone;
//! `.ends_with('>')` is the narrow, precisely-measured signature that is unique to the CENSUSED
//! machinery this stone actually removed, so the rune is honest about what it enforces.
//!
//! ## The escape hatch
//!
//! `crates/` is out of this lint's scan entirely (not merely allowlisted) — the one known
//! occurrence, `crates/wat-source-derive/src/lib.rs::declared_name`, is a proc-macro crate that
//! depends on `wat-reader` only (a deliberate cycle-avoidance: `wat-macros` → `wat-doc` → this
//! crate; see that crate's own module doc), so it structurally cannot share code with the `wat`
//! crate's `src/`. It is itself now a STALE hand-roll (same class, unmeasured) — flagged as a
//! finding for a follow-on stone, not swept here.
//!
//! `rune:lint(<name>)` is the repo's project-custom-lint exemption form (owner `lint` = the
//! project lint suite, NOT a grimoire spell); see `one_param_spec.rs` / `one_name_grammar.rs` for
//! the established shape.

use std::path::{Path, PathBuf};

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

/// The one banned call-shape, as its literal source text.
const BANNED: &str = r"ends_with('>')";

/// Does `line` contain the banned shape as CODE — i.e. before any `//` comment opener on the
/// same physical line? A doc/prose mention (this file's own module doc) sits entirely inside or
/// after a `//`, so it is excluded without needing its own exemption marker.
fn code_hit(line: &str, pat: &str) -> bool {
    let comment_at = line.find("//");
    match line.find(pat) {
        Some(hit_at) => comment_at.is_none_or(|c| hit_at < c),
        None => false,
    }
}

#[test]
fn no_name_is_stripped_of_a_balanced_angle_suffix() {
    let manifest = env!("CARGO_MANIFEST_DIR");
    let mut files = Vec::new();
    // Deliberately `src` + `tests` only — NOT `crates`. See the module doc's "escape hatch".
    for sub in ["src", "tests"] {
        collect_rs(&Path::new(manifest).join(sub), &mut files);
    }
    files.sort();

    let mut violations = Vec::new();
    for f in &files {
        // This file names the forbidden pattern in its own detector — skip self.
        if f.file_name().and_then(|n| n.to_str()) == Some("no_angle_suffix_strip.rs") {
            continue;
        }
        let rel = f.strip_prefix(manifest).unwrap_or(f).display().to_string();
        let Ok(src) = std::fs::read_to_string(f) else { continue };

        for (idx, line) in src.lines().enumerate() {
            if line.contains("// rune:lint(no-angle-suffix-strip)") {
                continue;
            }
            if code_hit(line, BANNED) {
                violations.push(format!("{}:{}   {}", rel, idx + 1, BANNED));
            }
        }
    }

    assert!(
        violations.is_empty(),
        "\n\n🔥🔥🔥 A BALANCED-ANGLE-SUFFIX STRIP RETURNED — {} site(s) test `ends_with('>')`,\n\
         the exact fingerprint of the four functions STONE reap-the-angle-machinery (arc 109)\n\
         deleted (`canonical_callable_name`, `split_name_and_type_params`, `split_type_params`,\n\
         `split_method_name_type_params`). `<K,V>` is unexpressible — written, minted, or\n\
         rendered (`0811c3009`) — so no keyword or symbol can end in a REAL `<...>` suffix any\n\
         more; a name found this way is used directly, never stripped.\n\
         \n\
         THE FIX — delete the strip and use the name as-is. If the site looks up a call head or\n\
         a declared name, the census (16.2M calls, 0 type-heads found) already proved the strip\n\
         was a no-op there; if it is a genuinely new question, ask why THIS name might carry a\n\
         suffix when the reader (`crates/wat-reader`) and every minting door\n\
         (`keyword/from-string`, `keyword-node`, `symbol-node`) already refuse to produce one —\n\
         that refusal is what makes the strip provably dead, not merely untested.\n\
         \n\
         Genuinely NOT a name (e.g. unrelated text ending in a literal `>`) — earn a co-located,\n\
         same-line `// rune:lint(no-angle-suffix-strip) — <reason>`. A reason of \"it's just this\n\
         one site\" does NOT earn its standing on its own; say why this `>` is not a name's.\n\
         \n\
         Offenders:\n\n{}\n",
        violations.len(),
        violations.join("\n"),
    );
}

#[cfg(test)]
mod tests {
    use super::code_hit;

    #[test]
    fn flags_code_before_any_comment() {
        assert!(code_hit(
            "    if !kw.ends_with('>') { return kw; }",
            "ends_with('>')"
        ));
    }

    #[test]
    fn ignores_a_prose_mention_in_a_comment() {
        assert!(!code_hit(
            "    // the balanced-suffix rule tests `ends_with('>')` before stripping",
            "ends_with('>')"
        ));
    }

    #[test]
    fn ignores_a_trailing_comment_after_unrelated_code() {
        assert!(!code_hit(
            "    do_thing(x); // historically used ends_with('>') here",
            "ends_with('>')"
        ));
    }

    #[test]
    fn a_hit_before_a_trailing_comment_still_flags() {
        assert!(code_hit(
            "    let hit = kw.ends_with('>'); // balanced suffix",
            "ends_with('>')"
        ));
    }
}
