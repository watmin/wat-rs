//! STONE 255.78 — the floor never reads a clock to decide.
//!
//! `T1` (the builder, 2026-10-01): *"i strongly dislike timing based on a single machine's
//! capabilities … W1 attacking a time property makes sense."* Ruled: no verdict on the floor
//! depends on a clock. A claim that is only about speed leaves the floor for `benches/`
//! (`harness = false`, prints, never gates — precedent `benches/binding_repr.rs`).
//!
//! This is the wall that keeps a clock-based verdict from regrowing in `src/**/tests/**` or
//! `tests/**`. It is syntactic, single-file, single-pass — **say what it can and cannot see**:
//!
//! ## What it CAN see
//!
//! 1. A variable is **tainted** (clock-derived) when the right-hand side of its assignment (or
//!    the argument of a `.push`/`.push_back`/`.push_back_mut`/`.fetch_add` call it feeds)
//!    contains, anywhere in the SAME statement (tracked across its own `;`-terminated span, so a
//!    multi-line closure body is still seen):
//!      - `Instant::now(`  — a direct clock read
//!      - `.elapsed(`      — a direct clock read
//!      - `ns_per_iter(` / `elapsed_ns(` — this file's own timing helpers (`tests/mod.rs`)
//!      - `(_, ns, …)` / `(_, ns)` — the phase-census tuple-destructure idiom this whole test
//!        family uses to pull nanoseconds out of a `(name, ns, count)` / `(name, ns)` row
//!      - a reference to an **already-tainted** name (so taint propagates: `t` → `t.elapsed()`
//!        → `h` → `ms(h)` → …)
//! 2. Every `assert!`, `assert_eq!`, `assert_ne!` call, found by a string/comment-aware balanced
//!    scan (so a `(` or `)` inside a format-string message cannot desync the parser).
//! 3. For `assert!`, the first top-level argument (the condition), split on top-level `&&`. Each
//!    conjunct referencing a tainted name must be the **liveness form** `IDENT > 0` /
//!    `IDENT > 0.0` (optionally parenthesised) — anything else (`<`, `<=`, `>=`, comparing two
//!    tainted names, arithmetic thresholds…) is a violation.
//! 4. For `assert_eq!`/`assert_ne!`, either of the first two top-level arguments being tainted is
//!    always a violation — equality of two clock readings has no liveness form.
//! 5. A co-located `// rune:lint(clock-verdict) — <reason>` anywhere inside the matched macro call
//!    exempts it, the same shape as `no_rpds_rebuild_loop.rs`'s own exemption.
//!
//! ## What it CANNOT see
//!
//! - **Cross-function / cross-file taint.** A helper that reads a clock and returns a plain
//!   `u64`/`f64` with no literal marker at the call site (no `.elapsed(`, no `ns_per_iter(`, no
//!   the `ns`-tuple idiom) is invisible — this is a textual, not a type-level, census. The two
//!   markers that exist today (`ns_per_iter`/`elapsed_ns` by name, and the `(_, ns, …)`
//!   destructure idiom) were chosen because every known clock-returning helper in this tree uses
//!   one of them; a THIRD convention would need a third marker here.
//! - **Shadowing / re-binding inside nested scopes it cannot distinguish from the outer one** —
//!   taint is a per-file name set, not scope-aware. A local `h` shadowing a tainted outer `h`
//!   with fresh, non-clock data would still read as tainted. None of this tree's cost tests do
//!   that today.
//! - **A conjunct that is itself a function call** (`waste_is_applicable(..)`) — only a bare
//!   `IDENT > 0`/`IDENT > 0.0` shape is recognised as liveness; anything syntactically richer on
//!   a NON-tainted conjunct is simply not examined (fine — it cannot be tainted without a tainted
//!   identifier in it).

use std::path::{Path, PathBuf};
use std::sync::LazyLock;

use regex::Regex;

// ── file discovery ────────────────────────────────────────────────────────────────────────

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

/// `src/**/tests/**` — any `.rs` under `src/` whose path has a `tests` component — plus the
/// whole `tests/` tree (the brief's own two clauses).
fn in_scope_files(manifest: &Path) -> Vec<PathBuf> {
    let mut out = Vec::new();
    let mut under_src = Vec::new();
    collect_rs(&manifest.join("src"), &mut under_src);
    for p in under_src {
        let rel = p.strip_prefix(manifest).unwrap_or(&p);
        if rel.components().any(|c| c.as_os_str() == "tests") {
            out.push(p);
        }
    }
    collect_rs(&manifest.join("tests"), &mut out);
    out
}

