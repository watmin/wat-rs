//! **A NAME CITED IN A RETE COMMENT MUST RESOLVE — OR SAY, IN PLACE, WHY IT CANNOT.**
//!
//! Two kinds of citation live in `src/rete/`'s comments and nothing checked either one:
//!
//! 1. **A backticked identifier.** `` `head_is_boolean_rete_predicate` `` guards a silent
//!    `_ => None` on the fix-list F path. If it names nothing, a reader cannot find the thing the
//!    comment is vouching for, and the guarantee the sentence asserts is unfalsifiable.
//! 2. **A bare `*.rs` / `*.wat` filename.** `src/rete/kernel/mod.rs:4` said *"Tests are
//!    `tests.rs`."* Driven 2026-09-01: `src/rete/kernel/tests.rs` does not exist — it became
//!    `kernel/tests/` in the 2026-08-30 `partire` split, so that sentence was stale the day the
//!    split landed.
//!
//! ── WHY THIS SITS BESIDE `no_stale_path_in_doc.rs` RATHER THAN INSIDE IT ─────────────────────
//!
//! That gate cannot see kind 2 **by construction**: `no_stale_path_in_doc.rs:47` requires
//! `tok.contains('/')`, so a bare filename is invisible to it. Extending it was rejected for two
//! reasons, both structural rather than stylistic:
//!
//! * Its resolver is **ancestor-relative** (repo root, `root/src`, then every ancestor of the
//!   naming file) — right for `fire/rules.rs`, wrong for a bare name. Measured 2026-09-01: applied
//!   to the 244 bare filenames cited under `src/rete`, that rule reports **55** stale, of which
//!   **31 exist** — `arm.rs` cited from `compiled_cond.rs` is `kernel/arm.rs`, a *sibling
//!   subdirectory* the upward walk can never reach. Widening its resolver to serve bare names
//!   would change verdicts it already gives on slashed paths.
//! * Kind 1 needs a code-versus-prose universe that gate has no concept of, and one walker serving
//!   both kinds needs to strip comments exactly once.
//!
//! So: a new gate, and `no_stale_path_in_doc`'s population is untouched.
//!
//! ── ⛔ THE UNIVERSE IS THE WHOLE DESIGN, AND A NAIVE ONE REPORTS ZERO ────────────────────────
//!
//! A scan whose resolution universe is raw text reports **0 unresolved of 760** — because a name
//! appearing only in prose **resolves against itself**. Comments are therefore stripped from every
//! resolving corpus, string-literal-aware, and [`prose_cannot_vouch_for_prose`] holds a live
//! control over that: `NoMatchingArm` is a name `src/rete/vocabulary.rs` states does not exist,
//! and it must stay unattested.
//!
//! Too NARROW a universe is the mirror failure and it manufactures findings. Measured: six names
//! cited in rete comments are attested **only outside `src/`** — `spec_equals_native_on_every_where_family`
//! is a `tests/rete/` fn, `alpha_class_lookup_is_still_the_linear_scan_the_benchmark_calls_the_engine`
//! is a `tests/lint/` fn. A `src`-only universe would demand those correct citations be reworded.
//!
//! **The universe, stated so it can be argued with — a name resolves if ANY of these holds:**
//!
//! | half | corpus | why it is not subsumed by the others |
//! |---|---|---|
//! | Rust code | idents in CODE positions under `src/ crates/ tests/ benches/ examples/` | the bulk |
//! | wat code | idents in `wat/` (the frozen stdlib is code) | `SiftRulesResponse` lives only there |
//! | file stem | the stem of any `.rs`/`.wat` in the corpus | this tree cites its gates and probe worlds by bare module name — `span_substitution_justified`, `probe_arc278_then_user_forms` |
//!
//! Each half is load-bearing and each has its own control below, because a half that decides
//! nothing is a half whose emptying would change no verdict — the failure the `wat-scripts` name
//! gate found in its own first draft.
//!
//! ── WHAT COUNTS AS A CITATION — the population boundary, and the three families outside it ───
//!
//! A backticked token is a citation iff it is a bare Rust identifier **and** carries a shape only
//! code has: an `_`, or an interior capital. A single lowercase word in backticks is not
//! distinguishable from emphasis — this tree backticks `partire`, `solvere` and `probare` (session
//! names), `fetch`, `pack` and English words — and a gate that demanded those resolve would be
//! demanding prose be rewritten to satisfy a parser. **This is a boundary, not a blind spot:** a
//! one-word `fn` name cited in backticks is unchecked here, and that is stated so it can be moved.
//!
//! Three families are outside by their SPELLING, and for each the fix at a site that spells one as
//! a bare identifier is to spell it in the convention this tree already uses elsewhere:
//!
//! * **A lint name** is `clippy::needless_borrow`, not `needless_borrow` — a `::` path, and
//!   `src/function/parse.rs:48` already writes it that way.
//! * **A name FRAGMENT** is `*_pass`, not `_pass` — and the very sentence that said `_pass` also
//!   says `*_delta` two lines above it.
//! * **A memory slug** is `[[feedback_…]]`, not backticks — the form used at 8 sites under
//!   `src/rete/` already.
//!
//! This is deliberately NOT three exemption lists. An excluded-by-list name goes on being exempt
//! after it stops being noise; an excluded-by-spelling name is one the author had to write
//! correctly, and the gate teaches the spelling in its failure text.
//!
//! ── THE EARNED EXEMPTION: a name whose ABSENCE is the point ──────────────────────────────────
//!
//! Some citations are correct precisely because they name nothing. `src/rete/vocabulary.rs` argues
//! that a justification was refuted because *"`NoMatchingArm` DOES NOT EXIST in this codebase"*;
//! `src/rete/kernel/fire/pass/hash_join.rs` records that there is *"no take, no `restore_parent`"*.
//! Deleting those names would delete the evidence. Each such site carries a per-name declaration:
//!
//! ```text
//! // rune:lint(cited-name-absent) <the name> — <why it is absent, and what the reader should know>
//! ```
//!
//! A DECLARATION, not a suppression: the rune must name the exact token, a reason under
//! [`MIN_REASON_CHARS`] is refused, and the shrug vocabulary is refused by name. A newly-rotted
//! citation with no rune is still a red.
//!
//! Shape and precedent: `tests/lint/rete_names_in_wat_scripts_resolve.rs` (the same code-vs-prose
//! split, two strikes back) and `tests/lint/every_walking_gate_declares_non_vacuity.rs`.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

