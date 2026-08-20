//! Arc 278 — DESIGN-STONE-compiled-conditions.md: conditions compile once; the matcher stops
//! re-deriving a static program.
//!
//! ## ⚠ Not a perf stone (read the DESIGN's 2026-08-01 amendment)
//!
//! `alpha_match_inner` (`matcher.rs`) re-derives its program from a `WatAST` on **every fact**:
//! it re-classifies clause shapes that never change, linear-scans field names for an index
//! fixed at compile time, and performs two heap allocations rebuilding the constant binding
//! key `"?l"` on every call — including every call that is about to FAIL, which is most of
//! them. This module compiles each alpha condition **once**, at the same setup site the alpha
//! discrimination tree (`alpha_tree.rs`) is built, into a pre-resolved instruction sequence
//! ([`Op`]) that an executor ([`exec_compiled`]) runs with array indexing and no per-call
//! allocation on the failure path. The point is correctness (a static program should not be
//! re-derived dynamically) and an allocation-pressure threat to this engine's jitter-free-tail
//! claim — NOT a timing win; post-tree, `alpha:match` is 1.1% of a fact-heavy fire.
//!
//! ## ★ THE ONE CONTRACT DECISION
//!
//! **Slots internally; populate materializes into the fire-scoped bind pool
//! ON SUCCESS ONLY** (`DESIGN-STONE-bind-pool`). Rematch still returns an
//! `Arc` and becomes a `PMap`. The executor threads a `Vec<Option<Value>>`
//! scratch buffer (reused call to call — see [`exec_compiled`]'s `scratch`
//! parameter) indexed by slot. A failing populate never writes the pool.
//!
//! ## Consumes `classify_rete_clause`, adds no second parser
//!
//! [`compile_condition`] walks the exact same [`crate::rete::matcher::ReteClauseShape`] shapes
//! `eval_clause` does (arc 294 item 9a's single grammar) — it does not re-derive "what shape is
//! this form" from the raw `WatAST` a second time. What it does differently is WHEN it resolves
//! each shape's parts: field names to `usize` indices, `?var` references to slot indices, and
//! literals to `Value`s, all once, at compile time, instead of on every fact.
//!
//! ## Scope: Bind/Constraint (+ nested `and`) is the measured path; `or`/`not` are correctness-
//! only
//!
//! Every alpha condition in the live corpus (`wat-scripts/perf/grid/*.wat`, the arc's grid axes)
//! is a straight-line sequence of `Bind` and `Constraint` clauses, optionally nested in
//! `:wat::rete::and` (which is flattened at compile time — sequential-AND has identical
//! semantics whether written as one flat list or nested `and`s, so there is no runtime cost to
//! flattening it away). `:wat::rete::or`/`:wat::rete::not` do not appear at the CLAUSE level
//! anywhere in the corpus (`grep` confirms every hit is a top-level `:when`-entry wrapper,
//! consumed by `compile-condition` in `wat/rete.wat` into a NegationNode long before
//! `alpha_match_inner`/this compiler ever sees a clause list) — but [`compile_condition`] still
//! compiles them correctly for STOP-1's sake: a branch's slot writes never survive the branch
//! (mirroring `eval_clause`'s `Or`/`Not` arms, which always return the ENTRY bindings unchanged,
//! discarding whatever a sub-clause bound), implemented via a scratch-slot clone-and-discard.
//! That clone is the one path that still allocates; it is never exercised by anything in the
//! live corpus, so it costs nothing measured, and STOP-1's correctness proof holds regardless
//! of whether the shape appears today.
//!
//! `:wat::rete::where`/`:wat::rete::exists`/an `Accumulate` wrapper/anything
//! `classify_rete_clause` cannot recognize compile to [`Op::Fail`] — an unconditional non-match,
//! identical to `eval_clause`'s own handling of those shapes (they never legitimately reach a
//! condition's clause list; `where` is stone 6 territory).

use std::collections::HashMap;
use std::sync::Arc;

use rustc_hash::FxHashMap;

use crate::ast::WatAST;
use crate::rete::expr_ir::Expr;
use crate::rete::matcher::{
    classify_constraint_head, classify_rete_clause, compare_values, Bindings, CmpKind,
    ReteClauseShape,
};
use crate::runtime::Value;

pub(crate) type SlotFrame = Vec<Option<Value>>;
pub(crate) type BindPairs = Option<Arc<[(Value, Value)]>>;

/// Fire-scoped intern: keys / fillers / val-ids / pair pool travel as one
/// place (`DESIGN-STONE-bind-pool`). Callers name the intern, not four
/// positional `&mut`s.
pub(crate) struct BindIntern<'a> {
    pub keys: &'a mut Vec<Value>,
    pub vals: &'a mut Vec<Value>,
    pub ids: &'a mut ValIntern,
    pub pool: &'a mut Vec<(u32, u32)>,
}

// ─── The compiled program ──────────────────────────────────────────────────────

// The comparison operator is `matcher::CmpKind` — the SAME enum the grammar and the interpreter
// read, decided once here at compile time instead of re-matched on every call. This file used to
// declare its own identical `CmpKind`; two enums meant two places a new comparison had to be added,
// which is the duplication the ONE DOOR exists to delete.

