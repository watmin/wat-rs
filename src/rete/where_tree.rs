//! (b) WhereDiscNode — index the armed `where` circuits.
//!
//! Alpha already has `alpha_tree.rs` (fact → candidate alphas). This tree
//! is the filter dual: token bindings → candidate TestNodes. The lab sketch
//! is `ShadowNode` (ddos tree.rs); live type is `WhereDiscNode`.
//! rune:lint(cited-name-absent) ShadowNode — a type in the SIBLING holon-lab-ddos repo, cited as the lab sketch.
//! rune:lint(cited-name-absent) tree.rs — that same sibling repo's file; no file here bears the name.
//! We use `Arc`, never `Rc`.
//!
//! ## Contract
//!
//! **The tree may OVER-approximate. It may never UNDER-approximate.**
//! `exec_where` remains the sole authority on the verdict (and on raises
//! for predicates we actually run). `candidates(token)` ⊇ { TestNodes
//! whose `where` returns true }.
//!
//! Anything this analyzer cannot prove as `(= dim literal)` or a
//! single range `(< > <= >= dim lit)` — `not=`, `or`, user-fn,
//! var-to-var, two constraints on one dim, a dim it cannot
//! canonicalize — rides the wildcard edge and is always walked.
//! A conservative tree is a correct tree. Range edges are guards
//! (`DESIGN-STONE-where-range-edges`).
//!
//! Dimensions are derived from the compiled `Expr` DAG (item 12), not
//! from `WatAST`. Two programs share a dim when their non-literal `=`
//! operands are structurally equal after slots are rewritten to binding
//! keys.

use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use crate::rete::expr_ir::{apply_op, Expr, Program};
use crate::rete::clause::{classify_constraint_head, CmpKind};
use crate::rete::matcher::{compare_values, Bindings};
use crate::rete::vocabulary::RETE_OPS;
use crate::runtime::{
    EvalBreak, RuntimeError, RuntimeErrorKind, Value,
};
use crate::span::Span;

type EqBuckets = HashMap<Value, Vec<i64>>;
type RangeBuckets = HashMap<(CmpKind, Value), Vec<i64>>;


/// Canonical dim — slots rewritten to `?var` names so two TestNodes
/// with independent slot tables still share a level.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub(crate) enum DimKey {
    Bind(Value),
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

/// Guard edge: dim compared to a literal (`DESIGN-STONE-where-range-edges`).
/// Not equality fan-out. Two constraints on one dim never land here.
#[derive(Clone, Debug)]
struct RangeEdge {
    op: CmpKind,
    threshold: Value,
}

/// One analyzed constraint on a dim. Two on the same dim delete the dim
/// (wildcard — over-approx; STOP-2).
#[derive(Clone, Debug)]
enum DimCon {
    Eq(Value),
    Range(CmpKind, Value),
}

type DimCons = HashMap<DimKey, DimCon>;
type WhereDiscs = HashMap<i64, DimCons>;
type EqChildren = HashMap<Value, Arc<WhereDiscNode>>;
type RangeChildren = Vec<(RangeEdge, Arc<WhereDiscNode>)>;
type WhereWildcard = Option<Arc<WhereDiscNode>>;

/// One level: branch on a compiled dim. Equality fan-out + range guards + wildcard.
pub(crate) struct WhereDiscNode {
    dim: Option<DimKey>,
    children: EqChildren,
    wildcard: WhereWildcard,
    range_children: RangeChildren,
    leaves: Vec<i64>,
}

/// Discrimination tree over TestNode ids.
pub(crate) struct WhereTree {
    root: Arc<WhereDiscNode>,
    /// Test ids this tree covers (every TestNode in the network).
    ids: HashSet<i64>,
    /// `where` is only `And` of dim-lit eq **or** range — skip `exec_where`
    /// when proven (`DESIGN-STONE-where-dim-reuse`,
    /// `DESIGN-STONE-where-range-edges`).
    pure_cmp: HashSet<i64>,
}

