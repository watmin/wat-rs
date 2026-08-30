//! Interned compiled network (`InternedNetwork`) and the thread-owned intern table.
//!
//! ## What "the arm" is, and why it is built exactly once
//!
//! The arm is the rete CONTROL PLANE: everything derivable from a network that the fire loop
//! would otherwise re-derive on every round. Compiled conditions, driver trees, `where`
//! programs, accumulator folds, and a dozen indices — all keyed by node id, all immutable once
//! built. The round loop matches against the arm, never against raw clause forms
//! (`classify_rete_clause` runs HERE, at setup, and never in the loop).
//!
//! `build_rete_arm` is the one constructor and reads top-to-bottom as the build order:
//! node ids → alpha index → alpha tree → compiled conds → drivers → `where`s → user folds →
//! acc folds → `derive_indices`. Each stage consumes the one above it, which is why the order
//! is not arrangeable to taste.
//!
//! ## The intern table is thread-owned — ZERO MUTEX by construction
//!
//! `ARM_TABLE` is a `thread_local!` `RefCell`, not a shared map behind a lock. There is no
//! contention to manage because there is no sharing: a network identity is armed on the thread
//! that fires it. Entries are LEASED — `rete_arm_intern` bumps a count, `rete_arm_release` drops
//! one and evicts at zero — so an arm outlives any single fire but not the last holder.
//!
//! ## Two indices over the same alphas, keyed differently — do not merge them
//!
//! - `alpha_index_by_cond_text` maps condition TEXT → one alpha id, and exists so
//!   `compile_cond_driver` can resolve a driver leaf. Textually identical conditions in
//!   different rules therefore SHARE an alpha — that sharing is the point.
//! - `build_alpha_index` maps fact TYPE → many alpha ids (plus id → cond AST), and exists so the
//!   alpha pass can find the candidates for an incoming fact.
//!
//! One is a lookup, the other a grouping; one is keyed by how a condition is WRITTEN, the other
//! by what it MATCHES. They answer different questions over the same nodes.

use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use crate::ast::WatAST;
use crate::runtime::{EvalBreak, RuntimeError, RuntimeErrorKind, SymbolTable, Value};
use crate::span::Span;
use rustc_hash::FxHashMap;

use super::{
    alpha_cond_from_node, alpha_cond_of, cond_text, get_node, kind_of, node_children,
    node_named_ast, session_named_field,
    session_network, rule_asts_field, rule_bag_consumes, rule_consumes, rule_name_of, rule_negates,
    rule_produces, sorted_node_ids,
    AlphasByType, ChildrenOf, JoinsFedBy, NodeKind, ParentsOf, TestChildren,
    TestSibs, StratifyView,
};
use crate::runtime::ValueSnapshot;

/// Residual stratify row for one named rule (Export ABI + interned network).
#[derive(Clone, Debug)]
pub(crate) struct RuleDep {
    pub name: String,
    pub view: StratifyView,
}

/// Rete control plane, specialized once at fire setup. The round loop
/// matches this, never `classify_rete_clause`.
#[derive(Clone)]
pub(crate) enum CondDriver {
    Leaf(i64),
    And(Vec<CondDriver>),
    Or(Vec<CondDriver>),
    Not(Box<CondDriver>),
    Exists(Box<CondDriver>),
    Where(Arc<crate::rete::expr_ir::Program>),
}

/// Lower one classified condition into the `CondDriver` tree the fire loop walks.
///
/// The combinators (`and`/`or`/`not`/`exists`) recurse structurally and `where` lowers to an
/// `expr_ir::Program`; everything else is FACT-SHAPED and becomes a `Leaf` holding an alpha id,
/// resolved by the condition's TEXT through `alpha_by_text`.
///
/// Resolving by text is what makes two rules with an identical condition share one alpha — the
/// sharing is deliberate, not incidental. A miss is therefore a setup contradiction (a
/// fact-shaped cond that was never minted an alpha) and raises rather than inventing one, since
/// a fabricated leaf would match nothing and read as an empty result.
pub(crate) fn compile_cond_driver(
    cond: &WatAST,
    alpha_by_text: &HashMap<String, i64>,
    sym: &SymbolTable,
) -> Result<CondDriver, EvalBreak> {
    use crate::rete::clause::{classify_rete_clause, ReteClauseShape};
    match classify_rete_clause(cond) {
        ReteClauseShape::And(kids) => {
            let mut out = Vec::with_capacity(kids.len());
            for k in kids {
                out.push(compile_cond_driver(k, alpha_by_text, sym)?);
            }
            Ok(CondDriver::And(out))
        }
        ReteClauseShape::Or(kids) => {
            let mut out = Vec::with_capacity(kids.len());
            for k in kids {
                out.push(compile_cond_driver(k, alpha_by_text, sym)?);
            }
            Ok(CondDriver::Or(out))
        }
        ReteClauseShape::Not(inner) => Ok(CondDriver::Not(Box::new(compile_cond_driver(
            inner, alpha_by_text, sym,
        )?))),
        ReteClauseShape::Exists(inner) => Ok(CondDriver::Exists(Box::new(compile_cond_driver(
            inner, alpha_by_text, sym,
        )?))),
        ReteClauseShape::Where(expr) => {
            let program = crate::rete::expr_ir::lower(expr, sym)
                .map_err(crate::rete::expr_ir::LowerError::into_eval)?;
            Ok(CondDriver::Where(Arc::new(program)))
        }
        _ => {
            let id = alpha_by_text.get(&cond_text(cond)).copied().ok_or_else(|| {
                RuntimeError::new(
                    cond.span().clone(),
                    RuntimeErrorKind::MalformedForm {
                        head: ":wat::rete::fire-rules".into(),
                        reason: format!(
                            "fact-shaped cond has no minted alpha — cannot compile driver: {}",
                            cond_text(cond)
                        ),
                    },
                )
            })?;
            Ok(CondDriver::Leaf(id))
        }
    }
}