/// One instruction. Execution is a straight walk over a `[Op]` slice with short-circuit AND
/// (the first `Op` that fails to hold ends the whole match), mirroring `eval_clauses`'s
/// left-to-right fold exactly — `Or`/`Not` are the only ops that recurse into a sub-sequence.
///
/// Flip 3 (CURRENT-STATE): `Cmp` operands are the one `Expr` core. Bind / BindCheck / Or /
/// Not / Fail stay driver-level. A `:field` operand is prologue — a (possibly hidden) Bind
/// into a slot — so `Expr` never grows a cond-only FactField arm. Lists stay uncompiled
/// (`None` → `Fail`), matching `resolve_operand` (a list operand is not a field/var/lit).
#[derive(Clone, Debug)]
pub(crate) enum Op {
    /// `(?v <- :field)`, first occurrence of `?v` in its scope: write the field's value into
    /// `slot` unconditionally (always holds unless the field itself is out of range, which
    /// cannot happen for a condition compiled against its own class's field list).
    Bind { field_idx: usize, slot: usize },
    /// `(?v <- :field)`, `?v` already bound in this scope: the field's value must equal the
    /// slot's existing value (the runtime conflict check `eval_clause`'s `Bind` arm performs
    /// when `existing` is `Some`).
    BindCheck { field_idx: usize, slot: usize },
    /// `(:wat::rete::core::<ty>::<op> a b)` — both operands are `Expr` (Slot / Lit after
    /// prologue field binds). The comparison itself stays `CmpKind` so it cannot drift
    /// from `eval_clause` / `compare_values`.
    Cmp { op: CmpKind, lhs: Expr, rhs: Expr },
    /// Same comparison as [`Op::Cmp`], but one operand is a leftover `?var` not bound
    /// by this condition (a join / exists / not / accumulate-`:from` seed). Populate
    /// ([`exec_compiled`]) skips it so the fact still enters alpha; rematch
    /// ([`exec_compiled_under`]) fills the seed slot and runs the compare.
    SeedCmp { op: CmpKind, lhs: Expr, rhs: Expr },
    /// `(:wat::rete::or c1 c2 …)` — each branch is its OWN op sequence, tried against a scratch
    /// clone of the current slots (never the live slots): mirrors `eval_clause`'s `Or`, which
    /// always returns the pre-`or` bindings unchanged even on a successful branch.
    Or(OrBranches),
    /// `(:wat::rete::not inner)` — `inner`'s op sequence, run against a scratch clone; holds iff
    /// `inner` does NOT. Mirrors `eval_clause`'s `Not`, which never lets a negated branch's
    /// binds escape.
    Not(Vec<Op>),
    /// Any clause shape `eval_clause` maps unconditionally to `None` — `where`/`exists`/
    /// `accumulate`/`Unrecognized`, or an operand/field that provably can never resolve for
    /// this class (an unbound `?var`, an unknown field name). Compiling these to an
    /// always-fail op (rather than skipping them) keeps this an honest specialization of
    /// `eval_clause`, not a silent narrowing of what it accepts.
    Fail,
}

pub(crate) type OrBranches = Vec<Vec<Op>>;

/// A condition compiled once, at setup, from the immutable network — the pre-resolved dual of
/// `alpha_match_inner`. Built by [`compile_condition`]; run by [`exec_compiled`].
#[derive(Clone)]
pub(crate) struct CompiledCond {
    /// The top-level clause sequence (nested `and` flattened in), in source order.
    ops: Vec<Op>,
    /// The binding keys, in FIRST-BIND order — built once, `Value::String(Arc<str>)`, cloned
    /// (a refcount bump, never a fresh allocation) into the output array on success. Parallel to
    /// `output_slots`.
    slot_keys: Arc<[Value]>,
    /// `output_slots[i]` is the scratch-slot index whose value pairs with `slot_keys[i]` — the
    /// two arrays together are the zip the design doc describes. Only slots reachable through
    /// the top-level/`and`-flattened path appear here; a slot a `Or`/`Not` branch privately
    /// bound never does, matching `eval_clause`'s discard of branch-local binds.
    output_slots: Arc<[usize]>,
    /// Total scratch slots this program needs (>= `output_slots.len()`; larger when an
    /// `or`/`not` branch binds its own scratch-only vars, or a leftover `?var` occupies a
    /// seed slot). The caller's reusable scratch buffer must be at least this long.
    n_slots: usize,
    /// Leftover `?var` keys this rematch reads from the token seed, in first-seen order.
    /// Not in `output_slots` — a leftover is the left token's bind, not this cond's.
    seed_reads: Arc<[(Value, usize)]>,
    /// `(?p <- :Type …)` — the fact itself, not a field. Set at compile from
    /// `alpha_pattern`; fire attaches without walking the cond AST.
    fact_bind: Option<Value>,
}

impl CompiledCond {
    /// The scratch-buffer length `exec_compiled` needs for this program.
    pub(crate) fn n_slots(&self) -> usize {
        self.n_slots
    }