// ── byte classification: code / line-comment / string literal ───────────────────────────────

#[derive(Clone, Copy, PartialEq, Eq)]
enum Class {
    Code,
    Comment,
    Str,
}

/// Single pass, byte-level. Escapes inside a string are honoured so a `\"` cannot end it early;
/// a `//` inside a string does not start a comment. Raw strings (`r"…"`, `r#"…"#`) are NOT
/// specially handled — none of the files this lint scans use them in assert bodies today
/// (checked by the non-vacuity count below staying sane); a raw string containing an unescaped
/// `"` would desync this classifier, same caveat `no_rpds_rebuild_loop.rs`'s line-based scan
/// carries for anything it does not model.
fn classify(src: &[u8]) -> Vec<Class> {
    let mut out = vec![Class::Code; src.len()];
    let mut i = 0usize;
    let mut in_comment = false;
    let mut in_string = false;
    while i < src.len() {
        let c = src[i];
        if in_comment {
            out[i] = Class::Comment;
            if c == b'\n' {
                in_comment = false;
            }
            i += 1;
            continue;
        }
        if in_string {
            out[i] = Class::Str;
            if c == b'\\' && i + 1 < src.len() {
                out[i + 1] = Class::Str;
                i += 2;
                continue;
            }
            if c == b'"' {
                in_string = false;
            }
            i += 1;
            continue;
        }
        if c == b'/' && i + 1 < src.len() && src[i + 1] == b'/' {
            in_comment = true;
            out[i] = Class::Comment;
            i += 1;
            continue;
        }
        if c == b'"' {
            in_string = true;
            out[i] = Class::Str;
            i += 1;
            continue;
        }
        out[i] = Class::Code;
        i += 1;
    }
    out
}

/// From a CODE byte, the index of the first top-level (bracket-depth <= 0) `;`, or `src.len()`.
/// Comment/string bytes neither open/close a bracket nor terminate a statement.
fn statement_end(src: &[u8], classes: &[Class], start: usize) -> usize {
    let mut depth: i32 = 0;
    let mut i = start;
    while i < src.len() {
        if classes[i] == Class::Code {
            match src[i] {
                b'(' | b'{' | b'[' => depth += 1,
                b')' | b'}' | b']' => depth -= 1,
                b';' if depth <= 0 => return i,
                _ => {}
            }
        }
        i += 1;
    }
    src.len()
}

/// Every top-level-or-nested `fn … { … }` body span `(body_start, body_end)`, where `body_start`
/// is the byte right after the opening `{` and `body_end` is the byte of the matching `}`.
///
/// ★ WHY SCOPED TO THE FUNCTION BODY, NOT THE WHOLE FILE: this test family's house style reuses
/// a tiny alphabet of single/double-letter names (`a b c d e f g h i j k l m n s t w`) PER
/// FUNCTION — `h` means "harvest ns" in one `#[test] fn` and is a totally unrelated loop counter
/// two functions later. A file-wide taint set made every one of those unrelated `h`/`n`/`k`
/// reuses read as clock-derived, which is not a syntactic limitation worth documenting — it is a
/// wrong answer. Scoping taint (and the assert scan) to one function body at a time matches
/// Rust's own scoping closely enough for this file family: these tests do not share state
/// between sibling `#[test] fn`s or across a closure boundary into an enclosing fn's names.
fn find_fn_bodies(src: &[u8], classes: &[Class]) -> Vec<(usize, usize)> {
    static FN_HEAD: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"\bfn\s+[A-Za-z_][A-Za-z0-9_]*\s*(?:<[^{};]*>)?\s*\(").unwrap());
    let text = std::str::from_utf8(src).unwrap_or("");
    let mut out = Vec::new();
    for m in FN_HEAD.find_iter(text) {
        if classes[m.start()] != Class::Code {
            continue;
        }
        let params_open = m.end() - 1; // the '('
        let params_close = matching_close(src, classes, params_open);
        // Scan forward from the params' close paren for the body's opening `{` at depth 0 —
        // skips a return type / where-clause, including any `(` `)` inside it (e.g. `-> Result<(), E>`).
        let mut depth: i32 = 0;
        let mut i = params_close + 1;
        let mut body_open = None;
        while i < src.len() {
            if classes[i] == Class::Code {
                match src[i] {
                    b'(' | b'[' => depth += 1,
                    b')' | b']' => depth -= 1,
                    b'{' if depth == 0 => {
                        body_open = Some(i);
                        break;
                    }
                    b';' if depth <= 0 => break, // a signature with no body (trait decl) — skip
                    _ => {}
                }
            }
            i += 1;
        }
        let Some(open) = body_open else { continue };
        let close = matching_close(src, classes, open);
        out.push((open + 1, close));
    }
    out
}