/// The population this gate enforces. The arc's surface; a tree-wide sweep is a different strike.
const SUBJECT: &str = "src/rete";

/// Roots whose Rust CODE positions attest an identifier.
const CODE_ROOTS: &[&str] = &["src", "crates", "tests", "benches", "examples"];

/// ⛔ THIS FILE, EXCLUDED FROM ITS OWN UNIVERSE.
///
/// Not hypothetical — driven 2026-09-01, on the first run of this gate. `tests/` is a code root,
/// this gate lives in it, and every name it writes — in a `const`, in a failure message, in a
/// control — is a code position. So `NoMatchingArm` and `SiftRulesResponse` both resolved AGAINST
/// THIS FILE, and [`prose_cannot_vouch_for_prose`] went red naming its own author. Without the
/// exclusion a future hand could silence any red by adding the offending name to the error text.
/// [`the_gate_does_not_attest_its_own_text`] keeps the exclusion honest.
const SELF: &str = "tests/lint/rete_citation_resolves.rs";

/// Roots whose `.wat` CODE positions attest an identifier. The frozen stdlib is code.
const WAT_ROOTS: &[&str] = &["wat"];

/// Roots whose source FILES lend their stems as resolvable names.
const FILE_ROOTS: &[&str] = &["src", "crates", "tests", "benches", "examples", "wat", "wat-scripts"];

/// Roots that lend BASENAMES but not stems.
///
/// `docs/arc/**` holds the recorded harness worlds a comment cites by filename —
/// `experiri-acc-head.wat` and its wrapped twin are the driven pair behind `reachability.rs:1945`,
/// and they are real files a reader opens. They lend no STEMS: a doc filename is not an identifier,
/// and letting `docs/` attest identifiers would be prose vouching for prose one directory over.
const DOC_ROOTS: &[&str] = &["docs"];

/// The per-name exemption marker, written in a Rust comment beside the citation.
const RUNE: &str = "rune:lint(cited-name-absent)";

/// Shortest reason that can name a mechanism, matching the two sibling gates.
const MIN_REASON_CHARS: usize = 40;

/// The separator between a rune's token and its reason.
const EM_DASH: char = '\u{2014}';

/// Reasons that describe the absence of an answer rather than an answer.
const REFUSED_REASONS: &[&str] = &[
    "n/a",
    "not applicable",
    "none",
    "nothing",
    "no reason",
    "see above",
    "obvious",
    "by inspection",
    "not needed",
    "unnecessary",
];

/// Blank every comment run opened by `opener` and closed by a newline, leaving code and string
/// literals intact and every byte offset unchanged.
///
/// Both languages this serves open a line comment with two characters and close it at the newline,
/// and both quote with `"` and escape with `\`, so one scanner serves both.
fn mask(src: &str, opener: &str) -> String {
    let chars: Vec<char> = src.chars().collect();
    let open: Vec<char> = opener.chars().collect();
    let mut out: Vec<char> = Vec::with_capacity(chars.len());
    let mut i = 0;
    let mut in_str = false;
    let mut esc = false;
    while i < chars.len() {
        let c = chars[i];
        if in_str {
            out.push(c);
            if esc {
                esc = false;
            } else if c == '\\' {
                esc = true;
            } else if c == '"' {
                in_str = false;
            }
            i += 1;
            continue;
        }
        if c == '"' {
            in_str = true;
            out.push(c);
            i += 1;
            continue;
        }
        if c == open[0] && chars.get(i + 1) == Some(&open[1]) {
            while i < chars.len() && chars[i] != '\n' {
                out.push(' ');
                i += 1;
            }
            continue;
        }
        out.push(c);
        i += 1;
    }
    out.into_iter().collect()
}

