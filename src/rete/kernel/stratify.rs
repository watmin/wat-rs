//! Native type-stratum numbering (`native_stratify` / `rule_produces` / `rule_negates`).
//! Dual of `wat/rete/oracle/stratify.wat`. The public `fire-rules` door lives in `fire/`.

use std::collections::HashMap;

use crate::ast::WatAST;
use crate::rete::matcher::{classify_rete_clause, ReteClauseShape};
use crate::runtime::{EvalBreak, RuntimeError, RuntimeErrorKind, SymbolTable, Value};

// ── Arc 278 Stone 7-strat-native: STRATIFIED negation, native port ──────────────
//
// Faithful Rust port of the wat ORACLE's stratification (`wat/rete/oracle/stratify.wat`):
// `rule-produces` / `rule-negates` / `stratify-sweep` / `stratify-fix` / `rule-stratum` /
// `stratify` / `fire-stratified-loop` / `fire-stratified`. The oracle is the reference and
// does NOT change (`DESIGN-STONE-7strat-native.md`); this is a SEPARATE, self-contained Rust
// impl that moves in lockstep with it (the dual-impl doctrine — no `native?` flag anywhere).

/// A fact-form's type head, colon-stripped: `(:Type ...)` → `"Type"`.
/// Mirrors the inline `ast-name` + colon-strip done identically in both `rule-produces`
/// (`wat/rete/oracle/stratify.wat`) and `rule-negates` (`wat/rete/oracle/stratify.wat`).
pub(crate) fn fact_type_head(fact_form: &WatAST) -> Option<String> {
    if let WatAST::List(items, _) = fact_form {
        let raw = match items.first() {
            Some(WatAST::Keyword(k, _)) => k.clone(),
            Some(WatAST::Symbol(s, _)) => s.as_str().to_string(),
            _ => return None,
        };
        return Some(raw.trim_start_matches(':').to_string());
    }
    None
}

/// Extract the produced type FQDNs from a Rule's RHS forms.
/// Arc 278 Stone A: each RHS form IS the fact-form directly (the `:wat::rete::insert` wrapper
/// is gone) — no more unwrapping a second child. Mirrors `rule-produces` (`wat/rete/oracle/stratify.wat`).
pub(crate) fn rule_produces(rhs: &[WatAST], sym: &SymbolTable) -> Vec<String> {
    let mut out = Vec::new();
    for form in rhs {
        if let Some(name) = produced_type(form, sym) {
            out.push(name);
        }
    }
    out
}

/// Constructor head stays the class. A fn-headed `:then` produces its
/// declared return type (the fact `T` another rule can consume).
pub(crate) fn produced_type(form: &WatAST, sym: &SymbolTable) -> Option<String> {
    let head = fact_type_head(form)?;
    let path = if head.starts_with(':') {
        head.clone()
    } else {
        format!(":{head}")
    };
    if let Some(func) = sym.get(&path) {
        if let crate::types::TypeExpr::Path(p) = &func.ret_type {
            let t = p.trim_start_matches(':');
            if !t.is_empty() && !t.starts_with("wat::core::") {
                return Some(t.to_string());
            }
        }
    }
    Some(head)
}

/// Extract the negated type FQDNs from a Rule's LHS conditions.
/// `(:not <fact>)` and `(:not (:and/:or …))` both raise: the leaf types under
/// the combinator are the edges, not `"wat::rete::and"`. Walk via
/// `classify_rete_clause`. Positive `:exists` / accumulate / `:where` are not
/// negation edges (those are `rule_consumes`).
pub(crate) fn rule_negates(lhs: &[WatAST]) -> Vec<String> {
    let mut out = Vec::new();
    for form in lhs {
        negate_types(form, &mut out, false);
    }
    out
}

