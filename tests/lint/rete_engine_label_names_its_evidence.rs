//! AN ENGINE CLAIM CARRIES ITS EVIDENCE — a benchmark row that says `engine` must name either the
//! production function it CALLS or the gate that PINS its inline replication.
//!
//! ## Why this gate exists
//!
//! `b7d9d8e90` is titled *"the benchmark called the wrong arm 'the engine' for eleven days."* A
//! cost-table row that says `engine` and times something else is not a slow measurement, it is a
//! WRONG one wearing the production label — and it is the worst possible host for a stale claim,
//! because nobody re-derives a table's row names and the number beside it is right, which makes the
//! row look checked.
//!
//! The cure is the rung the rune-vocabulary strike taught: **spelling beats a checker that
//! guesses.** Rather than parse each table's format rows, match them positionally to the `ms(x)`
//! arguments, resolve `x` to its assigning block and scan that block — real parser work, fragile at
//! every step, and wrong in a way no one would notice — the LABEL CARRIES ITS OWN EVIDENCE and this
//! gate only has to resolve a name.
//!
//! ## The grammar
//!
//! An engine claim is a parenthesised label inside a string literal under [`SUBJECT`], reading
//! `engine` (case-insensitively, with an optional leading article, so the second spelling `THE
//! ENGINE` cannot hide). It must take one of two evidenced forms:
//!
//! - **It calls production** — `engine: <qualified::path>`. The path resolves to a definition under
//!   [`SUBJECT`] that is NOT test code, and the file carrying the label must actually call it.
//! - **It replicates production, and a gate pins the replication** — `engine: gated by <test fn>`.
//!   The named test must exist under [`GATE_DIR`] and carry a `#[test]` attribute. This form is not
//!   free — it writes a test's name into `src/`, which can retire that name as a control elsewhere
//!   in the tree. [`GATED`] carries the warning and the incident that produced it.
//!
//! A BARE `engine` with no evidence is refused. Without that, the old spelling survives silently and
//! this gate would police only the sites someone had already converted.
//!
//! ## ★ WHY THE SECOND FORM EXISTS — the rule that was proposed here was WRONG
//!
//! The rule originally drawn read: *an arm may carry `engine` only if its body CALLS the production
//! function.* Driven against the tree, that rule STRIPS A TRUE CLAIM. `accum_alpha_cost.rs`'s `L`
//! arm replicates the alpha class lookup inline — `lin.iter().find(..)` — because production there
//! is not a callable at all, it is the body of `root_for`. Its claim is true, and it is already
//! pinned by a gate that asserts `AlphaRoots`' type and `root_for`'s body line EXACTLY — evidence
//! strictly STRONGER than a name resolving, which no resolver can produce. A rule that deletes that
//! label would trade a stronger proof for a weaker one.
//!
//! So there are three arm shapes, not two: calls production, replicates production under a gate,
//! and neither. Only the third loses its claim.
//!
//! ## ⚠ THE RESOLUTION TARGET MUST BE PRODUCTION — AND A PATH PREFIX CANNOT SAY SO
//!
//! A gate that accepts any function of the named name is satisfied by a TEST HELPER — the label
//! vouching for itself with a fixture. The obvious exclusion, "not under `kernel/tests/`", is
//! INSUFFICIENT and the tree proves it: `compiled_rhs.rs` carries an engine label inside a
//! `#[cfg(test)]` module in a `src/` file, and 26 files under [`SUBJECT`] have such a module. A
//! decoy planted in any of them is outside `kernel/tests/` and would satisfy a path-prefix rule.
//!
//! So exclusion is by BRACE TRACKING — [`classify`] plus the scope walk in [`scan`] — over both
//! shapes of test code: an inline `#[cfg(test)]` module, and a whole file reached through a
//! `#[cfg(test)]` module declaration (which is how `kernel/tests/` is reached).
//!
//! Note the distinction the prefix rule blurred: **the LABEL may live in test code** — four of the
//! five do, and that is legitimate, since a cost table is a test. It is only the RESOLUTION TARGET
//! that must be production.
//!
//! Nor does the qualified path subsume this: a decoy `matcher::seen_insert` planted in
//! `matcher.rs`'s own test module spells a path that looks identical. Both halves are needed.
//!
//! ## What this gate does NOT check, stated rather than implied
//!
//! - **That the named function is the RIGHT one.** It checks that the name resolves to production
//!   and that the labelled file calls it. An arm that calls the wrong production function is a
//!   defect this cannot see; the `gated by` form is where that stronger claim is available.
//! - **Positional association.** It does not verify the label sits on the row whose timing closure
//!   makes the call — only that the file does. Inferring that association is exactly the fragile
//!   parser this design rejected.
//! - **Labels other than `engine`.** `(authority)`, `(today)` and `(query tax)` also live in these
//!   tables and make claims of their own. They are a separate population with a separate rule, and
//!   widening this gate to reach them is what would make its one rule unstatable.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