/// Replace every Comment/Str byte with a space (newlines always preserved, whatever their
/// class, so line numbers stay correct). Structural scanners (`statement_end`, `matching_close`,
/// the comma/`&&` splitters, `find_macro_calls`'s literal search) only ever look at
/// `Class::Code` positions, which this never touches — so they give identical answers on the
/// masked text as on the original. What changes is every REGEX / whole-word search downstream,
/// which is exactly the point: a string literal or a comment that happens to spell
/// `Instant::now(` (a unit-test fixture quoting a forbidden shape, say) must never seed taint.
fn mask(src: &[u8], classes: &[Class]) -> String {
    let mut out = Vec::with_capacity(src.len());
    for (i, &b) in src.iter().enumerate() {
        if b == b'\n' || classes[i] == Class::Code {
            out.push(b);
        } else {
            out.push(b' ');
        }
    }
    String::from_utf8(out).unwrap_or_default()
}

/// From a CODE `(` at `open`, the index of its matching `)` (depth-uniform across all three
/// bracket kinds — good enough to close a macro call; Rust source is well-bracketed).
fn matching_close(src: &[u8], classes: &[Class], open: usize) -> usize {
    let mut depth: i32 = 0;
    let mut i = open;
    while i < src.len() {
        if classes[i] == Class::Code {
            match src[i] {
                b'(' | b'{' | b'[' => depth += 1,
                b')' | b'}' | b']' => {
                    depth -= 1;
                    if depth == 0 {
                        return i;
                    }
                }
                _ => {}
            }
        }
        i += 1;
    }
    src.len().saturating_sub(1)
}

/// Split `text[0..]` on top-level (depth 0) commas, CODE bytes only.
fn top_level_commas(text: &[u8], classes: &[Class]) -> Vec<(usize, usize)> {
    let mut parts = Vec::new();
    let mut depth: i32 = 0;
    let mut start = 0usize;
    for (i, &b) in text.iter().enumerate() {
        if classes[i] != Class::Code {
            continue;
        }
        match b {
            b'(' | b'{' | b'[' => depth += 1,
            b')' | b'}' | b']' => depth -= 1,
            b',' if depth == 0 => {
                parts.push((start, i));
                start = i + 1;
            }
            _ => {}
        }
    }
    parts.push((start, text.len()));
    parts
}

/// Split `text` on top-level `&&`, CODE bytes only.
fn top_level_and(text: &[u8], classes: &[Class]) -> Vec<(usize, usize)> {
    let mut parts = Vec::new();
    let mut depth: i32 = 0;
    let mut start = 0usize;
    let mut i = 0usize;
    while i < text.len() {
        if classes[i] == Class::Code {
            match text[i] {
                b'(' | b'{' | b'[' => depth += 1,
                b')' | b'}' | b']' => depth -= 1,
                b'&' if depth == 0 && i + 1 < text.len() && text[i + 1] == b'&' => {
                    parts.push((start, i));
                    i += 2;
                    start = i;
                    continue;
                }
                _ => {}
            }
        }
        i += 1;
    }
    parts.push((start, text.len()));
    parts
}