    pub(crate) fn ops(&self) -> &[Op] {
        &self.ops
    }
    pub(crate) fn slot_keys(&self) -> &[Value] {
        &self.slot_keys
    }
    pub(crate) fn output_slots(&self) -> &[usize] {
        &self.output_slots
    }
    pub(crate) fn seed_reads(&self) -> &[(Value, usize)] {
        &self.seed_reads
    }
    pub(crate) fn fact_bind(&self) -> Option<&Value> {
        self.fact_bind.as_ref()
    }
    pub(crate) fn from_parts(
        ops: Vec<Op>,
        slot_keys: Arc<[Value]>,
        output_slots: Arc<[usize]>,
        n_slots: usize,
        seed_reads: Arc<[(Value, usize)]>,
        fact_bind: Option<Value>,
    ) -> Self {
        CompiledCond {
            ops,
            slot_keys,
            output_slots,
            n_slots,
            seed_reads,
            fact_bind,
        }
    }

    /// Leftover seed: a `?var` this cond does not bind. Populate skipped
    /// `SeedCmp`; rematch must still run. Absence is a proof the Element
    /// already holds every bind the fold will read (`DESIGN-STONE-accum-fold-the-wall`).
    pub(crate) fn has_seed_cmp(&self) -> bool {
        !self.seed_reads.is_empty() || ops_have_seed_cmp(&self.ops)
    }

    /// `?var`s this cond binds, including `(?p <- :Type …)`.
    pub(crate) fn bind_keys(&self) -> Vec<Value> {
        let mut ks = Vec::with_capacity(self.slot_keys.len() + 1);
        if let Some(k) = &self.fact_bind {
            ks.push(k.clone());
        }
        ks.extend(self.slot_keys.iter().cloned());
        ks
    }
}

fn ops_have_seed_cmp(ops: &[Op]) -> bool {
    ops.iter().any(|op| match op {
        Op::SeedCmp { .. } => true,
        Op::Or(bs) => bs.iter().any(|b| ops_have_seed_cmp(b)),
        Op::Not(inner) => ops_have_seed_cmp(inner),
        _ => false,
    })
}

// ─── The compiler ───────────────────────────────────────────────────────────────

struct Ctx<'a> {
    field_names: &'a [String],
    next_slot: usize,
    defer_unbound: bool,
    seed_reads: Vec<(Value, usize)>,
}

/// Compile one alpha condition (`(:ClassName clause…)`, exactly what `alpha_cond` stores) into a
/// [`CompiledCond`] against `field_names` (that class's declared field order — the same list
/// `alpha_match_inner`'s caller resolves via `class_field_names`).
///
/// Returns `None` only if `cond` is not the `(:Keyword clause…)` shape `build_alpha_index`
/// already guarantees for every entry it puts in `alpha_cond` — i.e. never, for any condition
/// this is actually called with in `kernel.rs`. Kept as `Option` (rather than assuming the
/// invariant) so a caller can fall back to `alpha_match_inner` instead of panicking if that
/// invariant is ever violated.
///
/// Strict: an unbound constraint `?var` is [`Op::Fail`]. Fire setup uses
/// [`compile_condition_local`] (leftover-as-seed). This entry is the populate
/// differential against `alpha_match_inner`.
#[cfg(test)]
pub(crate) fn compile_condition(cond: &WatAST, field_names: &[String]) -> Option<CompiledCond> {
    compile_condition_opts(cond, field_names, false)
}

/// Same as [`compile_condition`], but an unbound constraint `?var` becomes a
/// seed slot + [`Op::SeedCmp`] (not omitted, not [`Op::Fail`]). Populate skips
/// `SeedCmp`; rematch fills the slot from the token and runs the compare.
/// One compile for every alpha at fire setup.
pub(crate) fn compile_condition_local(
    cond: &WatAST,
    field_names: &[String],
) -> Option<CompiledCond> {
    compile_condition_opts(cond, field_names, true)
}

fn compile_condition_opts(
    cond: &WatAST,
    field_names: &[String],
    defer_unbound: bool,
) -> Option<CompiledCond> {
    let pat = crate::rete::matcher::alpha_pattern(cond)?;
    let clauses = pat.clauses;

    let mut ctx = Ctx {
        field_names,
        next_slot: 0,
        defer_unbound,
        seed_reads: Vec::new(),
    };
    let mut scope: HashMap<String, usize> = HashMap::new();
    let mut field_slots: HashMap<usize, usize> = HashMap::new();
    let mut order: Vec<(String, usize)> = Vec::new();
    let mut ops: Vec<Op> = Vec::new();
    compile_seq(
        clauses,
        &mut ctx,
        &mut scope,
        &mut field_slots,
        &mut order,
        &mut ops,
    );

    let slot_keys: Arc<[Value]> = order
        .iter()
        .map(|(name, _)| Value::String(Arc::new(name.clone())))
        .collect::<Vec<_>>()
        .into();
    let output_slots: Arc<[usize]> = order
        .iter()
        .map(|(_, slot)| *slot)
        .collect::<Vec<_>>()
        .into();
    let seed_reads: Arc<[(Value, usize)]> = ctx.seed_reads.into();
    let fact_bind = pat
        .fact_var
        .map(|v| Value::String(Arc::new(v.to_string())));

    Some(CompiledCond {
        ops,
        slot_keys,
        output_slots,
        n_slots: ctx.next_slot,
        seed_reads,
        fact_bind,
    })
}

