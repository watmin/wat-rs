//! **A CENSUS NAME A COST TEST READS MUST BE A NAME THE ENGINE EMITS — OR THE ROW IS A ZERO
//! WEARING A MEASUREMENT'S CLOTHES.**
//!
//! Every cost test in `src/rete/kernel/tests/` reads the phase census the same way:
//!
//! ```ignore
//! let of = |name: &str| -> u64 {
//!     rows.iter().find(|(nm, _, _)| *nm == name).map(|(_, ns, _)| *ns).unwrap_or(0)
//! };
//! ```
//!
//! `unwrap_or(0)` is the whole problem. It answers **"this mark does not exist"** and **"this mark
//! measured zero nanoseconds"** with the same `0`
//! (`[[feedback_a_catch_all_holds_two_facts]]`). A name nobody emits therefore does not fail, does
//! not warn, and does not print blank — it prints `0.00 ms`, indistinguishable from a real reading,
//! and then feeds arithmetic. Driven at HEAD before this gate existed, `accum_cost.rs` read the
//! never-emitted `"  │  setup:seen:insert"` and printed:
//!
//! ```text
//!   insert                          0.00 ms
//! in-fire insert − S              -2.55 ms
//! ```
//!
//! The second row is `0 − S`: the entire isolated cost, negated, presented as a difference between
//! two measurements when only one of them was ever measured. Nothing in the suite could go RED on
//! it, because a missing row and a fast row are the same value.
//!
//! **So the check moves to construction time.** A name read here must resolve to a name emitted
//! there. Absence becomes a build failure instead of a zero.
//!
//! ## THE TWO SETS
//!
//! **EMITTED** — every name the engine can put into the census, gathered from non-test `src/`:
//!
//! 1. The string-literal argument of [`EMITTERS`] (`phase_end`, `census_count`, `census_count_n`).
//! 2. **Computed families.** `delta.rs` calls `census_count(ebucket(n))` and
//!    `census_count(tbucket(n))` — helpers returning `&'static str` from a `match`. Those names are
//!    just as emittable as any literal one, but they never appear inside an emitter's parentheses.
//!    So this gate finds every emitter call whose argument is a bare identifier rather than a
//!    literal, then harvests that helper's own literals. Without this half the gate would
//!    false-RED the day a cost test read `"elem-card:3"` — a name the engine really does emit.
//!
//! **READ** — gathered from `src/rete/kernel/tests/*.rs`:
//!
//! 1. Any literal containing a census tree glyph ([`GLYPHS`] — `├`, `│`, `└`). The table nests a
//!    child row under its parent with these, so they mark a string as a census name unambiguously;
//!    nothing else in this corpus contains one. This is the half that reaches names held in a
//!    `const ALPHA_KIDS: [&str; 4]` and handed to the reader through a loop variable, where no
//!    grep of the call site can see them.
//! 2. Any literal passed to a **row reader** — a closure the file itself defines as
//!    `let <name> = |<p>: &str| -> …`. Discovered per file rather than hardcoded, because the
//!    corpus names it three different things: `of` (`accum_cost.rs:256`), `ns_of`
//!    (`node_share_cost.rs:447`), `get` (`gather_probe_cost.rs:1223`).
//!
//! ## ⛔ TWO SCOPE CUTS, EACH MADE AGAINST A MEASURED COUNTER-EXAMPLE
//!
//! Both were tried and both produce false REDs on a correct tree, so both are excluded **by
//! design** rather than by oversight:
//!
//! - **Every literal in a `const [&str; N]`.** 22 such arrays exist under the subject and 21 are
//!   phase-name lists — but `accum_cost.rs:357`'s `WANTED` holds `"apx::CountF"`, `"apx::SumF"`, …,
//!   which are rule classes, not census rows. Little is lost: the glyph half already reaches every
//!   *child* row, and a deleted *top-level* mark is already caught loudly by `REQUIRED_PHASES`'s
//!   subset assertion (`accum_cost.rs:101`), which this gate deliberately does not duplicate.
//! - **Every literal in an `== "…"` comparison.** This would reach `"compiled:calls"`
//!   (`accum_cost.rs:38`) and `"accum:index-builds"` (`gather_probe_cost.rs:35`), which are real
//!   census names — but also `"WHOLE EVAL (compile+seed+fire)"` (`mod.rs:570`), a row the test
//!   harness synthesises itself and the engine never emits, plus `"weather::Temperature"`,
//!   `"hs::q-T"` and `"apx::ExistsF"`. A rule that reds on a correct tree gets suppressed, and a
//!   suppressed gate is not a gate.
//!
//! ## THE EARNED EXEMPTION — `rune:lint(census-name-retired)`
//!
//! A read name that the engine no longer emits is not *always* the defect above. A mark can be
//! **deliberately retired** while its reader stays, and stays *correctly*: `accum_cost.rs`'s
//! `accum_leftover_split` still lists the four per-fact alpha child marks retired by `c9d751049`
//! ("bind-value intern through retiring per-fact alpha timers"). That reader is sound where the
//! `setup:seen:insert` one was not, and the difference is exactly the distinction `unwrap_or(0)`
//! destroys: it reads `(ns, pairs)` and branches on `kids_retired = pairs.all(== 0)` — it asks
//! whether the mark **fired**, never trusting the nanoseconds to tell it whether the mark
//! **exists**.
//!
//! So each such name carries a co-located `// rune:lint(census-name-retired) — <reason>`, and the
//! reason must be at least [`MIN_REASON_CHARS`] characters. A rune is a DECLARATION, not a
//! suppression: it asserts the absence is known and handled. It is also kept honest in both
//! directions — [`every_census_name_retired_rune_names_a_name_the_engine_no_longer_emits`] REDs if
//! a runed name comes back, so a revived mark cannot quietly keep an exemption it no longer needs.
//!
//! Shape and precedent: `tests/lint/rete_citation_resolves.rs` (`rune:lint(cited-name-absent)`) and
//! `tests/lint/retired_name_justified.rs` (`rune:lint(retired-name)`).

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