/// Whole-word `name` present in `haystack`, EXCLUDING a reference immediately followed by `.` —
/// i.e. `name` and `name(…)` and `name,`/`name)`/`name ` count; `name.field` does not.
///
/// ★ WHY: a struct can legitimately mix a clock-derived field with a plain count field under one
/// binding (`Shot { wall: <clock>, query_maps: <count> }` in `strat_cost.rs`/`fanout_cost.rs`).
/// This detector has no field-sensitive dataflow — a bare struct variable taints as a WHOLE —
/// so without this exclusion, `a.wall` (genuinely clock-derived) and `a.query_maps` (not) are
/// indistinguishable, and every sibling-field assert on the SAME struct value false-positives.
/// Excluding `IDENT.field` from counting as a taint REFERENCE (while still letting the direct
/// seed markers — `.elapsed(`, `ns_per_iter(`, the `ns`-tuple idiom — match through a dot, since
/// those are checked by [`SEED_MARKERS`] directly against the raw RHS text, not through this
/// function) trades a FALSE NEGATIVE (an assert directly on a tainted struct's clock field,
/// reached only via `.field`, stops being seen) for removing a whole class of false positives
/// that is actually hit in this corpus today. No assert in this corpus depends on the lost case.
fn whole_word_present(haystack: &str, name: &str) -> bool {
    let bytes = haystack.as_bytes();
    let nb = name.as_bytes();
    let is_ident = |b: u8| b.is_ascii_alphanumeric() || b == b'_';
    let mut start = 0;
    while let Some(pos) = haystack[start..].find(name) {
        let abs = start + pos;
        let before_ok = abs == 0 || !is_ident(bytes[abs - 1]);
        let after = abs + nb.len();
        let after_ok = after >= bytes.len() || !is_ident(bytes[after]);
        let not_field_access = after >= bytes.len() || bytes[after] != b'.';
        if before_ok && after_ok && not_field_access {
            return true;
        }
        start = abs + 1;
        if start >= haystack.len() {
            break;
        }
    }
    false
}

// ── taint markers ────────────────────────────────────────────────────────────────────────

static SEED_MARKERS: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"Instant::now\(|\.elapsed\s*\(|\bns_per_iter\s*\(|\belapsed_ns\s*\(|\(\s*_\s*,\s*ns\s*[,)]")
        .unwrap()
});

/// A binding site: `let [mut] NAME [: TYPE] =` or bare `NAME =`, not `==`/`<=`/`>=`/`!=`.
static BINDING: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?m)^[ \t]*(?:let[ \t]+(?:mut[ \t]+)?)?([A-Za-z_][A-Za-z0-9_]*)[ \t]*(?::[^=;\n]*)?=[^=]").unwrap()
});

/// `IDENT.push(`, `.push_back(`, `.push_back_mut(`, `.fetch_add(` — the container/cell being fed.
static FEED: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?m)^[ \t]*([A-Za-z_][A-Za-z0-9_]*)\s*\.\s*(?:push_back_mut|push_back|push|fetch_add)\s*\(")
        .unwrap()
});

/// `let (a, b, …) =` — a tuple-destructuring binding (no nested parens inside the pattern; every
/// site in this corpus is flat). Every named element taints identically from the shared RHS —
/// coarser than per-element attribution, but the alternative (treating the pattern as invisible,
/// which a bare [`BINDING`] does) is a FALSE NEGATIVE this corpus actually hits: `fanout_cost.rs`'s
/// `let (prod, hj) = (ns_of("production"), ns_of("hash-join"));` is exactly this shape.
static TUPLE_BINDING: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?m)^[ \t]*let[ \t]+(?:mut[ \t]+)?\(([^()=;\n]+)\)[ \t]*=[^=]").unwrap()
});

