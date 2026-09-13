//! THE ONE-VARIANT-SEPARATOR WALL — the `::` between an enum and its variant is spelled in
//! exactly one file: `crates/wat-reader/src/identifier.rs`'s `compose_variant` /
//! `decompose_variant` pair.
//!
//! ## Why this is a SECOND lint and not a sixth entry in `one_name_grammar.rs`
//!
//! That lint bans five literal call-shapes, and its module doc calls them *"precise — measured,
//! not assumed."* They were — against **arc 109's** 33-site census. `rsplit_once('/')` is among
//! them; **`rsplit_once("::")` appears nowhere in that file** — not in its `BANNED` list, and not
//! in its *"⚠ Not banned, deliberately"* list either. A hole, not a decision. Fourteen were live
//! when this wall was written and **nine were variant decomposers**, hidden for months behind a
//! gate whose own doc asserted it made the grammar stay one.
//!
//! ★ Extending that list would re-ship the same false confidence. A hand-list of **banned**
//! shapes fails SILENTLY on the shape nobody listed; a hand-list of **exempt** shapes fails
//! LOUDLY, on an honest site, in front of a human who then decides. This wall inverts the
//! direction — the same inversion the `:wat::*` blanket needed when it stopped being a prefix
//! list and became a registry question (arc 255, `c3fefc5ab`).
//!
//! ## What counts as spelling the separator
//!
//! ```text
//! DATA      "::" | b"::"                     the separator as a literal argument
//! COMPOSE   }:: | ::{   inside a string      the separator joining a computed part
//! ACCESSOR  identifier::path( | identifier::leaf( | .path() | .leaf()
//! ```
//!
//! ⚠ **ACCESSOR is a hit even though it spells no separator.** `rete/expr_ir.rs:1321` is the
//! whole argument for it: that site composed through the door and decomposed through the GENERAL
//! accessor, and the flip broke it anyway. A site can be half-migrated and look finished.
//!
//! ## The gate, and the control that set it
//!
//! A file is in scope when it talks about variants at all — `TypeDef::Enum`, `EnumVariant`, or
//! the word `variant`. The obvious narrower gate (*the file mentions `TypeDef::Enum`*) was run
//! against the twenty-one hand-read variant sites and **caught nineteen**: it missed
//! `src/match_arm.rs`'s `is_namespaced_variant`, a file that never names the type, and
//! `crates/wat-doc/src/print.rs`'s `axis_edn`. The wide gate caught 21/21. The tighter predicate
//! was 69 rune-lines cheaper and two known sites worse; the tax is the price of the guarantee.
//!
//! ## The rune — a CLOSED category, then a reason
//!
//! ```text
//! // rune:lint(one-variant-separator, <category>) — <why this is not a variant separator>
//! ```
//!
//! on the offending line or the line immediately above it (the window `no_bare_is_err.rs` and
//! `holon_is_vsa_only.rs` both use). The category must be one of:
//!
//! ```text
//! namespace     the "::" separates namespace segments        :wat::core::foldl
//! type-path     it composes/decomposes an ENUM's OWN path    format!("{}::Op", surface.name)
//! display       it renders a name into human-facing prose    "variant {t}::{v} declares …"
//! edn           it translates "::" <-> "." for EDN           ns.replace("::", ".")
//! not-a-name    the string is not a wat name at all          a Rust path, a doc string
//! ```
//!
//! ⛔ **`variant` is NOT a category.** A site that separates an enum from its variant must route
//! through the pair; this wall refuses a rune that claims otherwise. That is the one-way door.
//!
//! ★ The categories are a manifest, not paperwork: `display` names every site that must switch to
//! `.` when the variant separator flips, and they share no call shape — after this wall they
//! share a grep.

use std::path::{Path, PathBuf};

/// The categories a rune may claim. `variant` is deliberately absent — see the module doc.
const CATEGORIES: &[&str] = &["namespace", "type-path", "display", "edn", "not-a-name"];

const ACCESSORS: &[&str] = &[
    "identifier::path(",
    "identifier::leaf(",
    ".path()",
    ".leaf()",
];

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

/// The part of `line` that is CODE — everything before the first `//`. A prose mention of the
/// separator (this file's own module doc, a comment recording a retirement) sits after that
/// opener and is excluded without needing its own rune.
fn code_of(line: &str) -> &str {
    match line.find("//") {
        Some(i) => &line[..i],
        None => line,
    }
}

/// Every double-quoted span in `code`, delimiters included. Naive over escapes only in the sense
/// that it honours `\"`; a raw string's `r#"…"#` still yields its inner span, which is what the
/// COMPOSE test wants.
fn string_spans(code: &str) -> Vec<&str> {
    let b = code.as_bytes();
    let mut out = Vec::new();
    let mut i = 0usize;
    while i < b.len() {
        if b[i] == b'"' {
            let start = i;
            let mut j = i + 1;
            while j < b.len() {
                if b[j] == b'\\' {
                    j += 2;
                    continue;
                }
                if b[j] == b'"' {
                    break;
                }
                j += 1;
            }
            let end = j.min(b.len().saturating_sub(1));
            out.push(&code[start..=end]);
            i = j + 1;
        } else {
            i += 1;
        }
    }
    out
}