/// Where a census name is EMITTED. `src/rete/kernel/tests/` is carved out of this — a name a cost
/// test emits to its own fixture is not a name the engine can produce.
const EMIT_ROOT: &str = "src";

/// Excluded from [`EMIT_ROOT`]: the readers are the SUBJECT, never their own authority.
const NOT_EMIT: &str = "src/rete/kernel/tests";

/// Where a census name is READ.
const SUBJECT: &str = "src/rete/kernel/tests";

/// The three functions that put a name into the census (`src/rete/kernel/census.rs`).
const EMITTERS: [&str; 3] = ["phase_end", "census_count", "census_count_n"];

/// Box-drawing glyphs the census table uses to nest a child row under its parent. A string literal
/// in this corpus contains one only if it is a census row name.
const GLYPHS: [char; 3] = ['├', '│', '└'];

/// The exemption rune. See the module doc.
const RUNE: &str = "rune:lint(census-name-retired)";

/// A reason shorter than this cannot say which mark was retired, by what, or why the reader is
/// still sound without it.
const MIN_REASON_CHARS: usize = 40;

// ── source scanning ─────────────────────────────────────────────────────────────────────────────

/// One ordinary string literal, with the 1-based line it opened on and the identifier it was
/// called on, if it sits in an argument position (`foo("bar")` → `Some("foo")`).
#[derive(Debug, Clone)]
struct Lit {
    text: String,
    line: usize,
    callee: Option<String>,
}

fn is_ident_char(c: char) -> bool {
    c.is_alphanumeric() || c == '_'
}

/// The identifier of the call a literal opening at `start` is an argument to.
fn callee_before(b: &[char], start: usize) -> Option<String> {
    let mut i = start;
    while i > 0 && b[i - 1].is_whitespace() {
        i -= 1;
    }
    if i == 0 || b[i - 1] != '(' {
        return None;
    }
    i -= 1;
    while i > 0 && b[i - 1].is_whitespace() {
        i -= 1;
    }
    let end = i;
    while i > 0 && is_ident_char(b[i - 1]) {
        i -= 1;
    }
    if i == end {
        return None;
    }
    Some(b[i..end].iter().collect())
}