/// The tree whose cost tables carry engine claims, and the only tree a claim may resolve into.
const SUBJECT: &str = "src/rete";

/// Where a `gated by` claim's evidence must live. Narrow on purpose: this arc's guarantees live in
/// this directory, and a claim pointing anywhere else is not pointing at a gate.
const GATE_DIR: &str = "tests/lint";

/// The claimed word. Matched case-insensitively so the `THE ENGINE` spelling cannot hide.
const ENGINE: &str = "engine";

/// The optional article the second spelling carries.
const ARTICLE: &str = "the ";

/// Introduces the second evidenced form.
///
/// ⛔ **WRITING THIS FORM SPENDS SOMETHING — read before you add one.** A `gated by` claim puts a
/// TEST FUNCTION'S NAME into a string literal under `src/`. Anything that scrapes `src/` for
/// identifiers will then find that name, and it stops being a name attested only under `tests/`.
///
/// That is not hypothetical and it is not local. `tests/lint/rete_citation_resolves.rs` proves its
/// resolver's universe reaches outside `src/` by naming test-only functions; on 2026-09-01 the
/// first `gated by` claim written in this tree — `accum_alpha_cost.rs`'s `L` row — consumed one of
/// its two controls that way, and a search of the whole tree found NO replacement of the same kind.
///
/// So before naming a test here, check it is not doing another job elsewhere. The citation gate is
/// now floored by an uncitable control (`tests/lint/universe_control_name.rs`) so it can never
/// reach zero, but that floor is a backstop, not a licence: any OTHER gate that depends on a name
/// being test-only is still burnable, and nothing will warn you but this paragraph.
const GATED: &str = "gated by ";

/// Path segments that name a position relative to the caller rather than a module to resolve.
const RELATIVE: &[&str] = &["crate", "super", "self"];

/// The site that must come back FOUND and RESOLVED every run. A count cannot see a recogniser that
/// has stopped recognising: if the scanner, the resolver or the call check goes blind, this is what
/// says so, and every other green below it is unearned.
const CONTROL_FILE: &str = "accum_cost.rs";

/// The control's claim, resolved through the qualifier that distinguishes it from the forwarder of
/// the same name in `kernel/session.rs` — which is the ambiguity the qualified path exists to fix.
const CONTROL_TARGET: &str = "compiled_cond::intern_val";

/// Every engine claim in the tree, measured 2026-09-01 by driving this gate: 4 — one per arm that
/// still makes the claim, after `gather_probe_cost.rs`'s `S` row dropped its false one. An EXACT
/// floor, not a slack one, and deliberately so: the population is four, so volume can never
/// validate this gate, and a claim that quietly DISAPPEARS is the same defect as one that quietly
/// rots. Deleting a benchmark should cost a human one re-derivation here.
///
/// The first driven value of this constant was 5, taken from the tree BEFORE the false claim was
/// dropped, and this gate red on itself for it. A floor carried over from a pre-state is a
/// measurement of the wrong tree.
const LABEL_FLOOR: usize = 4;

/// Files under [`SUBJECT`], measured 2026-09-01: 57. The floor catches a walk gone blind — a moved
/// root, a renamed directory — without rotting as the tree grows.
const FILE_FLOOR: usize = 40;

/// What a character belongs to. The distinction is load-bearing twice over: labels are found ONLY
/// in string literals (so prose discussing a label is exempt, as this repo's comment rule requires),
/// and braces are counted ONLY in code (so a format string's `{:>7.2}` cannot move the depth).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Kind {
    Code,
    Str,
    Comment,
}

/// Is the character at `i` preceded by an identifier character? Used to tell a raw-string prefix
/// from the tail of a name that merely ends in the same letter.
fn ident_before(chars: &[char], i: usize) -> bool {
    i > 0 && (chars[i - 1].is_alphanumeric() || chars[i - 1] == '_')
}

