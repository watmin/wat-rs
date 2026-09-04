//! **A `:wat::rete::` NAME WRITTEN IN CODE UNDER `wat-scripts/` MUST RESOLVE. PROSE MAY NAME A
//! RETIRED FORM; CODE MAY NOT.**
//!
//! ── THE HOLE THIS CLOSES ─────────────────────────────────────────────────────────────────────
//!
//! `tests/lint/wat_scripts_fixes_load.rs` is the gate the scratch-pad convention rests on: it
//! parses and type-checks every `.wat` under `wat-scripts/` on the current runtime, so — the
//! claim went — rot cannot hide there. It proves PARSE plus FREEZE. It does not prove that the
//! NAMES in the file exist, and that is precisely the rot the convention was written to prevent.
//!
//! Driven at arc 278's `51b851c91` — an invented head, in a `def` body, in a file under
//! `wat-scripts/`:
//!
//! ```text
//! (def :probe-nonsense (… (:wat::rete::core::THIS-HEAD-NEVER-EXISTED …)))
//! (defn :user::main [] -> nil (println "ran"))
//!       →  ran
//! ```
//!
//! It type-checks and the program RUNS. A `def` nothing forces is never resolved, so a file under
//! `wat-scripts/` may name anything at all. Two phantoms had been living on that licence: a rename
//! table inside `wat-scripts/fixes/rete-where-per-type-spelling.wat` — the codemod `CLAUDE.md`
//! mandates for every `.wat` migration — pointed two of its rows at
//! `:wat::rete::core::map` / `:wat::rete::core::filter`, names that have never been `RETE_OPS`
//! rows, and `wat-scripts/scratch-pad/probe-arc278-57-round1b-parametric-and-hof.wat` used the
//! same two heads to "prove the new spellings resolve".
//!
//! ── WHAT THIS GATE DOES **NOT** DO — a cut, not an oversight ─────────────────────────────────
//!
//! It does not resolve every head in every `def`. That needs forcing (which may have effects) or a
//! full static resolve pass, and it is a language-level problem, not a lint. `:wat::rete::` names
//! are a closed, enumerable set and they are the family that actually ROTS, because retirement is
//! a routine event in the rete vocabulary. Textual resolution of exactly that family is
//! proportionate; the general problem is out of scope and stays out.
//!
//! ── THE THREE SOURCES OF THE RESOLVER, AND WHY NO TWO OF THEM ARE A UNION ────────────────────
//!
//! 1. **The registry.** `:wat::rete::core::…` and `:wat::rete::holon::…` are the operator surface,
//!    and `RETE_OPS` in `src/rete/vocabulary.rs` is their sole authority. A name in that namespace
//!    resolves iff it is a `rete_name` row — or one of [`KNOWN_FORMS`], the short list of names
//!    that live in the operator namespace without being operators.
//! 2. **Attestation.** Everything else — `:wat::rete::where`, `:wat::rete::defrule`,
//!    `:wat::rete::Session`, `:wat::rete::acc::sum`, the session verbs, the outcome enums — are
//!    ordinary substrate symbols with no single table behind them. A name there resolves iff it is
//!    attested in a CODE position elsewhere in the tree: the Rust substrate under `src/`, or the
//!    frozen stdlib under `wat/`. That set is enumerated from the tree on every run, never from a
//!    hand-written list that would rot the moment a verb is added.
//!
//! 3. **Declaration.** A field accessor `:wat::rete::<Type>/<field>` is text NOWHERE.
//!    `register_aggregate_methods` (`src/runtime.rs`) mints one function per declared field at
//!    `format!("{}/{}", agg.name, field_name)` at freeze, from the `defrecord`; the macro's own
//!    accessor emission was removed in arc 293.R2.2, so nothing else registers those paths. The
//!    third source is therefore the DECLARATION, parsed out of `wat/`. ⛔ **The field half is not
//!    optional**: a rule admitting `<Type>/<anything>` cannot refuse a misspelling, and a resolver
//!    that cannot refuse a misspelling has stopped resolving — it would trade one false RED for a
//!    permanent blind spot over every field of every rete record, which is the worse trade because
//!    nobody notices a gate that never fires.
//!
//! ⛔ **THE SPLIT IS LOAD-BEARING, AND A UNION WOULD BE A GATE THAT VOUCHED FOR ITSELF.** Measured
//! 2026-09-01: every one of the 79 `RETE_OPS` row names is ALSO attested in a code position outside
//! the registry file, so under a flat `rows ∪ attested` universe the registry half resolves exactly
//! ZERO names by itself — emptying it would change no verdict and the "blind the resolver" mutation
//! would pass green. Splitting by namespace makes each half the only authority for its own family:
//! emptying the rows leaves 71 distinct names unresolved, and emptying the attestation leaves 63.
//! Both halves can now fail loudly, which is the only reason either is worth having.
//!
//! ⛔ **AND THE THIRD IS NOT A SUBSET OF THE SECOND** — the same test, applied to the source added
//! last. Measured 2026-09-03: 25 `:wat::rete::` records under `wat/` mint 80 accessors, and 30 of
//! those appear in NO code position under `src/` or `wat/`, because nothing in the stdlib happens
//! to call them. So emptying the declaration source leaves 30 names unresolvable and it can fail
//! loudly on its own. The other 50 are attested only by accident, through their own use in the
//! stdlib — which is exactly why this hole read as "some accessors resolve and some do not".
//!
//! ── CODE vs PROSE — the distinction that IS the design ───────────────────────────────────────
//!
//! `:wat::rete::core::foldr` and `:wat::rete::core::nth` appear under `wat-scripts/` today, and
//! both are correct: they sit in comments, recording that those rows were retired. A gate that
//! flagged them would demand the deletion of the tree's own accurate history of its retirements —
//! and that history is exactly what this class of defect needs preserved. So comments are stripped
//! before anything is classified, string-literal-aware in both directions: a `;;` inside a wat
//! string is not a comment, and a `"` inside a comment does not open a string.
//!
//! A name inside a wat STRING LITERAL is code, not prose. The phantom rename rows were string
//! literals in a table, and that table is what the codemod actually rewrites with.
//!
//! ── THE EARNED EXEMPTION ─────────────────────────────────────────────────────────────────────
//!
//! Two honest reasons for an unresolvable name in a code position exist in this tree, and neither
//! is rot:
//!
//! * **A recorded codemod's OLD column.** A migration tool must be able to say what it removes;
//!   `wat-scripts/fixes/rete-oracle-sigil.wat` names three pre-`$oracle` spellings it exists to
//!   eliminate, and `wat-scripts/fixes/type-query-to-defquery.wat` names the head it detects.
//! * **A negative control.** `wat-scripts/scratch-pad/probe-f64-comparator-bogus-head.wat` calls a
//!   head that was never minted, on purpose: that the head is absent IS its experiment.
//!
//! Each such site carries a per-name declaration in a wat comment —
//! `;; rune:lint(rete-name-unminted) <the name> — <reason>` — with a reason long enough to name a
//! mechanism. It is a DECLARATION, not a suppression: a re-introduced phantom with no rune is still
//! a red, and a rune must name the exact token, so no site can wave through a name it did not
//! anticipate.
//!
//! Shape and precedent: `tests/lint/retired_name_justified.rs` (per-site runes over a string scan)
//! and `tests/lint/every_walking_gate_declares_non_vacuity.rs` (a rune whose reason must hold up).

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

