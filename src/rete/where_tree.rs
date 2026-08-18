//! (b) ShadowNode — index the armed `where` circuits.
//!
//! Alpha already has `alpha_tree.rs` (fact → candidate alphas). This tree
//! is the filter dual: token bindings → candidate TestNodes. The lab node
//! is `ShadowNode` (equality fan-out + wildcard + unpopulated ranges).
//! We use `Arc`, never `Rc`.
//!
//! ## Contract
//!
//! **The tree may OVER-approximate. It may never UNDER-approximate.**
//! `exec_where` remains the sole authority on the verdict (and on raises
//! for predicates we actually run). `candidates(token)` ⊇ { TestNodes
//! whose `where` returns true }.
//!
//! Anything this analyzer cannot prove as `(= dim literal)` — `not=`,
//! `or`, user-fn, var-to-var, a dim it cannot canonicalize — rides the
//! wildcard edge and is always walked. A conservative tree is a correct
//! tree.
//!
//! Dimensions are derived from the compiled `Expr` DAG (item 12), not
//! from `WatAST`. Two programs share a dim when their non-literal `=`
//! operands are structurally equal after slots are rewritten to binding
//! keys.

use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use crate::rete::expr_ir::{apply_core, Expr, Program};
use crate::rete::matcher::Bindings;
use crate::rete::vocabulary::RETE_OPS;
use crate::runtime::{EvalBreak, RuntimeError, RuntimeErrorKind, Value};
use crate::span::Span;


/// Canonical dim — slots rewritten to `?var` names so two TestNodes
/// with independent slot tables still share a level.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub(crate) enum DimKey {
    Bind(String),
    Lit(Value),
    Call {
        op: u16,
        args: Box<[DimKey]>,
    },
    CallFallback {
        op: u16,
        args: Box<[DimKey]>,
        fallback: Box<DimKey>,
    },
    Field {
        recv: Box<DimKey>,
        idx: usize,
    },
}

/// One level: branch on a compiled dim. `range_children` reserved, unpopulated.
pub(crate) struct ShadowNode {
    dim: Option<DimKey>,
    children: HashMap<Value, Arc<ShadowNode>>,
    wildcard: Option<Arc<ShadowNode>>,
    #[allow(dead_code)]
    range_children: Vec<()>,
    leaves: Vec<i64>,
}

/// Discrimination tree over TestNode ids.
pub(crate) struct WhereTree {
    root: Arc<ShadowNode>,
    /// Test ids this tree covers (every TestNode in the network).
    ids: HashSet<i64>,
}

impl WhereTree {
    /// Empty tree — no TestNodes. `candidates` returns empty.
    pub(crate) fn empty() -> Self {
        WhereTree {
            root: Arc::new(ShadowNode {
                dim: None,
                children: HashMap::new(),
                wildcard: None,
                range_children: Vec::new(),
                leaves: Vec::new(),
            }),
            ids: HashSet::new(),
        }
    }

    pub(crate) fn build(compiled_wheres: &HashMap<i64, Program>) -> Self {
        if compiled_wheres.is_empty() {
            return Self::empty();
        }
        let mut disc: HashMap<i64, HashMap<DimKey, Value>> =
            HashMap::with_capacity(compiled_wheres.len());
        let mut dim_set: HashSet<DimKey> = HashSet::new();
        for (id, prog) in compiled_wheres {
            let m = analyze_where(prog);
            for k in m.keys() {
                dim_set.insert(k.clone());
            }
            disc.insert(*id, m);
        }
        let dims: Vec<DimKey> = dim_set.into_iter().collect();
        let ids: HashSet<i64> = compiled_wheres.keys().copied().collect();
        let all: Vec<i64> = ids.iter().copied().collect();
        WhereTree {
            root: build_node(all, &disc, &dims, 0),
            ids,
        }
    }

    pub(crate) fn covers(&self, id: i64) -> bool {
        self.ids.contains(&id)
    }

    /// Candidate TestNode ids. A superset of those whose `where` is true.
    /// Dim eval that raises → take every child (over-approx; do not drop).
    pub(crate) fn candidates<B: Bindings>(
        &self,
        bindings: &B,
        span: &Span,
    ) -> Vec<i64> {
        let mut out = Vec::new();
        walk(&self.root, bindings, span, &mut out);
        out
    }
}

