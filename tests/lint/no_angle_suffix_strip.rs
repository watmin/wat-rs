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
//! STONE reap-the-twelve (arc 109, follow-on): the first stone's own module doc predicted a
//! further population — "a STONE reap-the-angle-machinery follow-on... already found a further
//! ~11 sites in `src/runtime.rs` / `src/check.rs` / `src/types.rs` still hand-rolling a bare
//! `.find('<')` strip on a DECLARATION name or aggregate type keyword" — and deliberately did NOT
//! ban it yet, because banning it then would have failed the baseline on those sites. That stone
//! measured all twelve (eleven in `src/`, one in `crates/wat-source-derive`) over a full floor
//! run: **15.7 million calls, zero type-heads found, three never called at all.** Widening the
//! rune to ban a bare `.find('<')` on a name — not just the balanced-suffix `ends_with('>')`
//! fingerprint — was the whole point of measuring them: with them gone, it can finally be
//! enforced. Widening the census also caught two more sites it had not gone looking for, in
//! `tests/function/wat_arc170_closure_extraction.rs` (`extract_define_name`,
//! `collect_type_decl_names`) — the same dead idiom, reading a real captured `WatAST::Keyword`
//! name out of a frozen-world prologue, fixed the same way.
//!
//! ## The two shapes banned, and why THESE two
//!
//! `s.ends_with('>')` is the fingerprint of the BALANCED-SUFFIX rule every one of the first
//! stone's four deleted functions used — *strip only when the name also ends in `>`* — which
//! exists specifically to protect the comparison operators (`:wat::core::<`, `:wat::core::>`,
//! `:wat::core::>=`, `<-`, `->`) from having a real character sliced off them.
//!
//! `s.find('<')` alone is the wider fingerprint the second stone measured and reaped: a bare
//! "does this name have a `<` in it, and if so slice before it" test, with no balanced-suffix
//! guard at all (the twelve deleted sites either never checked for a matching `>`, or checked it
//! via a separate condition rather than this literal call). Both are now zero occurrences in
//! `src/`, `tests/`, and `crates/` after this stone's deletions (`grep -rn "find('<')\|ends_with
//! ('>')" src/ tests/ crates/` — the positive control below reproduces the ban for each).
//! Reintroducing either anywhere in this crate is reintroducing exactly the mechanism whose sole
//! reason to exist — a `<...>` name suffix — no longer exists: no keyword or symbol can carry one
//! (`crates/wat-reader`'s lexer refuses `<` opening a type head UNCONDITIONALLY, in `lex_keyword`
//! AND `lex_symbol`, in source AND in a minted name — `LexErrorKind::AngleTypeHeadInName`; the
//! runtime backstop is `angle_type_head_in_name`, `src/runtime.rs`; the type-checker's own wall is
//! `src/types.rs:4688`, `if stripped.contains('<') { … "angle-bracket type parameters are illegal"
//! … }`).
//!
//! ⚠ **`stripped.contains('<')` — arc 109 ③'s WALL, `src/types.rs:4688` — is a DIFFERENT literal
//! than either banned shape and does not match this lint.** It is the backstop that makes this
//! whole rune true: it fires on a declaration name that never should have reached the type
//! checker with a `<` in it at all, and raises the error rather than stripping anything. Deleting
//! it because it superficially resembles the twelve would remove the guard this rune's own claim
//! depends on — it stays, unconditionally, and is out of this lint's scope by construction (its
//! literal text is `contains`, not `find`, and it does not slice a suffix off anything).
//!
//! ## The escape hatch
//!
//! `crates/` is now IN this lint's scan (STONE reap-the-twelve deleted the one prior occupant,
//! `crates/wat-source-derive/src/lib.rs::declared_name`'s angle half — it kept only the `:-`
//! binder peel, which does not match either banned shape). A future genuine hand-roll anywhere
//! earns a co-located, same-line `// rune:lint(no-angle-suffix-strip) — <reason>`, exactly the
//! shape `one_param_spec.rs` / `one_name_grammar.rs` already use.
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