/// Every ordinary string literal in a Rust source, in order.
///
/// Comments (line and nested block), char literals and raw strings are skipped rather than parsed.
/// All three are real hazards in THIS corpus, not hypothetical ones: `'"'` appears 5 times under
/// `src/`, and `src/rete/kernel/tests/` holds 57 raw strings (inlined wat, full of quotes). A
/// scanner that mistook any of them for a string boundary would mispair every quote after it and
/// silently lose the rest of the file — the quiet failure this gate exists to prevent, committed by
/// the gate itself.
fn literals(src: &str) -> Vec<Lit> {
    let b: Vec<char> = src.chars().collect();
    let mut out = Vec::new();
    let mut i = 0usize;
    let mut line = 1usize;
    while i < b.len() {
        let c = b[i];
        if c == '\n' {
            line += 1;
            i += 1;
            continue;
        }
        if c == '/' && b.get(i + 1) == Some(&'/') {
            while i < b.len() && b[i] != '\n' {
                i += 1;
            }
            continue;
        }
        if c == '/' && b.get(i + 1) == Some(&'*') {
            i += 2;
            let mut depth = 1usize;
            while i < b.len() && depth > 0 {
                if b[i] == '\n' {
                    line += 1;
                    i += 1;
                } else if b[i] == '/' && b.get(i + 1) == Some(&'*') {
                    depth += 1;
                    i += 2;
                } else if b[i] == '*' && b.get(i + 1) == Some(&'/') {
                    depth -= 1;
                    i += 2;
                } else {
                    i += 1;
                }
            }
            continue;
        }
        if c == '\'' {
            // A char literal, or a lifetime. `'\n'` and `'"'` are literals; `'static` is not.
            if b.get(i + 1) == Some(&'\\') {
                let mut j = i + 2;
                while j < b.len() && b[j] != '\'' {
                    j += 1;
                }
                i = j + 1;
            } else if b.get(i + 2) == Some(&'\'') {
                i += 3;
            } else {
                i += 1;
            }
            continue;
        }
        if (c == 'r' || c == 'b') && (i == 0 || !is_ident_char(b[i - 1])) {
            let mut j = i + 1;
            if c == 'b' && b.get(j) == Some(&'r') {
                j += 1;
            }
            if c == 'r' || (c == 'b' && j > i + 1) {
                let mut hashes = 0usize;
                while b.get(j) == Some(&'#') {
                    hashes += 1;
                    j += 1;
                }
                if b.get(j) == Some(&'"') {
                    let mut k = j + 1;
                    loop {
                        if k >= b.len() {
                            break;
                        }
                        if b[k] == '\n' {
                            line += 1;
                        }
                        if b[k] == '"' {
                            let mut h = 0usize;
                            let mut m = k + 1;
                            while h < hashes && b.get(m) == Some(&'#') {
                                h += 1;
                                m += 1;
                            }
                            if h == hashes {
                                k = m;
                                break;
                            }
                        }
                        k += 1;
                    }
                    i = k;
                    continue;
                }
            }
        }
        if c == '"' {
            let start = i;
            let start_line = line;
            let mut s = String::new();
            let mut j = i + 1;
            let mut esc = false;
            while j < b.len() {
                let ch = b[j];
                if esc {
                    match ch {
                        'n' => s.push('\n'),
                        't' => s.push('\t'),
                        'r' => s.push('\r'),
                        '0' => s.push('\0'),
                        '\n' => {
                            // A `\`-continued line: the newline and the indent that follows it are
                            // not part of the value.
                            line += 1;
                            j += 1;
                            while j < b.len() && (b[j] == ' ' || b[j] == '\t') {
                                j += 1;
                            }
                            esc = false;
                            continue;
                        }
                        other => s.push(other),
                    }
                    esc = false;
                    j += 1;
                    continue;
                }
                if ch == '\\' {
                    esc = true;
                    j += 1;
                    continue;
                }
                if ch == '"' {
                    break;
                }
                if ch == '\n' {
                    line += 1;
                }
                s.push(ch);
                j += 1;
            }
            out.push(Lit {
                text: s,
                line: start_line,
                callee: callee_before(&b, start),
            });
            i = j + 1;
            continue;
        }
        i += 1;
    }
    out
}