fn build_node(
    test_ids: Vec<i64>,
    disc: &HashMap<i64, HashMap<DimKey, Value>>,
    dims: &[DimKey],
    pos: usize,
) -> Arc<ShadowNode> {
    if pos >= dims.len() || test_ids.len() <= 1 {
        return Arc::new(ShadowNode {
            dim: None,
            children: HashMap::new(),
            wildcard: None,
            range_children: Vec::new(),
            leaves: test_ids,
        });
    }
    let dim = &dims[pos];
    let mut buckets: HashMap<Value, Vec<i64>> = HashMap::new();
    let mut wild: Vec<i64> = Vec::new();
    for id in test_ids {
        match disc.get(&id).and_then(|m| m.get(dim)) {
            Some(v) => buckets.entry(v.clone()).or_default().push(id),
            None => wild.push(id),
        }
    }
    // A dim nobody in this subset constrains is a no-op level — skip it.
    if buckets.is_empty() {
        return build_node(wild, disc, dims, pos + 1);
    }
    let children: HashMap<Value, Arc<ShadowNode>> = buckets
        .into_iter()
        .map(|(v, ids)| (v, build_node(ids, disc, dims, pos + 1)))
        .collect();
    let wildcard = if wild.is_empty() {
        None
    } else {
        Some(build_node(wild, disc, dims, pos + 1))
    };
    Arc::new(ShadowNode {
        dim: Some(dim.clone()),
        children,
        wildcard,
        range_children: Vec::new(),
        leaves: Vec::new(),
    })
}

fn walk<B: Bindings>(
    node: &ShadowNode,
    bindings: &B,
    span: &Span,
    out: &mut Vec<i64>,
) {
    out.extend(node.leaves.iter().copied());
    let Some(dim) = &node.dim else {
        return;
    };
    match exec_dim(dim, bindings, span) {
        Ok(v) => {
            if let Some(child) = node.children.get(&v) {
                walk(child, bindings, span, out);
            }
        }
        Err(_) => {
            // Cannot prove a branch — keep every child (over-approx).
            for child in node.children.values() {
                walk(child, bindings, span, out);
            }
        }
    }
    if let Some(wc) = &node.wildcard {
        walk(wc, bindings, span, out);
    }
}

fn is_eq_op(op: u16) -> bool {
    let n = RETE_OPS[op as usize].core_name;
    n.ends_with("::=") && !n.contains("not=")
}

fn analyze_where(prog: &Program) -> HashMap<DimKey, Value> {
    let mut slot_to_key: HashMap<u16, String> = HashMap::new();
    for (k, slot) in prog.reads.iter() {
        if let Value::String(s) = k {
            slot_to_key.insert(*slot, s.as_ref().clone());
        }
    }
    let mut out = HashMap::new();
    collect_eqs(&prog.root, &slot_to_key, &mut out);
    out
}

fn collect_eqs(e: &Expr, slots: &HashMap<u16, String>, out: &mut HashMap<DimKey, Value>) {
    match e {
        Expr::And(xs) => {
            for x in xs.iter() {
                collect_eqs(x, slots, out);
            }
        }
        Expr::Call { op, args } if is_eq_op(*op) && args.len() == 2 => {
            let a = to_dim(&args[0], slots);
            let b = to_dim(&args[1], slots);
            match (a, b) {
                (Some(DimKey::Lit(v)), Some(d)) if !matches!(d, DimKey::Lit(_)) => {
                    out.insert(d, v);
                }
                (Some(d), Some(DimKey::Lit(v))) if !matches!(d, DimKey::Lit(_)) => {
                    out.insert(d, v);
                }
                _ => {}
            }
        }
        _ => {}
    }
}

fn to_dim(e: &Expr, slots: &HashMap<u16, String>) -> Option<DimKey> {
    match e {
        Expr::Lit(v) => Some(DimKey::Lit(v.clone())),
        Expr::Slot(s) => slots.get(s).cloned().map(DimKey::Bind),
        Expr::Call { op, args } => {
            let mut out = Vec::with_capacity(args.len());
            for a in args.iter() {
                out.push(to_dim(a, slots)?);
            }
            Some(DimKey::Call {
                op: *op,
                args: out.into_boxed_slice(),
            })
        }
        Expr::CallFallback {
            op,
            args,
            fallback,
        } => {
            let mut out = Vec::with_capacity(args.len());
            for a in args.iter() {
                out.push(to_dim(a, slots)?);
            }
            Some(DimKey::CallFallback {
                op: *op,
                args: out.into_boxed_slice(),
                fallback: Box::new(to_dim(fallback, slots)?),
            })
        }
        Expr::Field { recv, idx } => Some(DimKey::Field {
            recv: Box::new(to_dim(recv, slots)?),
            idx: *idx,
        }),
        _ => None,
    }
}