/// The two banned call-shapes, as their literal source text.
const BANNED: &[&str] = &[r"ends_with('>')", r"find('<')"];

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
fn no_name_is_stripped_of_an_angle_suffix() {
    let manifest = env!("CARGO_MANIFEST_DIR");
    let mut files = Vec::new();
    // `src`, `tests`, AND `crates` — the STONE reap-the-twelve widening. The prior carve-out
    // (crates/wat-source-derive's one occupant) is gone; see the module doc's "escape hatch".
    for sub in ["src", "tests", "crates"] {
        collect_rs(&Path::new(manifest).join(sub), &mut files);
    }
    files.sort();

    let mut violations = Vec::new();
    for f in &files {
        // This file names the forbidden patterns in its own detector — skip self.
        if f.file_name().and_then(|n| n.to_str()) == Some("no_angle_suffix_strip.rs") {
            continue;
        }
        let rel = f.strip_prefix(manifest).unwrap_or(f).display().to_string();
        let Ok(src) = std::fs::read_to_string(f) else { continue };

        for (idx, line) in src.lines().enumerate() {
            if line.contains("// rune:lint(no-angle-suffix-strip)") {
                continue;
            }
            for pat in BANNED {
                if code_hit(line, pat) {
                    violations.push(format!("{}:{}   {}", rel, idx + 1, pat));
                }
            }
        }
    }

    assert!(
        violations.is_empty(),
        "\n\n🔥🔥🔥 AN ANGLE-SUFFIX STRIP RETURNED — {} site(s) test `ends_with('>')` or a bare\n\
         `find('<')` on a name. `<K,V>` is unexpressible — written, minted, or rendered\n\
         (`0811c3009`) — so no keyword or symbol can end in a REAL `<...>` suffix any more; a\n\
         name found this way is used directly, never stripped. Both shapes were measured and\n\
         reaped by STONE reap-the-angle-machinery (4 functions, 16.2M calls, 0 type-heads) and\n\
         its follow-on STONE reap-the-twelve (12 more sites, 15.7M calls, 0 type-heads) — this\n\
         rune is the step that makes BOTH stay reaped.\n\
         \n\
         THE FIX — delete the strip and use the name as-is. If the site looks up a call head or\n\
         a declared name, the two censuses (32M calls combined, 0 type-heads found) already\n\
         proved the strip was a no-op there; if it is a genuinely new question, ask why THIS\n\
         name might carry a suffix when the reader (`crates/wat-reader`) refuses `<` opening a\n\
         type head UNCONDITIONALLY (`LexErrorKind::AngleTypeHeadInName`, both `lex_keyword` and\n\
         `lex_symbol`) and every minting door (`keyword/from-string`, `keyword-node`,\n\
         `symbol-node`) already refuses to produce one — that refusal is what makes the strip\n\
         provably dead, not merely untested.\n\
         \n\
         Genuinely NOT a name (e.g. unrelated text ending in a literal `>`, or a bare `<` test\n\
         unrelated to a name's own suffix) — earn a co-located, same-line\n\
         `// rune:lint(no-angle-suffix-strip) — <reason>`. A reason of \"it's just this one\n\
         site\" does NOT earn its standing on its own; say why this `<`/`>` is not a name's.\n\
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
        assert!(code_hit(
            "    let base = match name.find('<') { Some(i) => &name[..i], None => name };",
            "find('<')"
        ));
    }

    #[test]
    fn ignores_a_prose_mention_in_a_comment() {
        assert!(!code_hit(
            "    // the balanced-suffix rule tests `ends_with('>')` before stripping",
            "ends_with('>')"
        ));
        assert!(!code_hit(
            "    // a hand-rolled strip used to test `find('<')` here",
            "find('<')"
        ));
    }

    #[test]
    fn ignores_a_trailing_comment_after_unrelated_code() {
        assert!(!code_hit(
            "    do_thing(x); // historically used ends_with('>') here",
            "ends_with('>')"
        ));
        assert!(!code_hit(
            "    do_thing(x); // historically used find('<') here",
            "find('<')"
        ));
    }

    #[test]
    fn a_hit_before_a_trailing_comment_still_flags() {
        assert!(code_hit(
            "    let hit = kw.ends_with('>'); // balanced suffix",
            "ends_with('>')"
        ));
        assert!(code_hit(
            "    let hit = name.find('<'); // bare strip",
            "find('<')"
        ));
    }

    #[test]
    fn the_wall_at_types_rs_4688_does_not_match_either_banned_shape() {
        // arc 109 ③'s wall — `if stripped.contains('<') { … "angle-bracket type
        // parameters are illegal" … }` — is a DIFFERENT literal (`contains`, not
        // `find`) and does not slice a suffix off anything. Row 2 of this stone's own
        // acceptance table: a purge that took this wall out along with the twelve
        // would still pass every other row while making the whole campaign undoable.
        assert!(!code_hit(
            "    if stripped.contains('<') {",
            "find('<')"
        ));
        assert!(!code_hit(
            "    if stripped.contains('<') {",
            "ends_with('>')"
        ));
    }
}