/// Candidate TestNode ids. `proven` arrived only via equality children
/// whose dim eval succeeded. `maybe` is wildcard / dim-raise over-approx.
pub(crate) struct WhereCands {
    pub proven: Vec<i64>,
    pub maybe: Vec<i64>,
}

impl WhereTree {
    /// Empty tree — no TestNodes. `candidates` returns empty.
    pub(crate) fn empty() -> Self {
        WhereTree {
            root: Arc::new(WhereDiscNode {
                dim: None,
                children: HashMap::new(),
                wildcard: None,
                range_children: Vec::new(),
                leaves: Vec::new(),
            }),
            ids: HashSet::new(),
            pure_cmp: HashSet::new(),
        }
    }

    /// Build the discrimination tree from every armed `where` program.
    ///
    /// Three passes, and they are separate because each needs the previous one whole: analyse
    /// each program into its per-dimension constraints, collect the UNION of dimensions across
    /// all programs (a tree level exists if ANY program constrains that dim), then decide which
    /// programs are `pure_cmp` — fully representable as tree edges, so a match on every level
    /// PROVES them rather than merely nominating them.
    ///
    /// An empty input short-circuits to `empty()`: a tree over no `where`s would answer every
    /// query with an empty candidate set, which is correct but pays a walk to say so.
    pub(crate) fn build(compiled_wheres: &HashMap<i64, Program>) -> Self {
        if compiled_wheres.is_empty() {
            return Self::empty();
        }
        let mut disc: WhereDiscs =
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
        let mut pure_cmp = HashSet::new();
        for (id, prog) in compiled_wheres {
            let mut slot_to_key: HashMap<u16, String> = HashMap::new();
            for (k, slot) in prog.reads.iter() {
                if let Value::String(s) = k {
                    slot_to_key.insert(*slot, s.as_ref().clone());
                }
            }
            if expr_is_pure_cmp(&prog.root, &slot_to_key) {
                pure_cmp.insert(*id);
            }
        }
        let all: Vec<i64> = ids.iter().copied().collect();
        WhereTree {
            root: build_node(all, &disc, &dims, 0),
            ids,
            pure_cmp,
        }
    }

    pub(crate) fn covers(&self, id: i64) -> bool {
        self.ids.contains(&id)
    }

    pub(crate) fn is_pure_cmp(&self, id: i64) -> bool {
        self.pure_cmp.contains(&id)
    }

    /// Candidate TestNode ids. A superset of those whose `where` is true.
    /// Dim eval that raises → take every child as **maybe** (over-approx; do not drop).
    pub(crate) fn candidates<B: Bindings + ?Sized>(
        &self,
        bindings: &B,
        span: &Span,
    ) -> WhereCands {
        let mut proven = Vec::new();
        let mut maybe = Vec::new();
        walk(&self.root, bindings, span, true, &mut proven, &mut maybe);
        WhereCands { proven, maybe }
    }
}