/// Rust source with every `//` comment blanked; offsets and lines preserved.
fn rs_code(src: &str) -> String {
    mask(src, "//")
}

/// The inverse of [`rs_code`]: everything that is NOT Rust code. This is the SUBJECT text.
fn rs_prose(src: &str) -> String {
    let code = rs_code(src);
    src.chars()
        .zip(code.chars())
        .map(|(o, c)| {
            // Newlines survive on both sides so a rune cannot borrow its reason from the prose
            // that happens to follow it on the next line.
            if o == '\n' {
                '\n'
            } else if c == ' ' && o != ' ' {
                o
            } else {
                ' '
            }
        })
        .collect()
}

/// wat source with every `;;` comment blanked.
fn wat_code(src: &str) -> String {
    mask(src, ";;")
}

/// Every identifier-shaped run in a stretch of code text.
fn idents(text: &str) -> BTreeSet<String> {
    let mut out = BTreeSet::new();
    let chars: Vec<char> = text.chars().collect();
    let mut i = 0;
    while i < chars.len() {
        if chars[i].is_ascii_alphabetic() || chars[i] == '_' {
            let start = i;
            while i < chars.len() && (chars[i].is_ascii_alphanumeric() || chars[i] == '_') {
                i += 1;
            }
            out.insert(chars[start..i].iter().collect::<String>());
        } else {
            i += 1;
        }
    }
    out
}

/// Every backtick-delimited run in `text`, with the 1-based line it sits on.
///
/// Scanned per line: a backtick opened and never closed on its line is not a citation, and treating
/// it as one would swallow the rest of the file.
fn backticked(text: &str) -> Vec<(String, usize)> {
    let mut out = Vec::new();
    for (n, line) in text.lines().enumerate() {
        let chars: Vec<char> = line.chars().collect();
        let mut i = 0;
        while i < chars.len() {
            if chars[i] != '`' {
                i += 1;
                continue;
            }
            let start = i + 1;
            let mut j = start;
            while j < chars.len() && chars[j] != '`' {
                j += 1;
            }
            if j >= chars.len() {
                break;
            }
            out.push((chars[start..j].iter().collect::<String>(), n + 1));
            i = j + 1;
        }
    }
    out
}

/// Is this backticked run a CITATION of a code name?
///
/// A bare Rust identifier carrying a shape only code has — an `_`, or an interior capital. See the
/// module header for the three families this deliberately leaves outside, and why each is excluded
/// by its spelling rather than by a list.
fn is_citation(tok: &str) -> bool {
    let mut chars = tok.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    if !(first.is_ascii_alphabetic() || first == '_') {
        return false;
    }
    if !tok.chars().all(|c| c.is_ascii_alphanumeric() || c == '_') {
        return false;
    }
    tok.contains('_') || tok.chars().skip(1).any(|c| c.is_ascii_uppercase())
}

/// Is this token a bare source filename — the kind `no_stale_path_in_doc.rs` cannot see?
///
/// A token carrying a `/` belongs to that gate and is left to it.
fn is_bare_filename(tok: &str) -> bool {
    (tok.ends_with(".rs") || tok.ends_with(".wat"))
        && !tok.contains('/')
        && !tok.starts_with('.')
        && !tok.contains("..")
}

/// Every bare `*.rs` / `*.wat` filename in a stretch of prose, with its line.
fn bare_filenames(text: &str) -> Vec<(String, usize)> {
    let mut out = Vec::new();
    for (n, line) in text.lines().enumerate() {
        for tok in line.split(|c: char| !(c.is_alphanumeric() || "._/-".contains(c))) {
            if is_bare_filename(tok) {
                out.push((tok.to_string(), n + 1));
            }
        }
    }
    out
}

/// Was `tok` (an `X.rs`) invalidated by a module SPLIT the naming file sits inside?
///
/// The 2026-08-30 `partire` cuts turned `X.rs` into `X/mod.rs` three times. A file with the old
/// basename may well still exist SOMEWHERE — `tests.rs` survives under `src/macros/` — so plain
/// existence cannot see this class, and the class is the one that was actually driven. Returns the
/// directory that replaced the file.
fn shadowed_by_split(root: &Path, naming: &Path, tok: &str) -> Option<String> {
    let stem = tok.strip_suffix(".rs")?;
    let mut dir = naming.parent();
    while let Some(d) = dir {
        if d.join(stem).join("mod.rs").is_file() && !d.join(tok).is_file() {
            let split = d.join(stem);
            return Some(split.strip_prefix(root).unwrap_or(&split).display().to_string());
        }
        if d == root {
            break;
        }
        dir = d.parent();
    }
    None
}

