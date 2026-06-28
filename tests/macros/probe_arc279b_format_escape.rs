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

use wat::freeze::{eval_in_frozen, startup_beside};
use wat::runtime::{Environment, Value};

fn eval_probe(world: &wat::freeze::FrozenWorld, fn_call: &str) -> Result<String, String> {
    let ast = wat::parse_one!(fn_call).map_err(|e| format!("parse: {e:?}"))?;
    let got = eval_in_frozen(&ast, world, &Environment::new())
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
    let world = startup_beside(file!()).expect("startup");
    let s = eval_probe(&world, "(:user::probe-1)")
        .expect("format with doubled braces must expand cleanly");
    assert_eq!(s, "{literal}", "{{{{ }}}} doubling renders one literal brace each; got {s:?}");
}

// Doubled braces mixed with a real placeholder.
#[test]
fn escape_doubled_braces_with_placeholder() {
    let world = startup_beside(file!()).expect("startup");
    let s = eval_probe(&world, "(:user::probe-2)")
        .expect("format with doubled braces + placeholder must expand cleanly");
    assert_eq!(s, "{x} = v", "literal {{x}} beside a live {{name}} placeholder; got {s:?}");
}

// A trailing literal close brace after a placeholder.
#[test]
fn escape_close_brace_after_placeholder() {
    let world = startup_beside(file!()).expect("startup");
    let s = eval_probe(&world, "(:user::probe-3)")
        .expect("format with placeholder + trailing }} must expand cleanly");
    assert_eq!(s, "v}", "placeholder then literal }}}} → close brace; got {s:?}");
}