/// Build one level of the tree, discriminating on `dims[pos]`.
///
/// ⛔ The comment inside is the invariant, and it is easy to "optimise" away: **do not leaf
/// early on `test_ids.len() <= 1`.** A single surviving TestNode still has to prove its
/// REMAINING dimensions — stopping here would hand it back as proven when later levels were
/// never checked, and `exec_where` would then be skipped on a lie. The contract permits
/// over-approximation, never under-approximation, and an early leaf is exactly an
/// under-approximation.
fn build_node(
    test_ids: Vec<i64>,
    disc: &WhereDiscs,
    dims: &[DimKey],
    pos: usize,
) -> Arc<WhereDiscNode> {
    // Do not leaf on `len() <= 1`: a solo pure-cmp residue still has to
    // prove its remaining dims or `exec_where` is skipped on a lie.
    if pos >= dims.len() {
        return Arc::new(WhereDiscNode {
            dim: None,
            children: HashMap::new(),
            wildcard: None,
            range_children: Vec::new(),
            leaves: test_ids,
        });
    }
    let dim = &dims[pos];
    let mut buckets: EqBuckets = HashMap::new();
    let mut range_buckets: RangeBuckets = HashMap::new();
    let mut wild: Vec<i64> = Vec::new();
    for id in test_ids {
        match disc.get(&id).and_then(|m| m.get(dim)) {
            Some(DimCon::Eq(v)) => buckets.entry(v.clone()).or_default().push(id),
            Some(DimCon::Range(op, thr)) => {
                range_buckets.entry((*op, thr.clone())).or_default().push(id)
            }
            None => wild.push(id),
        }
    }
    // A dim nobody in this subset constrains is a no-op level — skip it.
    if buckets.is_empty() && range_buckets.is_empty() {
        return build_node(wild, disc, dims, pos + 1);
    }
    let children: EqChildren = buckets
        .into_iter()
        .map(|(v, ids)| (v, build_node(ids, disc, dims, pos + 1)))
        .collect();
    let range_children: RangeChildren = range_buckets
        .into_iter()
        .map(|((op, threshold), ids)| {
            (
                RangeEdge { op, threshold },
                build_node(ids, disc, dims, pos + 1),
            )
        })
        .collect();
    let wildcard = if wild.is_empty() {
        None
    } else {
        Some(build_node(wild, disc, dims, pos + 1))
    };
    Arc::new(WhereDiscNode {
        dim: Some(dim.clone()),
        children,
        wildcard,
        range_children,
        leaves: Vec::new(),
    })
}

/// Descend the tree under one token's bindings, partitioning leaves into PROVEN and MAYBE.
///
/// The `proven` flag is carried down rather than recomputed: it stays true only while every
/// level so far matched an exact edge for a `pure_cmp` program. The moment the walk takes a
/// wildcard or a range edge — a guard, not a proof — everything below it is `maybe` and must be
/// re-checked by `exec_where`. Two output vectors instead of one because the caller can SKIP the
/// predicate for proven leaves and cannot for the others; collapsing them would throw away the
/// only thing the tree bought.
fn walk<B: Bindings + ?Sized>(
    node: &WhereDiscNode,
    bindings: &B,
    span: &Span,
    proven: bool,
    out_proven: &mut Vec<i64>,
    out_maybe: &mut Vec<i64>,
) {
    if proven {
        out_proven.extend(node.leaves.iter().copied());
    } else {
        out_maybe.extend(node.leaves.iter().copied());
    }
    let Some(dim) = &node.dim else {
        return;
    };
    match exec_dim(dim, bindings, span) {
        Ok(v) => {
            if let Some(child) = node.children.get(&v) {
                walk(child, bindings, span, proven, out_proven, out_maybe);
            }
            for (edge, child) in &node.range_children {
                match range_holds(&v, edge.op, &edge.threshold) {
                    Some(true) => walk(child, bindings, span, proven, out_proven, out_maybe),
                    Some(false) => {}
                    None => walk(child, bindings, span, false, out_proven, out_maybe),
                }
            }
        }
        Err(_) => {
            // Cannot prove a branch — keep every child as maybe (over-approx).
            for child in node.children.values() {
                walk(child, bindings, span, false, out_proven, out_maybe);
            }
            for (_, child) in &node.range_children {
                walk(child, bindings, span, false, out_proven, out_maybe);
            }
        }
    }
    if let Some(wc) = &node.wildcard {
        walk(wc, bindings, span, false, out_proven, out_maybe);
    }
}

fn range_holds(v: &Value, op: CmpKind, thr: &Value) -> Option<bool> {
    let ord = compare_values(v, thr)?;
    match op {
        CmpKind::Lt => Some(ord.is_lt()),
        CmpKind::Gt => Some(ord.is_gt()),
        CmpKind::Le => Some(ord.is_le()),
        CmpKind::Ge => Some(ord.is_ge()),
        CmpKind::Eq | CmpKind::NotEq => None,
    }
}