/// Condition TEXT → alpha id, for `compile_cond_driver`'s leaf resolution.
///
/// Note the `insert`: identical text collapses to ONE entry, which is exactly the alpha sharing
/// described above. Contrast `build_alpha_index`, which `push`es because many alphas legitimately
/// share a fact type. Same nodes, different question, different collection.
fn alpha_index_by_cond_text(network: &Value, node_ids: &[i64]) -> HashMap<String, i64> {
    let mut out = HashMap::new();
    for id in node_ids {
        let Some(node) = get_node(network, *id) else {
            continue;
        };
        if kind_of(node) != NodeKind::Alpha {
            continue;
        }
        let Some(stored) = alpha_cond_of(network, *id) else {
            continue;
        };
        out.insert(cond_text(&stored), *id);
    }
    out
}

/// Every alpha's driver, in one pass. Builds the text index once and reuses it across all
/// alphas — an index per alpha would be quadratic in a network where conditions cross-reference.
pub(crate) fn compile_all_cond_drivers(
    network: &Value,
    node_ids: &[i64],
    sym: &SymbolTable,
) -> Result<HashMap<i64, CondDriver>, EvalBreak> {
    let alpha_by_text = alpha_index_by_cond_text(network, node_ids);
    let mut out = HashMap::new();
    for id in node_ids {
        let Some(node) = get_node(network, *id) else {
            continue;
        };
        if kind_of(node) != NodeKind::Alpha {
            continue;
        }
        let Some(cond) = alpha_cond_of(network, *id) else {
            continue;
        };
        out.insert(*id, compile_cond_driver(&cond, &alpha_by_text, sym)?);
    }
    Ok(out)
}

pub(crate) fn driver_leaf_ids(d: &CondDriver) -> Vec<i64> {
    match d {
        CondDriver::Leaf(id) => vec![*id],
        CondDriver::And(ks) | CondDriver::Or(ks) => {
            ks.iter().flat_map(driver_leaf_ids).collect()
        }
        CondDriver::Not(inner) | CondDriver::Exists(inner) => driver_leaf_ids(inner),
        CondDriver::Where(_) => Vec::new(),
    }
}

/// Built-in or user accumulate fold, specialized at setup. Fire does not
/// read the `acc-form` AST.
#[derive(Clone)]
pub(crate) enum AccFold {
    Count,
    Sum(Value),
    Min(Value),
    Max(Value),
    Mean(Value),
    Distinct(Value),
    All,
    GroupBy(Value),
    User { var: Value, program: Arc<crate::rete::expr_ir::Program> },
}

impl AccFold {
    pub(crate) fn operand_keys(&self) -> Vec<Value> {
        match self {
            AccFold::Count | AccFold::All => Vec::new(),
            AccFold::Sum(k)
            | AccFold::Min(k)
            | AccFold::Max(k)
            | AccFold::Mean(k)
            | AccFold::Distinct(k)
            | AccFold::GroupBy(k)
            | AccFold::User { var: k, .. } => vec![k.clone()],
        }
    }
}

/// Lower an `accumulate` acc-form into an `AccFold`.
///
/// Eight built-in heads (`count`, `sum`, `min`, `max`, `mean`, `distinct`, `all`, `group-by`)
/// map to their own variants; anything else is a USER fold and needs the `Program` that setup
/// compiled for it. That `compiled_user` being `None` at this point is a setup bug rather than a
/// bad program — the user fold should have been compiled or refused earlier — and it raises
/// saying so.
///
/// The malformed-shape arms come first and are separate for a reason: "acc-form is not a list"
/// and "acc-form has no head" are different diagnoses, and collapsing them into one message
/// would make the more common mistake harder to read.
pub(crate) fn compile_acc_fold(
    acc_form: &WatAST,
    compiled_user: Option<Arc<crate::rete::expr_ir::Program>>,
) -> Result<AccFold, EvalBreak> {
    let items = match acc_form {
        WatAST::List(items, _) => items.as_slice(),
        _ => {
            return Err(RuntimeError::new(
                acc_form.span().clone(),
                RuntimeErrorKind::MalformedForm {
                    head: ":wat::rete::fire-rules".into(),
                    reason: "accumulate acc-form is not a list".into(),
                },
            )
            .into());
        }
    };
    let head = match items.first() {
        Some(WatAST::Keyword(k, _)) => k.as_str(),
        Some(WatAST::Symbol(s, _)) => s.as_str(),
        _ => {
            return Err(RuntimeError::new(
                acc_form.span().clone(),
                RuntimeErrorKind::MalformedForm {
                    head: ":wat::rete::fire-rules".into(),
                    reason: "accumulate acc-form has no head".into(),
                },
            )
            .into());
        }
    };
    let var_key = || -> Result<Value, EvalBreak> {
        let name = match items.get(1) {
            Some(WatAST::Symbol(s, _)) => s.as_str().to_string(),
            Some(WatAST::Keyword(k, _)) => k.as_str().to_string(),
            _ => {
                return Err(RuntimeError::new(
                    acc_form.span().clone(),
                    RuntimeErrorKind::MalformedForm {
                        head: head.into(),
                        reason: format!("accumulate: value-fold {head} missing ?var arg"),
                    },
                )
                .into());
            }
        };
        Ok(Value::String(Arc::new(name)))
    };
    Ok(match head {
        ":wat::rete::acc::count" => AccFold::Count,
        ":wat::rete::acc::sum" => AccFold::Sum(var_key()?),
        ":wat::rete::acc::min" => AccFold::Min(var_key()?),
        ":wat::rete::acc::max" => AccFold::Max(var_key()?),
        ":wat::rete::acc::mean" => AccFold::Mean(var_key()?),
        ":wat::rete::acc::distinct" => AccFold::Distinct(var_key()?),
        ":wat::rete::acc::all" => AccFold::All,
        ":wat::rete::acc::group-by" => AccFold::GroupBy(var_key()?),
        _ => {
            let Some(program) = compiled_user else {
                return Err(RuntimeError::new(
                    acc_form.span().clone(),
                    RuntimeErrorKind::MalformedForm {
                        head: head.into(),
                        reason: "user acc fold has no compiled Program — setup should have refused"
                            .into(),
                    },
                )
                .into());
            };
            AccFold::User {
                var: var_key()?,
                program,
            }
        }
    })
}

