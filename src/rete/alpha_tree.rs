//! Arc 278 — the alpha discrimination tree (DESIGN-STONE-alpha-discrimination-tree.md).
//!
//! Today's alpha-delta loop (`kernel.rs`, step 1) type-indexes (P8), then linearly probes
//! **every** alpha of that type via `alpha_match_inner`. A depth-D cascade has D alphas per
//! type where exactly one can succeed — `facts × D` calls, measured as 79% of the deep-cascade
//! depth cost (`a0_depth_cost_split_at_equal_work`). This module replaces the linear scan with
//! a per-type discrimination tree: a fact walks root-to-leaf, one declared field per level, and
//! arrives at only the alphas it could possibly satisfy.
//!
//! ## ★ THE ONE CONTRACT DECISION
//!
//! **The tree may OVER-approximate. It may never UNDER-approximate.** `alpha_match_inner`
//! remains the sole authority on whether a condition holds and the sole producer of bindings;
//! `AlphaTree::candidates` returns a **candidate set** that the matcher then runs on exactly as
//! it does today. For every fact, `candidates(fact) ⊇ { alphas that actually match }`.
//!
//! Any clause this analyzer cannot prove an equality discriminator for — `not=`, `or`, `not`, a
//! computed operand, an unfamiliar shape, anything at all — costs nothing but a wasted
//! `alpha_match_inner` call: it simply does not contribute a discriminator, so that alpha rides
//! the **wildcard** edge at every level and is always walked. A conservative tree is a correct
//! tree; this analyzer never guesses at a shape it does not recognise in order to prune it.
//!
//! The clause-shape analyzer **consumes** `classify_rete_clause` (`matcher.rs:365`) — the single
//! source of "what shape is this form" that arc 294 item 9a extracted precisely to close a drift
//! hole between the matcher and the validator. This module adds no second parser for clause
//! shapes; it only inspects the WatAST *literal* variants inside an already-classified
//! `Constraint`'s operands (which `classify_rete_clause` does not itself resolve).
//!
//! ## Scope: alpha only, prune only, per type
//!
//! Beta (the join network) is untouched — rules derive facts, so the beta network stays
//! runtime, unlike the kernel's packet tree (`holon-lab-ddos/veth-lab/filter/src/tree.rs`),
//! which bakes the whole rule in because nothing ever derives there. This tree only prunes
//! which alphas of a fact's own type are worth match-testing; bindings still come from
//! `alpha_match_inner`. `dim` indexes the class's own declared field order
//! (`kernel::class_field_names`), not a fixed global order. `range_children` is present in the
//! node shape (so a later stone can populate it without changing the shape) but is never
//! populated by this one — `< > <= >=` constraints simply contribute no discriminator and ride
//! the wildcard edge, which is correct and simply unpruned.

use std::collections::HashMap;
use std::sync::Arc;

use crate::ast::WatAST;
use crate::rete::matcher::{classify_constraint_head, classify_rete_clause, CmpKind, ReteClauseShape};
use crate::rete::kernel::class_field_names;
use crate::runtime::{SymbolTable, Value};

/// A range-guarded edge, reserved for a later stone
/// (`DESIGN-STONE-alpha-tree-range-edges`). Never populated by this one.
#[derive(Debug)]
#[allow(dead_code)]
pub(crate) struct RangeEdge {
    pub(crate) op: &'static str,
    pub(crate) threshold: Value,
}

/// One level of the discrimination tree: branch on field `dim` of the fact's own class.
pub(crate) struct Node {
    /// Index into the class's declared field order (`class_field_names`). Meaningless on a
    /// leaf (empty `children`/`wildcard`), where it is never read.
    dim: usize,
    /// Equality fan-out: a field value this dim was proven to require, to the subtree of
    /// alphas that could still match.
    children: HashMap<Value, Arc<Node>>,
    /// Alphas that do not constrain `dim` by a provable equality — always walked.
    wildcard: Option<Arc<Node>>,
    /// Reserved for range/mask edges (unpopulated this stone; see module docs).
    #[allow(dead_code)]
    range_children: Vec<(RangeEdge, Arc<Node>)>,
    /// Alpha ids terminating at this node (no further dimension left to discriminate on, or
    /// only one candidate remained).
    leaves: Vec<i64>,
}