/// Every bare identifier passed to an emitter in place of a literal — the computed-name helpers.
fn computed_emitter_args(src: &str) -> BTreeSet<String> {
    let b: Vec<char> = src.chars().collect();
    let mut out = BTreeSet::new();
    let mut i = 0usize;
    while i < b.len() {
        let rest: String = b[i..(i + 24).min(b.len())].iter().collect();
        let Some(name) = EMITTERS.iter().find(|e| {
            rest.starts_with(**e)
                && b.get(i + e.len()) == Some(&'(')
                && (i == 0 || !is_ident_char(b[i - 1]))
        }) else {
            i += 1;
            continue;
        };
        let mut j = i + name.len() + 1;
        while j < b.len() && b[j].is_whitespace() {
            j += 1;
        }
        let start = j;
        while j < b.len() && is_ident_char(b[j]) {
            j += 1;
        }
        // A bare identifier immediately applied: `census_count(ebucket(n))`.
        if j > start && b.get(j) == Some(&'(') {
            out.insert(b[start..j].iter().collect::<String>());
        }
        i += name.len();
    }
    out
}

/// The literals inside `fn <name>` in `src` — the arms of a computed-name helper.
fn fn_body_literals(src: &str, name: &str) -> Vec<String> {
    let needle = format!("fn {name}");
    let mut out = Vec::new();
    let mut from = 0usize;
    while let Some(rel) = src[from..].find(&needle) {
        let at = from + rel;
        from = at + needle.len();
        let after = src[at + needle.len()..].chars().next();
        if after.is_some_and(is_ident_char) {
            continue;
        }
        let Some(open_rel) = src[at..].find('{') else {
            continue;
        };
        let open = at + open_rel;
        let b: Vec<char> = src[open..].chars().collect();
        let mut depth = 0usize;
        let mut end = b.len();
        let mut in_str = false;
        let mut esc = false;
        for (k, ch) in b.iter().enumerate() {
            if in_str {
                if esc {
                    esc = false;
                } else if *ch == '\\' {
                    esc = true;
                } else if *ch == '"' {
                    in_str = false;
                }
                continue;
            }
            match ch {
                '"' => in_str = true,
                '{' => depth += 1,
                '}' => {
                    depth -= 1;
                    if depth == 0 {
                        end = k + 1;
                        break;
                    }
                }
                _ => {}
            }
        }
        let body: String = b[..end].iter().collect();
        out.extend(literals(&body).into_iter().map(|l| l.text));
    }
    out
}

/// The row-reader closures a subject file defines: `let of = |name: &str| -> u64 { … }`.
fn reader_names(src: &str) -> BTreeSet<String> {
    let mut out = BTreeSet::new();
    for line in src.lines() {
        if !line.contains(": &str|") {
            continue;
        }
        let t = line.trim_start();
        let Some(rest) = t.strip_prefix("let ") else {
            continue;
        };
        let Some(eq) = rest.find(" = ") else {
            continue;
        };
        let name = rest[..eq].trim().trim_start_matches("mut ").trim();
        if !name.is_empty() && name.chars().all(is_ident_char) {
            out.insert(name.to_string());
        }
    }
    out
}

// ── the corpus ──────────────────────────────────────────────────────────────────────────────────

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn collect(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for e in entries.flatten() {
        let p = e.path();
        if p.is_dir() {
            collect(&p, out);
        } else if p.extension().and_then(|x| x.to_str()) == Some("rs") {
            out.push(p);
        }
    }
}

fn rs_under(rel_dir: &str) -> Vec<PathBuf> {
    let mut v = Vec::new();
    collect(&root().join(rel_dir), &mut v);
    v.sort();
    v
}

fn rel(p: &Path) -> String {
    p.strip_prefix(root()).unwrap_or(p).display().to_string()
}

/// The names the engine can put into the census, and how many came from each half.
struct Emitted {
    names: BTreeSet<String>,
    from_literals: usize,
    from_computed: usize,
    computed_helpers: BTreeSet<String>,
}