/// Names whose assignment (or `.push`-family feed), anywhere in this file, is clock-derived.
/// Single file, single pass, fixed-point: a name taints anything that references it afterwards —
/// so `t` (`Instant::now()`) taints `h` (`t.elapsed()`), which taints `ms(h)`'s binding, etc.
fn tainted_names(src: &str, classes: &[Class]) -> std::collections::HashSet<String> {
    let bytes = src.as_bytes();
    let mut tainted: std::collections::HashSet<String> = std::collections::HashSet::new();

    // Collect every (name, statement_text) binding/feed site once.
    let mut sites: Vec<(String, String)> = Vec::new();
    for caps in BINDING.captures_iter(src) {
        let m = caps.get(0).unwrap();
        if classes[m.start()] != Class::Code {
            continue;
        }
        let name = caps.get(1).unwrap().as_str().to_string();
        let rhs_start = m.end() - 1; // the non-`=` byte BINDING's negative class consumed
        let end = statement_end(bytes, classes, rhs_start);
        sites.push((name, src[rhs_start..end].to_string()));
    }
    for caps in FEED.captures_iter(src) {
        let m = caps.get(0).unwrap();
        if classes[m.start()] != Class::Code {
            continue;
        }
        let name = caps.get(1).unwrap().as_str().to_string();
        let open = m.end() - 1;
        let close = matching_close(bytes, classes, open);
        sites.push((name, src[open..=close].to_string()));
    }
    for caps in TUPLE_BINDING.captures_iter(src) {
        let m = caps.get(0).unwrap();
        if classes[m.start()] != Class::Code {
            continue;
        }
        let rhs_start = m.end() - 1;
        let end = statement_end(bytes, classes, rhs_start);
        let names: Vec<&str> = caps
            .get(1)
            .unwrap()
            .as_str()
            .split(',')
            .map(str::trim)
            .filter(|n| !n.is_empty() && *n != "_" && !n.starts_with('_'))
            .collect();
        // A tuple's positions carry DIFFERENT meanings (`(ns_reading, count)` is common in this
        // corpus) — if the RHS is itself a parenthesised tuple LITERAL with exactly as many
        // top-level elements as names, attribute PER POSITION instead of pooling every name
        // under the whole statement's text. An opaque RHS (a bare call like `of(RHS)`, whose
        // callee's own body is out of this file's view) gets NO taint at all rather than a
        // guess — a stated false negative, safer than the false positive it replaces (fixes
        // `fanout_cost.rs`'s `let (rhs_raw, rhs_pairs) = of(RHS);`, where `rhs_pairs` is a
        // plain count and would otherwise inherit `rhs_raw`'s clock taint).
        let trimmed_rhs = src[rhs_start..end].trim_end_matches(';').trim();
        let positional = trimmed_rhs.starts_with('(') && trimmed_rhs.ends_with(')');
        if positional {
            let inner = &trimmed_rhs.as_bytes()[1..trimmed_rhs.len() - 1];
            let inner_classes = classify(inner);
            let parts = top_level_commas(inner, &inner_classes);
            if parts.len() == names.len() {
                for (name, (ps, pe)) in names.iter().zip(parts.iter()) {
                    let elem = String::from_utf8_lossy(&inner[*ps..*pe]).to_string();
                    sites.push((name.to_string(), elem));
                }
                continue;
            }
        }
        let rhs = src[rhs_start..end].to_string();
        for name in names {
            sites.push((name.to_string(), rhs.clone()));
        }
    }

    // Fixed point: a handful of passes is enough for this file family's shallow chains.
    for _ in 0..8 {
        let mut grew = false;
        for (name, rhs) in &sites {
            if tainted.contains(name) {
                continue;
            }
            let seeded = SEED_MARKERS.is_match(rhs)
                || tainted.iter().any(|t| whole_word_present(rhs, t));
            if seeded {
                tainted.insert(name.clone());
                grew = true;
            }
        }
        if !grew {
            break;
        }
    }
    tainted
}

// ── macro-call extraction ───────────────────────────────────────────────────────────────────

struct MacroCall {
    line: usize,
    start: usize,
    end: usize, // inclusive index of the closing `)`
    args: Vec<String>, // top-level comma-separated argument texts
}

fn find_macro_calls(src: &str, classes: &[Class], name: &str) -> Vec<MacroCall> {
    let bytes = src.as_bytes();
    let needle = format!("{name}!(");
    let nb = needle.as_bytes();
    let mut out = Vec::new();
    let mut i = 0usize;
    while i + nb.len() <= bytes.len() {
        if classes[i] == Class::Code && &bytes[i..i + nb.len()] == nb {
            let before_ok = i == 0 || !(bytes[i - 1].is_ascii_alphanumeric() || bytes[i - 1] == b'_');
            if before_ok {
                let open = i + nb.len() - 1; // the '('
                let close = matching_close(bytes, classes, open);
                let line = src[..i].matches('\n').count() + 1;
                let inner = &bytes[open + 1..close];
                let inner_classes = &classes[open + 1..close];
                let args: Vec<String> = top_level_commas(inner, inner_classes)
                    .into_iter()
                    .map(|(s, e)| String::from_utf8_lossy(&inner[s..e]).trim().to_string())
                    .filter(|s| !s.is_empty())
                    .collect();
                out.push(MacroCall {
                    line,
                    start: i,
                    end: close,
                    args,
                });
                i = close + 1;
                continue;
            }
        }
        i += 1;
    }
    out
}

const RUNE: &str = "rune:lint(clock-verdict)";