/// Which way this line spells the variant separator, if it does.
fn classify(code: &str) -> Option<&'static str> {
    if code.contains(r#""::""#) || code.contains(r#"b"::""#) {
        return Some("DATA");
    }
    if string_spans(code).iter().any(|s| s.contains("}::") || s.contains("::{")) {
        return Some("COMPOSE");
    }
    if ACCESSORS.iter().any(|a| code.contains(a)) {
        return Some("ACCESSOR");
    }
    None
}

/// Does this file talk about variants at all? See the module doc on why the narrow
/// `TypeDef::Enum`-only gate was disqualified by a known-answer control.
fn in_scope(src: &str) -> bool {
    src.contains("TypeDef::Enum") || src.contains("EnumVariant") || {
        // `\bvariant` case-insensitively, without a regex dependency.
        let lower = src.to_ascii_lowercase();
        // The boundary is ALPHANUMERIC-only, deliberately: `_` must NOT block it, or
        // `is_namespaced_variant` — the exact site the narrow gate missed and this wide gate
        // exists to admit — falls outside its own gate. `invariant` is still excluded, because
        // `n` is alphanumeric. This lint's own self-test caught the `_` version.
        lower
            .match_indices("variant")
            .any(|(i, _)| i == 0 || !lower.as_bytes()[i - 1].is_ascii_alphanumeric())
    }
}

/// The category a rune on `line` claims, or `None` if the line carries no rune of ours.
/// `Some(Err(text))` when the claimed category is off the closed list.
#[allow(clippy::type_complexity)]
fn rune_category(line: &str) -> Option<Result<&'static str, String>> {
    let at = line.find("rune:lint(one-variant-separator")?;
    let rest = &line[at + "rune:lint(one-variant-separator".len()..];
    let inner = match (rest.find(','), rest.find(')')) {
        (Some(c), Some(p)) if c < p => rest[c + 1..p].trim(),
        _ => return Some(Err("<no category>".into())),
    };
    match CATEGORIES.iter().find(|c| **c == inner) {
        Some(c) => Some(Ok(c)),
        None => Some(Err(inner.to_string())),
    }
}

/// Indices of the contiguous `//` comment block immediately above `idx`, nearest first.
/// Stops at the first line that is not a comment — so a rune separated from its line by code
/// does not reach it.
fn comment_block_above(lines: &[&str], idx: usize) -> impl Iterator<Item = usize> {
    let mut out = Vec::new();
    let mut i = idx;
    while i > 0 {
        i -= 1;
        if lines[i].trim_start().starts_with("//") {
            out.push(i);
        } else {
            break;
        }
    }
    out.into_iter()
}

#[test]
fn only_identifier_rs_spells_the_variant_separator() {
    let manifest = env!("CARGO_MANIFEST_DIR");
    let mut files = Vec::new();
    for sub in ["src", "crates", "tests"] {
        collect_rs(&Path::new(manifest).join(sub), &mut files);
    }
    files.sort();

    let mut violations = Vec::new();
    for f in &files {
        // This file names every forbidden shape in its own detector — skip self.
        if f.file_name().and_then(|n| n.to_str()) == Some("one_variant_separator.rs") {
            continue;
        }
        let rel = f.strip_prefix(manifest).unwrap_or(f).display().to_string();
        // THE door — where these shapes are the implementation, not a violation of it.
        if rel.ends_with("crates/wat-reader/src/identifier.rs") {
            continue;
        }
        let Ok(src) = std::fs::read_to_string(f) else { continue };
        if !in_scope(&src) {
            continue;
        }
        let lines: Vec<&str> = src.lines().collect();
        for (idx, raw) in lines.iter().enumerate() {
            let Some(kind) = classify(code_of(raw)) else { continue };
            // Co-located rune: this line, or anywhere in the contiguous comment block
            // immediately above it. The one-line window this started with forced every
            // reason onto one line — and the REASON is the point of the rune, so the
            // window bent, not the prose. A comment block directly above a line belongs
            // to that line by Rust convention; a rune further up, separated by code,
            // does not reach.
            let runed = comment_block_above(&lines, idx)
                .chain(std::iter::once(idx))
                .filter_map(|i| lines.get(i).and_then(|l| rune_category(l)))
                .next();
            match runed {
                Some(Ok(_)) => continue,
                Some(Err(bad)) => violations.push(format!(
                    "{}:{}  [{}]  ⛔ RUNE CLAIMS `{}` — not a category  {}",
                    rel,
                    idx + 1,
                    kind,
                    bad,
                    raw.trim()
                )),
                None => violations.push(format!("{}:{}  [{}]  {}", rel, idx + 1, kind, raw.trim())),
            }
        }
    }

    assert!(
        violations.is_empty(),
        "\n\n🔥🔥🔥 A SECOND VARIANT SEPARATOR — {} site(s) spell the `::` between an enum and \n\
         its variant OUTSIDE `crates/wat-reader/src/identifier.rs`.\n\
         \n\
         A variant's fully-qualified name is composed and decomposed in exactly ONE place, or two \n\
         spellings WILL disagree — nine of these hid for months inside `rsplit_once(\"::\")`, a \n\
         shape `one_name_grammar.rs` bans for `'/'` and does not name for `\"::\"`.\n\
         \n\
         THE FIX — route through the pair:\n\
         \n\
         \x20 compose_variant(enum_path, variant)   -> `{{enum}}.{{variant}}`\n\
         \x20 decompose_variant(name) -> Option<(&str, &str)>   the exact inverse\n\
         \n\
         If this site does NOT separate an enum from its variant, add a co-located\n\
         `// rune:lint(one-variant-separator, <category>) — <reason>` on the line or the one\n\
         above, with <category> one of: {}.\n\
         ⛔ `variant` is NOT a category — a variant site routes through the door.\n\
         \n\
         Offenders:\n\n{}\n",
        violations.len(),
        CATEGORIES.join(" | "),
        violations.join("\n"),
    );
}