/// Put the matched fact on `?p` when this cond is `(?p <- :Type …)`.
pub(crate) fn attach_fact(
    compiled: &CompiledCond,
    fact: &Value,
    bindings: Arc<[(Value, Value)]>,
) -> Arc<[(Value, Value)]> {
    match &compiled.fact_bind {
        Some(key) => {
            let mut out: Vec<(Value, Value)> = Vec::with_capacity(bindings.len() + 1);
            out.push((key.clone(), fact.clone()));
            out.extend(bindings.iter().map(|(k, v)| (k.clone(), v.clone())));
            out.into()
        }
        None => bindings,
    }
}

/// Compile a clause LIST in the caller's own scope — the top-level condition body, or an `and`'s
/// sub-list (flattened directly into the caller's `ops`/`scope`/`order`, since `and` shares the
/// enclosing sequential scope exactly).
fn compile_seq(
    clauses: &[WatAST],
    ctx: &mut Ctx,
    scope: &mut HashMap<String, usize>,
    field_slots: &mut HashMap<usize, usize>,
    order: &mut Vec<(String, usize)>,
    ops: &mut Vec<Op>,
) {
    for clause in clauses {
        compile_one(clause, ctx, scope, field_slots, order, ops);
    }
}

/// Compile one clause, appending to `ops`. `scope`/`order` are the CALLER's — for `and` (and the
/// top-level list) that is the real, surviving scope; for `or`/`not` branches the caller passes
/// a throwaway clone/scratch (see below), matching `eval_clause`'s discard of branch-local binds.
fn compile_one(
    clause: &WatAST,
    ctx: &mut Ctx,
    scope: &mut HashMap<String, usize>,
    field_slots: &mut HashMap<usize, usize>,
    order: &mut Vec<(String, usize)>,
    ops: &mut Vec<Op>,
) {
    match classify_rete_clause(clause) {
        ReteClauseShape::Bind { var, field } => {
            match ctx.field_names.iter().position(|n| n == field) {
                // Field not declared on this class: read_fact_field would return None on every
                // fact of this class — a compile-time-provable, permanent failure.
                None => ops.push(Op::Fail),
                Some(field_idx) => {
                    if let Some(&slot) = scope.get(var) {
                        field_slots.entry(field_idx).or_insert(slot);
                        ops.push(Op::BindCheck { field_idx, slot });
                    } else {
                        let slot = ctx.next_slot;
                        ctx.next_slot += 1;
                        scope.insert(var.to_string(), slot);
                        order.push((var.to_string(), slot));
                        field_slots.entry(field_idx).or_insert(slot);
                        ops.push(Op::Bind { field_idx, slot });
                    }
                }
            }
        }
        ReteClauseShape::Constraint { op, lhs, rhs } => {
            // The ONE DOOR (`matcher::classify_constraint_head`) — the constraint vocabulary is
            // written down once, in `matcher.rs`, and read here rather than re-listed. A `None`
            // means this file and the grammar disagree, which is our bug, not the caller's.
            let (cmp, _spelling) = classify_constraint_head(op).unwrap_or_else(|| {
                unreachable!(
                    "classify_rete_clause admitted a Constraint head the ONE DOOR rejects: {op}"
                )
            });
            // An operand that cannot be resolved AT ALL (an unbound `?var` — statically proven
            // unbound by this point in the scope, since nothing earlier in this exact walk
            // recorded it; or a `:field` this class does not declare) makes `resolve_operand`
            // return None on every call, which makes the whole Constraint (and hence the whole
            // match) fail. Compile that permanently, at build time, instead of re-discovering it
            // on every fact. A `:field` becomes a prologue Bind + `Expr::Slot` so Cmp sits
            // on the one core.
            let lhs_e = compile_operand_expr(lhs, scope, field_slots, ctx, ops);
            let rhs_e = compile_operand_expr(rhs, scope, field_slots, ctx, ops);
            match (lhs_e, rhs_e) {
                (Some(lhs), Some(rhs)) => {
                    if expr_reads_seed(&lhs, &ctx.seed_reads)
                        || expr_reads_seed(&rhs, &ctx.seed_reads)
                    {
                        ops.push(Op::SeedCmp { op: cmp, lhs, rhs });
                    } else {
                        ops.push(Op::Cmp { op: cmp, lhs, rhs });
                    }
                }
                _ => ops.push(Op::Fail),
            }
        }
        ReteClauseShape::And(subs) => compile_seq(subs, ctx, scope, field_slots, order, ops),
        ReteClauseShape::Or(subs) => {
            // Each branch compiles against its OWN clone of the current scope (so a bind made by
            // one branch is never visible to a sibling), and its binds are recorded into a
            // throwaway `order` (discarded — an Or's successful branch's bindings never survive
            // the Or, exactly like `eval_clause`'s `Or` arm returning the pre-`or` `entry`).
            // `field_slots` is cloned the same way: a hidden `:field` Bind in one arm must
            // not satisfy a sibling that never wrote that slot.
            let branches: Vec<Vec<Op>> = subs
                .iter()
                .map(|sub| {
                    let mut sub_scope = scope.clone();
                    let mut sub_fields = field_slots.clone();
                    let mut scratch_order = Vec::new();
                    let mut sub_ops = Vec::new();
                    compile_one(
                        sub,
                        ctx,
                        &mut sub_scope,
                        &mut sub_fields,
                        &mut scratch_order,
                        &mut sub_ops,
                    );
                    sub_ops
                })
                .collect();
            ops.push(Op::Or(branches));
        }
        ReteClauseShape::Not(sub) => {
            let mut sub_scope = scope.clone();
            let mut sub_fields = field_slots.clone();
            let mut scratch_order = Vec::new();
            let mut sub_ops = Vec::new();
            compile_one(
                sub,
                ctx,
                &mut sub_scope,
                &mut sub_fields,
                &mut scratch_order,
                &mut sub_ops,
            );
            ops.push(Op::Not(sub_ops));
        }
        ReteClauseShape::Where(_)
        | ReteClauseShape::Exists(_)
        | ReteClauseShape::Accumulate { .. }
        | ReteClauseShape::FactBind { .. }
        | ReteClauseShape::Unrecognized => ops.push(Op::Fail),
    }
}

