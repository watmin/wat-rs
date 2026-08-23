//! THE ONE-PARAM-SPEC LINT — `:-` has exactly one recogniser: `peel_param_spec`
//! (beside `is_binder_marker`, `src/types.rs`).
//!
//! STONE-finish-the-param-spec (arc 109): nine hand-rolled recognisers of the
//! `(marker, [types], rest…)` triple grew up with no door between them — four peeled the
//! whole triple by hand, five tested only the marker by re-deriving `k == ":-"` (or, in
//! `types/surface.rs`, a TENTH site the original census missed, found by this rune's own
//! sweep before it was drawn). All ten now call `peel_param_spec` / `is_binder_marker`.
//! This lint is the step that makes it STAY one: it bans the hand-rolled shapes anywhere
//! outside `src/types.rs`, so an eleventh cannot appear un-noticed.
//!
//! ## Why these two, and only these two
//!
//! `k == ":-"` is the literal marker-equality test every one of the ten hand-rolls used
//! (whether alone, testing only "is this the marker," or as the guard clause of a full
//! `[Keyword, Vector, rest @ ..]` triple-peel). `[WatAST::Keyword(k, _),
//! WatAST::Vector(inner, _), rest @ ..]` is the exact slice-pattern shape the four
//! triple-peels used to destructure the marker+bracket+rest in one match arm. Measured,
//! not assumed: a plain `grep -rn` for these two exact literal spellings across `src/` +
//! `crates/` (excluding `types.rs`, the door's own file) finds precisely the sites this
//! stone converted, no more — no allowlist spam was needed to convert them.
//!
//! ⚠ **Not banned, deliberately**: calling `is_binder_marker` itself — that IS the door's
//! own exposed "is this node the marker" accessor, and every legitimate consumer
//! (`function/parse.rs`, `argspec/parse.rs`, `types/surface.rs`, `resolve/walk.rs`,
//! `macros/expand.rs`) is expected to call it freely. The rune bans RE-DERIVING the
//! marker test via a literal string comparison, never USING the door's own function.
//! Also not banned: any OTHER keyword-equality test that happens to share the `==`
//! operator (`k == ":keys"` and similar) — the ban is keyed to the exact `:-` literal,
//! not "any keyword comparison near a Vector."
//!
//! ## The escape hatch
//!
//! `crates/wat-source-derive` is a proc-macro crate that depends on `wat-reader` only —
//! deliberately, to avoid a dependency cycle (`wat-macros` → `wat-doc` → this crate; see
//! its own module doc) — so it structurally CANNOT call `crate::types::peel_param_spec`,
//! a `pub(crate)` item of the separate `wat` crate. Its one genuine hand-roll earns a
//! co-located, same-line `// rune:lint(one-param-spec) — <reason>`, exactly the shape
//! `one_name_grammar.rs` / `retired_name_justified.rs` already use.
//!
//! `rune:lint(<name>)` is the repo's project-custom-lint exemption form (owner `lint` =
//! the project lint suite, NOT a grimoire spell).

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
const BANNED: &[&str] = &[
    r#"k == ":-""#,
    r"WatAST::Keyword(k, _), WatAST::Vector(inner, _), rest @ ..]",
];

/// Does `line` contain a banned shape as CODE — i.e. before any `//` comment opener on the
/// same physical line? A doc/prose mention of the shape (this file's own module doc, or a
/// sibling doc comment naming `k == ":-"` in prose) sits entirely inside or after a `//`,
/// so it is excluded without needing its own exemption marker.
fn code_hit(line: &str, pat: &str) -> bool {
    let comment_at = line.find("//");
    match line.find(pat) {
        Some(hit_at) => comment_at.is_none_or(|c| hit_at < c),
        None => false,
    }
}

#[test]
fn only_types_rs_peels_a_param_spec() {
    let manifest = env!("CARGO_MANIFEST_DIR");
    let mut files = Vec::new();
    for sub in ["src", "crates", "tests"] {
        collect_rs(&Path::new(manifest).join(sub), &mut files);
    }
    files.sort();

    let mut violations = Vec::new();
    for f in &files {
        // This file names the forbidden patterns in its own detector — skip self.
        if f.file_name().and_then(|n| n.to_str()) == Some("one_param_spec.rs") {
            continue;
        }
        let rel = f.strip_prefix(manifest).unwrap_or(f).display().to_string();
        // THE door itself — `types.rs` is where these shapes are the implementation
        // (`is_binder_marker`, `peel_param_spec`), not a violation of it.
        if rel.ends_with("src/types.rs") {
            continue;
        }
        let Ok(src) = std::fs::read_to_string(f) else { continue };

        for (idx, line) in src.lines().enumerate() {
            if line.contains("// rune:lint(one-param-spec)") {
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
        "\n\n🔥🔥🔥 A SECOND `:-` RECOGNISER — {} site(s) hand-roll the param-spec marker\n\
         test (`k == \":-\"`) or the triple-peel slice pattern (`[Keyword, Vector, rest @ ..]`)\n\
         OUTSIDE `src/types.rs`. `:-` is wat's ONE parameterization operator; the\n\
         `(marker, [types], rest…)` triple must be read exactly ONE way, or two readers WILL\n\
         disagree (STONE-finish-the-param-spec, arc 109 — nine hand-rolls already had, plus a\n\
         tenth this rune's own sweep found).\n\
         \n\
         THE FIX — route through the door (`src/types.rs`):\n\
         \n\
         \x20 is_binder_marker(node)         is this node the `:-` marker?  (pub(crate))\n\
         \x20 peel_param_spec(args)          `[:- [T…] rest…]` -> (Some(&[T…]), rest);\n\
         \x20                                no marker -> (None, args). `:- []` peels to\n\
         \x20                                Some(&[]), NEVER None — the empty binder is\n\
         \x20                                EXPRESSED, not absent.\n\
         \n\
         If the hit is genuinely unreachable from the door (a sibling crate that cannot\n\
         depend on `wat` without a dependency cycle — the exact `wat-source-derive` shape) —\n\
         earn a co-located, same-line `// rune:lint(one-param-spec) — <reason>`. A rune\n\
         reason of \"it's just this one site\" does NOT earn its standing — that site is a\n\
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
        assert!(code_hit(r#"        if k == ":-" {"#, r#"k == ":-""#));
        assert!(code_hit(
            r"        [WatAST::Keyword(k, _), WatAST::Vector(inner, _), rest @ ..] => {}",
            r"WatAST::Keyword(k, _), WatAST::Vector(inner, _), rest @ ..]"
        ));
    }

    #[test]
    fn ignores_a_prose_mention_in_a_comment() {
        // Exactly this file's own module doc shape: the pattern named INSIDE a `///`
        // doc comment, never reaching code.
        assert!(!code_hit(
            r#"/// bans a hand-rolled `k == ":-"` test anywhere outside types.rs"#,
            r#"k == ":-""#
        ));
    }

    #[test]
    fn ignores_a_trailing_comment_after_unrelated_code() {
        assert!(!code_hit(
            r#"    do_thing(x); // historically tested k == ":-" here"#,
            r#"k == ":-""#
        ));
    }

    #[test]
    fn a_hit_before_a_trailing_comment_still_flags() {
        assert!(code_hit(
            r#"    if k == ":-" { // the binder marker"#,
            r#"k == ":-""#
        ));
    }
}
