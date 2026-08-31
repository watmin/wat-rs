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

use wat_macros::wat_intrinsic;

use crate::ast::WatAST;
use crate::runtime::{EvalBreak, Environment, RuntimeError, RuntimeErrorKind, SymbolTable, Value, ValueSnapshot};
use crate::span::Span;

/// `(:wat::rete::collect-rules ns) -> (:wat::core::PersistentVector :- [:wat::rete::Rule])`
///
/// Selects every zero-arg fn in namespace `ns` whose declared return type is `:wat::rete::Rule`, sorts by
/// name (deterministic rule order — the compiled network + differential oracle must be reproducible), invokes
/// each, and collects the resulting `Rule` values. Non-rule defns are excluded by the zero-arg + ret-type
/// filter; an empty PersistentVector is returned for a namespace with no rules (never raises).
///
/// Arc 255 Stone P6-c-W5c — moved verbatim into `#[wat_intrinsic]` with its real (1) arity
/// declared; the hand-rolled `args.len() != 1` guard this wave retires lived right here.
///
/// **Purity ground — this one surprised the ruling.** `ns` is evaluated by ordinary call-by-value
/// (not itself an effect). But the SELECTION step only checks shape (zero params, ret-type
/// `:wat::rete::Rule`) — it does NOT verify the body was produced by `defrule`'s
/// `(make-rule name (quote when) (quote then))` expansion, which is what makes a `defrule`-shaped
/// body side-effect-free (both vectors stay quoted, never evaluated). A hand-written zero-arg fn
/// with that same return type but an arbitrary body — `(:wat::core::do (:wat::io::print-line "x")
/// (:wat::rete::make-rule …))` — is picked up by the same filter and INVOKED via `eval_inner` on
/// a freshly built `(:ns::name)` call (line below), with nothing at this boundary bounding what
/// that body does. That is exactly the "unbounded caller-supplied code via apply_function /
/// eval_inner" shape `:wat::rete::eval-test`/`eval-insert` were ruled `Effectful` for (W5b) — the
/// difference is only that the code was defined earlier (as a `:ns::name` defn) rather than
/// passed as a literal argument on this call, not that it is any more bounded. Nothing this
/// verb's OWN body allocates outlives the call (the `names` Vec and the output `PVec` are both
/// local and returned/dropped normally, no cache/intern) — but the values it hands back to
/// callers are the return values of arbitrary invoked code, so `Effectful`/`Nondeterministic` is
/// the honest ruling on the reflect-and-invoke mechanism itself.
///
/// @added         1.0.0
/// @Purity        Effectful
/// @Determinism   Nondeterministic
/// @Totality         Unreviewed
/// @ExpandTime    Unreviewed
/// @Category      Combine
/// @arg     ns :wat::core::keyword the namespace to reflect (e.g. `:weather`); matches `"{ns}::"` as a prefix, subtree included
/// @ret     (:wat::core::PersistentVector :- [:wat::rete::Rule]) every discovered rule, sorted by name; empty if none
/// @example-norun (:wat::rete::collect-rules :probe::weather)
#[wat_intrinsic(":wat::rete::collect-rules")]
pub(crate) fn eval_collect_rules(
    ns: &WatAST,
    list_span: &Span,
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, EvalBreak> {
    const OP: &str = ":wat::rete::collect-rules";
    let ns_span = ns.span().clone();
    let ns_val = crate::runtime::eval_inner(ns, env, sym)?.value_owned();
    let ns = match ns_val {
        Value::wat__core__keyword(k) => k,
        other => {
            return Err(RuntimeError::new(ns_span, RuntimeErrorKind::TypeMismatch {
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