/// Resolve one operand AST node to an [`Expr`] at compile time — the same three shapes
/// `resolve_operand` distinguishes at runtime (`?var` from the scope-so-far, `:field` from the
/// class's declared fields, or a bare literal). `:field` is prologue (a Bind into a slot,
/// reused if this condition already bound that field), so the Cmp itself only sees `Expr`.
/// Returns `None` exactly when `resolve_operand` would unconditionally return `None` for every
/// fact of this class: an unbound `?var` (strict compile), an unknown field, or a nested list
/// (lists are `where`-territory on both sides; do not compile them here alone).
/// Leftover-as-seed (`defer_unbound`): an unbound `?var` allocates a seed slot and
/// returns `Expr::Slot` so the Constraint arm can emit [`Op::SeedCmp`].
fn expr_reads_seed(e: &Expr, seed_reads: &[(Value, usize)]) -> bool {
    match e {
        Expr::Slot(i) => seed_reads.iter().any(|(_, slot)| *slot == *i as usize),
        _ => false,
    }
}

fn expr_slot(slot: usize) -> Option<Expr> {
    u16::try_from(slot).ok().map(Expr::Slot)
}

fn compile_operand_expr(
    operand: &WatAST,
    scope: &mut HashMap<String, usize>,
    field_slots: &mut HashMap<usize, usize>,
    ctx: &mut Ctx<'_>,
    ops: &mut Vec<Op>,
) -> Option<Expr> {
    match operand {
        WatAST::Symbol(ident, _) => {
            let name = ident.as_str();
            if name.starts_with('?') {
                if let Some(slot) = scope.get(name).copied() {
                    expr_slot(slot)
                } else if ctx.defer_unbound {
                    let slot = ctx.next_slot;
                    ctx.next_slot += 1;
                    scope.insert(name.to_string(), slot);
                    ctx.seed_reads
                        .push((Value::String(Arc::new(name.to_string())), slot));
                    expr_slot(slot)
                } else {
                    None
                }
            } else {
                None
            }
        }
        WatAST::Keyword(k, _) => {
            let field_name = k.strip_prefix(':').unwrap_or(k.as_str());
            let field_idx = ctx.field_names.iter().position(|n| n == field_name)?;
            if let Some(&slot) = field_slots.get(&field_idx) {
                return expr_slot(slot);
            }
            let slot = ctx.next_slot;
            ctx.next_slot += 1;
            field_slots.insert(field_idx, slot);
            ops.push(Op::Bind { field_idx, slot });
            expr_slot(slot)
        }
        WatAST::IntLit(n, _) => Some(Expr::Lit(Value::i64(*n))),
        WatAST::FloatLit(x, _) => Some(Expr::Lit(Value::f64(*x))),
        WatAST::BoolLit(b, _) => Some(Expr::Lit(Value::bool(*b))),
        WatAST::StringLit(s, _) => Some(Expr::Lit(Value::String(Arc::new(s.clone())))),
        _ => None,
    }
}

// ─── The executor ───────────────────────────────────────────────────────────────

/// Run a compiled condition against one fact's fields. `scratch` is a caller-owned, caller-
/// reused `Vec<Option<Value>>` — `kernel.rs` allocates it once (sized to the largest
/// `CompiledCond::n_slots` across every compiled alpha) before the round loop starts, so this
/// function's own body never allocates on the Bind/Constraint (+ flattened `and`) path: `clear`
/// followed by `resize` back up to `compiled.n_slots` never reallocates once `scratch`'s
/// capacity has reached its high-water mark, and every write after that is a plain slice index.
/// The one exception is `Op::Or`/`Op::Not`, which clone `scratch` into a fresh temporary — not
/// exercised by anything in the live grid corpus (see the module doc), so it never fires on the
/// path this stone's zero-allocation gate measures.
///
/// Populate: write pairs into `pool`, `?p` first when this cond is
/// `(?p <- :Type …)`. Returns the span. Same keys/values/order as
/// `alpha_match_inner` + `attach_fact` (STOP-1).
#[cfg(test)]
// rune:excusare(arity-is-the-pool) — test wrapper builds BindIntern; production
// `exec_compiled_with_key_ids` takes the intern.
#[allow(clippy::too_many_arguments)]
pub(crate) fn exec_compiled(
    compiled: &CompiledCond,
    fact_fields: &[Value],
    scratch: &mut SlotFrame,
    pool: &mut Vec<(u32, u32)>,
    keys: &mut Vec<Value>,
    vals: &mut Vec<Value>,
    val_ids: &mut ValIntern,
    fact: &Value,
) -> Option<(u32, u16)> {
    let mut intern = BindIntern {
        keys,
        vals,
        ids: val_ids,
        pool,
    };
    exec_compiled_with_key_ids(
        compiled,
        fact_fields,
        scratch,
        &mut intern,
        fact,
        None,
    )
}

