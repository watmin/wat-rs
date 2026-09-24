//! `wat --check` diagnostic rendering: the output-format enum + the
//! text/EDN/JSON renderers `run_with_args` dispatches to on a freeze
//! failure. Split out of `distribution/mod.rs` (arc 170) — a self-
//! contained rendering concern, distinct from argv parsing and the
//! fork/proxy/reap run path.

/// Arc 115 slice 1 — output format for `wat --check` diagnostics.
/// Default (None) is text via stderr; `--check-output edn` emits EDN
/// records on stdout (one per diagnostic, line-delimited per arc 092
/// v4); `--check-output json` emits JSON records on stdout (same
/// shape, JSON encoding via wat-edn's sentinel-tagged-object
/// convention).
#[derive(Debug, Clone, Copy)]
pub(super) enum CheckOutputFormat {
    Edn,
    Json,
}

/// Emit `--check` failure diagnostics in the requested format.
///
/// **Data first.** All three modes consume the same source:
/// `StartupError::to_edn_values()` (arc 296 — one `OwnedValue` per
/// finding). Renderers vary; data shape is shared.
///
/// - **Text mode** (default): writes the StartupError's Display to
///   stderr — same shape `wat <file>` shows on freeze failure.
/// - **EDN mode**: prefixes each error record with a `:file` field
///   identifying the entry path, then emits one EDN record per error
///   to stdout (line-delimited; arc 092 v4 wire format).
/// - **JSON mode**: same record-per-error shape; JSON encoding via
///   `wat_edn::to_json_string` (sentinel-tagged-object convention).
///
/// Tagged EDN envelope: `#wat.kernel/<VariantName> {:file "..." :callee "..." ...}`.
/// JSON envelope: `{"#tag":"wat.kernel/VariantName","body":{":file":"...",...}}`.
pub(super) fn emit_check_failure(
    entry_path: &str,
    err: &crate::freeze::StartupError,
    format: Option<CheckOutputFormat>,
) {
    match format {
        None => {
            eprintln!("{}", err);
        }
        Some(CheckOutputFormat::Edn) => {
            for edn in err.to_edn_values() {
                let with_file = prepend_file_field(edn, entry_path);
                println!("{}", wat_edn::write(&with_file));
            }
        }
        Some(CheckOutputFormat::Json) => {
            for edn in err.to_edn_values() {
                let with_file = prepend_file_field(edn, entry_path);
                println!("{}", wat_edn::to_json_string(&with_file));
            }
        }
    }
}

/// Prepend a `:file "path"` field to the body of a tagged OwnedValue.
/// When the body is a Map, inserts at position 0. When it is not a Map,
/// wraps the body in a map with a `:value` key.
fn prepend_file_field(edn: wat_edn::OwnedValue, file: &str) -> wat_edn::OwnedValue {
    use std::borrow::Cow;
    use wat_edn::{Keyword, OwnedValue};

    let file_entry = (
        OwnedValue::Keyword(Keyword::new("file")),
        OwnedValue::String(Cow::Owned(file.to_owned())),
    );

    match edn {
        OwnedValue::Tagged(tag, body) => {
            let mut fields = match *body {
                OwnedValue::Map(m) => m,
                other => vec![(OwnedValue::Keyword(Keyword::new("value")), other)],
            };
            fields.insert(0, file_entry);
            OwnedValue::Tagged(tag, Box::new(OwnedValue::Map(fields)))
        }
        other => OwnedValue::Map(vec![
            file_entry,
            (OwnedValue::Keyword(Keyword::new("value")), other),
        ]),
    }
}

/// the-little-wat excursus 002 stone 2 — print what `crate::check::type_record` recorded, one
/// node per line on stdout, TAB-separated (a type can hold spaces), sorted by position, a line
/// said twice said once:
///
/// ```text
/// TYPE <file> <line> <col> <type> <wide>         the type infer gave the node starting there
/// UNRESOLVED <file> <line> <col> <type> <wide>   a variable survived the final substitution
/// TYPES recorded <n> distinct <d> spans <s> multi <m> unresolved <u> orphans <o> check-errors <e>
/// ```
///
/// Positions are the reader's: 1-based line and column of the node's first character (a list
/// begins at its `(`), columns in characters. `<type>` is `format_type` of the type after the
/// final substitution with aliases expanded; `<wide>` is the same type with every variant
/// widened to its enum (`Recorded::wide`). `multi` counts spans that carry more than one
/// distinct type -- every one is printed, none is chosen. `check-errors` is the number of
/// diagnostics the recording check itself produced (their text goes to stderr).
pub(super) fn emit_types(rec: &crate::check::type_record::Recording, errors: usize) {
    use std::collections::{BTreeMap, BTreeSet};
    use std::io::Write;
    let mut seen: BTreeSet<(String, i64, i64, bool, String, String)> = BTreeSet::new();
    for r in &rec.types {
        seen.insert((
            r.span.file.as_str().to_string(),
            r.span.line,
            r.span.col,
            r.unresolved,
            crate::check::format_type(&r.ty),
            crate::check::format_type(&r.wide),
        ));
    }
    let mut per_span: BTreeMap<(&str, i64, i64), usize> = BTreeMap::new();
    let mut unresolved = 0usize;
    let out = std::io::stdout();
    let mut out = out.lock();
    for (file, line, col, unres, ty, wide) in &seen {
        *per_span.entry((file.as_str(), *line, *col)).or_insert(0) += 1;
        if *unres {
            unresolved += 1;
        }
        let tag = if *unres { "UNRESOLVED" } else { "TYPE" };
        let _ = writeln!(out, "{tag}\t{file}\t{line}\t{col}\t{ty}\t{wide}");
    }
    let multi = per_span.values().filter(|n| **n > 1).count();
    let _ = writeln!(
        out,
        "TYPES recorded {} distinct {} spans {} multi {} unresolved {} orphans {} check-errors {}",
        rec.types.len(),
        seen.len(),
        per_span.len(),
        multi,
        unresolved,
        rec.orphans,
        errors
    );
}