/// Per-type discrimination tree over alpha conditions: fact class → root `Node`.
///
/// Built once at setup time (P8, alongside `alpha_by_type`/`alpha_cond`) from the immutable
/// network; never rebuilt per round.
pub(crate) struct AlphaTree {
    roots: HashMap<String, Arc<Node>>,
}

impl AlphaTree {
    /// Build the tree from the setup-time alpha index. `alpha_by_type`/`alpha_cond` are exactly
    /// P8's maps (`kernel.rs`); `sym` resolves each class's declared field order.
    pub(crate) fn build(
        alpha_by_type: &HashMap<String, Vec<i64>>,
        alpha_cond: &HashMap<i64, WatAST>,
        sym: &SymbolTable,
    ) -> Self {
        let mut roots = HashMap::with_capacity(alpha_by_type.len());
        for (class, alpha_ids) in alpha_by_type {
            let field_names = class_field_names(sym, class);

            // Per-alpha provable equality discriminators: alpha id -> {dim -> required value}.
            let mut disc: HashMap<i64, HashMap<usize, Value>> =
                HashMap::with_capacity(alpha_ids.len());
            for aid in alpha_ids {
                let clauses: &[WatAST] = match alpha_cond.get(aid) {
                    Some(cond) => crate::rete::matcher::alpha_pattern(cond)
                        .map(|p| p.clauses)
                        .unwrap_or(&[]),
                    None => &[],
                };
                disc.insert(*aid, analyze_condition(clauses, &field_names));
            }

            // Only branch on dims at least one alpha of this class actually discriminates on —
            // a field nobody constrains generates zero tree nodes (mirrors the kernel packet
            // tree's `any_constrains` skip).
            let dims: Vec<usize> = (0..field_names.len())
                .filter(|d| disc.values().any(|m| m.contains_key(d)))
                .collect();

            let root = build_node(alpha_ids.clone(), &disc, &dims, 0);
            roots.insert(class.clone(), root);
        }
        AlphaTree { roots }
    }

    /// Import-time tree: every alpha of a class is a candidate. Correct
    /// (a superset); unpruned. The residual does not carry WatAST, so
    /// `build` cannot re-derive discriminators. `(b)` indexes later.
    pub(crate) fn unpruned(alpha_by_type: &HashMap<String, Vec<i64>>) -> Self {
        let mut roots = HashMap::with_capacity(alpha_by_type.len());
        for (class, alpha_ids) in alpha_by_type {
            roots.insert(
                class.clone(),
                Arc::new(Node {
                    dim: 0,
                    children: HashMap::new(),
                    wildcard: None,
                    range_children: Vec::new(),
                    leaves: alpha_ids.clone(),
                }),
            );
        }
        AlphaTree { roots }
    }

    /// Walk `class`'s tree for a fact's field values, returning the **candidate set** of alpha
    /// ids the caller must still run `alpha_match_inner` on. A superset of the alphas that
    /// actually match — never a subset. Unknown class (no alpha of that type at all): empty.
    pub(crate) fn candidates(&self, class: &str, fields: &[Value]) -> Vec<i64> {
        let mut out = Vec::new();
        if let Some(root) = self.roots.get(class) {
            walk(root, fields, &mut out);
        }
        out
    }
}