pub(crate) fn exec_compiled_with_key_ids(
    compiled: &CompiledCond,
    fact_fields: &[Value],
    scratch: &mut SlotFrame,
    intern: &mut BindIntern<'_>,
    fact: &Value,
    key_ids: Option<&[u32]>,
) -> Option<(u32, u16)> {
    // Arc 278 DESIGN-STONE-compiled-conditions.md — the compiled path's call counter, parallel to
    // `alpha_match_inner`'s `match:calls`. Since this stone re-points the round loop's step 1 at
    // this function (`kernel.rs`), `match:calls` alone would read zero on a real fire from here
    // on; this is what a diagnostic census reads instead to see the production path is live.
    crate::rete::kernel::census_count("compiled:calls");
    scratch.clear();
    scratch.resize(compiled.n_slots, None);

    if !exec_ops(&compiled.ops, scratch, fact_fields, true) {
        return None;
    }
    materialize_into(compiled, scratch, intern, fact, key_ids)
}

pub(crate) fn intern_key(keys: &mut Vec<Value>, k: &Value) -> u32 {
    if let Some(i) = keys.iter().position(|x| x == k) {
        return i as u32;
    }
    let i = keys.len() as u32;
    keys.push(k.clone());
    i
}

/// Intern this cond's `fact_bind?` then `slot_keys` once
/// (`DESIGN-STONE-cond-key-ids`). Fire SETUP, not per fact.
pub(crate) fn intern_cond_keys(compiled: &CompiledCond, keys: &mut Vec<Value>) -> Vec<u32> {
    let extra = usize::from(compiled.fact_bind.is_some());
    let mut ids = Vec::with_capacity(extra + compiled.slot_keys.len());
    if let Some(k) = &compiled.fact_bind {
        ids.push(intern_key(keys, k));
    }
    for k in compiled.slot_keys.iter() {
        ids.push(intern_key(keys, k));
    }
    ids
}

/// Fire-scoped filler intern (`DESIGN-STONE-intern-val-i64`).
/// Nonnegative i64 below `I64_SMALL` skip hashing `Value`.
const I64_SMALL: usize = 4096;

pub(crate) struct ValIntern {
    any: FxHashMap<Value, u32>,
    small: Vec<u32>,
}

impl Default for ValIntern {
    fn default() -> Self {
        ValIntern {
            any: FxHashMap::default(),
            small: vec![u32::MAX; I64_SMALL],
        }
    }
}

impl ValIntern {
    pub(crate) fn clear(&mut self) {
        self.any.clear();
        self.small.fill(u32::MAX);
    }

    /// Read-only intern lookup (`DESIGN-STONE-gather-val-id`).
    pub(crate) fn get(&self, v: &Value) -> Option<u32> {
        if let Value::i64(n) = v {
            if *n >= 0 {
                let i = *n as usize;
                if i < self.small.len() {
                    let slot = self.small[i];
                    if slot != u32::MAX {
                        return Some(slot);
                    }
                }
            }
        }
        self.any.get(v).copied()
    }
}

pub(crate) fn intern_val(vals: &mut Vec<Value>, ids: &mut ValIntern, v: Value) -> u32 {
    if let Value::i64(n) = v {
        if n >= 0 {
            let i = n as usize;
            if i < ids.small.len() {
                let slot = ids.small[i];
                if slot != u32::MAX {
                    return slot;
                }
                let id = vals.len() as u32;
                ids.small[i] = id;
                vals.push(Value::i64(n));
                return id;
            }
        }
    }
    if let Some(&id) = ids.any.get(&v) {
        return id;
    }
    let id = vals.len() as u32;
    ids.any.insert(v.clone(), id);
    vals.push(v);
    id
}