/// A `> 0` / `> 0.0` liveness atom, optionally parenthesised, optionally with a trailing cast
/// (`as f64`) — the one shape this wall lets a tainted identifier appear in.
static LIVENESS: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^\(*\s*[A-Za-z_][A-Za-z0-9_.]*\s*(?:as\s+\w+\s*)?\)*\s*>\s*0(?:\.0)?\s*\)*$").unwrap()
});

/// A top-level (depth-0, CODE-classified) comparison operator `< > <= >= == !=` in `text` —
/// NOT inside a nested call/closure (so `.all(|t| x == y)` does not count: that tests a
/// PREDICATE over elements, it does not compare a scalar clock value at this conjunct's own
/// level) and not a `=>`/`->` false match.
fn has_top_level_comparison(text: &[u8], classes: &[Class]) -> bool {
    let mut depth: i32 = 0;
    let mut i = 0usize;
    while i < text.len() {
        if classes[i] == Class::Code {
            match text[i] {
                b'(' | b'{' | b'[' => depth += 1,
                b')' | b'}' | b']' => depth -= 1,
                b'<' | b'>' if depth == 0 => return true,
                b'=' if depth == 0 && i + 1 < text.len() && text[i + 1] == b'=' => return true,
                b'!' if depth == 0 && i + 1 < text.len() && text[i + 1] == b'=' => return true,
                _ => {}
            }
        }
        i += 1;
    }
    false
}

fn condition_violations(cond: &str, tainted: &std::collections::HashSet<String>) -> Vec<String> {
    let bytes = cond.as_bytes();
    let classes = classify(bytes);
    let mut out = Vec::new();
    for (s, e) in top_level_and(bytes, &classes) {
        let conjunct = cond[s..e].trim();
        if conjunct.is_empty() {
            continue;
        }
        // Not a comparison at all (a bare boolean flag, `.is_empty()`, `.all(|t| …)`, a
        // structural predicate call) — nothing is being weighed against a clock reading here,
        // whatever names happen to appear in it.
        if !has_top_level_comparison(conjunct.as_bytes(), &classify(conjunct.as_bytes())) {
            continue;
        }
        let touches_tainted = tainted.iter().any(|t| whole_word_present(conjunct, t));
        if touches_tainted && !LIVENESS.is_match(conjunct) {
            out.push(conjunct.to_string());
        }
    }
    out
}

/// Scan ONE span of source (a function body, or — for the detector's own unit tests, which hand
/// over a whole tiny fixture `fn` — the whole fixture) for clock-verdict violations.
///
/// `masked` (Str/Comment bytes blanked — see [`mask`]) drives every taint/liveness decision;
/// `original` is the SAME span, unmasked, consulted ONLY for the `// {RUNE}` exemption text,
/// which is deliberately a comment and would vanish under masking. `line_base` is added to every
/// reported line so callers scanning a sub-span of a larger file (a function body cut out of its
/// enclosing file) still report the real file line.
fn scan_span(rel: &str, masked: &str, original: &str, line_base: usize) -> Vec<String> {
    let classes = classify(masked.as_bytes());
    let tainted = tainted_names(masked, &classes);
    if tainted.is_empty() {
        return Vec::new();
    }
    // The window runs through the END OF THE LINE the macro call closes on — a trailing
    // `// {RUNE} — reason` after the closing `)` and `;` (the natural place to write one,
    // and where `no_rpds_rebuild_loop.rs`'s own exemption looks) must still be seen.
    let is_exempt = |call: &MacroCall| {
        let line_end = original[call.end..].find('\n').map(|i| call.end + i).unwrap_or(original.len());
        original.get(call.start..line_end).is_some_and(|t| t.contains(RUNE))
    };
    let mut violations = Vec::new();

    for call in find_macro_calls(masked, &classes, "assert") {
        if is_exempt(&call) {
            continue;
        }
        let Some(cond) = call.args.first() else { continue };
        for bad in condition_violations(cond, &tainted) {
            violations.push(format!(
                "  {rel}:{}: assert!(…) — clock-derived conjunct `{bad}` is not the liveness form `IDENT > 0` / `IDENT > 0.0`",
                line_base + call.line - 1
            ));
        }
    }
    for macro_name in ["assert_eq", "assert_ne"] {
        for call in find_macro_calls(masked, &classes, macro_name) {
            if is_exempt(&call) {
                continue;
            }
            let lhs_tainted = call.args.first().is_some_and(|a| tainted.iter().any(|t| whole_word_present(a, t)));
            let rhs_tainted = call.args.get(1).is_some_and(|a| tainted.iter().any(|t| whole_word_present(a, t)));
            if lhs_tainted || rhs_tainted {
                violations.push(format!(
                    "  {rel}:{}: {macro_name}!(…) — a clock-derived operand has no liveness form",
                    line_base + call.line - 1
                ));
            }
        }
    }
    violations
}