/// The verdict on one rune line.
#[derive(Debug, PartialEq, Eq)]
enum Rune {
    /// The name is declared deliberately absent, with a reason that names a mechanism.
    Declared,
    /// A rune for this name exists but does not hold up.
    Hollow(String),
}

/// Read the runes in one file's prose, keyed by the exact token each names.
fn runes(prose: &str) -> BTreeMap<String, Rune> {
    let mut out = BTreeMap::new();
    for line in prose.lines() {
        let Some(at) = line.find(RUNE) else { continue };
        let tail = line[at + RUNE.len()..].trim();
        let mut parts = tail.splitn(2, char::is_whitespace);
        let Some(token) = parts.next().filter(|t| is_citation(t) || is_bare_filename(t)) else {
            continue;
        };
        let rest = parts.next().unwrap_or("").trim();
        let Some(reason) = rest.strip_prefix(EM_DASH).or_else(|| rest.strip_prefix('-')) else {
            out.insert(
                token.to_string(),
                Rune::Hollow(format!(
                    "carries no reason (expected `{RUNE} <name> \u{2014} <reason>`, found `{rest}`)"
                )),
            );
            continue;
        };
        let reason = reason.trim();
        let folded = reason.to_ascii_lowercase();
        if let Some(shrug) = REFUSED_REASONS.iter().find(|r| {
            folded == **r
                || folded.trim_end_matches('.') == **r
                || folded
                    .strip_prefix(*r)
                    .is_some_and(|t| t.starts_with(' ') || t.starts_with(',') || t.starts_with(';'))
        }) {
            out.insert(
                token.to_string(),
                Rune::Hollow(format!(
                    "the reason is `{reason}` \u{2014} `{shrug}` names no mechanism. Say why this \
                     name is absent and what a reader looking for it should know instead"
                )),
            );
            continue;
        }
        if reason.chars().count() < MIN_REASON_CHARS {
            out.insert(
                token.to_string(),
                Rune::Hollow(format!(
                    "the reason is {} chars (`{reason}`) \u{2014} under {MIN_REASON_CHARS}, too \
                     short to name a mechanism",
                    reason.chars().count()
                )),
            );
            continue;
        }
        out.insert(token.to_string(), Rune::Declared);
    }
    out
}

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn collect(dir: &Path, ext: &str, out: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for e in entries.flatten() {
        let p = e.path();
        if p.is_dir() {
            collect(&p, ext, out);
        } else if p.extension().and_then(|x| x.to_str()) == Some(ext) {
            out.push(p);
        }
    }
}

fn files_under(roots: &[&str], ext: &str) -> Vec<PathBuf> {
    let mut v = Vec::new();
    for r in roots {
        collect(&root().join(r), ext, &mut v);
    }
    v.sort();
    v
}

/// The three halves of the resolver, kept apart so each can be emptied and seen to matter.
struct Universe {
    /// Identifiers attested in a Rust CODE position.
    rust: BTreeSet<String>,
    /// Identifiers attested in a `.wat` CODE position under `wat/`.
    wat: BTreeSet<String>,
    /// Stems of source files: this tree cites its gates and probe worlds by bare module name.
    stems: BTreeSet<String>,
    /// Basenames of source files, for the bare-filename half.
    basenames: BTreeSet<String>,
}

impl Universe {
    fn read() -> Self {
        let mut rust = BTreeSet::new();
        let me = root().join(SELF);
        for p in files_under(CODE_ROOTS, "rs") {
            if p == me {
                continue;
            }
            if let Ok(src) = std::fs::read_to_string(&p) {
                rust.extend(idents(&rs_code(&src)));
            }
        }
        let mut wat = BTreeSet::new();
        for p in files_under(WAT_ROOTS, "wat") {
            if let Ok(src) = std::fs::read_to_string(&p) {
                wat.extend(idents(&wat_code(&src)));
            }
        }
        let mut stems = BTreeSet::new();
        let mut basenames = BTreeSet::new();
        for ext in ["rs", "wat"] {
            for p in files_under(FILE_ROOTS, ext) {
                if let Some(s) = p.file_stem().and_then(|s| s.to_str()) {
                    stems.insert(s.to_string());
                }
                if let Some(b) = p.file_name().and_then(|s| s.to_str()) {
                    basenames.insert(b.to_string());
                }
            }
            for p in files_under(DOC_ROOTS, ext) {
                if let Some(b) = p.file_name().and_then(|s| s.to_str()) {
                    basenames.insert(b.to_string());
                }
            }
        }
        Self { rust, wat, stems, basenames }
    }

    fn resolves(&self, tok: &str) -> bool {
        self.rust.contains(tok) || self.wat.contains(tok) || self.stems.contains(tok)
    }
}

/// The subject: every `.rs` file under [`SUBJECT`], with its prose.
fn subject() -> Vec<(PathBuf, String)> {
    files_under(&[SUBJECT], "rs")
        .into_iter()
        .filter_map(|p| {
            let src = std::fs::read_to_string(&p).ok()?;
            let prose = rs_prose(&src);
            Some((p, prose))
        })
        .collect()
}

