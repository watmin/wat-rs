//! Macro span-fidelity — a macro-CONSTRUCTED node must carry the CALL-SITE span.
//!
//! The bridge constructors (`keyword/from-string`, `keyword-node`, `symbol-node`, `read-string`)
//! stamp `wat::rust_caller_span!()` on the nodes they build (edn/render.rs). So a node a macro *constructs*
//! (vs splices from the template or via `with-children`, which carry real spans) lands in the
//! expansion with no span — and a type error on it points at "unknown" instead of the user's
//! macro call. `restamp_unknown_spans` (applied at `expand_macro_call`'s return) fills every
//! `is_unknown()` hole in a macro's result with the call-site span.
//!
//! Observable: a macro builds a keyword node via `keyword/from-string`; `macroexpand-1` the call,
//! `ast-span` the constructed node, read `:line`. RED at HEAD: the node carries `Span::unknown`
//! → line 0. GREEN after: the call-site line (> 0).
//!
//! Wat source lives in the co-located fixture: probe_arc209_macro_span_fidelity.wat
//! (slurped via startup_beside(file!())).
//!
//! Run: cargo test --release -p wat --test probe_arc209_macro_span_fidelity

use wat::freeze::call_beside_value;
use wat::runtime::Value;

// just-eval (rubric): the probe is a zero-arg entry fn in the co-located fixture, driven via
// call_beside_value — no inline wat driver expression.

#[test]
fn macro_constructed_node_carries_call_site_span() {
    let got = call_beside_value(file!(), ":user::probe-line")
        .expect("startup should succeed (macro span-fidelity probe)");
    match got {
        Value::i64(line) => assert!(
            line > 0,
            "expected the macro-constructed keyword node to carry the call-site line (> 0); got \
             {line} (Span::unknown — the diagnostic gap)"
        ),
        other => panic!("expected an i64 line; got {other:?}"),
    }
}
