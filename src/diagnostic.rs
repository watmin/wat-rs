//! Structured diagnostic data — arc 115 slice 1.
//!
//! **The substrate's errors are data first; renderers are layered.**
//! `Display` impls produce the canonical text form; this module
//! exposes the SAME information as a structured `Diagnostic`
//! record with named fields. Tooling (the `wat --check` CLI's
//! `--check-output edn`/`json` modes; future LSP servers; agent
//! orchestrators) consumes the data directly without parsing
//! Display strings.
//!
//! User direction (2026-04-30):
//!
//! > the thing building the error context needs to compose it as
//! > data.. and then we can choose how to render this.. raw text,
//! > edn, json
//! >
//! > we are data first - always
//!
//! Each error variant in [`CheckError`] / [`StartupError`]
//! corresponds to one `Diagnostic` with `kind` = the variant name
//! and field-name → field-value pairs that mirror the Rust struct
//! fields. Multiple errors → multiple `Diagnostic` records (one
//! per).

use std::fmt::Write;

/// One structured error record — kind discriminator + named fields.
///
/// Fields preserve insertion order so renderers produce stable output.
#[derive(Debug, Clone)]
pub struct Diagnostic {
    /// Variant name from the source error type. Examples:
    /// `"TypeMismatch"`, `"CommCallOutOfPosition"`, `"Parse"`,
    /// `"Macro"`. Renderers map this to a tag (EDN
    /// `#wat.diag/<kind>`) or a `kind` field (JSON).
    pub kind: String,
    /// Field name → field value. Order-preserving Vec rather than
    /// HashMap so renderers produce stable output.
    pub fields: Vec<(String, DiagnosticValue)>,
}

/// Value-type for diagnostic fields. Slice-1 minimal: strings and
/// optional integers (line / col when known). Future slices may
/// widen (nested Diagnostics for chained-cause; lists for
/// match-arm coverage; etc.).
#[derive(Debug, Clone)]
pub enum DiagnosticValue {
    String(String),
    Int(i64),
}

impl Diagnostic {
    pub fn new(kind: impl Into<String>) -> Self {
        Self {
            kind: kind.into(),
            fields: Vec::new(),
        }
    }

    pub fn field(mut self, name: impl Into<String>, value: impl Into<DiagnosticValue>) -> Self {
        self.fields.push((name.into(), value.into()));
        self
    }
}

impl From<String> for DiagnosticValue {
    fn from(s: String) -> Self {
        Self::String(s)
    }
}

impl From<&str> for DiagnosticValue {
    fn from(s: &str) -> Self {
        Self::String(s.to_string())
    }
}

impl From<i64> for DiagnosticValue {
    fn from(n: i64) -> Self {
        Self::Int(n)
    }
}

impl From<usize> for DiagnosticValue {
    fn from(n: usize) -> Self {
        Self::Int(n as i64)
    }
}

/// Render a diagnostic as one line of EDN (arc 092 v4 wire shape).
/// Tag form: `#wat.diag/<kind> {:field1 "value1" :field2 42 ...}`.
pub fn render_edn(diag: &Diagnostic) -> String {
    let mut out = String::with_capacity(64 + diag.fields.len() * 32);
    write!(&mut out, "#wat.diag/{} {{", diag.kind).expect("write");
    let mut first = true;
    for (name, value) in &diag.fields {
        if !first {
            out.push(' ');
        }
        first = false;
        write!(&mut out, ":{} ", name).expect("write");
        render_edn_value(value, &mut out);
    }
    out.push('}');
    out
}

fn render_edn_value(value: &DiagnosticValue, out: &mut String) {
    match value {
        DiagnosticValue::String(s) => {
            render_edn_string(s, out);
        }
        DiagnosticValue::Int(n) => {
            write!(out, "{}", n).expect("write");
        }
    }
}

fn render_edn_string(s: &str, out: &mut String) {
    out.push('"');
    for c in s.chars() {
        match c {
            '\\' => out.push_str("\\\\"),
            '"' => out.push_str("\\\""),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            other => out.push(other),
        }
    }
    out.push('"');
}

/// Render a diagnostic as one line of JSON.
/// Object form: `{"kind":"<kind>","field1":"value1","field2":42,...}`.
pub fn render_json(diag: &Diagnostic) -> String {
    let mut out = String::with_capacity(64 + diag.fields.len() * 32);
    out.push('{');
    write!(&mut out, "\"kind\":").expect("write");
    render_json_string(&diag.kind, &mut out);
    for (name, value) in &diag.fields {
        out.push(',');
        render_json_string(name, &mut out);
        out.push(':');
        render_json_value(value, &mut out);
    }
    out.push('}');
    out
}

fn render_json_value(value: &DiagnosticValue, out: &mut String) {
    match value {
        DiagnosticValue::String(s) => {
            render_json_string(s, out);
        }
        DiagnosticValue::Int(n) => {
            write!(out, "{}", n).expect("write");
        }
    }
}

fn render_json_string(s: &str, out: &mut String) {
    // JSON escape — same shape as EDN's minimal escape; both formats
    // accept the subset we emit.
    out.push('"');
    for c in s.chars() {
        match c {
            '\\' => out.push_str("\\\\"),
            '"' => out.push_str("\\\""),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            other if (other as u32) < 0x20 => {
                write!(out, "\\u{:04x}", other as u32).expect("write");
            }
            other => out.push(other),
        }
    }
    out.push('"');
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn render_edn_simple() {
        let d = Diagnostic::new("TypeMismatch")
            .field("callee", ":wat::core::let*")
            .field("expected", ":i64")
            .field("got", ":String");
        assert_eq!(
            render_edn(&d),
            r#"#wat.diag/TypeMismatch {:callee ":wat::core::let*" :expected ":i64" :got ":String"}"#
        );
    }

    #[test]
    fn render_json_simple() {
        let d = Diagnostic::new("ArityMismatch")
            .field("callee", ":wat::kernel::send")
            .field("expected", 2_i64)
            .field("got", 1_i64);
        assert_eq!(
            render_json(&d),
            r#"{"kind":"ArityMismatch","callee":":wat::kernel::send","expected":2,"got":1}"#
        );
    }

    #[test]
    fn escape_quotes_in_strings() {
        let d = Diagnostic::new("Foo").field("msg", r#"has "quotes" in it"#);
        let edn = render_edn(&d);
        assert!(edn.contains(r#"\"quotes\""#));
        let json = render_json(&d);
        assert!(json.contains(r#"\"quotes\""#));
    }
}