fn rel(p: &Path) -> String {
    p.strip_prefix(root()).unwrap_or(p).display().to_string()
}

#[test]
fn every_backticked_name_in_a_rete_comment_resolves() {
    let universe = Universe::read();
    let files = subject();

    // NON-VACUITY, part 1 — the RESOLVER's three halves. Any one silently coming back empty would
    // leave this gate demanding that correct citations be reworded. Measured 2026-09-01: 22_425
    // Rust idents, 4_000+ wat idents, 2_400+ file stems.
    assert!(
        universe.rust.len() >= 15_000 && universe.wat.len() >= 1_000 && universe.stems.len() >= 1_500,
        "the resolver went blind: {} Rust ident(s), {} wat ident(s), {} file stem(s). None of \
         these can legitimately collapse, so every verdict below is unearned until it is fixed",
        universe.rust.len(),
        universe.wat.len(),
        universe.stems.len()
    );

    // NON-VACUITY, part 2 — the SUBJECT walk and the classifier over it. A count of files is not
    // enough: a prose extractor that blanked everything would find 57 files, zero citations, and
    // report a clean tree. Measured 2026-09-01: 57 files, 2_083 citations, 760 distinct.
    let cited: Vec<(String, usize, PathBuf)> = files
        .iter()
        .flat_map(|(p, prose)| {
            backticked(prose)
                .into_iter()
                .filter(|(t, _)| is_citation(t))
                .map(|(t, l)| (t, l, p.clone()))
                .collect::<Vec<_>>()
        })
        .collect();
    let distinct: BTreeSet<&String> = cited.iter().map(|(t, _, _)| t).collect();
    assert!(
        files.len() >= 40 && cited.len() >= 1_500 && distinct.len() >= 500,
        "the subject walk went blind: {} file(s) under {SUBJECT}, {} citation(s), {} distinct — \
         this gate is judging almost nothing",
        files.len(),
        cited.len(),
        distinct.len()
    );

    let mut unresolved: Vec<String> = Vec::new();
    let mut hollow: Vec<String> = Vec::new();

    for (path, prose) in &files {
        let declared = runes(prose);
        let here = rel(path);
        let mut seen: BTreeSet<String> = BTreeSet::new();
        for (token, line) in backticked(prose) {
            if !is_citation(&token) || universe.resolves(&token) || !seen.insert(token.clone()) {
                continue;
            }
            match declared.get(&token) {
                Some(Rune::Declared) => {}
                Some(Rune::Hollow(why)) => hollow.push(format!("  {here}:{line}  {token}\n      {why}")),
                None => unresolved.push(format!("  {here}:{line}  `{token}`")),
            }
        }
    }

    assert!(
        hollow.is_empty(),
        "\n\n{} deliberately-absent name(s) DECLARE a reason that does not hold up. A rune with a \
         hollow reason is worse than no rune: it reads, forever, as though someone answered.\n\n{}\n",
        hollow.len(),
        hollow.join("\n")
    );

    assert!(
        unresolved.is_empty(),
        "\n\n\u{1f525} {} name(s) cited in a comment under {SUBJECT} resolve to NOTHING — not a \
         Rust identifier in any code position under {CODE_ROOTS:?}, not a wat identifier under \
         {WAT_ROOTS:?}, and not the stem of any source file. A reader following one of these finds \
         nothing, and the claim the sentence makes cannot be checked.\n\
         \n\
         THE FIX, one of four:\n\
         \n\
         1. It MOVED or was RENAMED \u{2014} spell it as the identifier that exists today. Do not \
         guess a twin; grep a code position for it.\n\
         \n\
         2. The thing is GONE and there is no successor \u{2014} then reword the sentence to say \
         what is true now. A rename that makes the sentence false is worse than the rot.\n\
         \n\
         3. It is not a code name at all, and the spelling says so. A clippy lint is \
         `clippy::needless_borrow`; a name FRAGMENT is `*_pass`, not `_pass`; a memory slug is \
         `[[feedback_...]]`, not backticks. Each of those is already how this tree writes them \
         elsewhere, and each falls outside this gate by its spelling.\n\
         \n\
         4. Its ABSENCE is the point \u{2014} `NoMatchingArm` is cited to prove it does not exist. \
         Declare it, per name, in a comment beside the citation:\n\
         `// {RUNE} <the name> \u{2014} <why it is absent, and what a reader should know instead>`\n\
         A reason under {MIN_REASON_CHARS} chars is refused.\n\
         \n\
         \u{26d4} NOT A FIX: deleting the backticks. That hides the citation from this gate while \
         leaving the reader exactly as lost.\n\
         \n\
         Unresolved:\n\n{}\n",
        unresolved.len(),
        unresolved.join("\n")
    );
}