fn expr_is_pure_cmp(e: &Expr, slots: &HashMap<u16, String>) -> bool {
    if !shape_is_pure_cmp(e, slots) {
        return false;
    }
    let mut out = HashMap::new();
    let mut conflicts = HashSet::new();
    collect_cons(e, slots, &mut out, &mut conflicts);
    conflicts.is_empty()
}

/// Is this expression *entirely* an `and` of `(cmp dim literal)` comparisons — the shape the
/// tree can represent edge-for-edge?
///
/// Every arm must hold for the answer to be true (`And` requires ALL children, and a non-empty
/// list — an empty `and` proves nothing and must not read as vacuously pure). The literal-plus-
/// dimension test accepts either operand order but refuses literal-vs-literal, since that names
/// no dimension to discriminate on. Anything else falls through to `false` and rides the
/// wildcard, which is the conservative answer and therefore always safe.
fn shape_is_pure_cmp(e: &Expr, slots: &HashMap<u16, String>) -> bool {
    match e {
        Expr::And(xs) => !xs.is_empty() && xs.iter().all(|x| shape_is_pure_cmp(x, slots)),
        Expr::Call { op, args }
            if args.len() == 2 && (is_eq_op(*op) || range_kind_of(*op).is_some()) =>
        {
            let a = to_dim(&args[0], slots);
            let b = to_dim(&args[1], slots);
            matches!(
                (a, b),
                (Some(DimKey::Lit(_)), Some(d)) | (Some(d), Some(DimKey::Lit(_)))
                    if !matches!(d, DimKey::Lit(_))
            )
        }
        _ => false,
    }
}

/// Classify a RESOLVED `RETE_OPS` row through `clause.rs`'s ★ ONE DOOR.
///
/// These two used to hand-match `core_name` — `ends_with("::=")` for equality, and the
/// leaf `<`/`>`/`<=`/`>=` for orderings — which made this a SECOND closed set beside the
/// one `classify_constraint_head` exists to be. That door's own doc names the defect:
/// "the six generic core spellings were matched by literal string in FOUR independent
/// places, each re-asserting a closed set nothing enforced… That is the arc's recurring
/// defect class — a match on a literal STRING no exhaustiveness check can see." This was
/// a fifth place, and it is the one that read `core_name` rather than a head.
///
/// The hand-match was RIGHT today and wrong in shape, which is why the swap was proved
/// row-by-row before it was made rather than argued:
///   - `range_kind_of` matched the leaf regardless of TYPE, while the door admits
///     orderings only for `i64`/`f64` ("orderings exist only where the type totally
///     orders"). A `string::<` row would have been classified a range here and refused
///     there.
///   - `ends_with("::=")` happens to hold for `:wat::rete::core::enum::=`, whose
///     `core_name` is the GENERIC `:wat::core::=` by head-substitution — right by a
///     coincidence of spelling, not by consulting the table.
///
/// It reads `rete_name`, not `core_name`: the door takes a head as WRITTEN, and
/// deliberately recognises the generic core spelling in order to REFUSE it. A resolved
/// row's rete spelling is the admissible one. `NotEq` is neither an equality nor a
/// range here, and is dropped — the same verdict the hand-match gave by excluding
/// `not=`.
fn constraint_kind(op: u16) -> Option<CmpKind> {
    classify_constraint_head(RETE_OPS[op as usize].rete_name).map(|(k, _)| k)
}

fn is_eq_op(op: u16) -> bool {
    matches!(constraint_kind(op), Some(CmpKind::Eq))
}

fn range_kind_of(op: u16) -> Option<CmpKind> {
    match constraint_kind(op) {
        Some(k @ (CmpKind::Lt | CmpKind::Gt | CmpKind::Le | CmpKind::Ge)) => Some(k),
        _ => None,
    }
}

fn flip_range(k: CmpKind) -> CmpKind {
    match k {
        CmpKind::Lt => CmpKind::Gt,
        CmpKind::Gt => CmpKind::Lt,
        CmpKind::Le => CmpKind::Ge,
        CmpKind::Ge => CmpKind::Le,
        other => other,
    }
}