/// Are there `n` consecutive hashes at `start`?
fn hashes_at(chars: &[char], start: usize, n: usize) -> bool {
    (0..n).all(|k| chars.get(start + k) == Some(&'#'))
}

/// Is the quote at `i` opening a character literal rather than a lifetime? A lifetime has no
/// closing quote, and consuming one as a literal would swallow real code.
fn is_char_literal(chars: &[char], i: usize) -> bool {
    match chars.get(i + 1) {
        Some('\\') => (i + 2..(i + 8).min(chars.len())).any(|k| chars[k] == '\''),
        Some(_) => chars.get(i + 2) == Some(&'\''),
        None => false,
    }
}

/// Index of the closing quote of the character literal opening at `i`.
fn char_literal_end(chars: &[char], i: usize) -> usize {
    if chars.get(i + 1) == Some(&'\\') {
        (i + 2..chars.len())
            .find(|&k| chars[k] == '\'')
            .unwrap_or(i + 1)
    } else {
        i + 2
    }
}

/// Classify every character as code, string content, or comment.
///
/// Handles line and block comments (nested), normal strings with escapes and line continuations,
/// raw strings of any hash count, and character literals. All four shapes occur under [`SUBJECT`] —
/// `purity.rs` holds a `'"'` literal that would open a phantom string for a naive scanner, and
/// three files hold raw strings.
fn classify(chars: &[char]) -> Vec<Kind> {
    let mut kind = vec![Kind::Code; chars.len()];
    let mut i = 0;
    while i < chars.len() {
        let c = chars[i];
        let next = chars.get(i + 1).copied();
        if c == '/' && next == Some('/') {
            while i < chars.len() && chars[i] != '\n' {
                kind[i] = Kind::Comment;
                i += 1;
            }
            continue;
        }
        if c == '/' && next == Some('*') {
            let mut depth = 0usize;
            while i < chars.len() {
                if chars[i] == '/' && chars.get(i + 1) == Some(&'*') {
                    depth += 1;
                    kind[i] = Kind::Comment;
                    kind[i + 1] = Kind::Comment;
                    i += 2;
                    continue;
                }
                if chars[i] == '*' && chars.get(i + 1) == Some(&'/') {
                    depth -= 1;
                    kind[i] = Kind::Comment;
                    kind[i + 1] = Kind::Comment;
                    i += 2;
                    if depth == 0 {
                        break;
                    }
                    continue;
                }
                kind[i] = Kind::Comment;
                i += 1;
            }
            continue;
        }
        if c == 'r' && !ident_before(chars, i) {
            let mut j = i + 1;
            let mut n = 0usize;
            while chars.get(j) == Some(&'#') {
                n += 1;
                j += 1;
            }
            if chars.get(j) == Some(&'"') {
                for slot in kind.iter_mut().take(j + 1).skip(i) {
                    *slot = Kind::Str;
                }
                let mut k = j + 1;
                while k < chars.len() {
                    if chars[k] == '"' && hashes_at(chars, k + 1, n) {
                        for slot in kind.iter_mut().take((k + n + 1).min(chars.len())).skip(k) {
                            *slot = Kind::Str;
                        }
                        k += n + 1;
                        break;
                    }
                    kind[k] = Kind::Str;
                    k += 1;
                }
                i = k;
                continue;
            }
        }
        if c == '"' {
            kind[i] = Kind::Str;
            let mut k = i + 1;
            while k < chars.len() {
                if chars[k] == '\\' {
                    kind[k] = Kind::Str;
                    if k + 1 < chars.len() {
                        kind[k + 1] = Kind::Str;
                    }
                    k += 2;
                    continue;
                }
                kind[k] = Kind::Str;
                k += 1;
                if chars[k - 1] == '"' {
                    break;
                }
            }
            i = k;
            continue;
        }
        if c == '\'' && is_char_literal(chars, i) {
            let end = char_literal_end(chars, i).min(chars.len() - 1);
            for slot in kind.iter_mut().take(end + 1).skip(i) {
                *slot = Kind::Str;
            }
            i = end + 1;
            continue;
        }
        i += 1;
    }
    kind
}

