//! Arc 279.1 — disconfirming probe: `format` has no `{{`/`}}` literal-brace escape (RED at HEAD).
//!
//! The resolved design (279 DESIGN): a literal brace in a template is written DOUBLED —
//! `{{` → `{`, `}}` → `}` — collapsed by the format MACRO at expand time (zero lexer change;
//! `{`/`}` are ordinary string chars). A single `{`/`}` that is not part of a placeholder or a
//! double is a macro-error.
//!
//! At HEAD the parser splits naively by `{` then `}` (wat/core.wat:597-705). `"{{x}}"` splits by
//! `{` into `["", "", "x}}"]`; the empty chunk trips the `n-cp >= 2` "unclosed `{`" guard → the
//! macro ERRORS at startup → RED. GREEN when 279.1 ships the char-walk tokenizer that collapses
//! the doubles.
//!
//! Probe pattern: format is a MACRO — embed it in a defn body so startup_from_source expands it,
//! then call the compiled defn via eval_in_frozen (mirrors probe_arc279_format.rs).
//!
//! Run: cargo test --release -p wat --test probe_arc279b_format_escape -- --include-ignored

use std::sync::Arc;
use wat::freeze::{eval_in_frozen, startup_from_source};
use wat::load::InMemoryLoader;
use wat::runtime::{Environment, Value};

fn eval_format(body_expr: &str) -> Result<String, String> {
    let src = format!(
        "(:wat::core::defn :user::probe [] -> :wat::core::String {body_expr})\n\
         (:wat::core::defn :user::main [] -> :wat::core::nil nil)"
    );
    let world = startup_from_source(&src, None, Arc::new(InMemoryLoader::new()))
        .map_err(|e| format!("startup: {e:?}"))?;
    let ast = wat::parse_one!("(:user::probe)").map_err(|e| format!("parse: {e:?}"))?;
    let got = eval_in_frozen(&ast, &world, &Environment::new())
        .map_err(|e| format!("eval: {e:?}"))?
        .value_owned();
    match got {
        Value::String(ref s) => Ok(s.to_string()),
        other => Err(format!("probe must return a String; got {other:?}")),
    }
}

// `{{` and `}}` with no placeholder → literal braces.
#[test]
fn escape_doubled_braces_render_literal() {
    let s = eval_format(r#"(:wat::core::format "{{literal}}")"#)
        .expect("format with doubled braces must expand cleanly");
    assert_eq!(s, "{literal}", "{{{{ }}}} doubling renders one literal brace each; got {s:?}");
}

// Doubled braces mixed with a real placeholder.
#[test]
fn escape_doubled_braces_with_placeholder() {
    let s = eval_format(r#"(:wat::core::format "{{x}} = {name}" :name "v")"#)
        .expect("format with doubled braces + placeholder must expand cleanly");
    assert_eq!(s, "{x} = v", "literal {{x}} beside a live {{name}} placeholder; got {s:?}");
}

// A trailing literal close brace after a placeholder.
#[test]
fn escape_close_brace_after_placeholder() {
    let s = eval_format(r#"(:wat::core::format "{name}}}" :name "v")"#)
        .expect("format with placeholder + trailing }} must expand cleanly");
    assert_eq!(s, "v}", "placeholder then literal }}}} → close brace; got {s:?}");
}