/// The registry file. Its `rete_name` field is the authority for the operator namespace, and it is
/// read STRUCTURALLY (that one field) rather than scraped, so its own prose cannot admit a name.
const REGISTRY: &str = "src/rete/vocabulary.rs";

/// The prefix that makes a token this gate's business.
const PREFIX: &str = ":wat::rete::";

/// The namespaces `RETE_OPS` is the sole authority for.
const REGISTRY_NAMESPACES: &[&str] = &[":wat::rete::core::", ":wat::rete::holon::"];

/// Names living in the operator namespace that are NOT operator rows.
///
/// `defn` is a declaration FORM — `src/value/environment.rs` and `src/value/signal.rs` treat it as
/// the site where a rete contract is attested — and 15 files under `wat-scripts/` use it correctly.
/// The list is short by nature: minting a form is a language-surface change, not a table edit, so a
/// new entry here should be a deliberate act with a reason beside it. Every entry is checked against
/// the tree on each run — see [`known_forms_are_real`] — so this cannot become a private fiction.
const KNOWN_FORMS: &[&str] = &[":wat::rete::core::defn"];

/// The per-name exemption marker, written in a wat comment.
const RUNE: &str = "rune:lint(rete-name-unminted)";

/// Shortest reason that can name a mechanism, matching
/// `tests/lint/every_walking_gate_declares_non_vacuity.rs`.
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

/// Characters a `:wat::rete::` name is built from, after the prefix.
///
/// ⚠ The charset is where a naive scan breaks, in BOTH directions. A first pass over this corpus
/// stopped before `=` and reported `:wat::rete::core::enum::` and `:wat::rete::core::f64::` as
/// unknown names — they were fragments of `…::enum::=` and `…::f64::=`, so the operators are in.
/// Two characters are deliberately OUT, and both exclusions are visible rather than silent:
///
/// * `.` — no rete name contains one, and including it would turn a sentence-ending
///   `:wat::rete::defrule.` in prose into a phantom token.
/// * `'` — the retired IPC-prime spelling. Including it would surface seven more codemod OLD-column
///   names (`:wat::rete::insert'` and friends in `wat-scripts/fixes/rete-oracle-sigil.wat`) that are
///   retired for a different reason and are already the subject of
///   `tests/lint/retired_name_justified.rs`. A primed name therefore reads here as its unprimed
///   stem, which resolves. This is a boundary, not a blind spot: it is stated so it can be moved.
fn is_name_char(c: char) -> bool {
    c.is_ascii_alphanumeric() || "_:!?*/<>=+$-".contains(c)
}

/// Blank out wat `;;` comments, preserving byte offsets and line structure.
///
/// String-literal aware: a `;;` inside a `"…"` is content, and a `"` inside a comment never opens
/// one. The returned string has the same length as the input, so a match offset in it is a match
/// offset in the original.
fn wat_code_mask(src: &str) -> String {
    mask(src, ";;")
}