/// Fact TYPE → the alphas that test it, plus each alpha's condition AST.
///
/// The type key is the condition's fact-type head read through `alpha_pattern` — deliberately
/// the SAME reader `alpha_match_inner` uses, so the index cannot disagree with the matcher about
/// what an alpha is for. An alpha whose condition has no readable pattern is skipped rather than
/// filed under a guessed type.
pub(crate) fn build_alpha_index(
    network: &Value,
    node_ids: &[i64],
) -> (AlphasByType, HashMap<i64, WatAST>) {
    let mut alpha_by_type: AlphasByType = HashMap::new();
    let mut alpha_cond: HashMap<i64, WatAST> = HashMap::new();
    for node_id in node_ids {
        // Group C: use &Value ref — no clone needed; only reads the network here.
        let node = match get_node(network, *node_id) {
            Some(n) => n,
            None => continue,
        };
        if kind_of(node) != NodeKind::Alpha {
            continue;
        }
        let Some(cond_ast) = alpha_cond_from_node(node) else {
            continue;
        };
        // The condition's fact-type head (colon-free), exactly as alpha_match_inner reads it.
        if let Some(pat) = crate::rete::matcher::alpha_pattern(&cond_ast) {
            let ty = pat.type_head.to_string();
            alpha_by_type.entry(ty).or_default().push(*node_id);
            alpha_cond.insert(*node_id, cond_ast);
        }
    }
    (alpha_by_type, alpha_cond)
}

/// Leftover-as-seed compile of every alpha. Populate skips `SeedCmp`; rematch
/// requires it. A miss is a setup hole — refuse, do not walk `alpha_match_inner`.
pub(crate) fn compile_alpha_conds_from_index(
    alpha_by_type: &AlphasByType,
    alpha_cond: &HashMap<i64, WatAST>,
    sym: &SymbolTable,
) -> Result<HashMap<i64, crate::rete::compiled_cond::CompiledCond>, EvalBreak> {
    let mut compiled_conds = HashMap::with_capacity(alpha_cond.len());
    for (class, ids) in alpha_by_type {
        let field_names = class_field_names(sym, class);
        for aid in ids {
            let Some(cond) = alpha_cond.get(aid) else {
                continue;
            };
            let compiled = crate::rete::compiled_cond::compile_condition_local(cond, &field_names, sym)
                .ok_or_else(|| {
                    RuntimeError::new(
                        cond.span().clone(),
                        RuntimeErrorKind::MalformedForm {
                            head: ":wat::rete::fire-rules".into(),
                            reason: format!(
                                "alpha {aid} cond did not compile — setup should compile every fact-shaped alpha"
                            ),
                        },
                    )
                })?;
            compiled_conds.insert(*aid, compiled);
        }
    }
    Ok(compiled_conds)
}

/// A `:then` item that `compile_rhs` cannot represent. Refuse — do not walk
/// `build_insert_fact` on native fire.
pub(crate) fn rhs_must_compile(
    form: &WatAST,
    sym: &SymbolTable,
) -> Result<crate::rete::compiled_rhs::CompiledRhs, EvalBreak> {
    crate::rete::compiled_rhs::compile_rhs(form, sym)?.ok_or_else(|| {
        RuntimeError::new(
            form.span().clone(),
            RuntimeErrorKind::MalformedForm {
                head: ":wat::rete::fire-rules".into(),
                reason: "then item did not compile — fire does not walk build_insert_fact"
                    .into(),
            },
        )
        .into()
    })
}

pub(crate) use crate::rete::matcher::class_field_names;

pub(crate) type UserFoldPrograms = HashMap<i64, Arc<crate::rete::expr_ir::Program>>;

/// Flip 5 — lower each AccumulateNode whose acc-form head is a user rete-defn.
/// Built-in `:wat::rete::acc::*` heads are skipped. A `LowerError` refuses
/// the fire (same door as `compile_test_programs`). The old
/// `(user-fn __acc__)` / `eval_inner` arm is gone.
pub(crate) fn compile_user_fold_programs(
    network: &Value,
    node_ids: &[i64],
    sym: &SymbolTable,
) -> Result<UserFoldPrograms, EvalBreak> {
    let mut out = HashMap::new();
    for node_id in node_ids {
        let node = match get_node(network, *node_id) {
            Some(n) => n,
            None => continue,
        };
        if kind_of(node) != NodeKind::Accumulate {
            continue;
        }
        let Some(acc_form) = node_named_ast(node, "acc-form") else {
            continue;
        };
        let items = match acc_form {
            WatAST::List(items, _) => items.as_slice(),
            _ => continue,
        };
        let head = match items.first() {
            Some(WatAST::Keyword(k, _)) => k.as_str(),
            Some(WatAST::Symbol(s, _)) => s.as_str(),
            _ => continue,
        };
        if head.starts_with(":wat::rete::acc::") {
            continue;
        }
        let program = crate::rete::expr_ir::lower_named_rete_fn(head, acc_form.span(), sym)
            .map_err(crate::rete::expr_ir::LowerError::into_eval)?;
        out.insert(*node_id, program);
    }
    Ok(out)
}

/// Compile every TestNode's `:expr` once, beside `compiled_conds`.
/// Compile-condition already refused anything `lower` cannot take; a miss here
/// is a bug in that fence, not a fire-time interpreter door.
pub(crate) fn compile_test_programs(
    network: &Value,
    node_ids: &[i64],
    sym: &SymbolTable,
) -> Result<HashMap<i64, crate::rete::expr_ir::Program>, EvalBreak> {
    let mut out = HashMap::new();
    for node_id in node_ids {
        let node = match get_node(network, *node_id) {
            Some(n) => n,
            None => continue,
        };
        if kind_of(node) != NodeKind::Test {
            continue;
        }
        let Some(expr) = node_named_ast(node, "expr") else {
            continue;
        };
        let program = crate::rete::expr_ir::lower(expr, sym)
            .map_err(crate::rete::expr_ir::LowerError::into_eval)?;
        out.insert(*node_id, program);
    }
    Ok(out)
}

// ── Item 12 — the arm, persisted next to the network ─────────────────────────

/// Kind-partitioned node ids, each a subsequence of `node_ids` (topo).
/// Fire-path passes iterate these instead of `node_ids` + `get_node` +
/// `kind_of` (`DESIGN-STONE-arm-kind-lists`).
pub(crate) struct KindIdLists {
    pub(crate) alpha: Vec<i64>,
    pub(crate) join_parent: Vec<i64>,
    pub(crate) acc: Vec<i64>,
    pub(crate) filter: Vec<i64>,
    pub(crate) prod: Vec<i64>,
    pub(crate) filter_or_acc: Vec<i64>,
    pub(crate) query: Vec<i64>,
}

