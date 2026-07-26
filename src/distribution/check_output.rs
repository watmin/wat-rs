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
