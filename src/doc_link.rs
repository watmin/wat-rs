//! Shrink-only ledger of intra-doc links rustdoc cannot resolve.
//!
//! The floor runs `cargo doc` and then `doc_link_ledger`. This module is the
//! one list and the one comparison. A link not in the list is red. A listed
//! link that now resolves is red. The list only shrinks.
//!
//! `cargo doc` documents library crates, not tests, so the ledger lives here.

use std::collections::BTreeMap;

/// `(file, link-target, sites-in-that-file)`. Seeded by arc 278 E3 and
/// re-seeded at replay #270 to this tree's own 26 keys. `src/value/signal.rs`
/// is absent on purpose.
pub const KNOWN_BROKEN_DOC_LINKS: &[(&str, &str, usize)] = &[
    ("crates/wat-reader/src/parser.rs", "parse_all", 2),
    ("crates/wat-reader/src/parser.rs", "parse_one", 2),
    ("src/bin/cargo-wat.rs", "1", 2),
    ("src/channel/mod.rs", "crate::io::PipeWriter::write_all", 1),
    ("src/check.rs", "CheckError::BareLegacyPrimitive", 1),
    ("src/check/env.rs", "register", 1),
    ("src/check/env.rs", "register_overlay", 1),
    ("src/check/error_edn.rs", "CheckErrorKind", 1),
    ("src/config.rs", "DEFAULT_DIMS", 2),
    ("src/freeze.rs", "RuntimeError::EvalVerificationFailed", 2),
    ("src/kernel/address.rs", "feedback_dont_build_the_forcing_function", 1),
    ("src/kernel/address.rs", "feedback_vended_primitives_never_deadlock", 2),
    ("src/kernel/listener.rs", "feedback_vended_primitives_never_deadlock", 1),
    ("src/macros/expand.rs", "expand::expand_form", 1),
    ("src/macros/mod.rs", "ScopeId", 1),
    ("src/resolve/mod.rs", "RESERVED_PREFIXES", 1),
    ("src/resolve/mod.rs", "SymbolTable", 1),
    ("src/resolve/mod.rs", "check_form", 1),
    ("src/runtime.rs", "RuntimeError::ArityMismatch", 1),
    ("src/services/client.rs", "docs/ZERO-MUTEX.md", 1),
    ("src/types.rs", "crate::macros::MacroRegistry::register_stdlib", 1),
    ("src/types.rs", "crate::resolve::RESERVED_PREFIXES", 1),
    ("src/value/environment.rs", "SymbolTable", 1),
    ("src/value/mod.rs", "must_use", 1),
    ("src/value/symbol_table.rs", "2", 1),
    ("src/value/symbol_table.rs", "RuntimeError::NoEncodingCtx", 1),
];

/// Extract `(file, link-target) -> sites` from a cargo/rustdoc log.
///
/// ```text
/// warning: unresolved link to `parse_one`
///   --> crates/wat-reader/src/parser.rs:10:9
/// ```
///
/// Line and column are discarded. They move when anything above them is edited.
pub fn unresolved_links(output: &str) -> BTreeMap<(String, String), usize> {
    const HEAD: &str = "warning: unresolved link to `";
    let mut found: BTreeMap<(String, String), usize> = BTreeMap::new();
    let mut lines = output.lines();
    while let Some(line) = lines.next() {
        let Some(rest) = line.strip_prefix(HEAD) else {
            continue;
        };
        let Some(target) = rest.strip_suffix('`') else {
            continue;
        };
        let Some(loc) = lines.next() else {
            break;
        };
        let Some(loc) = loc.trim_start().strip_prefix("--> ") else {
            continue;
        };
        let file = loc.split(':').next().unwrap_or(loc);
        *found
            .entry((file.to_string(), target.to_string()))
            .or_insert(0) += 1;
    }
    found
}

/// Compare a cargo-doc log to [`KNOWN_BROKEN_DOC_LINKS`].
///
/// `Ok(())` when the named set matches, including site counts. `Err` is the
/// report `scripts/floor.sh` prints. It names arrivals, site-count moves, and
/// departures.
pub fn judge(output: &str) -> Result<(), String> {
    let found = unresolved_links(output);
    let known: BTreeMap<(String, String), usize> = KNOWN_BROKEN_DOC_LINKS
        .iter()
        .map(|(f, t, n)| (((*f).to_string(), (*t).to_string()), *n))
        .collect();

    let mut arrivals: Vec<String> = Vec::new();
    let mut departures: Vec<String> = Vec::new();
    let mut moved: Vec<String> = Vec::new();

    for (key, sites) in &found {
        match known.get(key) {
            None => arrivals.push(format!("{}: [`{}`] × {sites}", key.0, key.1)),
            Some(n) if n != sites => moved.push(format!(
                "{}: [`{}`] — ledger says {n} site(s), rustdoc found {sites}",
                key.0, key.1
            )),
            Some(_) => {}
        }
    }
    for key in known.keys() {
        if !found.contains_key(key) {
            departures.push(format!("{}: [`{}`]", key.0, key.1));
        }
    }

    let mut report = String::new();
    if !arrivals.is_empty() {
        report.push_str(&format!(
            "\n{} NEW broken intra-doc link(s) — not in KNOWN_BROKEN_DOC_LINKS:\n  {}\n\
             \n  FIX THE LINK. Do NOT add a line to this ledger: it is a shrink-only ratchet, and \
             adding to it is the laundering the gate exists to refuse.\n",
            arrivals.len(),
            arrivals.join("\n  ")
        ));
    }
    if !moved.is_empty() {
        report.push_str(&format!(
            "\n{} listed link(s) changed site count:\n  {}\n\
             \n  MORE sites than listed means a new broken citation of an already-broken target — \
             fix it. FEWER means you fixed one of several — update that line's count, or delete \
             the line if it reached zero.\n",
            moved.len(),
            moved.join("\n  ")
        ));
    }
    if !departures.is_empty() {
        report.push_str(&format!(
            "\n{} listed link(s) now RESOLVE — the ledger is stale:\n  {}\n\
             \n  Delete these lines from KNOWN_BROKEN_DOC_LINKS. A ledger that keeps names of \
             debt already paid stops describing anything, and the next reader cannot tell which \
             entries are real.\n",
            departures.len(),
            departures.join("\n  ")
        ));
    }
    if report.is_empty() {
        Ok(())
    } else {
        Err(format!(
            "broken intra-doc links moved away from the frozen ledger:\n{report}\n\
             Instrument (run from the repo root):\n  \
             RUSTDOCFLAGS=\"-W rustdoc::broken_intra_doc_links\" cargo doc --release --no-deps --workspace\n"
        ))
    }
}
