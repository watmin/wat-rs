//! Arc 279.1 — disconfirming probe: `format` has no `{{`/`}}` literal-brace escape (RED at HEAD).
//!
//! The resolved design (279 DESIGN): a literal brace in a template is written DOUBLED —
//! `{{` → `{`, `}}` → `}` — collapsed by the format MACRO at expand time (zero lexer change;
//! `{`/`}` are ordinary string chars). A single `{`/`}` that is not part of a placeholder or a
//! double is a macro-error.
//!
//! Probe pattern: format is a MACRO — embed it in a defn body so startup expands it,
//! then call the compiled defn via eval_in_frozen (mirrors probe_arc279_format.rs).
//!
//! Wat source lives in the co-located fixture: probe_arc279b_format_escape.wat
//! (slurped via startup_beside(file!())).
//!
//! Run: cargo test --release -p wat --test probe_arc279b_format_escape -- --include-ignored

use wat::freeze::StartupError;
use wat::runtime::{RuntimeError, RuntimeErrorKind, Value, ValueSnapshot};

// just-eval (rubric): each probe is a zero-arg entry fn in the co-located fixture, driven via
// call_beside_value — no inline wat driver expression.
fn call_probe(fn_name: &str) -> Result<String, StartupError> {
    let got = wat::freeze::call_beside_value(file!(), fn_name)
        .map_err(|e| StartupError::Runtime(Box::new(e)))?;
    match got {
        Value::String(ref s) => Ok(s.to_string()),
        other => Err(StartupError::Runtime(Box::new(RuntimeError::new(
            wat::rust_caller_span!(),
            RuntimeErrorKind::TypeMismatch {
                op: fn_name.to_string(),
                expected: "String",
                got: Box::new(ValueSnapshot::of(&other)),
            },
        )))),
    }
}

// `{{` and `}}` with no placeholder → literal braces.
#[test]
fn escape_doubled_braces_render_literal() {
    let s = call_probe(":user::probe-1")
        .expect("format with doubled braces must expand cleanly");
    assert_eq!(s, "{literal}", "{{{{ }}}} doubling renders one literal brace each; got {s:?}");
}

// Doubled braces mixed with a real placeholder.
#[test]
fn escape_doubled_braces_with_placeholder() {
    let s = call_probe(":user::probe-2")
        .expect("format with doubled braces + placeholder must expand cleanly");
    assert_eq!(s, "{x} = v", "literal {{x}} beside a live {{name}} placeholder; got {s:?}");
}

// A trailing literal close brace after a placeholder.
#[test]
fn escape_close_brace_after_placeholder() {
    let s = call_probe(":user::probe-3")
        .expect("format with placeholder + trailing }} must expand cleanly");
    assert_eq!(s, "v}", "placeholder then literal }}}} → close brace; got {s:?}");
}