/// Split `src` into its function bodies (see [`find_fn_bodies`] for why) and scan each
/// independently. Text OUTSIDE any function body (top-level `const`/`static`/`type`) is not
/// scanned — no assert lives there.
fn scan_file(rel: &str, src: &str) -> Vec<String> {
    let classes = classify(src.as_bytes());
    let masked = mask(src.as_bytes(), &classes);
    let mut violations = Vec::new();
    for (start, end) in find_fn_bodies(masked.as_bytes(), &classes) {
        let line_base = src[..start].matches('\n').count() + 1;
        violations.extend(scan_span(rel, &masked[start..end], &src[start..end], line_base));
    }
    violations
}

/// Whether ANY function body in `src` carries a tainted (clock-derived) name — used only for
/// the gate's own non-vacuity check, not for reporting.
fn file_has_taint(src: &str) -> bool {
    let classes = classify(src.as_bytes());
    let masked = mask(src.as_bytes(), &classes);
    find_fn_bodies(masked.as_bytes(), &classes).into_iter().any(|(start, end)| {
        let span = &masked[start..end];
        let span_classes = classify(span.as_bytes());
        !tainted_names(span, &span_classes).is_empty()
    })
}

#[test]
fn an_assertion_never_compares_a_clock_reading_except_for_liveness() {
    let manifest = Path::new(env!("CARGO_MANIFEST_DIR"));
    let files = in_scope_files(manifest);

    // NON-VACUITY: the walk must actually reach the family of files this wall exists for.
    assert!(
        files.len() > 100,
        "the clock-verdict walk found only {} .rs files under src/**/tests/** + tests/** — it is \
         not reaching the tree it claims to guard",
        files.len()
    );

    let mut violations = Vec::new();
    let mut files_with_taint = 0usize;
    for f in &files {
        let Ok(src) = std::fs::read_to_string(f) else { continue };
        let rel = f.strip_prefix(manifest).unwrap_or(f).to_string_lossy().replace('\\', "/");
        if file_has_taint(&src) {
            files_with_taint += 1;
        }
        violations.extend(scan_file(&rel, &src));
    }

    // NON-VACUITY on the INSTRUMENT: this family of files is known (by the stone's own census)
    // to carry clock-derived bindings today (the timing PRINTOUTS the brief says may stay). If
    // this reads 0 the taint detector stopped recognising its own markers.
    assert!(
        files_with_taint > 0,
        "no file under src/**/tests/** or tests/** was seen as clock-tainted — the marker set \
         (Instant::now/.elapsed/ns_per_iter/elapsed_ns/the ns-tuple idiom) is not matching \
         anything, so this gate cannot be trusted to have examined anything either"
    );

    assert!(
        violations.is_empty(),
        "\n\n🔥🔥🔥 THE FLOOR READ A CLOCK TO DECIDE — {} site(s).\n\
         \n\
         `docs/arc/2026/06/255-builtin-registry/BRIEF-STONE-255.78-the-floor-never-reads-a-clock.md`:\n\
         no verdict on the floor may depend on a clock. A liveness check (`IDENT > 0` /\n\
         `IDENT > 0.0` — \"the loop ran at all\") is fine and stays. Any other comparison of a\n\
         clock-derived value (Instant::now/.elapsed/ns_per_iter/elapsed_ns/a phase-census `ns`\n\
         reading) is not: replace it with a deterministic witness (a count, a byte-allocation\n\
         delta, a structural invariant) or move the claim to `benches/` (printed, not asserted).\n\
         \n\
         A genuinely structural, machine-independent ordering of two timed arms (e.g. \"the walk\n\
         did not get optimised away, which this ladder's own adjacent-rung note already found too\n\
         noisy to assert\") still needs a non-clock proof — or a co-located\n\
         `// {RUNE} — <reason>` if the orchestrator has ruled it out of scope for THIS stone.\n\
         \n\
         Offenders:\n\n{}\n",
        violations.len(),
        violations.join("\n"),
    );
}

#[cfg(test)]
mod detector_tests {
    use super::*;