fn classify_dim_con(op: u16, lit_on_left: bool, lit: Value) -> Option<DimCon> {
    if is_eq_op(op) {
        return Some(DimCon::Eq(lit));
    }
    let k = range_kind_of(op)?;
    Some(DimCon::Range(
        if lit_on_left { flip_range(k) } else { k },
        lit,
    ))
}

/// Record one constraint for a dimension — and make a SECOND constraint on the same dimension
/// poison it.
///
/// This is where the header's "two constraints on one dim rides the wildcard" rule is actually
/// enforced, and the mechanism is deliberately blunt: a repeat write REMOVES the existing entry
/// and marks the dim conflicted, so neither constraint survives to become an edge. Keeping
/// either one would build a tree edge that admits tokens the other rejects — an
/// under-approximation. Once conflicted, a dim stays conflicted; later writes return early.
fn put_con(
    out: &mut HashMap<DimKey, DimCon>,
    conflicts: &mut HashSet<DimKey>,
    dim: DimKey,
    con: DimCon,
) {
    if conflicts.contains(&dim) {
        return;
    }
    if out.remove(&dim).is_some() {
        conflicts.insert(dim);
        return;
    }
    out.insert(dim, con);
}

fn analyze_where(prog: &Program) -> HashMap<DimKey, DimCon> {
    let mut slot_to_key: HashMap<u16, String> = HashMap::new();
    for (k, slot) in prog.reads.iter() {
        if let Value::String(s) = k {
            slot_to_key.insert(*slot, s.as_ref().clone());
        }
    }
    let mut out = HashMap::new();
    let mut conflicts = HashSet::new();
    collect_cons(&prog.root, &slot_to_key, &mut out, &mut conflicts);
    out
}

/// Walk a program's expression DAG and collect the per-dimension constraints it implies,
/// routing every one through `put_con` so conflicts poison rather than overwrite.
///
/// `And` recurses (its children's constraints all apply jointly); a two-argument comparison
/// contributes a constraint when exactly one side canonicalises to a dimension and the other to
/// a literal. Everything else contributes NOTHING — silently, and correctly: an unrepresentable
/// form simply leaves its dims unconstrained, and an unconstrained dim rides the wildcard.
fn collect_cons(
    e: &Expr,
    slots: &HashMap<u16, String>,
    out: &mut HashMap<DimKey, DimCon>,
    conflicts: &mut HashSet<DimKey>,
) {
    match e {
        Expr::And(xs) => {
            for x in xs.iter() {
                collect_cons(x, slots, out, conflicts);
            }
        }
        Expr::Call { op, args } if args.len() == 2 => {
            let a = to_dim(&args[0], slots);
            let b = to_dim(&args[1], slots);
            match (a, b) {
                (Some(DimKey::Lit(v)), Some(d)) if !matches!(d, DimKey::Lit(_)) => {
                    if let Some(con) = classify_dim_con(*op, true, v) {
                        put_con(out, conflicts, d, con);
                    }
                }
                (Some(d), Some(DimKey::Lit(v))) if !matches!(d, DimKey::Lit(_)) => {
                    if let Some(con) = classify_dim_con(*op, false, v) {
                        put_con(out, conflicts, d, con);
                    }
                }
                _ => {}
            }
        }
        _ => {}
    }
}