fn emitted() -> Emitted {
    let skip = root().join(NOT_EMIT);
    let mut names = BTreeSet::new();
    let mut computed_helpers = BTreeSet::new();
    let mut from_computed_names = BTreeSet::new();
    for p in rs_under(EMIT_ROOT) {
        if p.starts_with(&skip) {
            continue;
        }
        let Ok(src) = std::fs::read_to_string(&p) else {
            continue;
        };
        for lit in literals(&src) {
            if lit
                .callee
                .as_deref()
                .is_some_and(|c| EMITTERS.contains(&c))
            {
                names.insert(lit.text);
            }
        }
        for helper in computed_emitter_args(&src) {
            for n in fn_body_literals(&src, &helper) {
                from_computed_names.insert(n);
            }
            computed_helpers.insert(helper);
        }
    }
    let from_literals = names.len();
    names.extend(from_computed_names.iter().cloned());
    Emitted {
        names,
        from_literals,
        from_computed: from_computed_names.len(),
        computed_helpers,
    }
}

/// One census name read by a cost test.
#[derive(Debug, Clone)]
struct Read {
    name: String,
    file: String,
    line: usize,
    /// The literal's own source line preceded by the one above it, so a co-located rune can be
    /// seen. Both are accepted because a name held in a `const [&str; N]` has no room for a
    /// trailing comment that rustfmt will leave alone — the rune goes on the line above instead.
    raw: String,
}

fn reads() -> Vec<Read> {
    let mut out = Vec::new();
    for p in rs_under(SUBJECT) {
        let Ok(src) = std::fs::read_to_string(&p) else {
            continue;
        };
        let readers = reader_names(&src);
        let lines: Vec<&str> = src.lines().collect();
        for lit in literals(&src) {
            let is_glyph = lit.text.chars().any(|c| GLYPHS.contains(&c));
            let is_read_arg = lit.callee.as_deref().is_some_and(|c| readers.contains(c));
            if !is_glyph && !is_read_arg {
                continue;
            }
            out.push(Read {
                name: lit.text,
                file: rel(&p),
                line: lit.line,
                raw: format!(
                    "{}\n{}",
                    lit.line
                        .checked_sub(2)
                        .and_then(|i| lines.get(i))
                        .unwrap_or(&""),
                    lines.get(lit.line - 1).unwrap_or(&"")
                ),
            });
        }
    }
    out
}

/// The reason a rune gives, if either line carries one. The reason ends at ITS OWN line's end — a
/// rune on the line above cannot borrow the code beneath it as its justification.
fn rune_reason(raw: &str) -> Option<String> {
    let at = raw.find(RUNE)?;
    let after = &raw[at + RUNE.len()..];
    let tail = after.split('\n').next().unwrap_or("").trim();
    let tail = tail
        .strip_prefix('—')
        .or_else(|| tail.strip_prefix("--"))
        .or_else(|| tail.strip_prefix('-'))
        .unwrap_or(tail);
    Some(tail.trim().to_string())
}

// ── the gate ────────────────────────────────────────────────────────────────────────────────────