/// One evidenced (or unevidenced) engine claim.
#[derive(Debug, PartialEq, Eq)]
enum Claim {
    /// `engine`, with nothing behind it.
    Bare,
    /// `engine: <qualified::path>` — the production function this arm calls.
    Calls(String),
    /// `engine: gated by <test fn>` — the gate that pins this arm's inline replication.
    Gated(String),
}

/// Case-insensitive prefix strip, returning the ORIGINAL-CASE remainder. Targets carry capitals
/// (`Bindings`), so folding the whole label would destroy the name it names.
fn strip_prefix_ci<'a>(s: &'a str, prefix: &str) -> Option<&'a str> {
    let folded = s.to_ascii_lowercase();
    folded.starts_with(prefix).then(|| &s[prefix.len()..])
}

/// Read a parenthesised label's content as an engine claim, or `None` if it makes no engine claim.
///
/// The trailing check is what keeps a word that merely BEGINS the same — an engineering note, say —
/// out of the population: the claim is the whole word, followed by end-of-label or its evidence.
fn engine_claim(content: &str) -> Option<Claim> {
    let t = content.trim();
    let body = strip_prefix_ci(t, ARTICLE).unwrap_or(t);
    let after = strip_prefix_ci(body, ENGINE)?.trim();
    if after.is_empty() {
        return Some(Claim::Bare);
    }
    let tail = after.strip_prefix(':')?.trim();
    if tail.is_empty() {
        return Some(Claim::Bare);
    }
    match strip_prefix_ci(tail, GATED) {
        Some(g) => Some(Claim::Gated(g.trim().to_string())),
        None => Some(Claim::Calls(tail.to_string())),
    }
}

/// A label found in the tree.
#[derive(Debug)]
struct Label {
    line: usize,
    claim: Claim,
}

/// A function definition, with the block names enclosing it and whether it is test-only.
#[derive(Debug)]
struct Def {
    name: String,
    scopes: Vec<String>,
    in_test: bool,
}

/// A path-qualified or bare call site — never a method call reached through a dot, which would let
/// any `.get(` in the tree vouch for a claim naming a trait method called `get`.
#[derive(Debug)]
struct Call {
    segments: Vec<String>,
}

/// Everything one file yields.
#[derive(Debug)]
struct FileScan {
    labels: Vec<Label>,
    defs: Vec<Def>,
    calls: Vec<Call>,
}

/// Is this trimmed code line a `#[cfg(test)]` attribute?
///
/// Matched piecewise rather than as one literal: `no_inlined_edn` flags any string literal opening
/// with `#`, and its rubric says an EDN look-alike is not a rune candidate — restructure instead.
fn is_cfg_test_attr(t: &str) -> bool {
    t.starts_with('#') && t.ends_with(']') && t.contains("cfg(test)")
}

/// Block names this line opens: a trait, an inherent or trait impl, a module, a struct, an enum.
///
/// An `impl A for B` contributes BOTH names, because either is a truthful qualifier for a method
/// defined inside it.
fn scope_names(t: &str) -> Vec<String> {
    let tok: Vec<&str> = t.split_whitespace().collect();
    let clean = |s: &str| -> String {
        s.chars()
            .take_while(|c| c.is_alphanumeric() || *c == '_')
            .collect()
    };
    let mut out = Vec::new();
    for (i, w) in tok.iter().enumerate() {
        match *w {
            "trait" | "mod" | "struct" | "enum" | "union" => {
                if let Some(n) = tok.get(i + 1) {
                    out.push(clean(n));
                }
            }
            "impl" => {
                if let Some(n) = tok.get(i + 1) {
                    out.push(clean(n));
                }
                if let Some(j) = tok.iter().position(|x| *x == "for") {
                    if let Some(n) = tok.get(j + 1) {
                        out.push(clean(n));
                    }
                }
            }
            _ => {}
        }
    }
    out.retain(|s| !s.is_empty());
    out
}

/// The function name this line defines, if it defines one.
fn fn_name(t: &str) -> Option<String> {
    let tok: Vec<&str> = t.split_whitespace().collect();
    let i = tok.iter().position(|w| *w == "fn")?;
    let n: String = tok
        .get(i + 1)?
        .chars()
        .take_while(|c| c.is_alphanumeric() || *c == '_')
        .collect();
    (!n.is_empty()).then_some(n)
}

