//! `:wat::core::symbol` — a symbol value from text, or from a namespace and a name.

use wat_macros::wat_intrinsic;

use crate::ast::WatAST;
use crate::span::Span;
use crate::value::{Environment, EvalBreak, SymbolTable, TrackedValue, Value};
use crate::runtime::{RuntimeError, RuntimeErrorKind, Provenance};

/// `(:wat::core::symbol text)` splits `text` at the first `/`.
/// `(:wat::core::symbol namespace name)` is that pair.
///
/// @added         1.0.0
/// @Purity        Pure
/// @Determinism   Deterministic
/// @Totality      Partial
/// @ExpandTime    Legal
/// @Category      Transform
/// @arg     args :wat::type::String the symbol text, or a namespace and a name
/// @ret     :wat::type::symbol a symbol value
/// @example (:wat::core::symbol "a.b/c") #=> a.b/c
#[wat_intrinsic(":wat::core::symbol")]
pub(crate) fn eval_symbol(
    args: &[WatAST],
    env: &Environment,
    sym: &SymbolTable,
    span: &Span,
) -> Result<TrackedValue, EvalBreak> {
    let eval_str = |node: &WatAST| -> Result<std::sync::Arc<String>, EvalBreak> {
        match crate::runtime::eval(node, env, sym)?.value_owned() {
            Value::String(s) => Ok(s),
            other => Err(RuntimeError::new(
                node.span().clone(),
                RuntimeErrorKind::TypeMismatch {
                    op: ":wat::core::symbol".into(),
                    expected: "String",
                    got: Box::new(crate::value::ValueSnapshot::of(&other)),
                },
            )
            .into()),
        }
    };
    let name = match args {
        [text] => {
            let s = eval_str(text)?;
            crate::scope::Name::from_symbol_text(s.as_str())
        }
        [ns, name] => {
            let ns = eval_str(ns)?;
            let name = eval_str(name)?;
            crate::scope::Name::from_ns_and_name(ns.as_str(), name.as_str())
        }
        _ => {
            return Err(RuntimeError::new(
                span.clone(),
                RuntimeErrorKind::ArityMismatch {
                    op: ":wat::core::symbol".into(),
                    expected: 1,
                    got: args.len(),
                },
            )
            .into());
        }
    };
    let name = name.ok_or_else(|| {
        RuntimeError::new(
            span.clone(),
            RuntimeErrorKind::MalformedForm {
                head: ":wat::core::symbol".into(),
                reason: "symbol text uses a reserved namespace or is empty".into(),
            },
        )
    })?;
    Ok(TrackedValue::new(
        Value::symbol(name),
        Provenance::RuntimeBuilt {
            producer: ":wat::core::symbol",
            call_span: span.clone(),
        },
    ))
}