/// Partition node ids by kind, once, so the fire loop can iterate one kind without re-testing
/// every node each round.
///
/// Two groupings are coarser than `NodeKind` on purpose: `join_parent` merges RootJoin with
/// HashJoin (both are left-parents to a join), and `filter` merges Test/Negation/Exists (all
/// three gate tokens rather than producing them). `filter_or_acc` is then the sorted merge of
/// two already-sorted lists — cheaper than concatenating and re-sorting, and it preserves the
/// ascending node order the passes rely on for topological correctness.
pub(crate) fn kind_id_lists(network: &Value, node_ids: &[i64]) -> KindIdLists {
    let mut alpha = Vec::new();
    let mut join_parent = Vec::new();
    let mut acc = Vec::new();
    let mut filter = Vec::new();
    let mut prod = Vec::new();
    let mut query = Vec::new();
    for &id in node_ids {
        let Some(node) = get_node(network, id) else {
            continue;
        };
        match kind_of(node) {
            NodeKind::Alpha => alpha.push(id),
            NodeKind::RootJoin | NodeKind::HashJoin => join_parent.push(id),
            NodeKind::Accumulate => acc.push(id),
            NodeKind::Test | NodeKind::Negation | NodeKind::Exists => filter.push(id),
            NodeKind::Production => prod.push(id),
            NodeKind::Query => query.push(id),
        }
    }
    let filter_or_acc = merge_sorted_ids(&filter, &acc);
    KindIdLists {
        alpha,
        join_parent,
        acc,
        filter,
        prod,
        filter_or_acc,
        query,
    }
}

pub(crate) fn invert_feeding_alpha(feeding_alpha_of: &HashMap<i64, i64>) -> JoinsFedBy {
    let mut out: JoinsFedBy = HashMap::new();
    for (join_id, alpha_id) in feeding_alpha_of {
        out.entry(*alpha_id).or_default().push(*join_id);
    }
    out
}

/// One intern-topology decision: children / feeding-alpha / parents / beta-readers.
/// `build_rete_arm`, `subset_rete_arm`, and Export import share this walk.
pub(crate) struct NetworkEdges {
    pub feeding_alpha_of: HashMap<i64, i64>,
    pub parents_of: ParentsOf,
    pub children_of: ChildrenOf,
    pub beta_readers: HashSet<i64>,
}

/// The four edge indices, in two walks.
///
/// The first walk splits parenthood in TWO, and the asymmetry is the whole point: a node has at
/// most ONE feeding alpha (`feeding_alpha_of`, a plain map) but may have MANY beta parents
/// (`parents_of`, a map of lists). Storing them in one structure would force the alpha case to
/// carry a vector it can never fill past one.
///
/// The second walk computes `beta_readers`: nodes with a HashJoin or Query child. This is the
/// set whose beta memory anyone will actually READ, and the fire loop uses it to skip writing
/// beta for everyone else (`fire/pass/mod.rs`'s `record_token`). It is a separate walk because
/// it needs to look at each child's KIND, which the first walk does not fetch.
pub(crate) fn index_network_edges(network: &Value, node_ids: &[i64]) -> NetworkEdges {
    let mut feeding_alpha_of: HashMap<i64, i64> = HashMap::new();
    let mut parents_of: ParentsOf = HashMap::new();
    let mut children_of: ChildrenOf = HashMap::new();
    for node_id in node_ids {
        let Some(node) = get_node(network, *node_id) else {
            continue;
        };
        let kids = node_children(node);
        children_of.insert(*node_id, kids.clone());
        let is_alpha = kind_of(node) == NodeKind::Alpha;
        for child in kids {
            if is_alpha {
                feeding_alpha_of.insert(child, *node_id);
            } else {
                parents_of.entry(child).or_default().push(*node_id);
            }
        }
    }
    let mut beta_readers = HashSet::new();
    for node_id in node_ids {
        let Some(node) = get_node(network, *node_id) else {
            continue;
        };
        for child in node_children(node) {
            if let Some(child_node) = get_node(network, child) {
                let k = kind_of(child_node);
                if k == NodeKind::HashJoin || k == NodeKind::Query {
                    beta_readers.insert(*node_id);
                    break;
                }
            }
        }
    }
    NetworkEdges {
        feeding_alpha_of,
        parents_of,
        children_of,
        beta_readers,
    }
}


/// Sorted union of two sorted id lists, deduplicating equal heads.
///
/// A merge rather than `concat` + `sort` + `dedup` because both inputs are ALREADY sorted —
/// ascending node id is the topological order every pass depends on, so the output must keep it
/// and a re-sort would be paying to rediscover what the inputs already knew.
pub(crate) fn merge_sorted_ids(a: &[i64], b: &[i64]) -> Vec<i64> {
    let mut out = Vec::with_capacity(a.len() + b.len());
    let (mut i, mut j) = (0, 0);
    while i < a.len() && j < b.len() {
        if a[i] < b[j] {
            out.push(a[i]);
            i += 1;
        } else if b[j] < a[i] {
            out.push(b[j]);
            j += 1;
        } else {
            out.push(a[i]);
            i += 1;
            j += 1;
        }
    }
    out.extend_from_slice(&a[i..]);
    out.extend_from_slice(&b[j..]);
    out
}