/// Recursively partition `alpha_ids` over the remaining dims. Each alpha with a proven
/// discriminator at `dims[pos]` goes to that value's child; every alpha without one (including
/// every alpha once `pos` runs off the end of `dims`) goes to `leaves` directly, or — while dims
/// remain — to the wildcard subtree, so a later dim still gets a chance to discriminate it.
fn build_node(
    alpha_ids: Vec<i64>,
    disc: &HashMap<i64, HashMap<usize, Value>>,
    dims: &[usize],
    pos: usize,
) -> Arc<Node> {
    if pos >= dims.len() || alpha_ids.len() <= 1 {
        return Arc::new(Node {
            dim: 0,
            children: HashMap::new(),
            wildcard: None,
            range_children: Vec::new(),
            leaves: alpha_ids,
        });
    }

    let dim = dims[pos];
    let mut buckets: HashMap<Value, Vec<i64>> = HashMap::new();
    let mut wild: Vec<i64> = Vec::new();
    for aid in alpha_ids {
        match disc.get(&aid).and_then(|m| m.get(&dim)) {
            Some(v) => buckets.entry(v.clone()).or_default().push(aid),
            None => wild.push(aid),
        }
    }

    let children: HashMap<Value, Arc<Node>> = buckets
        .into_iter()
        .map(|(v, ids)| (v, build_node(ids, disc, dims, pos + 1)))
        .collect();
    let wildcard = if wild.is_empty() {
        None
    } else {
        Some(build_node(wild, disc, dims, pos + 1))
    };

    Arc::new(Node { dim, children, wildcard, range_children: Vec::new(), leaves: Vec::new() })
}

/// Descend the tree for one fact: take the specific-value edge (if the fact's field at `dim`
/// has a matching child) AND the wildcard edge (if present) — union their leaves. Both edges
/// are walked because the wildcard side holds alphas that never constrained this dim at all, so
/// they must be reached regardless of what value the fact carries there.
fn walk(node: &Node, fields: &[Value], out: &mut Vec<i64>) {
    out.extend(node.leaves.iter().copied());
    if let Some(v) = fields.get(node.dim) {
        if let Some(child) = node.children.get(v) {
            walk(child, fields, out);
        }
    }
    if let Some(wc) = &node.wildcard {
        walk(wc, fields, out);
    }
}

// ─── The analyzer: per-condition provable equality discriminators ─────────────────────────────

/// Analyze one alpha condition's clause list (`items[1..]` of the condition `WatAST::List`),
/// producing every `{field-index -> required literal value}` this analyzer can **prove** — via
/// `classify_rete_clause` alone — must hold for any fact this alpha could ever accept.
///
/// Anything not provable (a `not=`/`or`/`not`/`where` clause, a non-`=` comparison, a
/// cross-condition join var unbound in THIS condition, a computed or nested operand) is simply
/// absent from the result. An absent dim is always safe: the tree treats it as "this alpha does
/// not constrain this field," which puts it on the wildcard edge — always walked, never pruned
/// away. This is the over-approximation contract, by construction.
fn analyze_condition(clauses: &[WatAST], field_names: &[String]) -> HashMap<usize, Value> {
    // Pass 1: `(?v <- :field)` binds, so a later `(:wat::core::= ?v <literal>)` can be traced
    // back to a field name. Recurses into `:wat::rete::and` (still AND semantics at any depth);
    // a bind that only lives inside `or`/`not` cannot be trusted for the ENCLOSING scope's
    // clauses, but since a `?v` used across those clauses would be the same var, the ordinary
    // top-level/And walk already finds it wherever it was actually bound.
    let mut var_to_field: HashMap<String, String> = HashMap::new();
    collect_binds(clauses, &mut var_to_field);

    let mut result: HashMap<usize, Value> = HashMap::new();
    collect_equalities(clauses, &var_to_field, field_names, &mut result);
    result
}

fn collect_binds(clauses: &[WatAST], out: &mut HashMap<String, String>) {
    for clause in clauses {
        match classify_rete_clause(clause) {
            ReteClauseShape::Bind { var, field } => {
                out.insert(var.to_string(), field.to_string());
            }
            ReteClauseShape::And(subs) => collect_binds(subs, out),
            // Or/Not/Where/Exists/Accumulate/Constraint/Unrecognized bind nothing here.
            _ => {}
        }
    }
}

