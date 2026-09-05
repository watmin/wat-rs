//! THE HOLON-IS-VSA-ONLY LINT — a structural wall keeping `HolonAST` inside its algebra.
//!
//! `holon::HolonAST` is a hypervector composition — bundling, binding, cosine similarity, the
//! whole VSA/HDC algebra. `wat::WatAST` is the wat SYNTAX TREE. The substrate has printed this
//! distinction to users for a long time (`src/types/error.rs:332`, `src/function/parse.rs:1452`):
//! *"use `:wat::WatAST` for any wat form, `:wat::holon::HolonAST` ONLY for a VSA/HDC algebra
//! value…"* — and `src/special_forms.rs` violated it anyway, for months, storing a special
//! form's `(:head <slot> <slot>)` SIGNATURE (syntax, not a hypervector) as a `HolonAST::Bundle`.
//! `src/reflect/verbs.rs` then copied the shape deliberately, citing consistency with the first
//! misuse as its own justification. A convention stated in prose did not merely fail to stop
//! that: it got cited as the REASON to propagate it
//! (`docs/arc/2026/06/294-holon-returns-to-vsa/RULING-holon-is-for-vsa-only-and-a-wall-will-say-so.md`).
//!
//! This is rung 2 of that RULING's ladder — "a check at construction" — arming at the exact
//! residue the RULING measured and named (the special-form sketch, the stepper's `:wat::core::fn`
//! arm, `require_bundle`'s misfiled home, and one runed test fixture) so that going forward this
//! is a WALL that can go red, not a sentence nobody can check.
//! (`docs/arc/2026/06/255-builtin-registry/DESIGN-STONE-the-special-form-sketch-is-syntax-not-a-hypervector.md`,
//! `docs/arc/2026/06/255-builtin-registry/BRIEF-STONE-holon-is-vsa-only-the-sketch-and-the-wall.md`.)
//!
//! ## The rule — aimed at the ACT, never the WORD
//!
//! Outside the VSA homes and the one carrier, no module may **CONSTRUCT** a `HolonAST`
//! (`HolonAST::<ctor>(…)`, in either its lowercase free-fn spelling — `HolonAST::bundle`,
//! `::keyword`, `::symbol`, `::i64`, `::string`, `::nil`, `::bool_`, `::char_`, `::f64` — or its
//! CamelCase variant-constructor spelling, `HolonAST::Bundle(…)` used as an expression) or
//! **DECLARE** one in a field or return type (`x: HolonAST`, `-> HolonAST`, `Arc<HolonAST>`,
//! `Vec<HolonAST>`, `&HolonAST`, …).
//!
//! **Pattern-matching an existing holon is ALLOWED** — a match arm is downstream of a
//! construction the wall already governs (`HolonAST::Bundle(children) => …`,
//! `matches!(&*h, HolonAST::Bind(_, _))`). **Naming it in prose is ALLOWED** — a `HolonAST`
//! inside a comment or a Rust string literal (an error message, a `TypeExpr::Path` string,
//! a wat-source test fixture) is documentation or data, never a live Rust type.
//!
//! ## VSA homes and the one carrier
//!
//! ```text
//! VSA HOMES        src/holon/**  ·  src/intrinsic/holon/**  ·  src/lower.rs
//!                  src/record/update.rs  ·  src/edn/render.rs
//! THE ONE CARRIER  src/value/value.rs
//! SCOPE            src/  and  crates/*/src/
//! ```
//!
//! `crates/wat-reader` cannot name `HolonAST` at all — it has no dependency on the `holon`
//! crate — so every `HolonAST` string found there is a keyword literal inside a test fixture or a
//! doc comment, never a live type. That this lint scores it clean (via the same prose/string
//! exclusion used everywhere else, not a carve-out for the crate) is a sanity check on the
//! discrimination below, not a special case in it.
//!
//! ## The exemption
//!
//! A site outside the homes may carry a co-located `// rune:lint(holon-not-vsa, <category>) —
//! <reason>`. The reason must name **why the holon IS the subject**, never that it happens to be
//! convenient — "it round-trips losslessly" is not a reason (the RULING's own example: the
//! `:wat::core::fn` arm round-tripped losslessly and was still wrong, because losslessness is a
//! property of the CONVERSION, not a licence for the detour).
//!
//! ## Distinguishing a construction from a match arm
//!
//! The lowercase free-fn constructors are unambiguous — `HolonAST::bundle(…)` is never a pattern
//! (Rust patterns cannot call functions), so any occurrence is a CONSTRUCTION.
//!
//! The CamelCase variant spelling is genuinely ambiguous: `HolonAST::Bundle(x)` is a pattern in
//! match-arm position and a construction everywhere else. **The discrimination this lint uses,
//! stated in one sentence: a `HolonAST::Variant(…)` occurrence is treated as a PATTERN (allowed)
//! when it is the first non-whitespace token on its line — this codebase's uniform match-arm
//! style, `HolonAST::Bundle(children) => …` — or when it appears after a `matches!(` earlier on
//! the same line (`matches!(&*h, HolonAST::Bind(_, _))`); every other occurrence is a
//! CONSTRUCTION.**
//!
//! **What this cannot see** (stated honestly, not discovered by a red and patched around):
//! - A match arm whose pattern is NOT the first token on its line and is not inside a `matches!`
//!   call (e.g. `Ok(HolonAST::Bundle(x)) => …`, `if let HolonAST::Bind(a, b) = h`) reads as a
//!   CONSTRUCTION — a false positive this lint cannot rule out short of a real parse. None exist
//!   in the tree today (verified against every `HolonAST::[A-Z]…(` occurrence outside the homes
//!   and the carrier at authoring time); a genuine future one needs a rune, same as any other
//!   exemption, or this lint's discrimination needs to grow a case.
//! - A declaration split across lines (`signature:\n    HolonAST,`) is invisible — the
//!   CONSTRUCT/DECLARE classification below is line-based, like its sibling `no_rc_use.rs`.
//!   (Comments and strings, regular AND raw, ARE tracked across lines — see `mask_non_code` —
//!   this blind spot is narrower than it first looks: only a genuine multi-line CODE
//!   declaration is invisible, not a multi-line string.)
//! - `mask_non_code` does not special-case char literals or lifetimes (`'a`); harmless here
//!   because neither can spell `HolonAST`.

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