pub(crate) fn negate_types(form: &WatAST, out: &mut Vec<String>, under_not: bool) {
    match classify_rete_clause(form) {
        ReteClauseShape::Not(inner) => negate_types(inner, out, true),
        ReteClauseShape::And(xs) | ReteClauseShape::Or(xs) => {
            for x in xs {
                negate_types(x, out, under_not);
            }
        }
        ReteClauseShape::FactBind { type_head, .. } if under_not => {
            out.push(type_head.to_string());
        }
        ReteClauseShape::Unrecognized if under_not => {
            if let Some(name) = fact_type_head(form) {
                if !name.starts_with('?') && !name.starts_with("wat::rete::") {
                    out.push(name);
                }
            }
        }
        _ => {}
    }
}

/// The stratifier's dependency view of one rule.
/// `consumed` is task #94 — without it a rule that reads a higher-stratum fact sits too low.
/// `exists_and_from_types` is exists-inner / acc `:from` (+1 like negation when the type is derived).
#[derive(Clone, Debug)]
pub(crate) struct StratifyView {
    pub produced: Vec<String>,
    pub negated: Vec<String>,
    pub consumed: Vec<String>,
    pub exists_and_from_types: Vec<String>,
}

/// A compiled rule paired with its stratify view.
#[derive(Clone)]
pub(crate) struct RuleParts {
    pub rule: Value,
    pub produced: Vec<String>,
    pub negated: Vec<String>,
    pub consumed: Vec<String>,
    pub exists_and_from_types: Vec<String>,
}

impl RuleParts {
    pub(crate) fn view(&self) -> StratifyView {
        StratifyView {
            produced: self.produced.clone(),
            negated: self.negated.clone(),
            consumed: self.consumed.clone(),
            exists_and_from_types: self.exists_and_from_types.clone(),
        }
    }
}

/// The fact types a rule reads POSITIVELY (task #94 — the input the stratifier never had).
///
/// Correct stratification needs BOTH `stratum(r) >= stratum(p)` for positively-used `p` and
/// `stratum(r) > stratum(p)` for negated `p`. Only the second existed, so a rule consuming a
/// fact produced in a HIGHER stratum was left LOWER, fired before its input existed, and never
/// re-fired. `:not` / `:where` are not positive reads. `:exists` inner and accumulate
/// `:from` ARE — they were dropped as engine-form prefixes and the `:from` head
/// leaked as `"?n"`. Walk via `classify_rete_clause`.
pub(crate) fn rule_consumes(lhs: &[WatAST]) -> Vec<String> {
    let mut out = Vec::new();
    for form in lhs {
        consume_types(form, &mut out);
    }
    out
}

/// Exists-inner and accumulate `:from` types. Stratify +1 (closed bag).
pub(crate) fn rule_bag_consumes(lhs: &[WatAST]) -> Vec<String> {
    let mut out = Vec::new();
    for form in lhs {
        bag_types(form, &mut out);
    }
    out
}

pub(crate) fn bag_types(form: &WatAST, out: &mut Vec<String>) {
    match classify_rete_clause(form) {
        ReteClauseShape::Exists(inner) => consume_types(inner, out),
        ReteClauseShape::Accumulate { from, .. } => consume_types(from, out),
        ReteClauseShape::And(xs) | ReteClauseShape::Or(xs) => {
            for x in xs {
                bag_types(x, out);
            }
        }
        _ => {}
    }
}

pub(crate) fn consume_types(form: &WatAST, out: &mut Vec<String>) {
    match classify_rete_clause(form) {
        ReteClauseShape::Exists(inner) => consume_types(inner, out),
        ReteClauseShape::Accumulate { from, .. } => consume_types(from, out),
        ReteClauseShape::And(xs) | ReteClauseShape::Or(xs) => {
            for x in xs {
                consume_types(x, out);
            }
        }
        ReteClauseShape::FactBind { type_head, .. } => {
            out.push(type_head.to_string());
        }
        ReteClauseShape::Not(_)
        | ReteClauseShape::Where(_)
        | ReteClauseShape::Bind { .. }
        | ReteClauseShape::Constraint { .. } => {}
        ReteClauseShape::Unrecognized => {
            if let Some(name) = fact_type_head(form) {
                if !name.starts_with('?') {
                    out.push(name);
                }
            }
        }
    }
}