pub(crate) fn materialize_into(
    compiled: &CompiledCond,
    scratch: &[Option<Value>],
    intern: &mut BindIntern<'_>,
    fact: &Value,
    key_ids: Option<&[u32]>,
) -> Option<(u32, u16)> {
    let off = intern.pool.len();
    let mut kid = 0usize;
    let next_key = |keys: &mut Vec<Value>, kid: &mut usize, fallback: &Value| -> u32 {
        if let Some(ids) = key_ids {
            let id = ids[*kid];
            *kid += 1;
            id
        } else {
            intern_key(keys, fallback)
        }
    };
    let BindIntern {
        keys,
        vals,
        ids: val_ids,
        pool,
    } = intern;
    if let Some(key) = &compiled.fact_bind {
        pool.push((
            next_key(keys, &mut kid, key),
            intern_val(vals, val_ids, fact.clone()),
        ));
    }
    for (i, &slot) in compiled.output_slots.iter().enumerate() {
        let v = match scratch.get(slot).and_then(|o| o.clone()) {
            Some(v) => v,
            None => {
                debug_assert!(
                    false,
                    "compiled program guarantee violated: output slot {slot} unbound on success"
                );
                pool.truncate(off);
                return None;
            }
        };
        pool.push((
            next_key(keys, &mut kid, &compiled.slot_keys[i]),
            intern_val(vals, val_ids, v),
        ));
    }
    Some((off as u32, (pool.len() - off) as u16))
}

/// Rematch a compiled condition under a token seed. Writes `seed_reads` into
/// scratch, then runs every op including [`Op::SeedCmp`]. An unbound seed
/// slot fails the compare (same as `resolve_operand` returning `None`).
/// Does not increment `match:calls`.
pub(crate) fn exec_compiled_under(
    compiled: &CompiledCond,
    fact_fields: &[Value],
    scratch: &mut SlotFrame,
    seed: &(impl Bindings + ?Sized),
) -> BindPairs {
    crate::rete::kernel::census_count("rematch:compiled");
    scratch.clear();
    scratch.resize(compiled.n_slots, None);
    for (key, slot) in compiled.seed_reads.iter() {
        scratch[*slot] = seed.get(key).cloned();
    }
    if !exec_ops(&compiled.ops, scratch, fact_fields, false) {
        return None;
    }
    materialize(compiled, scratch)
}

fn materialize(
    compiled: &CompiledCond,
    scratch: &[Option<Value>],
) -> BindPairs {
    let mut out: Vec<(Value, Value)> = Vec::with_capacity(compiled.output_slots.len());
    for (i, &slot) in compiled.output_slots.iter().enumerate() {
        let v = match scratch.get(slot).and_then(|o| o.clone()) {
            Some(v) => v,
            None => {
                // Compile-time guarantee: an output slot is written by the Bind op that
                // introduced it, which must have run (and held) for `exec_ops` above to have
                // returned `true` at all. Should be unreachable; fail closed rather than hand
                // back a malformed array if it ever isn't.
                debug_assert!(
                    false,
                    "compiled program guarantee violated: output slot {slot} unbound on success"
                );
                return None;
            }
        };
        out.push((compiled.slot_keys[i].clone(), v));
    }
    Some(out.into())
}

pub(crate) fn exec_ops(
    ops: &[Op],
    slots: &mut [Option<Value>],
    fact_fields: &[Value],
    skip_seed: bool,
) -> bool {
    for op in ops {
        if skip_seed && matches!(op, Op::SeedCmp { .. }) {
            continue;
        }
        if !exec_op(op, slots, fact_fields, skip_seed) {
            return false;
        }
    }
    true
}

fn exec_op(op: &Op, slots: &mut [Option<Value>], fact_fields: &[Value], skip_seed: bool) -> bool {
    match op {
        Op::Fail => false,
        Op::Bind { field_idx, slot } => match fact_fields.get(*field_idx) {
            Some(v) => {
                slots[*slot] = Some(v.clone());
                true
            }
            None => false,
        },
        Op::BindCheck { field_idx, slot } => match fact_fields.get(*field_idx) {
            Some(v) => match &slots[*slot] {
                Some(existing) => existing == v,
                // Should not occur (a BindCheck's slot was, by construction, already written by
                // the Bind that first introduced it) — fall back to a fresh bind rather than fail
                // outright, matching eval_clause's own "None => fresh insert" arm.
                None => {
                    slots[*slot] = Some(v.clone());
                    true
                }
            },
            None => false,
        },
        Op::Cmp { op, lhs, rhs } | Op::SeedCmp { op, lhs, rhs } => {
            match (eval_cmp_operand(lhs, slots), eval_cmp_operand(rhs, slots)) {
                (Some(a), Some(b)) => eval_cmp(*op, &a, &b),
                _ => false,
            }
        }
        Op::Or(branches) => {
            for branch in branches {
                let mut clone: SlotFrame = slots.to_vec();
                if exec_ops(branch, &mut clone, fact_fields, skip_seed) {
                    return true;
                }
            }
            false
        }
        Op::Not(sub) => {
            let mut clone: SlotFrame = slots.to_vec();
            !exec_ops(sub, &mut clone, fact_fields, skip_seed)
        }
    }
}

fn eval_cmp_operand(operand: &Expr, slots: &[Option<Value>]) -> Option<Value> {
    match operand {
        Expr::Lit(v) => Some(v.clone()),
        Expr::Slot(i) => slots.get(*i as usize).and_then(|o| o.clone()),
        // Flip 3 emits only Slot / Lit. Any other arm is a compiler bug — fail
        // closed, same as an unresolved operand.
        _ => None,
    }
}