#[cfg(test)]
mod detector_selftests {
    use super::*;

    #[test]
    fn the_three_hit_shapes_are_each_caught() {
        // The fixture is SOURCE TEXT under classification, not an assertion idiom — bound to a
        // local so `no_loose_string_assert` reads the assert line, not the sample.
        let guard = r#"    if !head.contains("::") {"#;
        assert_eq!(classify(guard), Some("DATA"));
        assert_eq!(classify(r#"    k.as_bytes()[n..].starts_with(b"::")"#), Some("DATA"));
        assert_eq!(classify(r#"    format!("{}::Op::{}", p, v)"#), Some("COMPOSE"));
        assert_eq!(classify(r#"    let l = identifier::leaf(name);"#), Some("ACCESSOR"));
        assert_eq!(classify(r#"    decompose_variant(head)?"#), None);
        assert_eq!(classify(r#"    compose_variant(&e.type_path, &e.variant_name)"#), None);
    }

    #[test]
    fn prose_after_a_comment_opener_is_not_a_hit() {
        assert_eq!(classify(code_of(r#"    do_thing(x); // historically split on "::" here"#)), None);
        // ...but the same text as CODE still is.
        let same_text_as_code = r#"    let p = x.contains("::"); // guard"#;
        assert_eq!(classify(code_of(same_text_as_code)), Some("DATA"));
    }

    #[test]
    fn a_rune_must_name_a_category_from_the_closed_list() {
        assert_eq!(
            rune_category("// rune:lint(one-variant-separator, namespace) — ns split"),
            Some(Ok("namespace"))
        );
        assert_eq!(
            rune_category("// rune:lint(one-variant-separator, display) — human-facing"),
            Some(Ok("display"))
        );
        assert!(matches!(rune_category("// rune:lint(one-variant-separator) — no category"), Some(Err(_))));
        assert!(rune_category("// rune:lint(loose-assert) — a different lint").is_none());
    }

    #[test]
    fn a_rune_claiming_variant_is_refused() {
        // THE one-way door: the only exit for a real variant site is `identifier.rs`.
        assert_eq!(
            rune_category("// rune:lint(one-variant-separator, variant) — it's fine honestly"),
            Some(Err("variant".to_string()))
        );
    }

    #[test]
    fn a_rune_reaches_from_anywhere_in_the_comment_block_above() {
        let lines = vec![
            "    // rune:lint(one-variant-separator, edn) — the dot/colon translation",
            "    // spelled over two lines, because the reason needs two lines",
            r#"    let ns = p.replace("::", ".");"#,
            r#"    let q = other.contains("::");"#,
        ];
        // The block reaches line 2 (index 2)...
        assert!(comment_block_above(&lines, 2).any(|i| rune_category(lines[i]).is_some()));
        // ...and does NOT reach line 3, whose block-above is code.
        assert!(!comment_block_above(&lines, 3).any(|i| rune_category(lines[i]).is_some()));
    }

    #[test]
    fn the_gate_admits_a_file_that_only_says_the_word() {
        // `match_arm.rs` never names `TypeDef::Enum`; the narrow gate missed it, and it held
        // `is_namespaced_variant`. This is that control, kept as a row.
        assert!(in_scope("pub fn is_namespaced_variant(path: &str) -> bool {"));
        assert!(in_scope("match d { TypeDef::Enum(e) => e, _ => return }"));
        assert!(!in_scope("pub fn unrelated(x: &str) -> usize { x.len() }"));
        // `invariant` must not admit a file — the word boundary is load-bearing.
        assert!(!in_scope("// this invariant holds for all callers"));
    }
}