/// Scan one file: its engine labels, its function definitions with test-status, and its call sites.
fn scan(src: &str) -> FileScan {
    let chars: Vec<char> = src.chars().collect();
    let kind = classify(&chars);

    let mut line_of = vec![1usize; chars.len()];
    let mut line = 1usize;
    for (i, c) in chars.iter().enumerate() {
        line_of[i] = line;
        if *c == '\n' {
            line += 1;
        }
    }

    // Labels live in string literals; a `(` in code is a tuple or a call, never a label.
    let mut labels = Vec::new();
    for (i, c) in chars.iter().enumerate() {
        if kind[i] != Kind::Str || *c != '(' {
            continue;
        }
        let mut j = i + 1;
        let mut content = String::new();
        while j < chars.len() && kind[j] == Kind::Str && chars[j] != ')' {
            content.push(chars[j]);
            j += 1;
        }
        if chars.get(j) != Some(&')') || kind[j] != Kind::Str {
            continue;
        }
        if let Some(claim) = engine_claim(&content) {
            labels.push(Label {
                line: line_of[i],
                claim,
            });
        }
    }

    // Calls: walk back from every `(` in CODE over an identifier chain. A chain reached through a
    // dot is a method call on a value and is deliberately not recorded.
    let mut calls = Vec::new();
    for (i, c) in chars.iter().enumerate() {
        if kind[i] != Kind::Code || *c != '(' {
            continue;
        }
        let mut k = i;
        while k > 0 {
            let p = chars[k - 1];
            if p.is_alphanumeric() || p == '_' {
                k -= 1;
            } else if p == ':' && k >= 2 && chars[k - 2] == ':' {
                k -= 2;
            } else {
                break;
            }
        }
        if k == i || (k > 0 && chars[k - 1] == '.') {
            continue;
        }
        let text: String = chars[k..i].iter().collect();
        let segments: Vec<String> = text
            .split("::")
            .filter(|s| !s.is_empty() && !RELATIVE.contains(s))
            .map(str::to_string)
            .collect();
        if !segments.is_empty() {
            calls.push(Call { segments });
        }
    }

    // Definitions, with brace-tracked test regions and enclosing block names.
    let mut defs = Vec::new();
    let mut depth = 0usize;
    let mut pending_cfg_test = false;
    let mut test_depths: Vec<usize> = Vec::new();
    let mut scopes: Vec<(usize, String)> = Vec::new();
    let mut start = 0usize;
    while start <= chars.len() {
        let end = chars[start..]
            .iter()
            .position(|c| *c == '\n')
            .map(|p| start + p)
            .unwrap_or(chars.len());
        let code: String = (start..end)
            .filter(|&i| kind[i] == Kind::Code)
            .map(|i| chars[i])
            .collect();
        let t = code.trim();

        if is_cfg_test_attr(t) {
            pending_cfg_test = true;
        } else {
            if let Some(name) = fn_name(t) {
                defs.push(Def {
                    name,
                    scopes: scopes.iter().map(|(_, n)| n.clone()).collect(),
                    in_test: !test_depths.is_empty(),
                });
            }
            let mut opening = scope_names(t);
            for c in code.chars() {
                match c {
                    '{' => {
                        if pending_cfg_test {
                            test_depths.push(depth);
                            pending_cfg_test = false;
                        }
                        for n in opening.drain(..) {
                            scopes.push((depth, n));
                        }
                        depth += 1;
                    }
                    '}' => {
                        depth = depth.saturating_sub(1);
                        while scopes.last().is_some_and(|(d, _)| *d == depth) {
                            scopes.pop();
                        }
                        if test_depths.last() == Some(&depth) {
                            test_depths.pop();
                        }
                    }
                    // A statement end closes an attribute that opened no block — which is exactly
                    // how `#[cfg(test)] mod tests;` reaches a whole directory without a brace.
                    ';' => pending_cfg_test = false,
                    _ => {}
                }
            }
        }

        if end >= chars.len() {
            break;
        }
        start = end + 1;
    }

    FileScan {
        labels,
        defs,
        calls,
    }
}

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).to_path_buf()
}

fn collect_rs(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    let mut paths: Vec<PathBuf> = entries.flatten().map(|e| e.path()).collect();
    paths.sort();
    for p in paths {
        if p.is_dir() {
            collect_rs(&p, out);
        } else if p.extension().and_then(|x| x.to_str()) == Some("rs") {
            out.push(p);
        }
    }
}