#[test]
fn every_bare_filename_in_a_rete_comment_names_a_file() {
    let universe = Universe::read();
    let files = subject();
    let r = root();

    // NON-VACUITY — the basename corpus and the walk. An empty basename set would flag all 244
    // citations; an empty walk would flag none. Measured 2026-09-01: 2_463 basenames, 244 bare
    // filename citations under src/rete.
    let all: Vec<(String, usize, PathBuf)> = files
        .iter()
        .flat_map(|(p, prose)| {
            bare_filenames(prose).into_iter().map(|(t, l)| (t, l, p.clone())).collect::<Vec<_>>()
        })
        .collect();
    assert!(
        universe.basenames.len() >= 1_500 && all.len() >= 150,
        "the bare-filename walk went blind: {} basename(s) in the corpus, {} citation(s) under \
         {SUBJECT}",
        universe.basenames.len(),
        all.len()
    );

    let mut stale: Vec<String> = Vec::new();
    let mut hollow: Vec<String> = Vec::new();
    for (path, prose) in &files {
        let declared = runes(prose);
        let here = rel(path);
        for (token, line) in bare_filenames(prose) {
            let why = if !universe.basenames.contains(&token) {
                format!("`{token}` \u{2014} no file with that name exists anywhere in the corpus")
            } else if let Some(split) = shadowed_by_split(&r, path, &token) {
                format!(
                    "`{token}` \u{2014} that module was SPLIT into `{split}/`; the file with this \
                     basename that still exists is somewhere else entirely"
                )
            } else {
                continue;
            };
            match declared.get(&token) {
                Some(Rune::Declared) => {}
                Some(Rune::Hollow(w)) => hollow.push(format!("  {here}:{line}  {token}\n      {w}")),
                None => stale.push(format!("  {here}:{line}  {why}")),
            }
        }
    }

    assert!(
        hollow.is_empty(),
        "\n\n{} filename declaration(s) carry a reason that does not hold up.\n\n{}\n",
        hollow.len(),
        hollow.join("\n")
    );
    assert!(
        stale.is_empty(),
        "\n\n\u{1f525} {} bare filename(s) cited in a comment under {SUBJECT} name no file a \
         reader can open. `no_stale_path_in_doc.rs` cannot see these: it requires a `/` in the \
         token, so a bare filename is invisible to it by construction.\n\
         \n\
         Two shapes, and the second is the one that rots silently:\n\
         \n\
         1. NO SUCH FILE ANYWHERE \u{2014} the file was deleted, renamed, or lives in another repo. \
         Name the file that exists, or reword.\n\
         \n\
         2. SPLIT INTO A DIRECTORY \u{2014} `X.rs` became `X/mod.rs`, so a citation of `X.rs` now \
         points at nothing even though some other `X.rs` may survive elsewhere in the tree. Write \
         `X/mod.rs`, or the specific file inside `X/` the sentence actually means.\n\
         \n\
         3. It names a file in ANOTHER REPO, on purpose \u{2014} declare it in place:\n\
         `// {RUNE} <the filename> \u{2014} <which repo it lives in, and why it is cited here>`\n\
         \n\
         Stale:\n\n{}\n",
        stale.len(),
        stale.join("\n")
    );
}

/// ★ THE CONTROL THE WHOLE DESIGN TURNS ON: prose must not vouch for prose.
///
/// If the Rust comment stripper stops blanking comments, every name that lives only in a comment
/// resolves against its own citation and this gate reports a clean tree forever. The control is not
/// hypothetical: `src/rete/vocabulary.rs` argues a point by asserting `NoMatchingArm` does not
/// exist, so that name is present in prose under the subject and must be absent from every
/// resolving half.
#[test]
fn prose_cannot_vouch_for_prose() {
    const ONLY_IN_PROSE: &str = "NoMatchingArm";
    let universe = Universe::read();
    assert!(
        universe.rust.len() >= 15_000,
        "the attestation scrape went blind ({} idents), so this control proves nothing",
        universe.rust.len()
    );
    let in_prose: BTreeSet<String> = subject()
        .iter()
        .flat_map(|(_, prose)| backticked(prose).into_iter().map(|(t, _)| t))
        .collect();
    assert!(
        in_prose.contains(ONLY_IN_PROSE),
        "`{ONLY_IN_PROSE}` no longer appears in any comment under {SUBJECT}, so this control is \
         asserting over nothing \u{2014} pick another name the tree cites as absent"
    );
    assert!(
        !universe.resolves(ONLY_IN_PROSE),
        "`{ONLY_IN_PROSE}` now RESOLVES. `src/rete/vocabulary.rs` states it exists nowhere in this \
         codebase, so either the comment stripper has stopped blanking comments \u{2014} in which \
         case every green verdict from this gate is unearned \u{2014} or the name has genuinely \
         been minted and that comment is now wrong"
    );
}