/// The inverse: everything that is NOT wat code — used to read runes and to prove, in
/// [`prose_control_holds`], that the comment stripper is doing something.
fn wat_comment_text(src: &str) -> String {
    let code = wat_code_mask(src);
    src.chars()
        .zip(code.chars())
        .map(|(o, c)| {
            // Newlines survive on both sides: `runes` reads this text line by line, and a
            // reconstruction that flattened it to one line would let a rune borrow its reason from
            // whatever prose happened to follow it.
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

/// Blank out `//` line comments in Rust source, preserving byte offsets.
fn rs_code_mask(src: &str) -> String {
    mask(src, "//")
}

/// Blank every comment run opened by `opener` and closed by a newline, leaving code and string
/// literals intact and every byte offset unchanged.
///
/// Both languages this is used for open a line comment with a two-character token and close it at
/// the newline, and both quote with `"` and escape with `\`, so one scanner serves both.
fn mask(src: &str, opener: &str) -> String {
    let bytes: Vec<char> = src.chars().collect();
    let open: Vec<char> = opener.chars().collect();
    let mut out: Vec<char> = Vec::with_capacity(bytes.len());
    let mut i = 0;
    let mut in_str = false;
    let mut esc = false;
    while i < bytes.len() {
        let c = bytes[i];
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
        if c == open[0] && bytes.get(i + 1) == Some(&open[1]) {
            while i < bytes.len() && bytes[i] != '\n' {
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

/// Every `:wat::rete::…` token in `text`, with the 1-based line each sits on.
fn tokens_with_lines(text: &str) -> Vec<(String, usize)> {
    let mut out = Vec::new();
    let chars: Vec<char> = text.chars().collect();
    let pre: Vec<char> = PREFIX.chars().collect();
    let mut line = 1usize;
    let mut i = 0usize;
    while i < chars.len() {
        if chars[i] == '\n' {
            line += 1;
            i += 1;
            continue;
        }
        if chars[i] == pre[0] && chars[i..].starts_with(pre.as_slice()) {
            let mut j = i + pre.len();
            while j < chars.len() && is_name_char(chars[j]) {
                j += 1;
            }
            out.push((chars[i..j].iter().collect::<String>(), line));
            i = j;
            continue;
        }
        i += 1;
    }
    out
}

/// Just the token strings.
fn tokens(text: &str) -> BTreeSet<String> {
    tokens_with_lines(text).into_iter().map(|(t, _)| t).collect()
}

/// The `rete_name` rows of `RETE_OPS`, read from the registry's own field.
fn registry_rows(src: &str) -> BTreeSet<String> {
    let mut out = BTreeSet::new();
    for line in src.lines() {
        let t = line.trim_start();
        if t.starts_with("//") {
            continue;
        }
        let Some(rest) = t.strip_prefix("rete_name:") else {
            continue;
        };
        let rest = rest.trim_start();
        let Some(body) = rest.strip_prefix('"') else {
            continue;
        };
        let Some(end) = body.find('"') else { continue };
        out.insert(body[..end].to_string());
    }
    out
}

/// The declaration forms that MINT an aggregate's field accessors.
///
/// `register_aggregate_methods` (`src/runtime.rs`) walks every `TypeDef::Aggregate` and registers
/// one function per declared field at `format!("{}/{}", agg.name, field_name)`. Nothing else
/// registers those paths — the macro's own accessor emission was removed in arc 293.R2.2 — so a
/// `defrecord` in the tree IS the authority for which `<Type>/<field>` names exist, exactly as
/// `RETE_OPS` is the authority for the operator namespace.
const RECORD_DECL_FORMS: &[&str] = &[":wat::core::defrecord", ":wat::holon::defrecord"];

/// The separator inside a field vector. Fields come in groups of three — `name <- Type` — which is
/// the arithmetic `wat/Record.wat`'s own macro does (`n-fields = field-len / 3`).
const FIELD_ARROW: &str = "<-";

/// Split a form's body into top-level items: a balanced `(…)`/`[…]`/`{…}` group and a `"…"` string
/// each count as ONE item.
///
/// ⚠ THIS IS WHERE A NAIVE PARSE LOSES FIELDS, and it has. A regex that ended a field vector at the
/// first `])` read `:wat::rete::DerivationNode` as two fields, dropping `via` — whose type is
/// `(:wat::core::PersistentVector :- [:wat::rete::DerivationStep])`, so the vector's own `]` is
/// preceded by the nested type's. Balance is not optional here; see
/// [`the_record_parse_reads_a_nested_last_field`], which anchors on that exact record.
fn top_level_items(chars: &[char]) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    let mut depth = 0usize;
    let mut start: Option<usize> = None;
    let mut in_str = false;
    let mut esc = false;
    for (i, &c) in chars.iter().enumerate() {
        if in_str {
            if esc {
                esc = false;
            } else if c == '\\' {
                esc = true;
            } else if c == '"' {
                in_str = false;
                if depth == 0 {
                    if let Some(s) = start.take() {
                        out.push(chars[s..=i].iter().collect());
                    }
                }
            }
            continue;
        }
        match c {
            '"' => {
                in_str = true;
                if start.is_none() {
                    start = Some(i);
                }
            }
            '(' | '[' | '{' => {
                if start.is_none() {
                    start = Some(i);
                }
                depth += 1;
            }
            ')' | ']' | '}' => {
                depth = depth.saturating_sub(1);
                if depth == 0 {
                    if let Some(s) = start.take() {
                        out.push(chars[s..=i].iter().collect());
                    }
                }
            }
            _ if c.is_whitespace() && depth == 0 => {
                if let Some(s) = start.take() {
                    out.push(chars[s..i].iter().collect());
                }
            }
            _ => {
                if start.is_none() {
                    start = Some(i);
                }
            }
        }
    }
    if let Some(s) = start {
        out.push(chars[s..].iter().collect());
    }
    out
}

/// The index of the bracket closing the group opened at `open`, string-aware.
fn matching_close(chars: &[char], open: usize) -> Option<usize> {
    let mut depth = 0usize;
    let mut in_str = false;
    let mut esc = false;
    for (i, &c) in chars.iter().enumerate().skip(open) {
        if in_str {
            if esc {
                esc = false;
            } else if c == '\\' {
                esc = true;
            } else if c == '"' {
                in_str = false;
            }
            continue;
        }
        match c {
            '"' => in_str = true,
            '(' | '[' | '{' => depth += 1,
            ')' | ']' | '}' => {
                depth -= 1;
                if depth == 0 {
                    return Some(i);
                }
            }
            _ => {}
        }
    }
    None
}

/// The field vector among a declaration form's top-level items: the LAST `[…]`.
///
/// `wat/Record.wat` picks the same slot the same way — `(last args)`, never by counting — because
/// the optional generic binder sits BETWEEN the name and the fields:
/// `(defrecord :Name :- [T…] [fields])`. Taking the first `[…]` would read the type parameters as
/// the field list.
fn field_vector_of(items: &[String]) -> Option<&String> {
    items
        .last()
        .filter(|s| s.starts_with('[') && s.ends_with(']'))
}

/// The field NAMES declared INSIDE a `[name <- Type …]` vector (brackets already peeled), in order
/// — or why the declaration could not be read.
///
/// ⛔ AN UNREADABLE VECTOR IS AN ERROR, NEVER AN EMPTY LIST. A record silently dropped here keeps
/// its accessors out of the resolver's universe, which re-blocks exactly its fields while every
/// other verdict stays green — the invisible re-blockage this whole source exists to prevent,
/// arriving one record at a time instead of all at once.
fn declared_field_names(inner: &str) -> Result<Vec<String>, String> {
    let chars: Vec<char> = inner.chars().collect();
    let cells = top_level_items(&chars);
    if cells.is_empty() || !cells.len().is_multiple_of(3) {
        return Err(format!(
            "the field vector holds {} cell(s), not a multiple of 3 (`name {FIELD_ARROW} Type`)",
            cells.len()
        ));
    }
    let mut names = Vec::with_capacity(cells.len() / 3);
    for group in cells.chunks(3) {
        if group[1] != FIELD_ARROW {
            return Err(format!(
                "field `{}` is followed by `{}`, not `{FIELD_ARROW}`",
                group[0], group[1]
            ));
        }
        names.push(group[0].clone());
    }
    Ok(names)
}

/// What a source declares: `:wat::rete::<Type>` → its field names, plus every `:wat::rete::` record
/// whose declaration could NOT be read.
#[derive(Debug, Default)]
struct RecordDecls {
    fields: BTreeMap<String, Vec<String>>,
    malformed: Vec<String>,
}

/// Read every `:wat::rete::` record declaration in one wat source.
///
/// Comments are masked first — `wat/gen.wat` and `wat/cache.wat` both show `defrecord` forms in
/// prose — so a declaration that is commented out declares nothing, the same cut the rest of this
/// gate makes between code and prose.
fn record_decls(src: &str) -> RecordDecls {
    record_decls_in(src, PREFIX)
}

/// [`record_decls`] over an arbitrary namespace. The gate itself only ever asks for [`PREFIX`];
/// the parameter exists so [`the_record_parse_reads_a_nested_last_field`] can anchor the
/// generic-binder rule on `wat/gen.wat`'s `:wat::gen::Pick`, a REAL declaration carrying both a
/// `:- [T…]` binder and a nested type on its last field. No `:wat::rete::` record is generic, so
/// that rule is unreachable from the corpus this gate walks — which is why the anchor reaches for
/// a namespace that has one rather than for a hand-written example.
fn record_decls_in(src: &str, namespace: &str) -> RecordDecls {
    let chars: Vec<char> = wat_code_mask(src).chars().collect();
    let mut out = RecordDecls::default();
    for i in 0..chars.len() {
        if chars[i] != '(' {
            continue;
        }
        // Cheap head check before the balanced walk, so the O(form) split runs only on real
        // declarations rather than on every open paren in the stdlib.
        let mut h = i + 1;
        while h < chars.len() && chars[h].is_whitespace() {
            h += 1;
        }
        let head_text: String = chars[h..chars.len().min(h + 32)].iter().collect();
        if !RECORD_DECL_FORMS.iter().any(|f| head_text.starts_with(f)) {
            continue;
        }
        let Some(close) = matching_close(&chars, i) else {
            continue;
        };
        let items = top_level_items(&chars[i + 1..close]);
        let Some(head) = items.first() else { continue };
        if !RECORD_DECL_FORMS.contains(&head.as_str()) {
            continue;
        }
        let Some(name) = items.get(1) else { continue };
        // Only this gate's namespace. A `:wat::query::` record is declared the same way and mints
        // its accessors the same way; resolving those is the wider question the module header cuts.
        if !name.starts_with(namespace) {
            continue;
        }
        let Some(vector) = field_vector_of(&items) else {
            out.malformed.push(format!(
                "{name}: the last argument is not a `[…]` field vector, so its fields cannot be read"
            ));
            continue;
        };
        let inner: String = vector.chars().skip(1).take(vector.chars().count() - 2).collect();
        match declared_field_names(&inner) {
            Ok(names) => {
                out.fields.insert(name.clone(), names);
            }
            Err(why) => out.malformed.push(format!("{name}: {why}")),
        }
    }
    out
}

/// Every `:wat::rete::` record declared under `wat/`, with its fields.
fn rete_record_decls() -> RecordDecls {
    let mut wats = Vec::new();
    collect(&root().join("wat"), "wat", &mut wats);
    wats.sort();
    let mut out = RecordDecls::default();
    for p in wats {
        let Ok(src) = std::fs::read_to_string(&p) else {
            continue;
        };
        let one = record_decls(&src);
        let rel = p.strip_prefix(root()).unwrap_or(&p).display().to_string();
        out.malformed
            .extend(one.malformed.into_iter().map(|m| format!("  {rel}  {m}")));
        out.fields.extend(one.fields);
    }
    out
}

/// The accessor names a set of record declarations MINTS: one `<Type>/<field>` per declared field,
/// matching `register_aggregate_methods`' `format!("{}/{}", agg.name, field_name)`.
///
/// ⛔ THE FIELD HALF IS THE GATE. `<Type>/<anything>` would resolve a misspelled accessor, trading
/// one false RED for a permanent blind spot over every field of every rete record — and a gate that
/// never fires is the one nobody notices. `DerivationNode/vai` must not be in this set.
fn record_accessors(decls: &RecordDecls) -> BTreeSet<String> {
    decls
        .fields
        .iter()
        .flat_map(|(ty, fields)| fields.iter().map(move |f| format!("{ty}/{f}")))
        .collect()
}

/// Is this token in the namespace `RETE_OPS` owns?
fn in_registry_namespace(token: &str) -> bool {
    REGISTRY_NAMESPACES.iter().any(|ns| token.starts_with(ns))
}

/// The verdict on one rune line.
#[derive(Debug, PartialEq, Eq)]
enum Rune {
    /// The name is declared deliberately unminted, with a reason that names a mechanism.
    Declared,
    /// A rune for this name exists but does not hold up.
    Hollow(String),
}

/// Read the runes in one file's comment text, keyed by the exact token each names.
fn runes(comment_text: &str) -> BTreeMap<String, Rune> {
    let mut out = BTreeMap::new();
    for line in comment_text.lines() {
        let Some(at) = line.find(RUNE) else { continue };
        let tail = line[at + RUNE.len()..].trim();
        let mut parts = tail.splitn(2, char::is_whitespace);
        let Some(token) = parts.next().filter(|t| t.starts_with(PREFIX)) else {
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
                || folded.strip_prefix(*r).is_some_and(|t| {
                    t.starts_with(' ') || t.starts_with(',') || t.starts_with(';')
                })
        }) {
            out.insert(
                token.to_string(),
                Rune::Hollow(format!(
                    "the reason is `{reason}` \u{2014} `{shrug}` names no mechanism. Say why this \
                     name is deliberately unminted and what would break if it were minted"
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

fn wat_scripts_files() -> Vec<PathBuf> {
    let mut v = Vec::new();
    collect(&root().join("wat-scripts"), "wat", &mut v);
    v.sort();
    v
}

/// Every `:wat::rete::` name attested in a CODE position outside `wat-scripts/`: the Rust substrate
/// and the frozen stdlib. The registry file is EXCLUDED — its rows come from [`registry_rows`], and
/// scraping it as well would let its own prose (which discusses the retired lazy heads by name)
/// admit the very phantoms this gate hunts.
fn attested() -> BTreeSet<String> {
    let registry = root().join(REGISTRY);
    let mut out = BTreeSet::new();

    let mut rs = Vec::new();
    collect(&root().join("src"), "rs", &mut rs);
    for p in rs {
        if p == registry {
            continue;
        }
        let Ok(src) = std::fs::read_to_string(&p) else {
            continue;
        };
        out.extend(tokens(&rs_code_mask(&src)));
    }

    let mut wats = Vec::new();
    collect(&root().join("wat"), "wat", &mut wats);
    for p in wats {
        let Ok(src) = std::fs::read_to_string(&p) else {
            continue;
        };
        out.extend(tokens(&wat_code_mask(&src)));
    }
    out
}

#[test]
fn every_rete_name_in_wat_scripts_code_resolves() {
    let rows = registry_rows(
        &std::fs::read_to_string(root().join(REGISTRY))
            .unwrap_or_else(|e| panic!("read {REGISTRY}: {e}")),
    );
    let attested = attested();
    let decls = rete_record_decls();
    let accessors = record_accessors(&decls);
    let files = wat_scripts_files();

    // A `:wat::rete::` record whose declaration could not be READ contributes zero accessors, so
    // every one of its fields goes back to being unwritable — invisibly, because nothing under
    // `wat-scripts/` names one today. A parser that meets a form it does not understand says so.
    assert!(
        decls.malformed.is_empty(),
        "\n\n{} `:wat::rete::` record declaration(s) under wat/ could not be read, so their \
         accessors are missing from the resolver's universe:\n\n{}\n",
        decls.malformed.len(),
        decls.malformed.join("\n")
    );

    // NON-VACUITY, part 1 — the two halves of the RESOLVER. Either one silently coming back empty
    // would leave this gate flagging correct names, or (worse, if the walk emptied instead) waving
    // everything through. Measured 2026-09-01: 79 `RETE_OPS` rows, 328 attested names.
    assert!(
        rows.len() >= 70 && attested.len() >= 250,
        "the resolver went blind: {} RETE_OPS row(s) read from {REGISTRY} and {} name(s) attested \
         in src/ + wat/. Neither number can legitimately collapse, so this gate's verdicts are \
         unearned until it is fixed",
        rows.len(),
        attested.len()
    );

    // NON-VACUITY, part 1b — the THIRD source, the record declarations. A minted accessor is never
    // text anywhere `attested` looks, so if this parse silently returned nothing every rete
    // accessor would be unwritable again — and this gate would stay GREEN about it, because nothing
    // under `wat-scripts/` names one today. The blockage would come back invisibly and the next
    // hand would rediscover it from scratch, which is how it was found the first time. Measured
    // 2026-09-03: 25 `:wat::rete::` record(s) under wat/, minting 80 accessors, of which 30 are
    // attested NOWHERE else and so resolve by this source alone.
    assert!(
        decls.fields.len() >= 20 && accessors.len() >= 60,
        "the record parse went blind: {} `:wat::rete::` record(s) read from wat/ minting {} \
         accessor(s). A collapse here does not make this gate red — it makes every rete accessor \
         unwritable while the gate reports a clean tree",
        decls.fields.len(),
        accessors.len()
    );

    // NON-VACUITY, part 2 — the SUBJECT walk, and the classifier over it. A count of files is not
    // enough: a stripper that blanked everything would find 445 files and zero tokens and report a
    // clean tree. Measured 2026-09-01: 445 .wat file(s), 4445 name occurrences in code, 721 of
    // them in the registry namespace.
    let all_code: Vec<(String, usize, PathBuf)> = files
        .iter()
        .flat_map(|p| {
            let src = std::fs::read_to_string(p).unwrap_or_default();
            tokens_with_lines(&wat_code_mask(&src))
                .into_iter()
                .map(|(t, l)| (t, l, p.clone()))
                .collect::<Vec<_>>()
        })
        .collect();
    let registry_ns = all_code.iter().filter(|(t, _, _)| in_registry_namespace(t)).count();
    assert!(
        files.len() > 200 && all_code.len() > 2000 && registry_ns > 400,
        "the subject walk went blind: {} .wat file(s), {} rete name(s) in code, {} of them in the \
         registry namespace — this gate is judging almost nothing",
        files.len(),
        all_code.len(),
        registry_ns
    );

    let mut unresolved: Vec<String> = Vec::new();
    let mut hollow: Vec<String> = Vec::new();

    for path in &files {
        let Ok(src) = std::fs::read_to_string(path) else {
            continue;
        };
        let declared = runes(&wat_comment_text(&src));
        let rel = path.strip_prefix(root()).unwrap_or(path).display().to_string();

        let mut seen: BTreeSet<String> = BTreeSet::new();
        for (token, line) in tokens_with_lines(&wat_code_mask(&src)) {
            let ok = if in_registry_namespace(&token) {
                rows.contains(&token) || KNOWN_FORMS.contains(&token.as_str())
            } else {
                attested.contains(&token) || accessors.contains(&token)
            };
            if ok || !seen.insert(token.clone()) {
                continue;
            }
            match declared.get(&token) {
                Some(Rune::Declared) => {}
                Some(Rune::Hollow(why)) => {
                    hollow.push(format!("  {rel}:{line}  {token}\n      {why}"))
                }
                None => {
                    let family = if in_registry_namespace(&token) {
                        format!("no `RETE_OPS` row in {REGISTRY}, and not a known form")
                    } else {
                        "not attested in any code position under src/ or wat/, and not a \
                         field accessor minted by any `:wat::rete::` record declared there"
                            .to_string()
                    };
                    unresolved.push(format!("  {rel}:{line}  {token}\n      {family}"));
                }
            }
        }
    }

    assert!(
        hollow.is_empty(),
        "\n\n{} deliberately-unminted name(s) DECLARE a reason that does not hold up. A rune with a \
         hollow reason is worse than no rune: it reads, forever, as though someone answered.\n\n{}\n",
        hollow.len(),
        hollow.join("\n")
    );

    assert!(
        unresolved.is_empty(),
        "\n\n🔥 {} `:wat::rete::` name(s) are written in CODE under wat-scripts/ and resolve to \
         NOTHING. `wat_scripts_fixes_load.rs` cannot see this: a `def` body nothing forces is never \
         resolved, so an invented head parses, type-checks, and runs.\n\
         \n\
         THE FIX, one of three:\n\
         \n\
         1. It is a TYPO or a name that MOVED — spell it as the row that exists. Read {REGISTRY}; \
         do not guess a twin. The rete map/filter family is `mapv`/`filterv`/`foldl`/`reduce`. For \
         a `<Type>/<field>` accessor the authority is the `defrecord` under wat/ — a field that \
         is not declared there was never minted, whatever the type is.\n\
         \n\
         2. It is a name that was RETIRED and there is no correct replacement — then the code that \
         names it is dead, and the fix is to delete or rewrite it, not to invent a target. \
         `wat-scripts/fixes/rete-where-per-type-spelling.wat` records what that looks like: two \
         rename rows deleted rather than re-pointed, because the only candidate targets would have \
         swapped a lazy Stream for an eager Vector.\n\
         \n\
         3. The name is deliberately unminted and that IS the point — a recorded codemod's OLD \
         column, or a negative control whose experiment is the absence. Declare it, per name, in a \
         wat comment: `;; {RUNE} <the name> \u{2014} <why it is unminted, and what breaks if it is \
         minted>`. A reason under {MIN_REASON_CHARS} chars is refused.\n\
         \n\
         ⛔ NOT A FIX: moving the name into a comment to silence this gate. Comments are read as \
         prose here precisely so the tree can keep an accurate record of its retirements — turning \
         live code into a comment to dodge a red converts a defect into a lie.\n\
         \n\
         Unresolved:\n\n{}\n",
        unresolved.len(),
        unresolved.join("\n")
    );
}

/// Every entry in [`KNOWN_FORMS`] is a real name in the tree that is genuinely NOT a row.
///
/// Without this, the forms list is a private fiction: an entry could be a typo, or a name that was
/// since promoted to a `RETE_OPS` row (in which case the list is shadowing the registry), and either
/// way the exemption would go on working forever.
#[test]
fn known_forms_are_real() {
    let rows = registry_rows(
        &std::fs::read_to_string(root().join(REGISTRY))
            .unwrap_or_else(|e| panic!("read {REGISTRY}: {e}")),
    );
    let attested = attested();
    // NON-VACUITY: an empty attestation set would make the "is attested" half of this check pass
    // over nothing. Measured 2026-09-01: 328 names.
    assert!(
        attested.len() >= 250 && !KNOWN_FORMS.is_empty(),
        "known-forms self-check went blind: {} attested name(s), {} form(s) declared",
        attested.len(),
        KNOWN_FORMS.len()
    );
    for form in KNOWN_FORMS {
        assert!(
            attested.contains(*form),
            "KNOWN_FORMS names `{form}`, which appears in no code position under src/ or wat/ — \
             either it is a typo or the form was removed, and in both cases it is exempting a name \
             that no longer exists"
        );
        assert!(
            !rows.contains(*form),
            "KNOWN_FORMS names `{form}`, which IS a RETE_OPS row in {REGISTRY} — the registry \
             already resolves it, so this entry is shadowing the authority it was meant to sit \
             beside"
        );
    }
}

/// The record parse, anchored on the declarations a naive parse gets WRONG.
///
/// ⛔ THIS IS THE TEST THAT WOULD HAVE CAUGHT THE FIRST COUNT. A regex-shaped parse that ended the
/// field vector at the first `])` read `:wat::rete::DerivationNode` as two fields and reported 46
/// accessors instead of 80 — and the field it dropped was `via`, whose type is
/// `(:wat::core::PersistentVector :- [:wat::rete::DerivationStep])` and which is the exact accessor
/// that proved the resolver was short. A COUNT cannot notice that: same shape, fewer values. So the
/// anchors here are field NAMES, in order, for the records the nesting bites — read by eye from
/// `wat/rete.wat` before any total was quoted.
///
/// It also pins the two boundaries the source has: the declaration site is NOT one file (six
/// `:wat::rete::` records live under `wat/rete/`, not in `wat/rete.wat`, and an accessor of theirs
/// is exactly as minted), and the NAMESPACE is the cut (`wat/query.wat` declares its records the
/// same way and mints its accessors the same way — they are not this gate's business).
#[test]
fn the_record_parse_reads_a_nested_last_field() {
    let decls = rete_record_decls();
    assert!(
        decls.malformed.is_empty(),
        "unreadable `:wat::rete::` record declaration(s):\n{}",
        decls.malformed.join("\n")
    );

    assert_eq!(
        decls.fields.get(":wat::rete::DerivationNode").map(Vec::as_slice),
        Some(["fact", "rule", "via"].map(String::from).as_slice()),
        "the last field of `DerivationNode` carries a nested type; losing it is the defect this \
         source was added after"
    );
    assert_eq!(
        decls.fields.get(":wat::rete::DerivationStep").map(Vec::as_slice),
        Some(
            ["supporting", "pattern", "bindings", "constraints"]
                .map(String::from)
                .as_slice()
        )
    );
    // Two wide records, counted by eye in wat/rete.wat: `Export` declares 11 fields, `Session` 8.
    // A parse that lost a trailing field would still produce a plausible-looking total.
    assert_eq!(decls.fields.get(":wat::rete::Export").map(Vec::len), Some(11));
    assert_eq!(decls.fields.get(":wat::rete::Session").map(Vec::len), Some(8));
    // Declared in `wat/rete/compile.wat`, not `wat/rete.wat` — the authority is the declaration,
    // wherever it sits.
    assert_eq!(
        decls.fields.get(":wat::rete::CompileState").map(Vec::as_slice),
        Some(["network", "next-id", "dedup"].map(String::from).as_slice())
    );
    // `wat/query.wat` declares `:wat::query::Row` with the same form. Out of namespace, out of set.
    assert_eq!(decls.fields.get(":wat::query::Row"), None);

    // THE GENERIC BINDER, on a real declaration that has one AND a nested last field:
    // `(defrecord :wat::gen::Pick :- [T] [rest <- :wat::core::i64 got <- (Option :- [T])])`.
    // Taking the FIRST `[…]` would read the type parameters `[T]` as the field list; taking the
    // last cuts nothing. No rete record is generic today, so this rule is unreachable from the
    // corpus this gate walks — which is exactly why it is anchored somewhere it IS reachable.
    let gen_src = std::fs::read_to_string(root().join("wat/gen.wat"))
        .unwrap_or_else(|e| panic!("read wat/gen.wat: {e}"));
    let gen = record_decls_in(&gen_src, ":wat::gen::");
    assert!(
        gen.malformed.is_empty(),
        "unreadable `:wat::gen::` record declaration(s):\n{}",
        gen.malformed.join("\n")
    );
    assert_eq!(
        gen.fields.get(":wat::gen::Pick").map(Vec::as_slice),
        Some(["rest", "got"].map(String::from).as_slice())
    );

    // EXACT, not membership: this is every accessor `DerivationNode` mints. It says `via` is in
    // AND that nothing else is — so the misspelling `…/vai`, and the bare type name, are both
    // refused by the same assertion. A `contains` pair could not have said the second half.
    let node_accessors: BTreeSet<String> = record_accessors(&decls)
        .into_iter()
        .filter(|a| a.starts_with(":wat::rete::DerivationNode/"))
        .collect();
    assert_eq!(
        node_accessors,
        BTreeSet::from([
            ":wat::rete::DerivationNode/fact".to_string(),
            ":wat::rete::DerivationNode/rule".to_string(),
            ":wat::rete::DerivationNode/via".to_string(),
        ]),
        "a field that is not declared must not mint an accessor — without that, the resolver \
         accepts any token with a slash in it and has stopped resolving"
    );
}

/// The negative control for the RUST comment stripper, and it is not hypothetical.
///
/// `src/rete/vocabulary.rs` is excluded from the attestation scrape, but the retired lazy heads are
/// discussed BY NAME in `//` comments in other files under `src/` too. If [`rs_code_mask`] ever
/// stops blanking comments, those names leak into the attested set and the phantom this gate exists
/// to catch becomes resolvable — silently, and with every other verdict still green.
#[test]
fn prose_in_rust_does_not_attest_a_name() {
    let attested = attested();
    assert!(
        attested.len() >= 250,
        "attestation scrape went blind ({} names), so this control proves nothing",
        attested.len()
    );
    for phantom in [":wat::rete::core::map", ":wat::rete::core::filter"] {
        assert!(
            !attested.contains(phantom),
            "`{phantom}` is attested as a code name under src/ or wat/. It has never been a \
             RETE_OPS row, so either the Rust comment stripper has stopped blanking comments (it \
             is named in `//` prose there) or the name has genuinely been minted — check \
             {REGISTRY} before touching this control"
        );
    }
}

/// The positive control for the WAT comment stripper: a retired name that lives, correctly, only in
/// prose under `wat-scripts/` must be VISIBLE as prose and INVISIBLE as code.
///
/// This is the trap the whole design turns on. A gate that could not tell code from comment would be
/// "fixed" by deleting the tree's accurate record of what it retired — so the record is asserted
/// here rather than merely tolerated, and a stripper that started reading comments as code would red
/// this before it started demanding those deletions.
#[test]
fn prose_control_holds() {
    let files = wat_scripts_files();
    // NON-VACUITY: measured 2026-09-01, 445 .wat files under wat-scripts/.
    assert!(
        files.len() > 200,
        "the wat-scripts walk found only {} file(s); this control proves nothing",
        files.len()
    );
    let mut in_prose = BTreeSet::new();
    let mut in_code = BTreeSet::new();
    for p in &files {
        let Ok(src) = std::fs::read_to_string(p) else {
            continue;
        };
        in_prose.extend(tokens(&wat_comment_text(&src)));
        in_code.extend(tokens(&wat_code_mask(&src)));
    }
    for retired in [":wat::rete::core::foldr", ":wat::rete::core::nth"] {
        assert!(
            in_prose.contains(retired),
            "`{retired}` no longer appears in any comment under wat-scripts/. Either the wat \
             comment reader has gone blind, or the tree's record of that retirement was deleted"
        );
        assert!(
            !in_code.contains(retired),
            "`{retired}` now appears in a CODE position under wat-scripts/ — it is a retired row, \
             so that code cannot resolve"
        );
    }
}

#[cfg(test)]
mod classifier {
    use super::*;

    /// Fragments here are deliberately not parenthesised forms: `no_inlined_wat_in_tests` reads any
    /// string literal that parses as a list with a keyword or symbol head as an inlined world.
    const CODE_TOKEN: &str = ":wat::rete::core::mapv";

    #[test]
    fn a_comment_is_not_code() {
        let src = format!(";; a note about {CODE_TOKEN}\n");
        assert!(tokens(&wat_code_mask(&src)).is_empty());
        assert_eq!(tokens(&wat_comment_text(&src)).len(), 1);
    }

    #[test]
    fn a_name_in_a_string_literal_is_code() {
        let src = format!("\"{CODE_TOKEN}\"\n");
        assert_eq!(tokens(&wat_code_mask(&src)).len(), 1);
        assert!(tokens(&wat_comment_text(&src)).is_empty());
    }

    #[test]
    fn a_comment_marker_inside_a_string_is_not_a_comment() {
        let src = format!("\"a ;; b\" {CODE_TOKEN}\n");
        assert_eq!(tokens(&wat_code_mask(&src)).len(), 1);
    }

    #[test]
    fn a_quote_inside_a_comment_does_not_open_a_string() {
        let src = format!(";; a lone \" quote\n{CODE_TOKEN}\n");
        assert_eq!(tokens(&wat_code_mask(&src)).len(), 1);
    }

    #[test]
    fn an_operator_suffix_is_part_of_the_name() {
        assert_eq!(
            tokens(":wat::rete::core::enum::= x\n").into_iter().next(),
            Some(":wat::rete::core::enum::=".to_string())
        );
    }

    #[test]
    fn a_sentence_period_does_not_join_the_name() {
        assert_eq!(
            tokens(":wat::rete::defrule.\n").into_iter().next(),
            Some(":wat::rete::defrule".to_string())
        );
    }

    #[test]
    fn a_line_number_is_reported() {
        let src = format!("\n\n{CODE_TOKEN}\n");
        assert_eq!(tokens_with_lines(&src), vec![(CODE_TOKEN.to_string(), 3)]);
    }

    #[test]
    fn a_rust_line_comment_is_blanked_but_a_string_is_not() {
        let src = format!("let a = \"{CODE_TOKEN}\"; // and {CODE_TOKEN}\n");
        assert_eq!(tokens(&rs_code_mask(&src)).len(), 1);
    }

    #[test]
    fn the_registry_reader_takes_the_field_not_the_prose() {
        let src = "    // rete_name: \":wat::rete::core::ghost\",\n\
                   rete_name: \":wat::rete::core::real\",\n";
        assert_eq!(
            registry_rows(src),
            BTreeSet::from([":wat::rete::core::real".to_string()])
        );
    }

    #[test]
    fn a_rune_naming_a_mechanism_declares_the_name() {
        let src = format!(
            ";; {RUNE} {CODE_TOKEN} \u{2014} the pre-migration spelling this recorded codemod \
             exists to eliminate.\n"
        );
        assert_eq!(runes(&src).get(CODE_TOKEN), Some(&Rune::Declared));
    }

    #[test]
    fn a_rune_reading_n_slash_a_is_refused() {
        let src = format!(";; {RUNE} {CODE_TOKEN} \u{2014} N/A\n");
        match runes(&src).get(CODE_TOKEN) {
            Some(Rune::Hollow(_)) => {}
            other => panic!("`N/A` must be refused, got {other:?}"),
        }
    }

    #[test]
    fn a_rune_too_short_to_name_a_mechanism_is_refused() {
        let src = format!(";; {RUNE} {CODE_TOKEN} \u{2014} it is fine\n");
        match runes(&src).get(CODE_TOKEN) {
            Some(Rune::Hollow(_)) => {}
            other => panic!("a reason under the floor must be refused, got {other:?}"),
        }
    }

    #[test]
    fn a_rune_with_no_reason_is_refused() {
        let src = format!(";; {RUNE} {CODE_TOKEN}\n");
        match runes(&src).get(CODE_TOKEN) {
            Some(Rune::Hollow(_)) => {}
            other => panic!("a rune with no reason must be hollow, got {other:?}"),
        }
    }

    #[test]
    fn a_rune_declares_only_the_name_it_writes() {
        let src = format!(
            ";; {RUNE} {CODE_TOKEN} \u{2014} the pre-migration spelling this recorded codemod \
             exists to eliminate.\n"
        );
        assert!(!runes(&src).contains_key(":wat::rete::core::somethingelse"));
    }

    /// ⚠ The literals in these four are deliberately NOT parenthesised forms, for the same reason
    /// the rest of this module's are: `no_inlined_wat_in_tests` reads any string literal that
    /// parses as a list with a keyword or symbol head as an inlined world. A field vector is a
    /// `[…]`, a commented-out declaration reads as nothing, and `field_vector_of` takes the items
    /// already split — so the parser's three units are each reachable without one.

    #[test]
    fn a_field_vector_ending_in_a_nested_type_keeps_its_last_field() {
        assert_eq!(
            declared_field_names(
                "a <- :wat::core::i64 b <- (:wat::core::PersistentVector :- [:wat::rete::Yy])"
            ),
            Ok(vec!["a".to_string(), "b".to_string()])
        );
    }

    /// Both arms of the refusal, because they predict different mechanisms: a cell count that is
    /// not a multiple of three says the GROUPING is not `name <- Type`, while a wrong separator
    /// says the grouping held and the form is something else. Reporting either as an empty field
    /// list would drop the record's accessors silently.
    #[test]
    fn a_field_vector_this_parser_cannot_read_is_reported_not_skipped() {
        assert_eq!(
            declared_field_names("a :wat::core::i64"),
            Err("the field vector holds 2 cell(s), not a multiple of 3 (`name <- Type`)".to_string())
        );
        assert_eq!(
            declared_field_names("a :wat::core::i64 b"),
            Err("field `a` is followed by `:wat::core::i64`, not `<-`".to_string())
        );
        assert_eq!(
            declared_field_names(""),
            Err("the field vector holds 0 cell(s), not a multiple of 3 (`name <- Type`)".to_string())
        );
    }

    #[test]
    fn a_declaration_in_a_comment_declares_nothing() {
        let d = record_decls(";; (:wat::core::defrecord :wat::rete::Zz [a <- :wat::core::i64])\n");
        assert_eq!(d.fields, BTreeMap::new());
        assert_eq!(d.malformed, Vec::<String>::new());
    }

    #[test]
    fn the_registry_namespace_is_split_from_the_rest() {
        assert!(in_registry_namespace(":wat::rete::core::mapv"));
        assert!(in_registry_namespace(":wat::rete::holon::presence?"));
        assert!(!in_registry_namespace(":wat::rete::where"));
        assert!(!in_registry_namespace(":wat::rete::Session"));
    }
}