/// Compiled circuits for one network. Interned by the network PMap's
/// `rust_identity`; `insert` / clone share that identity and fire skips setup.
/// Thread-owned `Arc` per unique network (`DESIGN-STONE-intern-zero-mutex`).
/// The Session is a fact overlay, not the owner of the circuits. Never EDN.
pub(crate) struct InternedNetwork {
    pub(crate) node_ids: Vec<i64>,
    pub(crate) kind_ids: KindIdLists,
    pub(crate) compiled_conds: HashMap<i64, crate::rete::compiled_cond::CompiledCond>,
    pub(crate) compiled_drivers: HashMap<i64, CondDriver>,
    pub(crate) compiled_wheres: HashMap<i64, crate::rete::expr_ir::Program>,
    pub(crate) compiled_acc_folds: HashMap<i64, AccFold>,
    pub(crate) compiled_rhs: crate::rete::compiled_rhs::CompiledRhsByRule,
    pub(crate) alpha_tree: crate::rete::alpha_tree::AlphaTree,
    /// (b) WhereDiscNode — armed `where` index. Built from `compiled_wheres`.
    pub(crate) where_tree: crate::rete::where_tree::WhereTree,
    pub(crate) feeding_alpha_of: HashMap<i64, i64>,
    /// Invert of `feeding_alpha_of`: alpha id → HashJoin ids it feeds.
    /// Dirty-join-parent construction (`DESIGN-STONE-dirty-join-parents`).
    pub(crate) joins_fed_by: JoinsFedBy,
    pub(crate) parents_of: ParentsOf,
    pub(crate) beta_readers: HashSet<i64>,
    pub(crate) compiled_max_slots: usize,
    /// Residual stratify row per named rule (`RuleDep`).
    /// Import has no rule AST; fire-rules reads this instead.
    pub(crate) rule_deps: Vec<RuleDep>,
    /// TestNode id → all TestNodes sharing its sorted parent-set (where-tree dispatch).
    pub(crate) test_sibs: TestSibs,
    /// Node id → TestNode children (filter-after-join sibling walk).
    pub(crate) test_children: TestChildren,
    /// Node id → children ids interned at arm build (fire does not re-scan names).
    pub(crate) children_of: ChildrenOf,
}

pub(crate) fn network_identity(network: &Value) -> Option<u64> {
    match network {
        Value::wat__core__PersistentMap(m) => Some(m.rust_identity()),
        _ => None,
    }
}

/// Thread-owned intern entry. `leases` is the owner count (`DESIGN-STONE-intern-eviction`).
/// Fire HIT does not lease. `arm-session` does. Last lease drop removes the row.
struct InternEntry {
    arm: Arc<InternedNetwork>,
    leases: usize,
}

// rune:sequi(ambient-context) — ZERO-MUTEX intern index is thread-owned RefCell
// (`DESIGN-STONE-intern-zero-mutex` THE ONE CONTRACT: Session stays 8 fields;
// `DESIGN-STONE-intern-eviction` forbids an intern handle on Session). Circuits
// are a pure function of network+rules; fire threads `Arc<InternedNetwork>`
// after `get_or_build`. The table is the worker memo, not a Session overlay.
// It holds DOMAIN state (the armed network + its lease count) reached by id
// rather than through any signature, which is what makes it ambient-context and
// not host-idiom — cf. `EXEC_ARENA` (expr_ir.rs), the same shape, same category.
// Recategorised 2026-08-25: `sequi` found it labelled `host-idiom` beside an
// identical `ambient-context` neighbour. See CONVENTIONS.md, "The `rune:sequi`
// vocabulary" — the categories had no written definition, so nothing could
// notice the two disagreeing.
// rune:circumspicere(accepted-by-design) — lease is `arm-session`/`release-session`,
// not Session Drop (stone 29). Connection-thread affinity is the ZERO-MUTEX
// contract (DESIGN-STONE-intern-zero-mutex): fire/release on another thread
// miss the arming thread's row. Bound named in those two stones.
thread_local! {
    static ARM_TABLE: RefCell<FxHashMap<u64, InternEntry>> =
        RefCell::new(FxHashMap::default());
}

#[cfg(test)]
// rune:sequi(performance-counter) — test-only intern-miss count; not fire domain.
pub(crate) static ARM_BUILDS: std::sync::atomic::AtomicUsize =
    std::sync::atomic::AtomicUsize::new(0);

pub(crate) fn rete_arm_lookup(id: u64) -> Option<Arc<InternedNetwork>> {
    ARM_TABLE.with(|t| t.borrow().get(&id).map(|e| Arc::clone(&e.arm)))
}

#[cfg(test)]
pub(crate) fn rete_arm_leases(id: u64) -> Option<usize> {
    ARM_TABLE.with(|t| t.borrow().get(&id).map(|e| e.leases))
}

/// Publish an arm under a network identity and take a lease on it.
///
/// Re-interning an id REPLACES the arm and still increments — the newest build wins, and the
/// count tracks holders rather than builds. See `rete_arm_release` for the other half; the table
/// is thread-local, so neither needs a lock (module header).
pub(crate) fn rete_arm_intern(id: u64, arm: &Arc<InternedNetwork>) {
    ARM_TABLE.with(|t| {
        let mut m = t.borrow_mut();
        match m.get_mut(&id) {
            Some(e) => {
                e.arm = Arc::clone(arm);
                e.leases = e.leases.saturating_add(1);
            }
            None => {
                m.insert(
                    id,
                    InternEntry {
                        arm: Arc::clone(arm),
                        leases: 1,
                    },
                );
            }
        }
    });
}

/// Drop one lease. At zero the intern entry is gone. Missing id is a no-op
/// (hangup after already deprovisioned).
pub(crate) fn rete_arm_release(id: u64) {
    ARM_TABLE.with(|t| {
        let mut m = t.borrow_mut();
        let drop = match m.get_mut(&id) {
            Some(e) if e.leases <= 1 => true,
            Some(e) => {
                e.leases -= 1;
                false
            }
            None => false,
        };
        if drop {
            m.remove(&id);
        }
    });
}

fn rete_arm_build_put(
    network: &Value,
    rules: &Value,
    sym: &SymbolTable,
) -> Result<Arc<InternedNetwork>, EvalBreak> {
    let arm = Arc::new(build_rete_arm(network, rules, sym)?);
    if let Some(id) = network_identity(network) {
        rete_arm_intern(id, &arm);
    }
    Ok(arm)
}

pub(crate) fn rete_arm_get_or_build(
    network: &Value,
    rules: &Value,
    sym: &SymbolTable,
) -> Result<Arc<InternedNetwork>, EvalBreak> {
    if let Some(id) = network_identity(network) {
        if let Some(arm) = rete_arm_lookup(id) {
            return Ok(arm);
        }
    }
    rete_arm_build_put(network, rules, sym)
}

/// `arm-session` door: HIT increments the lease; MISS intern's with leases=1.
fn rete_arm_lease_or_build(
    network: &Value,
    rules: &Value,
    sym: &SymbolTable,
) -> Result<Arc<InternedNetwork>, EvalBreak> {
    if let Some(id) = network_identity(network) {
        let hit = ARM_TABLE.with(|t| {
            t.borrow_mut().get_mut(&id).map(|e| {
                e.leases = e.leases.saturating_add(1);
                Arc::clone(&e.arm)
            })
        });
        if let Some(arm) = hit {
            return Ok(arm);
        }
    }
    rete_arm_build_put(network, rules, sym)
}