/// Is this file reached only through a `#[cfg(test)]` module declaration?
///
/// This is the SECOND shape of test code, and the one a path prefix was invented to catch by hand:
/// `kernel/mod.rs` declares `mod tests;` under the attribute, so every file beneath that directory
/// is test-only without any of them saying so. Checked at every level, because the marked
/// declaration may sit anywhere up the chain.
fn is_test_module_file(path: &Path) -> bool {
    let mut cur = path.to_path_buf();
    loop {
        let Some(parent) = cur.parent().map(Path::to_path_buf) else {
            return false;
        };
        let name = match cur.file_stem().and_then(|s| s.to_str()) {
            Some("mod") => parent.file_name().and_then(|s| s.to_str()),
            other => other,
        };
        let decl_dir = if cur.file_stem().and_then(|s| s.to_str()) == Some("mod") {
            parent.parent().map(Path::to_path_buf)
        } else {
            Some(parent.clone())
        };
        if let (Some(name), Some(dir)) = (name, decl_dir.clone()) {
            if let Ok(src) = std::fs::read_to_string(dir.join("mod.rs")) {
                let lines: Vec<&str> = src.lines().collect();
                let wanted = format!("mod {name};");
                for (i, l) in lines.iter().enumerate() {
                    if !l.trim().ends_with(&wanted) {
                        continue;
                    }
                    let back = i.saturating_sub(2);
                    if lines[back..i].iter().any(|p| is_cfg_test_attr(p.trim())) {
                        return true;
                    }
                }
            }
        }
        match decl_dir {
            Some(d) if d.ends_with("src") => return false,
            Some(d) => cur = d.join("mod.rs"),
            None => return false,
        }
    }
}

/// Module path components of a file relative to `src/`, which is how a qualifier like
/// `compiled_cond` or `delta` is satisfied.
fn module_path(path: &Path, src_root: &Path) -> Vec<String> {
    let Ok(rel) = path.strip_prefix(src_root) else {
        return Vec::new();
    };
    rel.components()
        .filter_map(|c| c.as_os_str().to_str())
        .map(|s| s.trim_end_matches(".rs").to_string())
        .filter(|s| s != "mod")
        .collect()
}

/// A production definition the resolver may name.
#[derive(Debug)]
struct ProdDef {
    name: String,
    quals: BTreeSet<String>,
}

/// Every `#[test]` function under [`GATE_DIR`].
fn gate_tests() -> BTreeSet<String> {
    let mut out = BTreeSet::new();
    let mut files = Vec::new();
    collect_rs(&root().join(GATE_DIR), &mut files);
    for f in &files {
        let Ok(src) = std::fs::read_to_string(f) else {
            continue;
        };
        let lines: Vec<&str> = src.lines().collect();
        for (i, l) in lines.iter().enumerate() {
            let Some(name) = fn_name(l.trim()) else {
                continue;
            };
            let back = i.saturating_sub(3);
            if lines[back..i]
                .iter()
                .any(|p| p.trim().starts_with('#') && p.trim().contains("test]"))
            {
                out.insert(name);
            }
        }
    }
    out
}

/// Does the file carrying the label call this target? A recorded call's segments must be a SUFFIX
/// of the target's, so `super::seen_insert` satisfies `delta::seen_insert` and `Bindings::get`
/// satisfies `matcher::Bindings::get`.
fn file_calls(calls: &[Call], target: &[String]) -> bool {
    calls.iter().any(|c| {
        c.segments.len() <= target.len() && target[target.len() - c.segments.len()..] == c.segments
    })
}