/// The universe must reach OUTSIDE `src/`, or every test-only citation becomes a false finding.
///
/// The two names below are attested only under `tests/`. Narrowing the universe back to `src/`
/// would make this gate demand that a correct sentence be reworded, and this is what says so. They
/// are deliberately of DIFFERENT KINDS, and neither replaces the other:
///
/// - `spec_equals_native_on_every_where_family` is a REAL test, cited in a rete comment at
///   `src/rete/compiled_cond.rs:884`. It carries the actual stake: a live sentence that would go
///   wrong. This is the control that means something.
/// - `zz_universe_control_never_cite_this` (`tests/lint/universe_control_name.rs`) is OWNED by the
///   lint suite and describes no behaviour, so nothing can ever cite it. It is the FLOOR — the
///   guarantee that this gate cannot be left with zero controls.
///
/// ⛔ THERE WAS A THIRD, AND A BENCHMARK LABEL ATE IT — 2026-09-01.
/// `alpha_class_lookup_is_still_the_linear_scan_the_benchmark_calls_the_engine` was struck here
/// because `accum_alpha_cost.rs`'s `L` row now carries an `engine: gated by <that test>` claim,
/// which writes the test's name into a string literal UNDER `src/`. The name is therefore attested
/// in `src/` and proves nothing about the test corpus any more.
///
/// That coupling is GENERAL: every `engine: gated by` claim retires its gate's name as a test-only
/// control anywhere in the tree. A replacement of the same kind was searched for and NONE EXISTS —
/// the cited name above is the only other test fn both cited in a rete comment and absent from
/// `src/`. Left alone, the usable population only ever falls, and the LAST one dies silently: this
/// gate would keep reporting green while proving nothing. The owned floor is what stops that
/// ratchet, and it is why a second, uncitable control exists at all.
#[test]
fn the_universe_reaches_the_test_corpus() {
    const IN_TESTS_ONLY: &[&str] = &[
        "spec_equals_native_on_every_where_family",
        "zz_universe_control_never_cite_this",
    ];
    let universe = Universe::read();
    let mut src_only = BTreeSet::new();
    for p in files_under(&["src"], "rs") {
        if let Ok(src) = std::fs::read_to_string(&p) {
            src_only.extend(idents(&rs_code(&src)));
        }
    }
    assert!(
        src_only.len() >= 8_000,
        "the src-only scrape went blind ({} idents); this control proves nothing",
        src_only.len()
    );
    for name in IN_TESTS_ONLY {
        assert!(
            universe.resolves(name),
            "`{name}` no longer resolves \u{2014} the test corpus dropped out of the universe, and \
             every citation of a test-only name is now a false finding"
        );
        assert!(
            !src_only.contains(*name),
            "`{name}` is now attested under `src/` too, so it no longer proves the test corpus is \
             load-bearing \u{2014} pick another name that lives only in `tests/`"
        );
    }
}

/// The wat half and the file-stem half each decide something ALONE.
///
/// A resolver half that resolves nothing the other halves do not is a half whose emptying would
/// change no verdict — and then the "blind the resolver" guard above passes green over a broken
/// gate. That is the exact defect `rete_names_in_wat_scripts_resolve.rs` found in its own first
/// draft, so each half here is pinned to a name only it can answer.
#[test]
fn each_resolver_half_answers_a_name_no_other_half_can() {
    /// Cited at `src/rete/purity.rs:1644` against `wat/query.wat`; no Rust code names it.
    const WAT_ONLY: &str = "SiftRulesResponse";
    /// A `tests/lint/` gate cited by module name; its own test fn is `span_substitutions_are_justified`.
    const STEM_ONLY: &str = "span_substitution_justified";
    let u = Universe::read();
    assert!(
        u.wat.contains(WAT_ONLY) && !u.rust.contains(WAT_ONLY) && !u.stems.contains(WAT_ONLY),
        "`{WAT_ONLY}` no longer resolves through the wat half alone (wat: {}, rust: {}, stem: {}) \
         \u{2014} emptying `wat/` would now change no verdict, so pick another wat-only name",
        u.wat.contains(WAT_ONLY),
        u.rust.contains(WAT_ONLY),
        u.stems.contains(WAT_ONLY)
    );
    assert!(
        u.stems.contains(STEM_ONLY) && !u.rust.contains(STEM_ONLY) && !u.wat.contains(STEM_ONLY),
        "`{STEM_ONLY}` no longer resolves through the file-stem half alone (stem: {}, rust: {}, \
         wat: {}) \u{2014} emptying the stem half would now change no verdict, so pick another \
         module-name-only citation",
        u.stems.contains(STEM_ONLY),
        u.rust.contains(STEM_ONLY),
        u.wat.contains(STEM_ONLY)
    );
}