/// Build the whole control plane for one network. The single constructor for `InternedNetwork`.
///
/// The body reads as the build order because it IS the dependency order: node ids feed the alpha
/// index, which feeds both the alpha tree and the compiled conds; drivers need the network and
/// the symbol table; `where`s and user folds are independent; acc folds need the user folds that
/// precede them; and `derive_indices` closes over the compiled results. Reordering it is not a
/// style choice.
///
/// Accumulate nodes are walked here rather than in a helper because the fold needs BOTH the
/// node's `acc-form` and the user program compiled two lines above — a helper would have to take
/// the map back apart.
pub(crate) fn build_rete_arm(
    network: &Value,
    rules: &Value,
    sym: &SymbolTable,
) -> Result<InternedNetwork, EvalBreak> {
    #[cfg(test)]
    ARM_BUILDS.fetch_add(1, std::sync::atomic::Ordering::Relaxed);

    let node_ids = sorted_node_ids(network);
    let (alpha_by_type, alpha_cond) = build_alpha_index(network, &node_ids);
    let alpha_tree = crate::rete::alpha_tree::AlphaTree::build(&alpha_by_type, &alpha_cond, sym);
    let compiled_conds = compile_alpha_conds_from_index(&alpha_by_type, &alpha_cond, sym)?;
    let compiled_drivers = compile_all_cond_drivers(network, &node_ids, sym)?;
    let compiled_wheres = compile_test_programs(network, &node_ids, sym)?;
    let compiled_user_folds = compile_user_fold_programs(network, &node_ids, sym)?;
    let mut compiled_acc_folds: HashMap<i64, AccFold> = HashMap::new();
    for node_id in &node_ids {
        let Some(node) = get_node(network, *node_id) else {
            continue;
        };
        if kind_of(node) != NodeKind::Accumulate {
            continue;
        }
        let Some(acc_form) = node_named_ast(node, "acc-form") else {
            continue;
        };
        compiled_acc_folds.insert(
            *node_id,
            compile_acc_fold(acc_form, compiled_user_folds.get(node_id).cloned())?,
        );
    }

    // ONE RECIPE — see `derive_indices` (and its ⛔ note: a THIRD site, `import_export`,
    // bypasses this recipe). The two ARM BUILDERS derive the same ten indices;
    // the only difference is WHICH NETWORK they are handed, and that is the difference
    // that must not be gettable wrong.
    let DerivedIndices {
        kind_ids,
        where_tree,
        feeding_alpha_of,
        joins_fed_by,
        parents_of,
        children_of,
        beta_readers,
        compiled_max_slots,
        test_sibs,
        test_children,
    } = derive_indices(network, &node_ids, &compiled_conds, &compiled_wheres);

    let rule_vec: Vec<Value> = match rules {
        Value::wat__core__PersistentVector(pv) => pv.iter().cloned().collect(),
        _ => vec![],
    };
    let mut compiled_rhs: crate::rete::compiled_rhs::CompiledRhsByRule =
        HashMap::new();
    for r in &rule_vec {
        if let Some(rname) = rule_name_of(r) {
            let rhs = rule_asts_field(r, "rhs");
            let mut compiled: Vec<crate::rete::compiled_rhs::CompiledRhs> =
                Vec::with_capacity(rhs.len());
            for f in &rhs {
                compiled.push(rhs_must_compile(f, sym)?);
            }
            compiled_rhs.insert(rname.to_string(), compiled);
        }
    }

    let rule_deps = rule_deps_from_rules(rules, sym);

    Ok(InternedNetwork {
        node_ids,
        kind_ids,
        compiled_conds,
        compiled_drivers,
        compiled_wheres,
        compiled_acc_folds,
        compiled_rhs,
        alpha_tree,
        where_tree,
        feeding_alpha_of,
        joins_fed_by,
        parents_of,
        beta_readers,
        compiled_max_slots,
        rule_deps,
        test_sibs,
        test_children,
        children_of,
    })
}

/// Everything an [`InternedNetwork`] DERIVES from a network, its node set, and its
/// compiled maps — as opposed to what it merely CARRIES.
///
/// ONE RECIPE. `build_rete_arm` (the whole network) and `subset_rete_arm` (one stratum's
/// slice) each used to spell out the same ten derivations. The struct literal already
/// stopped a MISSING field — literals are exhaustive, so a new `InternedNetwork` field
/// fails to compile on both paths. What it could not stop is the far quieter mistake:
/// deriving a new index in the slice path from the FULL arm (`arm.foo`) instead of from
/// the slice, producing a stratum-sliced arm carrying a stale index. That is visible only
/// under stratified fire, which is the same "wrong only in a configuration nothing
/// exercises" shape as this arc's other silent defects.
///
/// The MECHANISM is sound: `derive_indices` is handed a network and has no `arm` parameter, so
/// it cannot reach a prior arm and every field it produces is derived from the source it was
/// given. A caller that uses it cannot make the stale-index mistake.
///
/// ⛔ **BUT ITS COVERAGE IS NOT TOTAL, AND AN EARLIER VERSION OF THIS DOC CLAIMED IT WAS.** It
/// said the mistake is "unrepresentable" and that "both callers" get it right. There are THREE
/// construction sites, not two, and the third does not use this recipe: `import_export`
/// (`rete/export.rs`) hand-rolls all eight derivations inline and never calls `derive_indices`.
/// `NetworkEdges` two hundred lines up has always named all three — "`build_rete_arm`,
/// `subset_rete_arm`, and Export import share this walk" — so the file contradicted itself.
///
/// **So: adding a derived index here reaches two of three sites, and leaves import stale.** Add
/// it to `import_export` too, or route that site through this function. `intueri` found this
/// 2026-08-30; the mechanism was verified sound the same day by `probare`, which is why the two
/// halves are stated separately above.
pub(crate) struct DerivedIndices {
    pub(crate) kind_ids: KindIdLists,
    pub(crate) where_tree: crate::rete::where_tree::WhereTree,
    pub(crate) feeding_alpha_of: HashMap<i64, i64>,
    pub(crate) joins_fed_by: JoinsFedBy,
    pub(crate) parents_of: ParentsOf,
    pub(crate) children_of: ChildrenOf,
    pub(crate) beta_readers: HashSet<i64>,
    pub(crate) compiled_max_slots: usize,
    pub(crate) test_sibs: TestSibs,
    pub(crate) test_children: TestChildren,
}

