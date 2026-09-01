//! THE ONE-NAME-GRAMMAR LINT — a wat name has exactly one parser: `Identifier`
//! (`crates/wat-reader/src/identifier.rs`).
//!
//! STONE-one-name-grammar (arc 109): a name is an atom, and structure encoded inside an atom
//! must be re-parsed by every consumer of that atom. Before this stone, 33 hand-rolled parsers
//! grew up beside `Identifier` — each its own `rfind("::")` / `rsplit("::")` / `rfind('/')` /
//! `rsplit_once('/')` / `strip_suffix('\'')`, re-deriving `leaf`/`path`/`receiver`/`method`/
//! `prime`/`deprimed` by hand, with no guarantee any two of them agreed. This lint is the step
//! that makes it STAY one: it bans those five literal call-shapes anywhere outside
//! `identifier.rs` itself.
//!
//! ## Why these five, and only these five
//!
//! They are the exact patterns the 33-site census (`DESIGN-STONE-one-name-grammar.md`) found in
//! use, and — measured, not assumed — they are precise: a plain `grep -rn` for these five exact
//! literal spellings across `src/` + `crates/` (excluding `identifier.rs`) returns almost exactly
//! the 33 converted sites, no more. Generic path/URL/EDN-tag splitting in this codebase uses
//! different idioms (`.find(...)`, `.split(...)` collecting ALL segments, `.contains(...)`), so
//! this narrow literal-pattern scan does not reach for those honest non-name uses — no allowlist
//! spam was needed to ship this. See the module-doc warning below for what to do if that ever
//! stops being true.
//!
//! ⚠ **Not banned, deliberately**: `.split("::")` (full segmentation — legitimate for a caller
//! that needs every segment, not just the last), `.find('/')` (first-occurrence — a different
//! question from "the last `/`"), and any `.rfind`/`.split`/`.strip_suffix` call whose argument is
//! not one of these five exact literals. The rune is drawn at the literal call-shape, not at
//! "any string surgery near a `/` or `::`" — the DESIGN doc's warning against a rune tight enough
//! to make an honest filesystem-path/URL/EDN-tag split non-compliant.
//!
//! ## The escape hatch
//!
//! A future site that legitimately needs one of these five exact shapes for something that is
//! NOT a wat name (a filesystem path, a URL, a doc string bearing the literal text) earns an
//! allowlist with a co-located, same-line `// rune:lint(one-name-grammar) — <reason>`, exactly
//! the shape `no_loose_string_assert.rs` and `retired_name_justified.rs` already use.
//!
//! `rune:lint(<name>)` is the repo's project-custom-lint exemption form (owner `lint` = the
//! project lint suite, NOT a grimoire spell).

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

/// The five banned call-shapes, as their literal source text.
const BANNED: &[&str] = &[
    r#"rfind("::")"#,
    r#"rsplit("::")"#,
    r"rfind('/')",
    r"rsplit_once('/')",
    r"strip_suffix('\'')",
];

/// Does `line` contain a banned shape as CODE — i.e. before any `//` comment opener on the
/// same physical line? A doc/prose mention of the shape (this file's own module doc, or
/// `registration.rs`'s comment naming `rsplit("::")` in prose) sits entirely inside or after a
/// `//`, so it is excluded without needing its own exemption marker.
fn code_hit(line: &str, pat: &str) -> bool {
    let comment_at = line.find("//");
    match line.find(pat) {
        Some(hit_at) => comment_at.is_none_or(|c| hit_at < c),
        None => false,
    }
}