#[test]
fn every_census_name_a_cost_test_reads_is_emitted() {
    let em = emitted();
    let rd = reads();

    // NON-VACUITY, both sides. A moved root, a renamed emitter or a scanner that lost its place
    // mid-file would leave one of these sets empty, and an empty read set makes this gate assert
    // nothing over nothing while reporting PASS.
    assert!(
        em.from_literals > 50,
        "only {} literal census names found under {EMIT_ROOT}/ — the emitter scan is not reading \
         the engine it claims to guard",
        em.from_literals
    );
    assert!(
        rd.len() > 40,
        "only {} census-name reads found under {SUBJECT}/ — the reader scan found nothing to check",
        rd.len()
    );
    // NON-VACUITY, positive control: names that must be on both sides. A recogniser that had
    // stopped recognising would still satisfy the counts above.
    for anchor in ["  ├ setup:seen", "  │  setup:seen:alloc", "alpha", "production"] {
        assert!(
            em.names.contains(anchor),
            "the emitted universe is missing `{anchor}`, which `fire/delta.rs` demonstrably emits \
             — the emitter scan has gone blind"
        );
    }
    assert!(
        rd.iter().any(|r| r.name == "  ├ setup:seen"),
        "no cost test appears to read `  ├ setup:seen` — the reader scan has gone blind"
    );

    let mut unresolved: Vec<&Read> = Vec::new();
    for r in &rd {
        if em.names.contains(&r.name) {
            continue;
        }
        if let Some(reason) = rune_reason(&r.raw) {
            assert!(
                reason.chars().count() >= MIN_REASON_CHARS,
                "{}:{} — `{}` carries a {RUNE} whose reason is {} chars (`{reason}`), under \
                 {MIN_REASON_CHARS}. A rune is a declaration: name the mark, what retired it, and \
                 why the reader is still sound without it.",
                r.file,
                r.line,
                r.name,
                reason.chars().count()
            );
            continue;
        }
        unresolved.push(r);
    }

    if !unresolved.is_empty() {
        let mut msg = format!(
            "\n{} census name(s) are READ by a cost test and EMITTED by nothing under {EMIT_ROOT}/.\n\
             Each one reaches its table through `unwrap_or(0)`, so it prints `0.00 ms` as though it \
             had been measured — and any row derived from it is arithmetic on a value that was \
             never taken.\n\n",
            unresolved.len()
        );
        for r in &unresolved {
            msg.push_str(&format!("  {}:{}  {:?}\n", r.file, r.line, r.name));
        }
        msg.push_str(
            "\nFIX — one of exactly two, and they are not interchangeable:\n\
             \n\
             (a) The row is FALSE. The name describes work that is not inside that phase (or any \
             phase). Delete the read and every row derived from it, and restate the table to what \
             the marks actually cover. Do NOT add a `phase_end` to make the name resolve — if the \
             work is already inside a timed phase, a new mark double-counts it.\n\
             \n\
             (b) The mark was DELIBERATELY RETIRED and the reader is still sound without it — it \
             branches on whether the mark FIRED (the pair count), never on its nanoseconds. Then \
             put a co-located rune on the line:\n\
             \n\
             \x20   // rune:lint(census-name-retired) — <which mark, what retired it, why the \
             reader is still correct>\n",
        );
        panic!("{msg}");
    }
}

#[test]
fn every_census_name_retired_rune_names_a_name_the_engine_no_longer_emits() {
    let em = emitted();
    let rd = reads();
    // NON-VACUITY: this gate is meant to reach zero (a clean tree has no stale rune), so a count
    // cannot guard it. The guard is on the population it filters instead — if `reads()` went
    // blind, the sibling gate above REDs first on its own positive control.
    assert!(
        !rd.is_empty(),
        "the reader scan found nothing — this gate would pass over an empty set"
    );
    let stale: Vec<String> = rd
        .iter()
        .filter(|r| rune_reason(&r.raw).is_some() && em.names.contains(&r.name))
        .map(|r| format!("  {}:{}  {:?}", r.file, r.line, r.name))
        .collect();
    assert!(
        stale.is_empty(),
        "\n{} {RUNE} rune(s) sit on a name the engine DOES emit:\n{}\n\nThe mark came back. Drop \
         the rune — an exemption that outlives its reason is how a gate stops gating.\n",
        stale.len(),
        stale.join("\n")
    );
}

#[test]
fn the_computed_name_half_reaches_the_bucket_helpers() {
    let em = emitted();
    // NON-VACUITY: if this half found nothing, the gate would false-RED the day a cost test read
    // an `elem-card:*` / `tok-card:*` name — a name the engine really does emit, just not as a
    // literal inside the emitter's parentheses.
    assert!(
        em.from_computed >= 16,
        "the computed-name half harvested only {} name(s) from helpers {:?} — `delta.rs`'s \
         `ebucket`/`tbucket` supply 8 each. The `census_count(<helper>(n))` shape has moved and \
         this half is now blind.",
        em.from_computed,
        em.computed_helpers
    );
    for anchor in ["elem-card:0", "elem-card:8+", "tok-card:0", "tok-card:8+"] {
        assert!(
            em.names.contains(anchor),
            "the computed-name half is missing `{anchor}` — helpers seen: {:?}",
            em.computed_helpers
        );
    }
}

#[cfg(test)]
mod extractor {
    //! The scanner is the gate. A blind extractor over a present file is the one failure the
    //! walk-count assertions cannot see, so each hazard this corpus actually contains is driven
    //! against a fixed sample.

    use super::*;