/// VSA homes — directories where `HolonAST` construction/declaration is the algebra doing its
/// job. Trailing `/` marks a directory prefix; an exact entry marks one file.
const VSA_HOME_DIRS: &[&str] = &["src/holon/", "src/intrinsic/holon/"];
const VSA_HOME_FILES: &[&str] = &["src/lower.rs", "src/record/update.rs", "src/edn/render.rs"];

/// THE ONE CARRIER — `Value::holon__HolonAST` and the two Hologram carriers. Named explicitly,
/// not lumped into the homes list, because it is rung 3's target (see the RULING).
const THE_ONE_CARRIER: &str = "src/value/value.rs";

fn is_home_or_carrier(rel: &str) -> bool {
    if rel == THE_ONE_CARRIER {
        return true;
    }
    if VSA_HOME_FILES.contains(&rel) {
        return true;
    }
    VSA_HOME_DIRS.iter().any(|d| rel.starts_with(d))
}

/// Blanks every non-code span in `src` — line comments, block comments (nesting-aware), regular
/// `"…"` strings (with `\"` escapes), and raw strings (`r"…"`, `r#"…"#`, `r##"…"##`, …) — while
/// preserving line breaks and column positions, so a `HolonAST` spelled in a comment, an error
/// message, or an embedded multi-line wat-source test fixture can never be mistaken for a live
/// Rust type, and the per-line position heuristics below (first-token-of-line, `matches!(`) still
/// see real code exactly where it was. A raw string is the load-bearing case: this codebase
/// embeds wat-source test fixtures naming `:wat::holon::HolonAST` inside `r#"…"#` blocks that
/// span many lines (`src/runtime.rs`, `src/macros/tests.rs`), and those lines carry no visible
/// quote character of their own — a single-line-only stripper (this function's first draft)
/// reads them as bare code and misfires. Returns one masked line per input line.
fn mask_non_code(src: &str) -> Vec<String> {
    #[derive(Clone, Copy, PartialEq)]
    enum St {
        Code,
        Line,
        Block(u32),
        Str,
        StrEsc,
        Raw(usize),
    }
    let mut out = Vec::new();
    let mut cur = String::new();
    let mut state = St::Code;
    let mut chars = src.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '\n' {
            out.push(std::mem::take(&mut cur));
            if state == St::Line {
                state = St::Code;
            }
            continue;
        }
        match state {
            St::Code => {
                if c == '/' && chars.peek() == Some(&'/') {
                    chars.next();
                    cur.push(' ');
                    cur.push(' ');
                    state = St::Line;
                } else if c == '/' && chars.peek() == Some(&'*') {
                    chars.next();
                    cur.push(' ');
                    cur.push(' ');
                    state = St::Block(1);
                } else if c == '"' {
                    cur.push(' ');
                    state = St::Str;
                } else if c == 'r' && {
                    let mut la = chars.clone();
                    while la.peek() == Some(&'#') {
                        la.next();
                    }
                    la.peek() == Some(&'"')
                } {
                    cur.push(' '); // the `r`
                    let mut n = 0usize;
                    while chars.peek() == Some(&'#') {
                        chars.next();
                        cur.push(' ');
                        n += 1;
                    }
                    chars.next(); // the opening quote
                    cur.push(' ');
                    state = St::Raw(n);
                } else {
                    cur.push(c);
                }
            }
            St::Line => cur.push(' '),
            St::Block(depth) => {
                if c == '/' && chars.peek() == Some(&'*') {
                    chars.next();
                    cur.push(' ');
                    cur.push(' ');
                    state = St::Block(depth + 1);
                } else if c == '*' && chars.peek() == Some(&'/') {
                    chars.next();
                    cur.push(' ');
                    cur.push(' ');
                    state = if depth <= 1 { St::Code } else { St::Block(depth - 1) };
                } else {
                    cur.push(' ');
                }
            }
            St::Str => {
                if c == '\\' {
                    cur.push(' ');
                    state = St::StrEsc;
                } else if c == '"' {
                    cur.push(' ');
                    state = St::Code;
                } else {
                    cur.push(' ');
                }
            }
            St::StrEsc => {
                cur.push(' ');
                state = St::Str;
            }
            St::Raw(hashes) => {
                if c == '"' {
                    let mut la = chars.clone();
                    let mut n = 0usize;
                    while n < hashes && la.peek() == Some(&'#') {
                        n += 1;
                        la.next();
                    }
                    if n == hashes {
                        cur.push(' ');
                        for _ in 0..hashes {
                            chars.next();
                            cur.push(' ');
                        }
                        state = St::Code;
                    } else {
                        cur.push(' ');
                    }
                } else {
                    cur.push(' ');
                }
            }
        }
    }
    out.push(cur);
    out
}