/// The exclusion of this file from its own universe is REAL.
///
/// The token below appears in this gate and nowhere else in the tree. If it ever resolves, [`SELF`]
/// has stopped being skipped and this gate can be silenced by writing a name into its own error
/// text — which is how its very first run reported a false green half.
#[test]
fn the_gate_does_not_attest_its_own_text() {
    const WRITTEN_ONLY_HERE: &str = "a_name_that_exists_only_inside_this_gate";
    let u = Universe::read();
    assert!(
        u.rust.len() >= 15_000,
        "the scrape went blind ({} idents); this control proves nothing",
        u.rust.len()
    );
    assert!(
        !u.resolves(WRITTEN_ONLY_HERE),
        "`{WRITTEN_ONLY_HERE}` resolves \u{2014} {SELF} is being scraped into its own universe \
         again, so any name this gate MENTIONS is a name this gate will accept"
    );
}

#[cfg(test)]
mod classifier {
    use super::*;

    const REAL: &str = "check_field_kw";

    #[test]
    fn a_rust_comment_is_prose_and_code_is_not() {
        let src = format!("let x = {REAL}(); // see `{REAL}`\n");
        assert_eq!(backticked(&rs_prose(&src)).len(), 1);
        assert!(backticked(&rs_code(&src)).is_empty());
    }

    #[test]
    fn a_name_in_a_string_literal_is_code_not_prose() {
        let src = format!("let s = \"`{REAL}`\";\n");
        assert!(backticked(&rs_prose(&src)).is_empty());
        assert!(idents(&rs_code(&src)).contains(REAL));
    }

    #[test]
    fn a_comment_marker_inside_a_string_does_not_open_a_comment() {
        let src = format!("let s = \"a // b\"; let y = {REAL};\n");
        assert!(idents(&rs_code(&src)).contains(REAL));
    }

    #[test]
    fn a_quote_inside_a_comment_does_not_open_a_string() {
        let src = format!("// a lone \" quote\nlet y = {REAL};\n");
        assert!(idents(&rs_code(&src)).contains(REAL));
    }

    #[test]
    fn a_snake_case_word_is_a_citation_and_a_plain_word_is_not() {
        assert!(is_citation("check_field_kw"));
        assert!(is_citation("DidNotDiscriminate"));
        assert!(!is_citation("partire"));
        assert!(!is_citation("fetch"));
    }

    #[test]
    fn the_three_excluded_families_are_excluded_by_their_spelling() {
        // A lint written canonically, a fragment written as a glob, and a memory slug.
        const SLUG: &str = "feedback_x_y";
        // Assembled rather than written: a literal opening with `[` reads to `no_inlined_edn` as
        // an EDN vector, and that lint's rubric says restructure the code, never rune it.
        let bracketed = ["[", "[", SLUG, "]", "]"].concat();
        assert!(is_citation("needless_borrow") && !is_citation("clippy::needless_borrow"));
        assert!(is_citation("_pass") && !is_citation("*_pass"));
        assert!(is_citation(SLUG) && !is_citation(&bracketed));
    }

    #[test]
    fn an_unclosed_backtick_does_not_swallow_the_line() {
        assert!(backticked("a ` b\n").is_empty());
    }

    #[test]
    fn a_line_number_is_reported() {
        let src = format!("\n\n// `{REAL}`\n");
        assert_eq!(backticked(&rs_prose(&src)), vec![(REAL.to_string(), 3)]);
    }

    #[test]
    fn a_bare_filename_is_taken_and_a_slashed_path_is_left_to_the_other_gate() {
        let names: Vec<String> =
            bare_filenames("// see arm.rs and src/rete/kernel/arm.rs\n").into_iter().map(|(t, _)| t).collect();
        assert_eq!(names, vec!["arm.rs".to_string()]);
    }

    #[test]
    fn a_rune_naming_a_mechanism_declares_the_name() {
        let src = format!(
            "// {RUNE} {REAL} \u{2014} cited to prove it does not exist; minting it would make the \
             sentence false.\n"
        );
        assert_eq!(runes(&src).get(REAL), Some(&Rune::Declared));
    }

    #[test]
    fn a_rune_reading_n_slash_a_is_refused() {
        let src = format!("// {RUNE} {REAL} \u{2014} N/A\n");
        match runes(&src).get(REAL) {
            Some(Rune::Hollow(_)) => {}
            other => panic!("`N/A` must be refused, got {other:?}"),
        }
    }

    #[test]
    fn a_rune_too_short_to_name_a_mechanism_is_refused() {
        let src = format!("// {RUNE} {REAL} \u{2014} it is fine\n");
        match runes(&src).get(REAL) {
            Some(Rune::Hollow(_)) => {}
            other => panic!("a reason under the floor must be refused, got {other:?}"),
        }
    }

    #[test]
    fn a_rune_with_no_reason_is_refused() {
        let src = format!("// {RUNE} {REAL}\n");
        match runes(&src).get(REAL) {
            Some(Rune::Hollow(_)) => {}
            other => panic!("a rune with no reason must be hollow, got {other:?}"),
        }
    }

    #[test]
    fn a_rune_declares_only_the_name_it_writes() {
        let src = format!(
            "// {RUNE} {REAL} \u{2014} cited to prove it does not exist; minting it would make the \
             sentence false.\n"
        );
        assert!(!runes(&src).contains_key("something_else"));
    }
}