/// Everything derivable from an already-compiled network: the edge indices, the kind
/// partitions, the where-tree, the test sibling/child maps, and `compiled_max_slots`.
///
/// Split from `build_rete_arm` because these are PURE functions of what came before — nothing
/// here compiles, everything here indexes. `compiled_max_slots` is the widest frame any compiled
/// cond needs, computed once so the fire loop can size its scratch frame a single time instead
/// of per node.
pub(crate) fn derive_indices(
    network: &Value,
    node_ids: &[i64],
    compiled_conds: &HashMap<i64, crate::rete::compiled_cond::CompiledCond>,
    compiled_wheres: &HashMap<i64, crate::rete::expr_ir::Program>,
) -> DerivedIndices {
    let NetworkEdges {
        feeding_alpha_of,
        parents_of,
        children_of,
        beta_readers,
    } = index_network_edges(network, node_ids);
    DerivedIndices {
        kind_ids: kind_id_lists(network, node_ids),
        where_tree: crate::rete::where_tree::WhereTree::build(compiled_wheres),
        joins_fed_by: invert_feeding_alpha(&feeding_alpha_of),
        test_sibs: build_test_sibs(network, node_ids, &parents_of),
        test_children: build_test_children(network, node_ids),
        compiled_max_slots: compiled_conds.values().map(|c| c.n_slots()).max().unwrap_or(0),
        feeding_alpha_of,
        parents_of,
        children_of,
        beta_readers,
    }
}

/// TestNodes that share a parent-set dispatch together through the where-tree.
pub(crate) fn build_test_sibs(
    network: &Value,
    node_ids: &[i64],
    parents_of: &ParentsOf,
) -> TestSibs {
    // rune:perspicere(read-once) — parent-set grouping for one sibs build; alias would be a one-site mumble.
    let mut by_parents: HashMap<Vec<i64>, Vec<i64>> = HashMap::new();
    for &id in node_ids {
        let Some(node) = get_node(network, id) else {
            continue;
        };
        if kind_of(node) != NodeKind::Test {
            continue;
        }
        let mut p = parents_of.get(&id).cloned().unwrap_or_default();
        p.sort_unstable();
        by_parents.entry(p).or_default().push(id);
    }
    let mut out = HashMap::new();
    for group in by_parents.into_values() {
        for &id in &group {
            out.insert(id, group.clone());
        }
    }
    out
}

/// TestNode children of each node — interned so fire does not re-kind-walk.
pub(crate) fn build_test_children(network: &Value, node_ids: &[i64]) -> TestChildren {
    let mut out = HashMap::new();
    for &id in node_ids {
        let Some(node) = get_node(network, id) else {
            continue;
        };
        let kids: Vec<i64> = node_children(node)
            .into_iter()
            .filter(|&c| {
                get_node(network, c)
                    .map(|n| kind_of(n) == NodeKind::Test)
                    .unwrap_or(false)
            })
            .collect();
        if !kids.is_empty() {
            out.insert(id, kids);
        }
    }
    out
}

/// Stratify inputs from the rule AST. Name, produced, negated, consumed.
pub(crate) fn rule_deps_from_rules(rules: &Value, sym: &SymbolTable) -> Vec<RuleDep> {
    let rule_vec: Vec<Value> = match rules {
        Value::wat__core__PersistentVector(pv) => pv.iter().cloned().collect(),
        _ => vec![],
    };
    let mut out = Vec::new();
    for r in &rule_vec {
        let Some(name) = rule_name_of(r) else {
            continue;
        };
        let lhs = rule_asts_field(r, "lhs");
        let rhs = rule_asts_field(r, "rhs");
        out.push(RuleDep {
            name,
            view: StratifyView {
                produced: rule_produces(&rhs, sym),
                negated: rule_negates(&lhs),
                consumed: rule_consumes(&lhs),
                exists_and_from_types: rule_bag_consumes(&lhs),
            },
        });
    }
    out
}

/// Filter an armed network down to a stratum slice. The slice is a new
/// PMap. The caller holds the `Arc`; fire does not intern the slice.
pub(crate) fn subset_rete_arm(
    arm: &InternedNetwork,
    active_ids: &HashSet<i64>,
    rule_names: &HashSet<String>,
    sliced_network: &Value,
) -> Arc<InternedNetwork> {
    let node_ids: Vec<i64> = arm
        .node_ids
        .iter()
        .copied()
        .filter(|id| active_ids.contains(id))
        .collect();
    let keep = |id: &i64| active_ids.contains(id);
    let compiled_conds: HashMap<i64, crate::rete::compiled_cond::CompiledCond> = arm
        .compiled_conds
        .iter()
        .filter(|(id, _)| keep(id))
        .map(|(id, c)| (*id, c.clone()))
        .collect();
    let compiled_drivers: HashMap<i64, CondDriver> = arm
        .compiled_drivers
        .iter()
        .filter(|(id, _)| keep(id))
        .map(|(id, d)| (*id, d.clone()))
        .collect();
    let compiled_wheres: HashMap<i64, crate::rete::expr_ir::Program> = arm
        .compiled_wheres
        .iter()
        .filter(|(id, _)| keep(id))
        .map(|(id, p)| (*id, p.clone()))
        .collect();
    let compiled_acc_folds: HashMap<i64, AccFold> = arm
        .compiled_acc_folds
        .iter()
        .filter(|(id, _)| keep(id))
        .map(|(id, f)| (*id, f.clone()))
        .collect();
    let compiled_rhs: crate::rete::compiled_rhs::CompiledRhsByRule = arm
        .compiled_rhs
        .iter()
        .filter(|(n, _)| rule_names.contains(*n))
        .map(|(n, r)| (n.clone(), r.to_vec()))
        .collect();
    let rule_deps: Vec<RuleDep> = arm
        .rule_deps
        .iter()
        .filter(|d| rule_names.contains(&d.name))
        .cloned()
        .collect();
    let alpha_tree = arm.alpha_tree.restrict(active_ids);

    // ONE RECIPE — see `derive_indices` (and its ⛔ note: a THIRD site, `import_export`,
    // bypasses this recipe). The two ARM BUILDERS derive the same ten indices;
    // the only difference is WHICH NETWORK they are handed, and that is the difference
    // that must not be gettable wrong.
    let DerivedIndices {
        kind_ids,
        where_tree,
        feeding_alpha_of,
        joins_fed_by,
        parents_of,
        children_of,
        beta_readers,
        compiled_max_slots,
        test_sibs,
        test_children,
    } = derive_indices(sliced_network, &node_ids, &compiled_conds, &compiled_wheres);

    Arc::new(InternedNetwork {
        node_ids,
        kind_ids,
        compiled_conds,
        compiled_drivers,
        compiled_wheres,
        compiled_acc_folds,
        compiled_rhs,
        alpha_tree,
        where_tree,
        feeding_alpha_of,
        joins_fed_by,
        parents_of,
        beta_readers,
        compiled_max_slots,
        rule_deps,
        test_sibs,
        test_children,
        children_of,
    })
}