/// One sweep over all rules' (produced, negated, consumed) triples, raising `type_strata` entries.
/// For each rule: `required = max(stratum[n]+1 for n in negated, default 0)`; for each produced
/// type `p`: `stratum[p] = max(stratum[p], required)`. Returns `true` iff any stratum rose.
/// Mirrors `stratify-sweep` (`wat/rete/oracle/stratify.wat`).
pub(crate) fn native_stratify_sweep(rule_parts: &[StratifyView], type_strata: &mut HashMap<String, i64>) -> bool {
    let mut changed = false;
    for view in rule_parts {
        let mut required = 0i64;
        for n in &view.negated {
            let v = *type_strata.get(n).unwrap_or(&0) + 1;
            if v > required {
                required = v;
            }
        }
        // exists / acc :from of a type THIS SET derives: +1 (closed bag).
        // Inserted-only bag types stay +0 so the unstratified path survives.
        // A rule that both produces and bags `b` (userfn-head gather that
        // returns the same type) is a self-cycle — do not count it as derived.
        for b in &view.exists_and_from_types {
            let derived = rule_parts.iter().any(|other| {
                other.produced.iter().any(|t| t == b) && !other.exists_and_from_types.iter().any(|t| t == b)
            });
            let v = *type_strata.get(b).unwrap_or(&0) + i64::from(derived);
            if v > required {
                required = v;
            }
        }
        // req-pos: a positive consumer may share its input's stratum but never sit BELOW it.
        // NOT +1 — same-stratum forward chaining is ordinary and must stay allowed.
        for c in &view.consumed {
            let v = *type_strata.get(c).unwrap_or(&0);
            if v > required {
                required = v;
            }
        }
        for p in &view.produced {
            let cur = *type_strata.get(p).unwrap_or(&0);
            if required > cur {
                type_strata.insert(p.clone(), required);
                changed = true;
            }
        }
    }
    changed
}

/// Recursive fixpoint for stratification: sweeps until converged or `remaining` runs out.
/// A negation cycle (non-terminating strata) raises the same "not stratifiable" error the
/// oracle raises. Mirrors `stratify-fix` (`wat/rete/oracle/stratify.wat`).
pub(crate) fn native_stratify_fix(
    rule_parts: &[StratifyView],
    mut type_strata: HashMap<String, i64>,
    mut remaining: i64,
) -> Result<HashMap<String, i64>, EvalBreak> {
    loop {
        let changed = native_stratify_sweep(rule_parts, &mut type_strata);
        if !changed {
            return Ok(type_strata);
        }
        if remaining <= 0 {
            return Err(RuntimeError::new(
                crate::rust_caller_span!(),
                RuntimeErrorKind::MalformedForm {
                    head: ":wat::rete::fire-rules".into(),
                    reason: "stratify: negation cycle detected — rule set is not stratifiable"
                        .into(),
                },
            )
            .into());
        }
        remaining -= 1;
    }
}

/// Compute the type→stratum map for a rule set (`length(rules)+1` sweeps is always enough for
/// a stratifiable set — same bound the oracle uses). Mirrors `stratify` (`wat/rete/oracle/stratify.wat`).
pub(crate) fn native_stratify(rule_parts: &[StratifyView]) -> Result<HashMap<String, i64>, EvalBreak> {
    let bound = rule_parts.len() as i64 + 1;
    native_stratify_fix(rule_parts, HashMap::new(), bound)
}

/// A single rule's stratum given the final type-strata:
/// `max(max strata[p] for produced p, max strata[n]+1 for negated n)`.
/// Mirrors `rule-stratum` (`wat/rete/oracle/stratify.wat`).
pub(crate) fn native_rule_stratum(
    produced: &[String],
    negated: &[String],
    type_strata: &HashMap<String, i64>,
) -> i64 {
    let from_p = produced
        .iter()
        .map(|p| *type_strata.get(p).unwrap_or(&0))
        .max()
        .unwrap_or(0);
    let from_n = negated
        .iter()
        .map(|n| *type_strata.get(n).unwrap_or(&0) + 1)
        .max()
        .unwrap_or(0);
    from_p.max(from_n)
}