#[test]
fn only_identifier_rs_parses_a_name() {
    let manifest = env!("CARGO_MANIFEST_DIR");
    let mut files = Vec::new();
    for sub in ["src", "crates", "tests"] {
        collect_rs(&Path::new(manifest).join(sub), &mut files);
    }
    files.sort();

    // NON-VACUITY: a walk that comes back empty asserts nothing over nothing and reports PASS, and
    // every verdict downstream inherits that silence. The floor sits well under the
    // 998 .rs file(s) this walk finds today — driven 2026-09-01, and the count comes
    // from `tests/lint/every_walking_gate_declares_non_vacuity.rs`, never from prose — so it
    // catches a walk gone blind — a moved root, a renamed directory — without rotting as the
    // tree grows.
    assert!(
        files.len() > 500,
        "the one-name-grammar walk found only {} .rs file(s) — it is not \
         reaching the tree it claims to guard, so its green means nothing",
        files.len()
    );

    let mut violations = Vec::new();
    for f in &files {
        // This file names the forbidden patterns in its own detector — skip self.
        if f.file_name().and_then(|n| n.to_str()) == Some("one_name_grammar.rs") {
            continue;
        }
        let rel = f.strip_prefix(manifest).unwrap_or(f).display().to_string();
        // THE door itself — `identifier.rs` is where these five patterns are the
        // implementation, not a violation of it.
        if rel.ends_with("crates/wat-reader/src/identifier.rs") {
            continue;
        }
        let Ok(src) = std::fs::read_to_string(f) else { continue };

        for (idx, line) in src.lines().enumerate() {
            if line.contains("// rune:lint(one-name-grammar)") {
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
        "\n\n🔥🔥🔥 A SECOND NAME PARSER — {} site(s) hand-roll one of the five name-grammar\n\
         call-shapes (`rfind(\"::\")`, `rsplit(\"::\")`, `rfind('/')`, `rsplit_once('/')`,\n\
         `strip_suffix('\\'')`) OUTSIDE `crates/wat-reader/src/identifier.rs`. A name is an atom;\n\
         structure encoded inside it must be parsed exactly ONE way, or two parsers WILL disagree\n\
         (STONE-one-name-grammar, arc 109 — the census found 33 that already had).\n\
         \n\
         THE FIX — route through the door's accessors (methods on `Identifier`, free functions on\n\
         `&str` for sites holding a raw keyword/symbol string):\n\
         \n\
         \x20 leaf(name)        the last `::` segment        :wat::cache::Lru  -> Lru\n\
         \x20 path(name)        everything before the leaf   :wat::cache::Lru  -> :wat::cache\n\
         \x20 receiver(name)    everything before the `/`    :S/mk             -> :S\n\
         \x20 method(name)      everything after the `/`     :S/mk             -> mk\n\
         \x20 prime(name)       is the name primed?          :sort'            -> true\n\
         \x20 deprimed(name)    the name without its `'`     :sort'            -> :sort\n\
         \n\
         If the hit is genuinely NOT a wat name (a filesystem path, a URL, an EDN tag, a doc\n\
         string) — earn a co-located, same-line `// rune:lint(one-name-grammar) — <reason>`.\n\
         A rune reason of \"it's just this one site\" does NOT earn its standing — that site is a\n\
         FIX (route through the door), not a rune (excusare — the reason must earn it).\n\
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
        assert!(code_hit(r#"    let pos = k.rfind("::")?;"#, r#"rfind("::")"#));
        assert!(code_hit(r"    let slash_pos = s.rfind('/').unwrap();", r"rfind('/')"));
    }

    #[test]
    fn ignores_a_prose_mention_in_a_comment() {
        // Exactly the registration.rs shape: the pattern is named INSIDE a `///` doc comment,
        // never reaching code.
        assert!(!code_hit(
            r#"/// leading `:` (parametric heads drop it), because `rsplit("::")`"#,
            r#"rsplit("::")"#
        ));
    }

    #[test]
    fn ignores_a_trailing_comment_after_unrelated_code() {
        assert!(!code_hit(
            r#"    do_thing(x); // historically used rfind("::") here"#,
            r#"rfind("::")"#
        ));
    }

    #[test]
    fn a_hit_before_a_trailing_comment_still_flags() {
        assert!(code_hit(
            r#"    let pos = k.rfind("::")?; // find the leaf boundary"#,
            r#"rfind("::")"#
        ));
    }
}