fn exec_dim<B: Bindings>(d: &DimKey, bindings: &B, span: &Span) -> Result<Value, EvalBreak> {
    match d {
        DimKey::Lit(v) => Ok(v.clone()),
        DimKey::Bind(name) => {
            let k = Value::String(Arc::new(name.clone()));
            bindings.get(&k).cloned().ok_or_else(|| {
                RuntimeError::new(span.clone(), RuntimeErrorKind::UnboundSymbol(name.clone()))
                    .into()
            })
        }
        DimKey::Call { op, args } => {
            let mut vs = Vec::with_capacity(args.len());
            for a in args.iter() {
                vs.push(exec_dim(a, bindings, span)?);
            }
            apply_core(RETE_OPS[*op as usize].core_name, &vs, span)
        }
        DimKey::CallFallback {
            op,
            args,
            fallback,
        } => {
            let row = &RETE_OPS[*op as usize];
            let mut vs = Vec::with_capacity(args.len());
            for a in args.iter() {
                vs.push(exec_dim(a, bindings, span)?);
            }
            match apply_core(row.core_name, &vs, span) {
                Ok(Value::f64(x)) if !x.is_finite() => exec_dim(fallback, bindings, span),
                Ok(Value::Option(opt)) => match opt.as_ref() {
                    Some(v) => Ok(v.clone()),
                    None => exec_dim(fallback, bindings, span),
                },
                Ok(v) => Ok(v),
                Err(EvalBreak::Diagnostic(e))
                    if matches!(
                        e.kind(),
                        RuntimeErrorKind::IntegerOverflow { .. }
                            | RuntimeErrorKind::DivisionByZero
                    ) =>
                {
                    exec_dim(fallback, bindings, span)
                }
                Err(EvalBreak::Diagnostic(e))
                    if matches!(
                        e.kind(),
                        RuntimeErrorKind::MalformedForm { head, .. } if head.as_str() == row.core_name
                    ) =>
                {
                    exec_dim(fallback, bindings, span)
                }
                Err(e) => Err(e),
            }
        }
        DimKey::Field { recv, idx } => {
            let v = exec_dim(recv, bindings, span)?;
            match v {
                Value::Aggregate(a) => a.fields.get(*idx).cloned().ok_or_else(|| {
                    RuntimeError::new(
                        span.clone(),
                        RuntimeErrorKind::UnknownField {
                            record_class: a.class.clone(),
                            field: format!("{idx}"),
                            available: (*a.names).clone(),
                        },
                    )
                    .into()
                }),
                other => Err(RuntimeError::new(
                    span.clone(),
                    RuntimeErrorKind::TypeMismatch {
                        op: ":wat::rete::lower".into(),
                        expected: "record",
                        got: Box::new(crate::runtime::ValueSnapshot::of(&other)),
                    },
                )
                .into()),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rete::expr_ir::{Expr, Program};

    fn eq_op() -> u16 {
        RETE_OPS
            .iter()
            .position(|r| r.core_name.ends_with("::=") && !r.core_name.contains("not="))
            .expect("RETE_OPS has an equality") as u16
    }

    fn eq_bind_lit(lit: i64) -> Program {
        let k = Value::String(Arc::new("?k".into()));
        Program {
            frame_len: 1,
            root: Expr::Call {
                op: eq_op(),
                args: Box::new([Expr::Lit(Value::i64(lit)), Expr::Slot(0)]),
            },
            reads: Arc::from([(k, 0u16)]),
            params: Box::from([]),
            names: Box::from([Some(Arc::from("?k"))]),
            span: crate::rust_caller_span!(),
        }
    }

    fn bindings_k(v: i64) -> Vec<(Value, Value)> {
        vec![(Value::String(Arc::new("?k".into())), Value::i64(v))]
    }

    #[test]
    fn tree_picks_the_matching_equality_leaf() {
        let mut compiled = HashMap::new();
        compiled.insert(10, eq_bind_lit(0));
        compiled.insert(11, eq_bind_lit(1));
        compiled.insert(12, eq_bind_lit(2));
        let tree = WhereTree::build(&compiled);
        let span = crate::rust_caller_span!();
        let cands = tree.candidates(&bindings_k(1), &span);
        assert!(
            cands.contains(&11),
            "token ?k=1 must keep the (= ?k 1) TestNode; got {cands:?}"
        );
        assert!(
            !cands.contains(&10) && !cands.contains(&12),
            "same-dim other literals must be pruned (over-approx would keep them); got {cands:?}"
        );
    }

    #[test]
    fn no_key_predicate_rides_wildcard() {
        let k = Value::String(Arc::new("?k".into()));
        let no_key = Program {
            frame_len: 1,
            root: Expr::Slot(0),
            reads: Arc::from([(k, 0u16)]),
            params: Box::from([]),
            names: Box::from([Some(Arc::from("?k"))]),
            span: crate::rust_caller_span!(),
        };
        let mut compiled = HashMap::new();
        compiled.insert(1, eq_bind_lit(0));
        compiled.insert(2, no_key);
        let tree = WhereTree::build(&compiled);
        let span = crate::rust_caller_span!();
        let cands = tree.candidates(&bindings_k(1), &span);
        assert!(
            cands.contains(&2),
            "a where with no (= dim lit) must stay on the wildcard; got {cands:?}"
        );
        assert!(
            !cands.contains(&1),
            "token ?k=1 must not keep (= ?k 0); got {cands:?}"
        );
    }
}