    #[test]
    fn a_name_in_a_comment_is_not_a_read() {
        let src = "// of(\"  ├ ghost\")\nlet of = |n: &str| -> u64 { rows.iter(); 0 };\n";
        assert!(literals(src).iter().all(|l| l.text != "  ├ ghost"));
    }

    #[test]
    fn a_char_literal_holding_a_quote_does_not_open_a_string() {
        let src = "let q = '\"';\nlet real = \"  ├ setup:seen\";\n";
        let got: Vec<String> = literals(src).into_iter().map(|l| l.text).collect();
        assert_eq!(got, vec!["  ├ setup:seen".to_string()], "got {got:?}");
    }

    #[test]
    fn a_lifetime_is_not_a_char_literal() {
        let src = "fn f(x: &'static str) -> &'static str { \"  ├ setup:seen\" }\n";
        let got: Vec<String> = literals(src).into_iter().map(|l| l.text).collect();
        assert_eq!(got, vec!["  ├ setup:seen".to_string()], "got {got:?}");
    }

    #[test]
    fn a_raw_string_full_of_quotes_does_not_mispair_the_rest_of_the_file() {
        let src = "let w = r#\"(:a \"x\" \"y\" \"z\")\"#;\nlet real = \"  ├ setup:seen\";\n";
        let got: Vec<String> = literals(src).into_iter().map(|l| l.text).collect();
        assert_eq!(got, vec!["  ├ setup:seen".to_string()], "got {got:?}");
    }

    #[test]
    fn a_block_comment_is_skipped_and_lines_still_count() {
        let src = "/* \"  ├ ghost\"\n   more */\nlet real = \"  ├ setup:seen\";\n";
        let lits = literals(src);
        assert_eq!(lits.len(), 1, "got {lits:?}");
        assert_eq!(lits[0].text, "  ├ setup:seen");
        assert_eq!(lits[0].line, 3, "line drifted: {lits:?}");
    }

    #[test]
    fn the_callee_of_an_argument_literal_is_recorded() {
        let src = "x = of(\"  ├ setup:seen\");\n";
        assert_eq!(literals(src)[0].callee.as_deref(), Some("of"));
    }

    #[test]
    fn a_reader_closure_is_discovered_under_any_name() {
        let src = "    let ns_of = |name: &str| -> u64 { 0 };\n    let get = |n: &str| -> u64 { 0 };\n";
        let got: Vec<String> = reader_names(src).into_iter().collect();
        assert_eq!(got, vec!["get".to_string(), "ns_of".to_string()], "got {got:?}");
    }

    #[test]
    fn a_computed_emitter_argument_is_found_and_its_arms_harvested() {
        let src = "fn ebucket(n: usize) -> &'static str { match n { 0 => \"e:0\", _ => \"e:8+\" } }\n\
                   census_count(ebucket(k));\n";
        let helpers: Vec<String> = computed_emitter_args(src).into_iter().collect();
        assert_eq!(helpers, vec!["ebucket".to_string()], "got {helpers:?}");
        let arms = fn_body_literals(src, "ebucket");
        assert_eq!(arms, vec!["e:0".to_string(), "e:8+".to_string()], "got {arms:?}");
    }

    #[test]
    fn a_literal_emitter_argument_is_not_mistaken_for_a_computed_one() {
        let src = "phase_end(\"  ├ setup:seen\", t);\n";
        assert!(computed_emitter_args(src).is_empty());
    }

    #[test]
    fn a_rune_reason_is_read_after_the_dash() {
        let raw = "        \"  ├ x\", // rune:lint(census-name-retired) — because the mark retired";
        assert_eq!(
            rune_reason(raw).as_deref(),
            Some("because the mark retired")
        );
        assert_eq!(rune_reason("        \"  ├ x\","), None);
    }

    #[test]
    fn a_rune_on_the_line_above_counts_and_stops_at_its_own_newline() {
        let raw = "        // rune:lint(census-name-retired) — the mark retired in abc1234\n        \"  ├ x\",";
        assert_eq!(
            rune_reason(raw).as_deref(),
            Some("the mark retired in abc1234"),
            "a rune above the literal must be seen, and must not swallow the line below it"
        );
    }
}