// ── Public entry: intern the arm at compile-all ──────────────────────────────

/// `(:wat::rete::arm-session <session>) -> :wat::rete::CompileOutcome`
///
/// Arc 278 — it answers the termination VERDICT as a value. See `kernel::outcome` for why that
/// one refusal converts while this function's ArityMismatch/TypeMismatch stay raises.
///
/// Item 12's contract: compile puts the arm next to the network. WAT
/// `compile-all` builds the Session and did not intern the rust `InternedNetwork`;
/// first `fire-rules` paid the build (`DESIGN-STONE-arm-at-compile`).
/// Value unchanged. Takes one intern lease (`DESIGN-STONE-intern-eviction`).
pub(crate) fn eval_arm_session(
    args: &[WatAST],
    list_span: &Span,
    env: &crate::runtime::Environment,
    sym: &SymbolTable,
) -> Result<Value, EvalBreak> {
    const OP: &str = ":wat::rete::arm-session";
    if args.len() != 1 {
        return Err(RuntimeError::new(
            list_span.clone(),
            RuntimeErrorKind::ArityMismatch {
                op: OP.into(),
                expected: 1,
                got: args.len(),
            },
        )
        .into());
    }
    let session = crate::runtime::eval_inner(&args[0], env, sym)?.value_owned();
    let (network, rules) = match (
        session_network(&session),
        session_named_field(&session, "rules"),
    ) {
        (Some(network), Some(rules)) => (network, rules),
        _ => {
            return Err(RuntimeError::new(
                list_span.clone(),
                RuntimeErrorKind::TypeMismatch {
                    op: OP.into(),
                    expected: ":wat::rete::Session",
                    got: Box::new(ValueSnapshot::of(&session)),
                },
            )
            .into());
        }
    };
    if network_identity(network).is_none() {
        return Err(RuntimeError::new(
            list_span.clone(),
            RuntimeErrorKind::TypeMismatch {
                op: OP.into(),
                expected: ":wat::core::PersistentMap network with intern identity",
                got: Box::new(ValueSnapshot::of(network)),
            },
        )
        .into());
    }
    // THE TERMINATION VERIFIER — before the arm is built, and before a fact can be inserted.
    // Here rather than in the freeze-time `defrule` wall because `compile-all` is the one door
    // EVERY rule passes: rules built at runtime as `Rule` values (both differential fuzzers do
    // this) never see that wall. See `stratify::refuse_non_terminating`.
    // ⛔ THE WALL, at the wat boundary. The termination verdict becomes a matchable
    // `(:wat::rete::CompileOutcome)` rather than a raise — see `kernel::outcome` for why THIS
    // refusal converts while `arm-session`'s ArityMismatch/TypeMismatch do not. The `?` is gone
    // deliberately: the verdict must reach the converter, not unwind past it.
    if let Err(e) = crate::rete::kernel::stratify::refuse_non_terminating(rules, sym) {
        return crate::rete::kernel::outcome::compile_result_to_outcome(Err(e));
    }
    rete_arm_lease_or_build(network, rules, sym)?;
    // THE SESSION'S ZERO POINT. `compile-all` calls `arm-session` for every session it builds, so
    // this is the one door a session is born through — the same reason the termination verifier
    // runs here. Everything the session allocates from now on is charged to it, at BOTH doors it
    // can grow through (`insert` and the fixpoint), against `max-session-bytes`.
    crate::alloc_counter::mark_session_origin();
    crate::rete::kernel::outcome::compile_result_to_outcome(Ok(session))
}

/// `(:wat::rete::release-session <session>) -> :wat::rete::Session`
///
/// Drop one intern lease for this Session's network identity
/// (`DESIGN-STONE-intern-eviction`). Value unchanged. Missing intern
/// is a no-op. At zero the intern entry is gone.
pub(crate) fn eval_release_session(
    args: &[WatAST],
    list_span: &Span,
    env: &crate::runtime::Environment,
    sym: &SymbolTable,
) -> Result<Value, EvalBreak> {
    // Keyword primitive (TypeScheme + runtime dispatch), not a dual-impl wat Fn.
    // Bound in DESIGN-STONE-intern-eviction.md.
    const OP: &str = ":wat::rete::release-session";
    if args.len() != 1 {
        return Err(RuntimeError::new(
            list_span.clone(),
            RuntimeErrorKind::ArityMismatch {
                op: OP.into(),
                expected: 1,
                got: args.len(),
            },
        )
        .into());
    }
    let session = crate::runtime::eval_inner(&args[0], env, sym)?.value_owned();
    let network = match session_network(&session) {
        Some(network) => network,
        None => {
            return Err(RuntimeError::new(
                list_span.clone(),
                RuntimeErrorKind::TypeMismatch {
                    op: OP.into(),
                    expected: ":wat::rete::Session",
                    got: Box::new(ValueSnapshot::of(&session)),
                },
            )
            .into());
        }
    };
    if let Some(id) = network_identity(network) {
        rete_arm_release(id);
    } else {
        return Err(RuntimeError::new(
            list_span.clone(),
            RuntimeErrorKind::TypeMismatch {
                op: OP.into(),
                expected: ":wat::core::PersistentMap network with intern identity",
                got: Box::new(ValueSnapshot::of(network)),
            },
        )
        .into());
    }
    Ok(session)
}