    fn violations_of(src: &str) -> Vec<String> {
        scan_file("fixture.rs", src)
    }

    #[test]
    fn a_direct_elapsed_threshold_is_a_hit() {
        let src = "fn f() {\n    let t = Instant::now();\n    let h = t.elapsed().as_nanos() as f64;\n    assert!(h < 25.0, \"too slow\");\n}\n";
        let v = violations_of(src);
        assert_eq!(
            v,
            ["  fixture.rs:4: assert!(…) — clock-derived conjunct `h < 25.0` is not the liveness form `IDENT > 0` / `IDENT > 0.0`"]
        );
    }

    #[test]
    fn the_harvest_apportionment_shape_is_a_hit() {
        // The exact shape removed from harvest_cost.rs's harvest_wrap_split.
        let src = "fn f() {\n    let s = ns_per_iter(1, || {});\n    let w = ns_per_iter(1, || {});\n    let h = ns_per_iter(1, || {});\n    assert!(\n        h >= (s + w) * 0.5 && h <= (s + w) * 2.0,\n        \"not accounted for\"\n    );\n}\n";
        let v = violations_of(src);
        // Two conjuncts (`h >= (s+w)*0.5` and `h <= (s+w)*2.0`), both clock-derived and neither
        // the liveness form — both are reported, not deduplicated into one.
        assert_eq!(v.len(), 2, "{v:?}");
    }

    #[test]
    fn a_phase_census_ns_tuple_chain_is_a_hit() {
        let src = "fn f() {\n    let ns_of = |n: &str| rows.iter().find(|(nm, _, _)| *nm == n).map(|(_, ns, _)| *ns).unwrap_or(0);\n    let fold_ms = ns_of(\"fold\") as f64 / 1e6;\n    assert!(fold_ms < 25.0, \"regressed\");\n}\n";
        let v = violations_of(src);
        assert_eq!(v.len(), 1, "{v:?}");
    }

    #[test]
    fn a_pure_liveness_check_is_not_a_hit() {
        let src = "fn f() {\n    let t = Instant::now();\n    let h = t.elapsed().as_nanos() as f64;\n    assert!(h > 0.0, \"the loop never ran\");\n}\n";
        assert!(violations_of(src).is_empty());
    }

    #[test]
    fn a_compound_liveness_check_is_not_a_hit() {
        let src = "fn f() {\n    let t = Instant::now();\n    let a = t.elapsed().as_nanos() as f64;\n    let b = t.elapsed().as_nanos() as f64;\n    assert!(a > 0.0 && b > 0.0, \"neither loop ran\");\n}\n";
        assert!(violations_of(src).is_empty());
    }

    #[test]
    fn an_untainted_assert_is_not_a_hit() {
        let src = "fn f() {\n    let count = 40_000usize;\n    assert_eq!(count, 40_000, \"wrong N\");\n}\n";
        assert!(violations_of(src).is_empty());
    }

    #[test]
    fn an_exempted_site_is_not_a_hit() {
        let src = "fn f() {\n    let t = Instant::now();\n    let h = t.elapsed().as_nanos() as f64;\n    assert!(h < 25.0, \"x\"); // rune:lint(clock-verdict) — reason\n}\n";
        assert!(violations_of(src).is_empty());
    }

    #[test]
    fn a_literal_paren_inside_the_message_string_does_not_desync_the_scan() {
        // The exact shape harvest_cost.rs's message used: literal `(` `)` inside the format
        // string. A paren-counter that is not string-aware would mis-close the macro call here.
        let src = "fn f() {\n    let t = Instant::now();\n    let h = t.elapsed().as_nanos() as f64;\n    assert!(h > 0.0, \"combined ({:.2} ms) is not accounted for\", h);\n}\n";
        assert!(violations_of(src).is_empty());
    }

    #[test]
    fn assert_eq_on_a_tainted_operand_is_a_hit() {
        let src = "fn f() {\n    let t = Instant::now();\n    let h = t.elapsed().as_nanos();\n    assert_eq!(h, 0);\n}\n";
        let v = violations_of(src);
        assert_eq!(v.len(), 1, "{v:?}");
    }

    #[test]
    fn a_commented_out_marker_is_not_a_hit() {
        let src = "fn f() {\n    // let t = Instant::now();\n    let h = 7;\n    assert!(h < 25, \"x\");\n}\n";
        assert!(violations_of(src).is_empty());
    }
}