#[test]
fn every_engine_label_names_its_evidence() {
    let src_root = root().join("src");
    let mut files = Vec::new();
    collect_rs(&root().join(SUBJECT), &mut files);

    // NON-VACUITY, first half: a walk that comes back empty asserts nothing over nothing and
    // reports PASS. The floor sits under the 57 files this walk finds today (driven 2026-09-01), so
    // it catches a moved or renamed root without rotting as the tree grows.
    assert!(
        files.len() >= FILE_FLOOR,
        "the engine-label walk found only {} .rs file(s) under {SUBJECT} — it is not reaching the \
         tree it claims to guard, so its green means nothing",
        files.len()
    );

    let scans: Vec<(PathBuf, FileScan, bool)> = files
        .iter()
        .filter_map(|p| {
            let src = std::fs::read_to_string(p).ok()?;
            let test_file = is_test_module_file(p);
            Some((p.clone(), scan(&src), test_file))
        })
        .collect();

    let mut prod: Vec<ProdDef> = Vec::new();
    for (path, s, test_file) in &scans {
        if *test_file {
            continue;
        }
        let module = module_path(path, &src_root);
        for d in &s.defs {
            if d.in_test {
                continue;
            }
            let mut quals: BTreeSet<String> = module.iter().cloned().collect();
            quals.extend(d.scopes.iter().cloned());
            prod.push(ProdDef {
                name: d.name.clone(),
                quals,
            });
        }
    }

    let gates = gate_tests();
    let mut problems: Vec<String> = Vec::new();
    let mut found = 0usize;
    let mut control_resolved = false;

    for (path, s, _) in &scans {
        let rel = path
            .strip_prefix(root())
            .unwrap_or(path)
            .display()
            .to_string();
        let base = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or_default();
        for label in &s.labels {
            found += 1;
            let at = format!("{rel}:{}", label.line);
            match &label.claim {
                Claim::Bare => problems.push(format!(
                    "{at} — an engine claim with NO EVIDENCE. Name the production function this \
                     arm calls, or the gate that pins its inline replication"
                )),
                Claim::Gated(g) => {
                    if !gates.contains(g) {
                        problems.push(format!(
                            "{at} — claims it is gated by `{g}`, which is not a test under \
                             {GATE_DIR}. The gate is the whole evidence for this form"
                        ));
                    }
                }
                Claim::Calls(target) => {
                    let segs: Vec<String> = target
                        .split("::")
                        .filter(|x| !x.is_empty() && !RELATIVE.contains(x))
                        .map(str::to_string)
                        .collect();
                    let Some((name, quals)) = segs.split_last() else {
                        problems.push(format!("{at} — names an empty path"));
                        continue;
                    };
                    let resolved = prod
                        .iter()
                        .any(|d| &d.name == name && quals.iter().all(|q| d.quals.contains(q)));
                    if !resolved {
                        problems.push(format!(
                            "{at} — names `{target}`, which resolves to NO production definition \
                             under {SUBJECT}. A definition inside a `#[cfg(test)]` module or a \
                             test-only module does not count: that is the label vouching for \
                             itself with a fixture"
                        ));
                        continue;
                    }
                    if !file_calls(&s.calls, &segs) {
                        problems.push(format!(
                            "{at} — names `{target}`, which resolves, but this file never calls \
                             it. The label claims this arm times production; it does not"
                        ));
                        continue;
                    }
                    if base == CONTROL_FILE && target == CONTROL_TARGET {
                        control_resolved = true;
                    }
                }
            }
        }
    }

    // NON-VACUITY, second half. The count comes first: five claims live in this tree, and a
    // scanner that stopped finding them would otherwise report a serene green over nothing.
    assert!(
        found >= LABEL_FLOOR,
        "the scanner found {found} engine claim(s) under {SUBJECT}, expected at least \
         {LABEL_FLOOR}. Either a benchmark was deleted — re-derive the floor and say so — or the \
         label scanner has gone blind, which would make every green here vacuous"
    );
    // And the positive control, because a count cannot see a recogniser that has stopped
    // recognising: the exemplar must come back FOUND, RESOLVED past the ambiguity its qualifier
    // exists to settle, and CALLED by the file that claims it.
    assert!(
        control_resolved,
        "the positive control did not resolve: {CONTROL_FILE} no longer carries a claim naming \
         `{CONTROL_TARGET}` that this gate can resolve and confirm is called. Either the label \
         moved — update the control — or the scanner, the resolver or the call check has gone \
         blind, and every green below is unearned"
    );

    assert!(
        problems.is_empty(),
        "\n\n🔥 {} engine claim(s) do not carry their evidence. A cost-table row that says engine \
         and times something else is not a slow measurement, it is a WRONG one wearing the \
         production label — and nobody re-derives a table's row names.\n\
         \n\
         THE FIX, one of three:\n\
         \n\
         1. The arm CALLS production — spell the qualified path: `engine: compiled_cond::intern_val`.\n\
         2. The arm REPLICATES production inline and a gate pins the replication — spell \
         `engine: gated by <that test's name>`.\n\
         3. The arm does NEITHER — it has no engine claim to make. DROP the label. Do not invent a \
         production name for it: that would make the row pass this gate while staying exactly as \
         false as it was, and this gate would then be certifying the defect.\n\
         \n\
         Unevidenced:\n\n{}\n",
        problems.len(),
        problems.join("\n")
    );
}