/// Single-line convenience wrapper over [`mask_non_code`] for the unit tests below — masking
/// always starts in `Code` state, so it cannot see a raw string opened on an earlier line (by
/// construction: there is no earlier line). The real scan below always runs the whole-file form.
fn mask_line(line: &str) -> String {
    mask_non_code(line).into_iter().next().unwrap_or_default()
}

fn is_ident_char(c: char) -> bool {
    c.is_alphanumeric() || c == '_'
}

/// A bare `HolonAST` occurrence NOT immediately followed by `::` (which would make it an enum
/// path — a pattern or a constructor call, handled separately) and not on a `use` line (an
/// import names the type into scope; it does not itself hold or return one). Every remaining
/// bare occurrence has no other legal Rust role than a type reference — a field, a parameter, a
/// return type, or a generic argument (`Arc<HolonAST>`, `Vec<HolonAST>`).
fn declare_hit(code: &str) -> bool {
    let trimmed = code.trim_start();
    if trimmed.starts_with("use ")
        || trimmed.starts_with("pub use ")
        || trimmed.starts_with("pub(crate) use ")
        || trimmed.starts_with("pub(super) use ")
        || trimmed.starts_with("pub(in ")
    {
        return false;
    }
    let bytes = code.as_bytes();
    let mut idx = 0;
    while let Some(pos) = code[idx..].find("HolonAST") {
        let start = idx + pos;
        let end = start + "HolonAST".len();
        let left_ok = start == 0 || !is_ident_char(bytes[start - 1] as char);
        let right_ok = end == bytes.len() || !is_ident_char(bytes[end] as char);
        if left_ok && right_ok {
            let followed_by_path = code.as_bytes().get(end..end + 2) == Some(b"::");
            if !followed_by_path {
                return true;
            }
        }
        idx = end;
    }
    false
}

