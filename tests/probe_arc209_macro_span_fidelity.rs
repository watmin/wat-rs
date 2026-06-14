//! Macro span-fidelity — a macro-CONSTRUCTED node must carry the CALL-SITE span.
//!
//! The bridge constructors (`keyword/from-string`, `keyword-node`, `symbol-node`, `read-string`)
//! stamp `Span::unknown()` on the nodes they build (edn_shim.rs). So a node a macro *constructs*
//! (vs splices from the template or via `with-children`, which carry real spans) lands in the
//! expansion with no span — and a type error on it points at "unknown" instead of the user's
//! macro call. `restamp_unknown_spans` (applied at `expand_macro_call`'s return) fills every
//! `is_unknown()` hole in a macro's result with the call-site span.
//!
//! Observable: a macro builds a keyword node via `keyword/from-string`; `macroexpand-1` the call,
//! `ast-span` the constructed node, read `:line`. RED at HEAD: the node carries `Span::unknown`
//! → line 0. GREEN after: the call-site line (> 0).
//!
//! Run: cargo test --release -p wat --test probe_arc209_macro_span_fidelity

use std::sync::Arc;
use wat::freeze::{eval_in_frozen, startup_from_source};
use wat::load::InMemoryLoader;
use wat::runtime::{Environment, Value};

const PROGRAM: &str = r#"
;; keyword-node returns a Value::wat__WatAST NODE with Span::unknown baked in (edn_shim:581);
;; value_to_watast passes such a node through DIRECT, so the unknown span survives — the gap.
;; (keyword/from-string returns a keyword VALUE, auto-stamped call-site by value_to_watast — no gap.)
(:wat::core::defmacro :user::mk-kw [] -> :wat::WatAST
  (:wat::core::keyword-node ":foo"))

(:wat::core::defn :user::probe-line [] -> :wat::core::i64
  (:wat::core::Option/expect -> :wat::core::i64
    (:wat::core::HashMap/get
      (:wat::core::ast-span (:wat::core::macroexpand-1 (:wat::core::quote (:user::mk-kw))))
      :line)
    "ast-span should carry :line"))

(:wat::core::defn :user::main [] -> :wat::core::nil nil)
"#;

#[test]
fn macro_constructed_node_carries_call_site_span() {
    let world = startup_from_source(PROGRAM, None, Arc::new(InMemoryLoader::new()))
        .expect("startup should succeed (macro span-fidelity probe)");
    let ast = wat::parse_one!("(:user::probe-line)").expect("parse");
    let got = eval_in_frozen(&ast, &world, &Environment::new())
        .map(|tv| tv.value_owned())
        .unwrap_or_else(|e| panic!("probe-line raised: {e:?}"));
    match got {
        Value::i64(line) => assert!(
            line > 0,
            "expected the macro-constructed keyword node to carry the call-site line (> 0); got \
             {line} (Span::unknown — the diagnostic gap)"
        ),
        other => panic!("expected an i64 line; got {other:?}"),
    }
}