/// Canonicalise one compiled operand into a `DimKey` — the identity two programs must agree on
/// to share a tree level.
///
/// Slots are rewritten to their BINDING KEY here, which is the whole reason dimensions are
/// derived from the compiled `Expr` DAG rather than from `WatAST` (header, item 12): two
/// programs that wrote the same predicate will have allocated different slot numbers, and
/// comparing slots would make them look like different dimensions. Calls recurse structurally,
/// so `(f ?x 1)` in two programs is one dimension. `None` means "cannot canonicalise" — the
/// caller then leaves the dim unconstrained, which rides the wildcard.
fn to_dim(e: &Expr, slots: &HashMap<u16, String>) -> Option<DimKey> {
    match e {
        Expr::Lit(v) => Some(DimKey::Lit(v.clone())),
        Expr::Slot(s) => slots
            .get(s)
            .cloned()
            .map(|name| DimKey::Bind(Value::String(Arc::new(name)))),
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

/// Evaluate one `DimKey` — the where-tree's compiled dimension expression — against a row's bindings.
///
/// Recursive over the `DimKey` tree: `Lit` and `Bind` are leaves, `Call`,
/// `CallFallback` and `Field` recurse into their operands first. `span` is the
/// whole `:where` form's span and is reused for every diagnostic raised in here,
/// because a `DimKey` is compiled from that form and carries no span of its own.
fn exec_dim<B: Bindings + ?Sized>(d: &DimKey, bindings: &B, span: &Span) -> Result<Value, EvalBreak> {
    match d {
        DimKey::Lit(v) => Ok(v.clone()),
        DimKey::Bind(k) => {
            bindings.get(k).cloned().ok_or_else(|| {
                let name = match k {
                    Value::String(s) => s.as_ref().clone(),
                    _ => format!("{k:?}"),
                };
                RuntimeError::new(span.clone(), RuntimeErrorKind::UnboundSymbol(name)).into()
            })
        }
        DimKey::Call { op, args } => {
            let mut vs = Vec::with_capacity(args.len());
            for a in args.iter() {
                vs.push(exec_dim(a, bindings, span)?);
            }
            apply_op(*op, &vs, span, None)
        }
        // `CallFallback` is `Call` plus a rule for "this op has no answer for THIS row",
        // and that rule is NOT stated here: `runtime::classify_fallback_outcome` is its
        // single home, with the five no-answer shapes and why each is narrow.
        //
        // It used to be restated here in full — and this file's copy was WRONG. It
        // sniffed the runtime value instead of guarding on the row's declared `ret`, so
        // a generic-`ret` row returning a non-finite float took the fallback here and
        // not in the core evaluator: native answering `1` where the `$oracle` answered
        // `0`. Three hand-written copies of one classification, and the prose beside each
        // read like the definition. Do not restate it again — call the classifier.
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
            // ONE classification, shared with `expr_ir`'s walk and the core evaluator.
            // It used to be hand-written here, and the copies diverged — see
            // `classify_fallback_outcome`. Only the RECURSION is this site's own.
            match crate::runtime::classify_fallback_outcome(
                apply_op(*op, &vs, span, None),
                &row.ret,
                row.core_name,
                row.rete_name,
                span,
            )? {
                crate::runtime::FallbackVerdict::Value(v) => Ok(v),
                crate::runtime::FallbackVerdict::UseFallback => exec_dim(fallback, bindings, span),
            }
        }
        DimKey::Field { recv, idx } => {
            let v = exec_dim(recv, bindings, span)?;
            match v {
                Value::Aggregate(a) => a.fields.get(*idx).cloned().ok_or_else(|| {
                    RuntimeError::new(
                        span.clone(),
                        RuntimeErrorKind::UnknownField {
                            record_class: a.class.to_string(),
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
            cands.proven.contains(&11) || cands.maybe.contains(&11),
            "token ?k=1 must keep the (= ?k 1) TestNode; proven {:?} maybe {:?}",
            cands.proven,
            cands.maybe
        );
        assert!(
            !cands.proven.contains(&10)
                && !cands.maybe.contains(&10)
                && !cands.proven.contains(&12)
                && !cands.maybe.contains(&12),
            "same-dim other literals must be pruned (over-approx would keep them); proven {:?} maybe {:?}",
            cands.proven,
            cands.maybe
        );
        assert!(
            cands.proven.contains(&11),
            "(= ?k 1) must be proven, not maybe; proven {:?}",
            cands.proven
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
            cands.maybe.contains(&2),
            "a where with no (= dim lit) must stay on the wildcard as maybe; proven {:?} maybe {:?}",
            cands.proven,
            cands.maybe
        );
        assert!(
            !cands.proven.contains(&1) && !cands.maybe.contains(&1),
            "token ?k=1 must not keep (= ?k 0); proven {:?} maybe {:?}",
            cands.proven,
            cands.maybe
        );
    }

    fn gt_op() -> u16 {
        RETE_OPS
            .iter()
            .position(|r| r.core_name == ":wat::core::i64::>")
            .expect("RETE_OPS has i64::>") as u16
    }

    fn lt_op() -> u16 {
        RETE_OPS
            .iter()
            .position(|r| r.core_name == ":wat::core::i64::<")
            .expect("RETE_OPS has i64::<") as u16
    }

    fn cmp_bind_lit(op: u16, lit: i64) -> Program {
        let k = Value::String(Arc::new("?k".into()));
        Program {
            frame_len: 1,
            root: Expr::Call {
                op,
                args: Box::new([Expr::Slot(0), Expr::Lit(Value::i64(lit))]),
            },
            reads: Arc::from([(k, 0u16)]),
            params: Box::from([]),
            names: Box::from([Some(Arc::from("?k"))]),
            span: crate::rust_caller_span!(),
        }
    }

    fn cmp_lit_bind(op: u16, lit: i64) -> Program {
        let k = Value::String(Arc::new("?k".into()));
        Program {
            frame_len: 1,
            root: Expr::Call {
                op,
                args: Box::new([Expr::Lit(Value::i64(lit)), Expr::Slot(0)]),
            },
            reads: Arc::from([(k, 0u16)]),
            params: Box::from([]),
            names: Box::from([Some(Arc::from("?k"))]),
            span: crate::rust_caller_span!(),
        }
    }

    #[test]
    fn range_gt_prunes_below_and_proves_above() {
        let mut compiled = HashMap::new();
        compiled.insert(20, cmp_bind_lit(gt_op(), 10));
        let tree = WhereTree::build(&compiled);
        let span = crate::rust_caller_span!();
        assert!(tree.is_pure_cmp(20), "(> ?k 10) is a pure range");

        let below = tree.candidates(&bindings_k(5), &span);
        assert!(
            !below.proven.contains(&20) && !below.maybe.contains(&20),
            "(> ?k 10) must prune ?k=5; proven {:?} maybe {:?}",
            below.proven,
            below.maybe
        );

        let above = tree.candidates(&bindings_k(15), &span);
        assert!(
            above.proven.contains(&20),
            "(> ?k 10) must prove ?k=15; proven {:?} maybe {:?}",
            above.proven,
            above.maybe
        );
        assert!(
            !above.maybe.contains(&20),
            "(> ?k 10) at 15 is proven, not maybe; maybe {:?}",
            above.maybe
        );
    }

    #[test]
    fn range_lit_on_left_flips_the_op() {
        // (> 10 ?k) ≡ (?k < 10)
        let mut compiled = HashMap::new();
        compiled.insert(21, cmp_lit_bind(gt_op(), 10));
        let tree = WhereTree::build(&compiled);
        let span = crate::rust_caller_span!();

        let below = tree.candidates(&bindings_k(5), &span);
        assert!(
            below.proven.contains(&21),
            "(> 10 ?k) must prove ?k=5; proven {:?} maybe {:?}",
            below.proven,
            below.maybe
        );

        let above = tree.candidates(&bindings_k(15), &span);
        assert!(
            !above.proven.contains(&21) && !above.maybe.contains(&21),
            "(> 10 ?k) must prune ?k=15; proven {:?} maybe {:?}",
            above.proven,
            above.maybe
        );
    }

    /// Fixture: `(and (> ?k 10) (< ?k 20))` — TWO constraints on ONE dimension.
    ///
    /// The shape `put_con` must poison. Both comparisons are individually representable as tree
    /// edges, which is what makes this the interesting case: a tree that kept either one would
    /// look correct and quietly under-approximate. Shared by the two tests that check the
    /// conflicted dim rides the wildcard instead.
    fn two_constraint_where() -> Program {
        let k = Value::String(Arc::new("?k".into()));
        Program {
            frame_len: 1,
            root: Expr::And(Box::new([
                Expr::Call {
                    op: gt_op(),
                    args: Box::new([Expr::Slot(0), Expr::Lit(Value::i64(10))]),
                },
                Expr::Call {
                    op: lt_op(),
                    args: Box::new([Expr::Slot(0), Expr::Lit(Value::i64(20))]),
                },
            ])),
            reads: Arc::from([(k, 0u16)]),
            params: Box::from([]),
            names: Box::from([Some(Arc::from("?k"))]),
            span: crate::rust_caller_span!(),
        }
    }

    #[test]
    fn two_constraints_on_one_dim_are_not_pure_cmp() {
        let mut compiled = HashMap::new();
        compiled.insert(30, two_constraint_where());
        let tree = WhereTree::build(&compiled);
        let span = crate::rust_caller_span!();
        let cands = tree.candidates(&bindings_k(15), &span);
        assert!(
            cands.proven.contains(&30) || cands.maybe.contains(&30),
            "two constraints on one dim must not drop the residue; proven {:?} maybe {:?}",
            cands.proven,
            cands.maybe
        );
        assert!(
            !tree.is_pure_cmp(30),
            "two constraints on one dim must not skip exec_where"
        );
    }

    #[test]
    fn two_constraints_on_one_dim_ride_wildcard_beside_equality() {
        let mut compiled = HashMap::new();
        compiled.insert(10, eq_bind_lit(0));
        compiled.insert(30, two_constraint_where());
        let tree = WhereTree::build(&compiled);
        let span = crate::rust_caller_span!();
        let cands = tree.candidates(&bindings_k(15), &span);
        assert!(
            cands.maybe.contains(&30),
            "conflicted dim rides the wildcard as maybe; proven {:?} maybe {:?}",
            cands.proven,
            cands.maybe
        );
        assert!(
            !cands.proven.contains(&30),
            "conflicted dim must not be proven; proven {:?}",
            cands.proven
        );
        assert!(
            !cands.proven.contains(&10) && !cands.maybe.contains(&10),
            "token ?k=15 must not keep (= ?k 0); proven {:?} maybe {:?}",
            cands.proven,
            cands.maybe
        );
    }

    #[test]
    fn equality_and_range_share_a_dim_without_colliding() {
        let mut compiled = HashMap::new();
        compiled.insert(10, eq_bind_lit(0));
        compiled.insert(20, cmp_bind_lit(gt_op(), 10));
        let tree = WhereTree::build(&compiled);
        let span = crate::rust_caller_span!();

        let at_eq = tree.candidates(&bindings_k(0), &span);
        assert!(at_eq.proven.contains(&10), "eq 0 proven; {:?}", at_eq.proven);
        assert!(
            !at_eq.proven.contains(&20) && !at_eq.maybe.contains(&20),
            "0 is not > 10; proven {:?} maybe {:?}",
            at_eq.proven,
            at_eq.maybe
        );

        let at_range = tree.candidates(&bindings_k(15), &span);
        assert!(
            at_range.proven.contains(&20),
            "> 10 at 15 proven; {:?}",
            at_range.proven
        );
        assert!(
            !at_range.proven.contains(&10) && !at_range.maybe.contains(&10),
            "15 is not = 0; proven {:?} maybe {:?}",
            at_range.proven,
            at_range.maybe
        );

        let neither = tree.candidates(&bindings_k(5), &span);
        assert!(
            neither.proven.is_empty() && neither.maybe.is_empty(),
            "5 is neither = 0 nor > 10; proven {:?} maybe {:?}",
            neither.proven,
            neither.maybe
        );
    }
}

