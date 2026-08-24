//! Arc 278 Stone 5b — `collect-rules`: gather a namespace's `defrule`'d rules by reflecting the symbol table.
//!
//! `(:wat::rete::collect-rules <ns: :wat::core::keyword>) -> (:wat::core::PersistentVector :- [wat::rete::Rule])`
//!
//! `defrule` (stone 5a) expands to a zero-arg `defn` returning `:wat::rete::Rule` — the return type is the
//! discovery marker, exactly as `deftest` marks tests by returning `:wat::test::TestResult` and the test
//! runner's `discover_tests` (src/test_runner.rs) finds them by reflecting the frozen symbol table. There is
//! no wat-level "enumerate the defns in a namespace" primitive and (wat being pure) no mutable global rule
//! registry — so `collect-rules` does the same reflection at eval time. This is the "wat orchestrates Rust"
//! pattern: `defrule` plants the discoverable zero-arg `defn`s; `collect-rules` reflects + invokes them.

use crate::ast::WatAST;
use crate::runtime::{EvalBreak, Environment, RuntimeError, RuntimeErrorKind, SymbolTable, Value, ValueSnapshot};
use crate::span::Span;

/// `(:wat::rete::collect-rules <ns>) -> PersistentVector<Rule>`.
///
/// Selects every zero-arg fn in namespace `ns` whose declared return type is `:wat::rete::Rule`, sorts by
/// name (deterministic rule order — the compiled network + differential oracle must be reproducible), invokes
/// each, and collects the resulting `Rule` values. Non-rule defns are excluded by the zero-arg + ret-type
/// filter; an empty PersistentVector is returned for a namespace with no rules (never raises).
pub(crate) fn eval_collect_rules(
    args: &[WatAST],
    list_span: &Span,
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, EvalBreak> {
    const OP: &str = ":wat::rete::collect-rules";
    if args.len() != 1 {
        return Err(RuntimeError::new(list_span.clone(), RuntimeErrorKind::ArityMismatch {
            op: OP.into(),
            expected: 1,
            got: args.len(),
        }).into());
    }
    let ns_val = crate::runtime::eval_inner(&args[0], env, sym)?.value_owned();
    let ns = match ns_val {
        Value::wat__core__keyword(k) => k,
        other => {
            return Err(RuntimeError::new(args[0].span().clone(), RuntimeErrorKind::TypeMismatch {
                op: OP.into(),
                expected: ":wat::core::keyword (a namespace, e.g. :weather)",
                got: Box::new(ValueSnapshot::of(&other)),
            }).into());
        }
    };
    // Namespace boundary: "weather::" — the trailing "::" guards against ":weatherfoo::".
    // Colon-robust: strip a leading ':' from both the namespace and each fn-name key before comparing.
    let ns_bare = ns.strip_prefix(':').unwrap_or(&ns);
    let prefix = format!("{ns_bare}::");

    // Select zero-arg fns in the namespace whose return type is the `defrule` marker `:wat::rete::Rule`.
    let mut names: Vec<String> = sym
        .functions_iter()
        .filter(|(name, f)| {
            let bare = name.strip_prefix(':').unwrap_or(name);
            bare.starts_with(&prefix)
                && f.param_types.is_empty()
                && matches!(&f.ret_type, crate::types::TypeExpr::Path(p) if p == ":wat::rete::Rule")
        })
        .map(|(name, _)| name.clone())
        .collect();
    names.sort();

    // Invoke each zero-arg rule fn → its Rule value; collect into a PersistentVector.
    // The call keyword carries the leading ':' (the form `(:ns::name)` that 5a proved evaluates the fn).
    let mut out: crate::value::pvec::PVec = crate::value::pvec::PVec::new();
    for name in &names {
        let kw = if name.starts_with(':') { name.clone() } else { format!(":{name}") };
        let call = WatAST::List(vec![WatAST::Keyword(kw, list_span.clone())], list_span.clone());
        let rule = crate::runtime::eval_inner(&call, env, sym)?.value_owned();
        out.push_back_mut(rule);
    }
    Ok(Value::wat__core__PersistentVector(out))
}