/// Gather provable `(:wat::core::= <field-ref> <literal>)` discriminators (either operand
/// order), recursing into `:wat::rete::and` sub-clauses (still AND semantics — a discriminator
/// proven inside an `and` is exactly as required as one at the top level). A clause this cannot
/// classify as such a `Constraint` — `not=`, any ordering comparison, `or`, `not`, `where`, an
/// unrecognized shape — contributes nothing, which is always safe: it just leaves that field
/// (or that alpha entirely) on the wildcard edge.
fn collect_equalities(
    clauses: &[WatAST],
    var_to_field: &HashMap<String, String>,
    field_names: &[String],
    out: &mut HashMap<usize, Value>,
) {
    for clause in clauses {
        match classify_rete_clause(clause) {
            // The ONE DOOR (`matcher::classify_constraint_head`) — an EQUALITY at any spelling, so
            // the per-type rete rows feed the discrimination tree exactly as the generic core `=`
            // used to. Matching a literal string here is what made this arm a silent
            // migration hazard: `_ => {}` below would have quietly stopped collecting
            // discriminators, degrading the tree to a linear alpha scan with the floor still green.
            // (Mutation-proven 2026-08-06: `alpha_tree_discriminates_candidates_to_about_one_at_50_100`
            // does catch it — 1.000 -> 50.000 candidates/fact — but only because that gate exists.)
            ReteClauseShape::Constraint { op, lhs, rhs }
                if matches!(classify_constraint_head(op), Some((CmpKind::Eq, _))) =>
            {
                let pair = field_literal_pair(lhs, rhs, var_to_field)
                    .or_else(|| field_literal_pair(rhs, lhs, var_to_field));
                if let Some((field, value)) = pair {
                    if let Some(idx) = field_names.iter().position(|n| *n == field) {
                        // First proof wins; a second `=` on the same field would only ever
                        // co-require an equal or contradictory value — either way this dim's
                        // bucket remains a valid (super-approximating) placement.
                        out.entry(idx).or_insert(value);
                    }
                }
            }
            ReteClauseShape::And(subs) => collect_equalities(subs, var_to_field, field_names, out),
            _ => {}
        }
    }
}

/// If `field_side` names a field of the fact (a bound `?var` or a direct `:field` keyword) and
/// `literal_side` is a bare literal, return `(field_name, value)`.
fn field_literal_pair(
    field_side: &WatAST,
    literal_side: &WatAST,
    var_to_field: &HashMap<String, String>,
) -> Option<(String, Value)> {
    let field: String = match field_side {
        WatAST::Symbol(name, _) if name.as_str().starts_with('?') => {
            var_to_field.get(name.as_str())?.clone()
        }
        // A direct field reference, `:field`, with no preceding `(?v <- :field)` bind —
        // `resolve_operand`'s own reading of a bare Keyword operand.
        WatAST::Keyword(k, _) => k.strip_prefix(':').unwrap_or(k).to_string(),
        _ => return None,
    };
    let value = literal_value(literal_side)?;
    Some((field, value))
}

/// The literal AST kinds `resolve_operand` resolves without touching a fact or bindings —
/// exactly the operand shapes that make a provable, fact-independent discriminator. `Keyword`
/// is deliberately excluded: in operand position it is always a field reference (see
/// `resolve_operand`), never a literal keyword value.
fn literal_value(ast: &WatAST) -> Option<Value> {
    match ast {
        WatAST::IntLit(n, _) => Some(Value::i64(*n)),
        WatAST::FloatLit(x, _) => Some(Value::f64(*x)),
        WatAST::BoolLit(b, _) => Some(Value::bool(*b)),
        WatAST::StringLit(s, _) => Some(Value::String(std::sync::Arc::new(s.clone()))),
        _ => None,
    }
}