/// Unambiguous constructions — Rust patterns cannot call a function, so these lowercase free-fn
/// spellings are never a match arm.
static LOWER_CTOR: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"\bHolonAST::(bundle|keyword|symbol|i64|string|nil|bool_|char_|f64)\s*\(").unwrap()
});

/// Ambiguous — a CamelCase variant constructor call, textually identical to a match-arm pattern.
static CAMEL_CTOR: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\bHolonAST::[A-Z][A-Za-z0-9_]*\s*\(").unwrap());

/// Classifies one already-comment/string-stripped line. `None` = no violation on this line.
fn classify(code: &str) -> Option<&'static str> {
    if !code.contains("HolonAST") {
        return None;
    }
    if LOWER_CTOR.is_match(code) {
        return Some("construct (lowercase free-fn constructor)");
    }
    if let Some(m) = CAMEL_CTOR.find(code) {
        let trimmed = code.trim_start();
        let is_first_token = trimmed.starts_with("HolonAST::");
        let is_matches_macro_arg = code
            .find("matches!(")
            .is_some_and(|macro_pos| macro_pos < m.start());
        if !is_first_token && !is_matches_macro_arg {
            return Some("construct (enum-variant constructor, not in pattern position)");
        }
    }
    if declare_hit(code) {
        return Some("declare (field / parameter / return type)");
    }
    None
}

#[test]
fn holon_ast_confined_to_vsa_homes_and_the_one_carrier() {
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

    // A discovering walk must prove it discovered something, or an empty sweep reads as clean
    // (same doctrine as `no_rc_use.rs`).
    assert!(
        files.len() > 50,
        "the holon-is-vsa-only walk found only {} .rs files — it is not reaching the tree, so \
         its green means nothing (a gate that discovers its inputs must floor-assert the count).",
        files.len()
    );

    let mut violations = Vec::new();
    for f in &files {
        let rel_raw = f.strip_prefix(manifest).unwrap_or(f).display().to_string();
        let rel = rel_raw.trim_start_matches('/').replace('\\', "/");
        if is_home_or_carrier(&rel) {
            continue;
        }
        let Ok(src) = std::fs::read_to_string(f) else { continue };
        let raw_lines: Vec<&str> = src.lines().collect();
        let masked_lines = mask_non_code(&src);
        for (idx, raw_line) in raw_lines.iter().enumerate() {
            if !raw_line.contains("HolonAST") {
                continue;
            }
            // Co-located rune: the offending line itself, or the line immediately above it
            // (same window `no_bare_is_err.rs` uses for its own per-site exemption).
            let runed = raw_lines
                .get(idx.saturating_sub(1)..=idx)
                .into_iter()
                .flatten()
                .any(|l| l.contains("rune:lint(holon-not-vsa"));
            if runed {
                continue;
            }
            let Some(code) = masked_lines.get(idx) else { continue };
            if let Some(kind) = classify(code) {
                violations.push(format!(
                    "{}:{}  [{}]  {}",
                    rel,
                    idx + 1,
                    kind,
                    raw_line.trim()
                ));
            }
        }
    }

    assert!(
        violations.is_empty(),
        "\n\n🔥🔥🔥 HOLON USED OUTSIDE VSA/HDC — {} site(s) construct or declare a `HolonAST` \
         outside its homes.\n\
         \n\
         `HolonAST` is a hypervector composition (bundle/bind/cosine); `WatAST` is the wat syntax \
         tree. The substrate has printed this rule to users for a long time (`src/types/error.rs`, \
         `src/function/parse.rs`) — this wall is what makes it more than a sentence nobody can \
         check (`docs/arc/2026/06/294-holon-returns-to-vsa/RULING-holon-is-for-vsa-only-and-a-wall-will-say-so.md`).\n\
         \n\
         THE FIX — use `:wat::WatAST` (`crate::ast::WatAST`) for syntax; reserve `HolonAST` for an \
         actual VSA/HDC algebra value.\n\
         \n\
         If a site PROVABLY needs the holon as its subject (not merely as a convenient carrier), \
         add a co-located `// rune:lint(holon-not-vsa, <category>) — <reason>` naming WHY the \
         holon is the subject.\n\
         \n\
         Offenders:\n\n{}\n",
        violations.len(),
        violations.join("\n"),
    );
}