#[cfg(test)]
mod detector {
    use super::*;

    fn kinds(src: &str) -> Vec<Kind> {
        classify(&src.chars().collect::<Vec<char>>())
    }

    #[test]
    fn a_label_in_a_string_is_read_and_one_in_a_comment_is_not() {
        let src = "// V FxHashMap (engine)\nlet t = \"V FxHashMap (engine: a::b)\";\n";
        let s = scan(src);
        assert_eq!(s.labels.len(), 1);
        assert_eq!(s.labels[0].claim, Claim::Calls(String::from("a::b")));
    }

    #[test]
    fn the_second_spelling_cannot_hide() {
        assert_eq!(engine_claim("THE ENGINE"), Some(Claim::Bare));
        assert_eq!(engine_claim("engine"), Some(Claim::Bare));
    }

    #[test]
    fn a_word_that_merely_begins_the_same_is_not_a_claim() {
        assert_eq!(engine_claim("engineering"), None);
        assert_eq!(engine_claim("engine room"), None);
    }

    #[test]
    fn the_gated_form_carries_the_test_name() {
        assert_eq!(
            engine_claim("engine: gated by some_gate_fn"),
            Some(Claim::Gated(String::from("some_gate_fn")))
        );
    }

    #[test]
    fn a_definition_in_a_cfg_test_module_is_test_code() {
        let src = "fn real() {}\n#[cfg(test)]\nmod t {\n    fn decoy() {}\n}\nfn other() {}\n";
        let s = scan(src);
        let by = |n: &str| s.defs.iter().find(|d| d.name == n).map(|d| d.in_test);
        assert_eq!(by("real"), Some(false));
        assert_eq!(by("decoy"), Some(true));
        assert_eq!(by("other"), Some(false));
    }

    #[test]
    fn a_module_declaration_does_not_leak_the_attribute_onto_the_next_block() {
        let src = "fn first() {}\n#[cfg(test)]\nmod tests;\nfn production() {}\n";
        let s = scan(src);
        assert_eq!(s.defs.iter().find(|d| d.name == "production").map(|d| d.in_test), Some(false));
    }

    #[test]
    fn a_trait_encloses_its_methods_as_a_qualifier() {
        let src = "pub(crate) trait Bindings {\n    fn get(&self, k: &Value) -> Option<&Value>;\n}\n";
        let s = scan(src);
        let d = s.defs.iter().find(|d| d.name == "get").expect("get");
        assert_eq!(d.scopes, vec![String::from("Bindings")]);
    }

    #[test]
    fn a_format_brace_cannot_move_the_depth() {
        let src = "fn f() {\n    println!(\"a {:>7.2} b }} {{\");\n}\nfn g() {}\n";
        let s = scan(src);
        assert_eq!(s.defs.iter().find(|d| d.name == "g").map(|d| d.in_test), Some(false));
    }

    #[test]
    fn a_char_literal_holding_a_quote_does_not_open_a_string() {
        let k = kinds("if c == '\"' { }\nlet s = \"x\";\n");
        assert_eq!(k[k.len() - 3], Kind::Str);
        assert_eq!(k[k.len() - 2], Kind::Code);
    }

    #[test]
    fn a_lifetime_is_not_a_char_literal() {
        let src = "fn f<'a>(x: &'a str) {}\nfn g() {}\n";
        let s = scan(src);
        assert!(s.defs.iter().any(|d| d.name == "g"));
    }

    #[test]
    fn a_method_call_through_a_dot_is_not_recorded() {
        let s = scan("fn f() {\n    let a = pmap.get(k);\n    let b = Bindings::get(p, k);\n}\n");
        let flat: Vec<Vec<String>> = s.calls.iter().map(|c| c.segments.clone()).collect();
        assert!(flat.contains(&vec![String::from("Bindings"), String::from("get")]));
        assert!(!flat.contains(&vec![String::from("get")]));
    }

    #[test]
    fn a_relative_prefix_is_stripped_so_a_suffix_match_still_holds() {
        let s = scan("fn f() {\n    super::seen_insert(a, b, c);\n}\n");
        let target = vec![String::from("delta"), String::from("seen_insert")];
        assert!(file_calls(&s.calls, &target));
    }
}