/// Mirrors `eval_clause`'s `Constraint` arm exactly, including the propagation `compare_values`
/// returning `None` (incompatible types) makes the WHOLE clause fail rather than yielding some
/// boolean — `compare_values` is REUSED from `matcher.rs`, not reimplemented, so an ordering
/// definition can never drift between the interpreter and the compiled executor.
fn eval_cmp(op: CmpKind, a: &Value, b: &Value) -> bool {
    match op {
        CmpKind::Eq => a == b,
        CmpKind::NotEq => a != b,
        CmpKind::Lt => matches!(compare_values(a, b), Some(std::cmp::Ordering::Less)),
        CmpKind::Gt => matches!(compare_values(a, b), Some(std::cmp::Ordering::Greater)),
        CmpKind::Le => matches!(compare_values(a, b), Some(o) if o != std::cmp::Ordering::Greater),
        CmpKind::Ge => matches!(compare_values(a, b), Some(o) if o != std::cmp::Ordering::Less),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rete::matcher::alpha_match_inner_seeded;
    use crate::value::pmap::PMap;

    fn qvar(name: &str) -> Value {
        Value::String(Arc::new(name.to_string()))
    }

    #[test]
    fn leftover_seed_cmp_populate_skips_rematch_enforces() {
        let ast = crate::parse_one!("(:wjl::Wind (?w <- :kph) (:wat::rete::core::i64::> ?w ?c))")
            .expect("parse leftover cond");
        let fields = vec!["kph".to_string()];
        let compiled = compile_condition_local(&ast, &fields).expect("compile leftover-as-seed");
        assert!(
            compiled.seed_reads.iter().any(|(k, _)| *k == qvar("?c")),
            "leftover ?c must be a seed slot, not omitted and not an output bind"
        );
        assert!(
            !compiled.slot_keys.iter().any(|k| *k == qvar("?c")),
            "leftover ?c must not leak into this cond's binds"
        );

        let fact = [Value::i64(20)];
        let mut scratch = Vec::new();
        assert!(
            exec_compiled(
                &compiled,
                &fact,
                &mut scratch,
                &mut Vec::new(),
                &mut Vec::new(),
                &mut Vec::new(),
                &mut ValIntern::default(),
                &Value::i64(20),
            )
                .is_some(),
            "populate skips SeedCmp so the fact enters alpha"
        );

        let seed_ok = PMap::from_pairs([(qvar("?c"), Value::i64(10))]);
        let rematch_ok = exec_compiled_under(&compiled, &fact, &mut scratch, &seed_ok);
        assert!(rematch_ok.is_some(), "20 > 10 holds under seed");

        let seed_fail = PMap::from_pairs([(qvar("?c"), Value::i64(30))]);
        assert!(
            exec_compiled_under(&compiled, &fact, &mut scratch, &seed_fail).is_none(),
            "20 > 30 fails under seed"
        );
        assert!(
            exec_compiled_under(&compiled, &fact, &mut scratch, &PMap::new()).is_none(),
            "unbound leftover seed is no match"
        );

        let seed_pairs = vec![(qvar("?c"), Value::i64(10))];
        let interp = alpha_match_inner_seeded(&ast, "wjl::Wind", &fact, &fields, &seed_pairs);
        assert_eq!(
            interp.is_some(),
            rematch_ok.is_some(),
            "compiled rematch verdict must match the interpreter oracle"
        );
    }

    #[test]
    fn leftover_strict_compile_is_still_fail() {
        let ast = crate::parse_one!("(:wjl::Wind (?w <- :kph) (:wat::rete::core::i64::> ?w ?c))")
            .expect("parse leftover cond");
        let fields = vec!["kph".to_string()];
        let compiled = compile_condition(&ast, &fields).expect("strict compile");
        assert!(
            compiled.seed_reads.is_empty(),
            "strict compile must not seed leftover ?vars"
        );
        let mut scratch = Vec::new();
        assert!(
            exec_compiled(
                &compiled,
                &[Value::i64(20)],
                &mut scratch,
                &mut Vec::new(),
                &mut Vec::new(),
                &mut Vec::new(),
                &mut ValIntern::default(),
                &Value::i64(20),
            )
            .is_none(),
            "strict populate still Fails an unbound constraint ?var"
        );
    }

    #[test]
    fn local_compile_is_some_whenever_alpha_pattern_holds() {
        // Setup refuses a None from compile_condition_local for any alpha_cond
        // entry. build_alpha_index only inserts when alpha_pattern holds, so
        // these shapes must compile. A None here is the populate-interp hatch
        // coming back.
        let cases = [
            "(:wjl::Wind (?w <- :kph))",
            "(:wjl::Wind (?w <- :kph) (:wat::rete::core::i64::> ?w 30))",
            "(:wjl::Wind (?w <- :kph) (:wat::rete::core::i64::> ?w ?c))",
            "(?p <- :wjl::Wind (?w <- :kph))",
        ];
        for src in cases {
            let ast = crate::parse_one!(src).unwrap_or_else(|_| panic!("parse {src}"));
            assert!(
                crate::rete::matcher::alpha_pattern(&ast).is_some(),
                "alpha_pattern must hold for {src}"
            );
            assert!(
                compile_condition_local(&ast, &["kph".to_string()]).is_some(),
                "compile_condition_local must not return None for {src}"
            );
        }
    }
}