#[cfg(test)]
mod detector_tests {
    use super::{classify, declare_hit, mask_line};

    fn clean(line: &str) -> String {
        mask_line(line)
    }

    #[test]
    fn lowercase_ctor_is_always_construction() {
        assert_eq!(
            classify(&clean("    let sketch = holon::HolonAST::bundle(children);")),
            Some("construct (lowercase free-fn constructor)")
        );
        assert_eq!(
            classify(&clean("children.push(HolonAST::keyword(head));")),
            Some("construct (lowercase free-fn constructor)")
        );
    }

    #[test]
    fn camel_ctor_at_line_start_is_a_pattern() {
        assert_eq!(classify(&clean("        HolonAST::Bundle(children) => Ok(children),")), None);
        assert_eq!(
            classify(&clean("            HolonAST::Bind(a, b) => find(a).or(find(b)),")),
            None
        );
    }

    #[test]
    fn camel_ctor_inside_matches_macro_is_a_pattern() {
        assert_eq!(
            classify(&clean("assert!(matches!(&*h, HolonAST::Bind(_, _)));")),
            None
        );
    }

    #[test]
    fn camel_ctor_not_at_line_start_and_not_in_matches_is_construction() {
        assert_eq!(
            classify(&clean("    let x = HolonAST::Bundle(vec![]);")),
            Some("construct (enum-variant constructor, not in pattern position)")
        );
    }

    #[test]
    fn bare_declare_shapes_are_caught() {
        assert!(declare_hit(&clean("    pub signature: HolonAST,")));
        assert!(declare_hit(&clean("fn sketch(head: &str) -> HolonAST {")));
        assert!(declare_hit(&clean("    pub(crate) fn require_bundle<'a>(holon: &'a HolonAST) -> Result<&'a Vec<HolonAST>, EvalBreak> {")));
    }

    #[test]
    fn use_import_is_not_a_declare() {
        assert!(!declare_hit(&clean("use holon::HolonAST;")));
        assert!(!declare_hit(&clean("pub(crate) use holon::HolonAST;")));
    }

    #[test]
    fn enum_path_prefix_is_not_a_bare_declare() {
        // Followed by `::`, so it routes to the constructor/pattern classifier instead.
        assert!(!declare_hit(&clean("        HolonAST::Bundle(children) => Ok(children),")));
    }

    #[test]
    fn prose_is_never_a_hit() {
        assert_eq!(
            classify(&clean(
                r#"                    expected: "a :wat::holon::HolonAST Bundle composition","#
            )),
            None
        );
        assert_eq!(
            classify(&clean(
                r#"        args: vec![TypeExpr::Path(":wat::holon::HolonAST".into())],"#
            )),
            None
        );
        assert_eq!(
            classify(&clean("    // HolonAST::Bundle(vec![]) would be a violation if it were code")),
            None
        );
        assert_eq!(
            classify(&clean("    /// See `HolonAST::keyword` for the stripping rule.")),
            None
        );
    }

    #[test]
    fn escaped_quote_inside_a_string_does_not_end_it_early() {
        // `\"` inside the string must not be read as the closing quote — if it were, `HolonAST`
        // after it would wrongly land outside the (supposedly still-open) string.
        let line = r#"    let s = "a \" mid-string HolonAST quote"; let real = HolonAST::bundle(v);"#;
        // The literal call after the string is still a real construction.
        assert_eq!(
            classify(&clean(line)),
            Some("construct (lowercase free-fn constructor)")
        );
    }
}
