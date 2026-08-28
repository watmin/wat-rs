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
//! ([`Op`]) that fire runs via [`exec_compiled_with_key_ids`] with array indexing and no
//! per-call allocation on the failure path. [`exec_compiled`] is the `#[cfg(test)]` door
//! (no interned keys). The point is correctness (a static program should not be
//! re-derived dynamically) and an allocation-pressure threat to this engine's jitter-free-tail
//! claim — NOT a timing win; post-tree, `alpha:match` is 1.1% of a fact-heavy fire.
//!
//! ## ★ THE ONE CONTRACT DECISION
//!
//! **Slots internally; populate materializes into the fire-scoped bind pool
//! ON SUCCESS ONLY** (`DESIGN-STONE-bind-pool`). Rematch still returns an
//! `Arc` and becomes a `PMap`. The executor threads a `Vec<Option<Value>>`
//! scratch buffer (reused call to call — see [`exec_compiled_with_key_ids`]'s
//! `scratch` parameter) indexed by slot. A failing populate never writes the pool.
//!
//! ## Consumes `classify_rete_clause`, adds no second parser
//!
//! [`compile_alpha_ops`] / [`compile_condition_local`] walk the exact same [`crate::rete::clause::ReteClauseShape`] shapes
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
//! consumed by `compile-condition` in `wat/rete/compile.wat` into a NegationNode long before
//! `alpha_match_inner`/this compiler ever sees a clause list) — but [`compile_alpha_ops`] still
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
use crate::span::Span;
use crate::rete::clause::{
    classify_constraint_head, classify_rete_clause, CmpKind, ConstraintSpelling, ReteClauseShape,
};
use crate::rete::expr_ir::Expr;
use crate::rete::matcher::{compare_values, Bindings};
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

// The comparison operator is `clause::CmpKind` — the SAME enum the grammar and the interpreter
// read, decided once here at compile time instead of re-matched on every call. This file used to
// declare its own identical `CmpKind`; two enums meant two places a new comparison had to be added,
// which is the duplication the ONE DOOR exists to delete.

/// One instruction. Execution is a straight walk over a `[Op]` slice with short-circuit AND
/// (the first `Op` that fails to hold ends the whole match), mirroring `eval_clauses`'s
/// left-to-right fold exactly — `Or`/`Not` are the only ops that recurse into a sub-sequence.
///
/// Flip 3 (CURRENT-STATE), same taxonomy as `Lands` next to `Op` in this file:
/// Driver = slot population (`Bind` only). BindCheck / Cmp / SeedCmp / Or / Not / Fail
/// are the expression core (they are still `Op` variants, not `Expr`). A `:field`
/// operand is prologue — a (possibly hidden) Bind into a slot — so `Expr` never grows
/// a cond-only FactField arm. Lists stay uncompiled (`None` → `Fail`), matching
/// `resolve_operand` (a list operand is not a field/var/lit).
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
    /// ([`exec_compiled_with_key_ids`]) skips it so the fact still enters alpha; rematch
    /// ([`exec_compiled_under`]) fills the seed slot and runs the compare.
    SeedCmp { op: CmpKind, lhs: Expr, rhs: Expr },
    /// **FIX-LIST F** — evaluate a COMPUTED operand into `slot`, before the `Cmp` that reads it.
    ///
    /// The same idea as [`Op::Bind`], one level up: `Bind` materialises a FIELD into a slot,
    /// `Eval` materialises an EXPRESSION into one. Doing it here rather than inside `Op::Cmp`
    /// keeps `Cmp`'s operands `Slot | Lit` exactly as before, so the per-fact fast path is
    /// untouched and `eval_cmp_operand` still returns a BORROW instead of cloning per comparison.
    ///
    /// `expr` is lowered by the one expression core (`expr_ir::lower_in_frame`) into THIS
    /// condition's slot numbering, so it reads the very scratch the prologue filled — no copy
    /// plan, no second frame.
    Eval { expr: Expr, slot: usize },
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
/// `alpha_match_inner`. Built by [`compile_condition_local`]; fire runs
/// [`exec_compiled_with_key_ids`]. [`exec_compiled`] is the `#[cfg(test)]` door.
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
    /// Frozen leftover bit — `join_extend` reads this, never re-walks `ops`.
    has_seed_cmp: bool,
    /// The condition's own wat span, for `Op::Eval`'s diagnostics. A REAL user span, not
    /// `rust_caller_span!()` — a computed operand that raises must point at the rule, which is
    /// exactly the `conformare` class this arc spent a day removing.
    span: Span,
    /// Slot index -> name, for the same diagnostics.
    slot_names: crate::rete::expr_ir::SlotNames,
}

impl CompiledCond {
    /// The scratch-buffer length fire's [`exec_compiled_with_key_ids`] (and the
    /// `#[cfg(test)]` [`exec_compiled`] door) needs for this program.
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
    // 8 args since fix-list F: `span` and `slot_names` joined so an `Op::Eval` raise can point at
    // the USER's rule instead of at `rust_caller_span!()` — the `conformare` class this arc spent a
    // day removing. A builder would be ceremony for a constructor with two call sites.
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn from_parts(
        ops: Vec<Op>,
        slot_keys: Arc<[Value]>,
        output_slots: Arc<[usize]>,
        n_slots: usize,
        seed_reads: Arc<[(Value, usize)]>,
        fact_bind: Option<Value>,
        span: Span,
        slot_names: crate::rete::expr_ir::SlotNames,
    ) -> Self {
        let has_seed_cmp = !seed_reads.is_empty() || ops_have_seed_cmp(&ops);
        CompiledCond {
            ops,
            slot_keys,
            output_slots,
            n_slots,
            seed_reads,
            fact_bind,
            has_seed_cmp,
            span,
            slot_names,
        }
    }

    /// The condition's wat span — where a computed operand's raise points.
    pub(crate) fn span(&self) -> &Span {
        &self.span
    }

    /// Slot names for diagnostics.
    pub(crate) fn slot_names(&self) -> &[crate::rete::expr_ir::SlotName] {
        &self.slot_names
    }

    /// Leftover seed: a `?var` this cond does not bind. Populate skipped
    /// `SeedCmp`; rematch must still run. Absence is a proof the Element
    /// already holds every bind the fold will read (`DESIGN-STONE-accum-fold-the-wall`).
    pub(crate) fn has_seed_cmp(&self) -> bool {
        self.has_seed_cmp
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

struct AlphaCompileCx<'a> {
    /// Needed to lower a NESTED operand through the one expression core (fix-list F). The alpha
    /// compile path is otherwise sym-free; this is a borrow, so it costs nothing at runtime.
    sym: &'a crate::runtime::SymbolTable,
    /// Slot names for `Op::Eval`'s diagnostics, rebuilt whenever a computed operand lowers.
    slot_names: crate::rete::expr_ir::SlotNames,
    field_names: &'a [String],
    next_slot: usize,
    defer_unbound: bool,
    seed_reads: Vec<(Value, usize)>,
}

/// Compile one alpha condition (`(:ClassName clause…)`, exactly what `alpha_cond` stores) into a
/// [`CompiledCond`] against `field_names` (that class's declared field order — the same list
/// `alpha_match_inner`'s caller resolves via `class_field_names`).
///
/// Returns `None` on a pattern miss (`alpha_pattern` refuses the cond) **or** when
/// `compile_seq` hits a Law-A `ConstraintSpelling::CoreGeneric` head. Fire setup
/// calls [`compile_condition_local`] and refuses a miss ("alpha N cond did not compile");
/// it does not fall back to `alpha_match_inner`.
///
/// Strict: an unbound constraint `?var` is [`Op::Fail`]. Fire setup uses
/// [`compile_condition_local`] (leftover-as-seed). This entry is the populate
/// differential against `alpha_match_inner`. Test-only; wat `compile-condition`
/// mints network nodes.
#[cfg(test)]
pub(crate) fn compile_alpha_ops(
    cond: &WatAST,
    field_names: &[String],
    sym: &crate::runtime::SymbolTable,
) -> Option<CompiledCond> {
    compile_condition_opts(cond, field_names, sym, false)
}

/// Same as [`compile_alpha_ops`], but an unbound constraint `?var` becomes a
/// seed slot + [`Op::SeedCmp`] (not omitted, not [`Op::Fail`], test_sym()). Populate skips
/// `SeedCmp`; rematch fills the slot from the token and runs the compare.
/// One compile for every alpha at fire setup.
pub(crate) fn compile_condition_local(
    cond: &WatAST,
    field_names: &[String],
    sym: &crate::runtime::SymbolTable,
) -> Option<CompiledCond> {
    compile_condition_opts(cond, field_names, sym, true)
}

fn compile_condition_opts(
    cond: &WatAST,
    field_names: &[String],
    sym: &crate::runtime::SymbolTable,
    defer_unbound: bool,
) -> Option<CompiledCond> {
    let pat = crate::rete::matcher::alpha_pattern(cond)?;
    let clauses = pat.clauses;

    let mut ctx = AlphaCompileCx {
        sym,
        slot_names: Box::from([]),
        field_names,
        next_slot: 0,
        defer_unbound,
        seed_reads: Vec::new(),
    };
    let mut scope: HashMap<String, usize> = HashMap::new();
    let mut field_slots: HashMap<usize, usize> = HashMap::new();
    let mut order: Vec<(String, usize)> = Vec::new();
    let mut ops: Vec<Op> = Vec::new();
    if !compile_seq(
        clauses,
        &mut ctx,
        &mut scope,
        &mut field_slots,
        &mut order,
        &mut ops,
    ) {
        // Law A: a CoreGeneric comparator is freeze-walled on defrule; compile-all /
        // compile-condition must refuse the same head (circumspicere 2026-08-20).
        return None;
    }

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
    let fact_bind = pat.fact_var.map(|v| Value::String(Arc::new(v.to_string())));

    Some(CompiledCond::from_parts(
        ops,
        slot_keys,
        output_slots,
        ctx.next_slot,
        seed_reads,
        fact_bind,
        cond.span().clone(),
        ctx.slot_names,
    ))
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
    ctx: &mut AlphaCompileCx,
    scope: &mut HashMap<String, usize>,
    field_slots: &mut HashMap<usize, usize>,
    order: &mut Vec<(String, usize)>,
    ops: &mut Vec<Op>,
) -> bool {
    for clause in clauses {
        if !compile_one(clause, ctx, scope, field_slots, order, ops) {
            return false;
        }
    }
    true
}

/// Compile one clause, appending to `ops`. `scope`/`order` are the CALLER's — for `and` (and the
/// top-level list) that is the real, surviving scope; for `or`/`not` branches the caller passes
/// a throwaway clone/scratch (see below), matching `eval_clause`'s discard of branch-local binds.
fn compile_one(
    clause: &WatAST,
    ctx: &mut AlphaCompileCx,
    scope: &mut HashMap<String, usize>,
    field_slots: &mut HashMap<usize, usize>,
    order: &mut Vec<(String, usize)>,
    ops: &mut Vec<Op>,
) -> bool {
    match classify_rete_clause(clause) {
        // A boolean rete expression where a constraint goes. Lowered through the one expression
        // core and required to be TRUE — which needs no new op: `Op::Eval` materialises the
        // predicate into a slot and the existing `Op::Cmp` compares it to `true`.
        ReteClauseShape::Predicate(expr) => {
            match compile_operand_expr(expr, scope, field_slots, ctx, ops) {
                OperandLowering::Lowered(e) => {
                    ops.push(Op::Cmp { op: CmpKind::Eq, lhs: e, rhs: Expr::Lit(Value::bool(true)) });
                }
                // Same three-way split the operands use: an unresolvable predicate can never hold
                // (`Op::Fail`), a refused one is a USER ERROR and refuses the whole condition.
                OperandLowering::Unresolvable => ops.push(Op::Fail),
                OperandLowering::Refused => return false,
            }
        }
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
            // The ONE DOOR (`clause::classify_constraint_head`) — the constraint vocabulary is
            // written down once, in `clause.rs`, and read here rather than re-listed. A `None`
            // means this file and the grammar disagree, which is our bug, not the caller's.
            let (cmp, spelling) = classify_constraint_head(op).unwrap_or_else(|| {
                unreachable!(
                    "classify_rete_clause admitted a Constraint head the ONE DOOR rejects: {op}"
                )
            });
            if matches!(spelling, ConstraintSpelling::CoreGeneric) {
                return false;
            }
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
                // A form that will not lower is a USER ERROR: refuse the whole condition so
                // `arm.rs` reports it, instead of compiling a silent never-match.
                (OperandLowering::Refused, _) | (_, OperandLowering::Refused) => return false,
                (OperandLowering::Lowered(lhs), OperandLowering::Lowered(rhs)) => {
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
        ReteClauseShape::And(subs) => {
            return compile_seq(subs, ctx, scope, field_slots, order, ops)
        }
        ReteClauseShape::Or(subs) => {
            // Each branch compiles against its OWN clone of the current scope (so a bind made by
            // one branch is never visible to a sibling), and its binds are recorded into a
            // throwaway `order` (discarded — an Or's successful branch's bindings never survive
            // the Or, exactly like `eval_clause`'s `Or` arm returning the pre-`or` `entry`).
            // `field_slots` is cloned the same way: a hidden `:field` Bind in one arm must
            // not satisfy a sibling that never wrote that slot.
            let mut branches: OrBranches = Vec::with_capacity(subs.len());
            for sub in subs {
                let mut sub_scope = scope.clone();
                let mut sub_fields = field_slots.clone();
                let mut scratch_order = Vec::new();
                let mut sub_ops = Vec::new();
                if !compile_one(
                    sub,
                    ctx,
                    &mut sub_scope,
                    &mut sub_fields,
                    &mut scratch_order,
                    &mut sub_ops,
                ) {
                    return false;
                }
                branches.push(sub_ops);
            }
            ops.push(Op::Or(branches));
        }
        ReteClauseShape::Not(sub) => {
            let mut sub_scope = scope.clone();
            let mut sub_fields = field_slots.clone();
            let mut scratch_order = Vec::new();
            let mut sub_ops = Vec::new();
            if !compile_one(
                sub,
                ctx,
                &mut sub_scope,
                &mut sub_fields,
                &mut scratch_order,
                &mut sub_ops,
            ) {
                return false;
            }
            ops.push(Op::Not(sub_ops));
        }
        ReteClauseShape::Where(_)
        | ReteClauseShape::Exists(_)
        | ReteClauseShape::Accumulate { .. }
        | ReteClauseShape::FactBind { .. }
        | ReteClauseShape::Unrecognized => ops.push(Op::Fail),
    }
    true
}

/// True when `e` is a slot allocated as a leftover-as-seed read.
fn expr_reads_seed(e: &Expr, seed_reads: &[(Value, usize)]) -> bool {
    match e {
        Expr::Slot(i) => seed_reads.iter().any(|(_, slot)| *slot == *i as usize),
        _ => false,
    }
}

fn expr_slot(slot: usize) -> Option<Expr> {
    u16::try_from(slot).ok().map(Expr::Slot)
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
/// What compiling one operand produced — and the three cases must NOT collapse.
///
/// ⛔ **THIS ENUM IS FIX-LIST F's LESSON APPLIED TO F's OWN FIX.** F was "could not lower" being
/// reported as `Op::Fail`, a compiled permanent never-match. The first repair lowered nested calls
/// through the core but kept `Option`, so a nested operand that STILL would not lower — a
/// non-rete head, which Law A must refuse — fell into the same silent bucket. Measured: a core
/// `(:wat::core::i64::+ …)` inline compiled and matched nothing, while the identical form in a
/// `where` fence was refused by name. The defect class re-entered through its own cure.
///
/// So the three outcomes are separated by TYPE rather than by discipline:
///   · `Lowered`      — an expression the executor can run.
///   · `Unresolvable` — an unbound `?var` or a field this class does not declare. It can NEVER
///     match, and compiling that as `Op::Fail` is correct and always was.
///   · `Refused`      — a form that will not lower at all. It is a USER ERROR and must reach the
///     user as one; the whole condition refuses, which `arm.rs` turns into a located
///     `MalformedForm`. Never `Op::Fail`, because silence is what F was.
enum OperandLowering {
    Lowered(Expr),
    Unresolvable,
    Refused,
}

fn compile_operand_expr(
    operand: &WatAST,
    scope: &mut HashMap<String, usize>,
    field_slots: &mut HashMap<usize, usize>,
    ctx: &mut AlphaCompileCx<'_>,
    ops: &mut Vec<Op>,
) -> OperandLowering {
    match operand {
        WatAST::Symbol(ident, _) => {
            let name = ident.as_str();
            if name.starts_with('?') {
                if let Some(slot) = scope.get(name).copied() {
                    lowered(expr_slot(slot))
                } else if ctx.defer_unbound {
                    let slot = ctx.next_slot;
                    ctx.next_slot += 1;
                    scope.insert(name.to_string(), slot);
                    ctx.seed_reads
                        .push((Value::String(Arc::new(name.to_string())), slot));
                    lowered(expr_slot(slot))
                } else {
                    OperandLowering::Unresolvable
                }
            } else {
                OperandLowering::Unresolvable
            }
        }
        WatAST::Keyword(k, _) => {
            let field_name = k.strip_prefix(':').unwrap_or(k.as_str());
            let Some(field_idx) = ctx.field_names.iter().position(|n| n == field_name) else {
                // ⛔ NOT A FIELD -> A CONSTANT. This `else` used to be `Unresolvable`, and that is
                // the whole reason `keyword::=` / `enum::=` could not be written inline: a keyword
                // in operand position was read as a field reference UNCONDITIONALLY, so
                // `(keyword::= :v :alpha)` was refused with "`:probe::In` has no field `:alpha`".
                //
                // ⚠ The engine was already deciding this correctly ONE LEVEL DOWN. The identical
                // comparison, nested as an operand of another call, fires and answers correctly —
                // because `bind_field_refs` (this same file, ~120 lines up) runs the SAME
                // `position(...)` lookup and falls through to a keyword literal. Same question,
                // two answers, one file. Measured 2026-08-28.
                //
                // `keyword_value` is the resolver that path already used: an enum unit variant if
                // the symbol table knows one, else a plain keyword. `:probe::E::A` therefore lands
                // as an enum value — and note it could never have been a field reference at all,
                // since it carries `::` and a field name is a bare identifier.
                //
                // This can only ADMIT programs, never change one: a non-field keyword here was a
                // hard freeze error, so no program that compiles today contains one.
                return lowered(Some(Expr::Lit(crate::rete::expr_ir::keyword_value(k, ctx.sym))));
            };
            if let Some(&slot) = field_slots.get(&field_idx) {
                return lowered(expr_slot(slot));
            }
            let slot = ctx.next_slot;
            ctx.next_slot += 1;
            field_slots.insert(field_idx, slot);
            ops.push(Op::Bind { field_idx, slot });
            lowered(expr_slot(slot))
        }
        // ── FIX-LIST F: a NESTED CALL operand, lowered through the one expression core ────────
        //
        // This arm did not exist. A nested call fell to the literal case below, returned `None`,
        // and the caller turned that into `Op::Fail` — a compiled, permanent, SILENT never-match.
        // So `(:wat::rete::core::i64::= (:wat::rete::core::i64::+ :v 2 :undefined 0) 12)` was
        // accepted at every gate, ran, and matched nothing for every fact, with no diagnostic.
        //
        // The builder settled the design question it resembled: *"we made it such that every rete
        // form can be compiled to a jump table... why is this any exception?"* It is not one. Same
        // `Expr::Call`, same opcode, same `RETE_OPS` table as the `where` fence — flip 3 gave
        // `cond` the core's `Expr` TYPE and stopped short of its LOWERING.
        WatAST::List(..) => {
            // An operand naming a `?var` this condition does not bind cannot resolve, and that is
            // the ORIGINAL, correct meaning of `Op::Fail`. Detect it before lowering, because
            // `lower_in_frame` would happily mint a fresh slot for it and leave it unfilled.
            if unbound_qvar_in(operand, scope) {
                return OperandLowering::Unresolvable;
            }
            // Field refs are alpha-specific spelling: the core lowers a bare keyword to a keyword
            // LITERAL, so `:v` must become a slot read first. Each one gets the prologue `Op::Bind`
            // it would have had as a direct operand, under a reserved name that no user symbol can
            // collide with.
            let mut names: HashMap<String, u16> =
                scope.iter().map(|(k, v)| (k.clone(), *v as u16)).collect();
            let Some(rewritten) = bind_field_refs(operand, scope, field_slots, ctx, ops, &mut names)
            else {
                return OperandLowering::Refused;
            };
            let Ok(mut next) = u16::try_from(ctx.next_slot) else {
                return OperandLowering::Refused;
            };
            // ⛔ A LOWERING FAILURE IS A REFUSAL, NEVER `Op::Fail`. `lower_in_frame` rejects a
            // non-rete head — which is Law A doing its job — and reporting that as "this fact does
            // not match" is precisely the silence fix-list F was.
            let Ok(expr) =
                crate::rete::expr_ir::lower_in_frame(&rewritten, ctx.sym, &mut names, &mut next)
            else {
                return OperandLowering::Refused;
            };
            ctx.next_slot = next as usize;
            // Materialise into a slot so `Cmp` keeps its `Slot | Lit` operands.
            let result = ctx.next_slot;
            ctx.next_slot += 1;
            ctx.slot_names = invert_slot_names(&names, ctx.next_slot);
            ops.push(Op::Eval { expr, slot: result });
            lowered(expr_slot(result))
        }
        other => match crate::rete::matcher::ast_literal_value(other) {
            Some(v) => OperandLowering::Lowered(Expr::Lit(v)),
            None => OperandLowering::Unresolvable,
        },
    }
}

/// `expr_slot` yields `Option<Expr>`; a `None` there is an internal slot-index failure, which is
/// not a user error and not a match outcome.
fn lowered(e: Option<Expr>) -> OperandLowering {
    match e {
        Some(x) => OperandLowering::Lowered(x),
        None => OperandLowering::Refused,
    }
}

/// Slot index -> name, for `Op::Eval`'s runtime diagnostics. Built from the lowering's final
/// name map so an unfilled slot reports what it was called rather than a bare index.
fn invert_slot_names(
    names: &HashMap<String, u16>,
    len: usize,
) -> crate::rete::expr_ir::SlotNames {
    let mut out: Vec<Option<Arc<str>>> = vec![None; len];
    for (name, &slot) in names {
        if let Some(cell) = out.get_mut(slot as usize) {
            *cell = Some(Arc::from(name.as_str()));
        }
    }
    out.into_boxed_slice()
}

/// True when the operand names a `?var` this condition has not bound.
///
/// Kept separate from the lowering so the ORIGINAL meaning of [`Op::Fail`] survives intact: an
/// operand that can never resolve should still compile to a permanent failure. What changed is
/// only that a nested CALL no longer counts as unresolvable.
fn unbound_qvar_in(ast: &WatAST, scope: &HashMap<String, usize>) -> bool {
    match ast {
        WatAST::Symbol(id, _) => {
            let n = id.as_str();
            n.starts_with('?') && !scope.contains_key(n)
        }
        WatAST::List(items, _) => items.iter().any(|i| unbound_qvar_in(i, scope)),
        _ => false,
    }
}

/// Rewrite every FIELD-naming keyword in operand position into a slot-backed symbol, emitting the
/// prologue `Op::Bind` that fills it.
///
/// Only non-head positions are considered, and only keywords that name a DECLARED field of this
/// class — so a call's head (`:wat::rete::core::i64::+`) and a marker like `:undefined` are left
/// exactly as they are and reach the core as themselves.
fn bind_field_refs(
    ast: &WatAST,
    scope: &mut HashMap<String, usize>,
    field_slots: &mut HashMap<usize, usize>,
    ctx: &mut AlphaCompileCx<'_>,
    ops: &mut Vec<Op>,
    names: &mut HashMap<String, u16>,
) -> Option<WatAST> {
    match ast {
        WatAST::Keyword(k, span) => {
            let field_name = k.strip_prefix(':').unwrap_or(k.as_str());
            let Some(field_idx) = ctx.field_names.iter().position(|n| n == field_name) else {
                return Some(ast.clone());
            };
            let slot = if let Some(&s) = field_slots.get(&field_idx) {
                s
            } else {
                let s = ctx.next_slot;
                ctx.next_slot += 1;
                field_slots.insert(field_idx, s);
                ops.push(Op::Bind { field_idx, slot: s });
                s
            };
            // `%` cannot begin a user symbol read from source, so this name cannot collide with a
            // `?var` or a `let` binder inside the operand.
            let reserved = format!("%alpha-field%{field_name}");
            names.insert(reserved.clone(), u16::try_from(slot).ok()?);
            let _ = scope;
            Some(WatAST::Symbol(crate::scope::Identifier::bare(reserved), span.clone()))
        }
        WatAST::List(items, span) => {
            let mut out = Vec::with_capacity(items.len());
            for (i, item) in items.iter().enumerate() {
                if i == 0 {
                    out.push(item.clone());
                } else {
                    out.push(bind_field_refs(item, scope, field_slots, ctx, ops, names)?);
                }
            }
            Some(WatAST::List(out, span.clone()))
        }
        // A VECTOR has no head, so every element is an operand position. This is where a `let`
        // binder lives (`(let [x :v] ...)`), and it was the hole: `Vector` fell to the old
        // `other => clone` catch-all, so `:v` was never rewritten, stayed a bare keyword, compared
        // unequal to every i64 forever, and the rule COMPILED, FIRED and MATCHED NOTHING with no
        // diagnostic. Measured 2026-08-28 against both engines, which shared the defect.
        WatAST::Vector(items, span) => {
            let mut out = Vec::with_capacity(items.len());
            for item in items {
                out.push(bind_field_refs(item, scope, field_slots, ctx, ops, names)?);
            }
            Some(WatAST::Vector(out, span.clone()))
        }
        // ⛔ REFUSED, never cloned through. A `{...}` / `#{...}` cannot appear in a rete expression
        // today — `:wat::rete::lower` rejects both ("cannot lower", measured) — so this arm is
        // unreachable, and that is exactly why it must not be a silent pass-through. If either
        // literal ever becomes lowerable, a field ref in a map KEY versus a map VALUE is a real
        // semantic question, and answering it by accident is the failure this whole strike is
        // pulling out. `None` here reaches `OperandLowering::Refused` at the call site — a refusal,
        // never `Op::Fail`.
        WatAST::Map(..) | WatAST::Set(..) => None,
        // The leaves, named rather than swept up: each one IS itself and is correct to clone.
        // ⛔ THE WILDCARD IS DELETED ON PURPOSE. `other => clone` meant two different things at
        // once — "this node is a leaf, leave it alone" AND "I have no arm for this node" — and the
        // second meaning is what made `Vector` silent. With every variant named, a new `WatAST`
        // variant is a COMPILE ERROR here instead of a silent never-match.
        WatAST::IntLit(..)
        | WatAST::FloatLit(..)
        | WatAST::RationalLit(..)
        | WatAST::BigIntLit(..)
        | WatAST::BoolLit(..)
        | WatAST::StringLit(..)
        | WatAST::CharLit(..)
        | WatAST::NilLit(..)
        | WatAST::Symbol(..) => Some(ast.clone()),
    }
}

// ─── The executor ───────────────────────────────────────────────────────────────

/// Run a compiled condition against one fact's fields. `scratch` is a caller-owned, caller-
/// reused `Vec<Option<Value>>` — `kernel/` allocates it once (sized to the largest
/// `CompiledCond::n_slots` across every compiled alpha) before the round loop starts, so this
/// function's own body never allocates on the Bind/Constraint (+ flattened `and`) path: `clear`
/// followed by `resize` back up to `compiled.n_slots` never reallocates once `scratch`'s
/// capacity has reached its high-water mark, and every write after that is a plain slice index.
/// The one exception is `Op::Or`/`Op::Not`, which copy `scratch` into a temporary — one frame
/// per disjunction and one per negation (T7 hoisted the `Or` case out of its branch loop on
/// 2026-08-25; the `Not` case is a single shot and was affirmatively cut, reasons at the arm).
///
/// **Exercised for CORRECTNESS, unreached by any PERF axis — the two are not the same claim,
/// and this comment used to blur them.** It read "not exercised by anything in the live grid
/// corpus", which was true when written and became false once `where-or-inline.{wat,clj}` landed
/// (native + oracle on every floor via `spec_equals_native_on_every_where_family`, Clara via
/// `check-where-shapes.sh`). What remains true is the narrower thing: no perf axis reaches these
/// arms, so they never fire on the path this stone's zero-allocation gate measures, and any
/// allocation change here is arithmetic rather than a measured win.
///
/// Populate: write pairs into `pool`, `?p` first when this cond is
/// `(?p <- :Type …)`. Returns the span. Same keys/values/order as
/// `alpha_match_inner` + `attach_fact` (STOP-1).
#[cfg(test)]
pub(crate) fn exec_compiled(
    sym: &crate::runtime::SymbolTable,
    compiled: &CompiledCond,
    fact_fields: &[Value],
    scratch: &mut SlotFrame,
    intern: &mut BindIntern<'_>,
    fact: &Value,
) -> Option<(u32, u16)> {
    exec_compiled_with_key_ids(sym, compiled, fact_fields, scratch, intern, fact, None)
}

pub(crate) fn exec_compiled_with_key_ids(
    sym: &crate::runtime::SymbolTable,
    compiled: &CompiledCond,
    fact_fields: &[Value],
    scratch: &mut SlotFrame,
    intern: &mut BindIntern<'_>,
    fact: &Value,
    key_ids: Option<&[u32]>,
) -> Option<(u32, u16)> {
    // Arc 278 DESIGN-STONE-compiled-conditions.md — the compiled path's call counter, parallel to
    // `alpha_match_inner`'s `match:calls`. Since this stone re-points the round loop's step 1 at
    // this function (`kernel/`), `match:calls` alone would read zero on a real fire from here
    // on; this is what a diagnostic census reads instead to see the production path is live.
    crate::rete::kernel::census_count("compiled:calls");
    scratch.clear();
    scratch.resize(compiled.n_slots, None);

    let cx = ExecCx { sym, span: compiled.span(), names: compiled.slot_names() };
    if !exec_ops(&compiled.ops, scratch, fact_fields, true, cx) {
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

/// Output field indexes iff every op is [`Op::Bind`] and each output
/// slot's field fits [`crate::rete::kernel::I64_ROW_CAP`]. Empty ops
/// (class-only) are bind-only with no fields
/// (`DESIGN-STONE-fire-i64-columns`).
pub(crate) fn bind_only_fields(compiled: &CompiledCond) -> Option<Vec<u8>> {
    if !compiled.ops.iter().all(|op| matches!(op, Op::Bind { .. })) {
        return None;
    }
    let mut out = Vec::with_capacity(compiled.output_slots.len());
    for &slot in compiled.output_slots.iter() {
        let fi = compiled.ops.iter().find_map(|op| match op {
            Op::Bind { field_idx, slot: s } if *s == slot => Some(*field_idx),
            _ => None,
        })?;
        if fi >= crate::rete::kernel::I64_ROW_CAP {
            return None;
        }
        out.push(fi as u8);
    }
    Some(out)
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
        if i >= compiled.slot_keys.len() {
            pool.truncate(off);
            return None;
        }
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
    sym: &crate::runtime::SymbolTable,
    compiled: &CompiledCond,
    fact_fields: &[Value],
    scratch: &mut SlotFrame,
    seed: &(impl Bindings + ?Sized),
) -> BindPairs {
    crate::rete::kernel::census_count("rematch:compiled");
    if !exec_compiled_under_holds(sym, compiled, fact_fields, scratch, seed) {
        return None;
    }
    materialize(compiled, scratch)
}

/// Seed `seed_reads`, run ops including [`Op::SeedCmp`]. No materialize / no PMap.
pub(crate) fn exec_compiled_under_holds(
    sym: &crate::runtime::SymbolTable,
    compiled: &CompiledCond,
    fact_fields: &[Value],
    scratch: &mut SlotFrame,
    seed: &(impl Bindings + ?Sized),
) -> bool {
    scratch.clear();
    scratch.resize(compiled.n_slots, None);
    for (key, slot) in compiled.seed_reads.iter() {
        scratch[*slot] = seed.get(key).cloned();
    }
    let cx = ExecCx { sym, span: compiled.span(), names: compiled.slot_names() };
    exec_ops(&compiled.ops, scratch, fact_fields, false, cx)
}

fn materialize(compiled: &CompiledCond, scratch: &[Option<Value>]) -> BindPairs {
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
        let sk = compiled.slot_keys.get(i)?;
        out.push((sk.clone(), v));
    }
    Some(out.into())
}

/// A leaked bare `SymbolTable` for tests that compile or execute a condition.
///
/// Threading `sym` is what let `Op::Eval` run a computed operand through the one expression core
/// (fix-list F). Tests that never write a computed operand still need SOME table to pass, and a
/// bare world is the cheapest honest one — built once, shared, never mutated.
#[cfg(test)]
pub(crate) fn test_sym() -> &'static crate::runtime::SymbolTable {
    use std::sync::OnceLock;
    static W: OnceLock<crate::freeze::FrozenWorld> = OnceLock::new();
    W.get_or_init(|| crate::freeze::startup_bare().expect("bare world for tests")).symbols()
}

/// What `Op::Eval` needs to run a computed operand — bundled so the four `exec_ops` call sites and
/// the recursive `Or`/`Not` branches take ONE extra parameter rather than three.
///
/// It is three borrows: nothing is allocated, nothing is cloned, and the per-fact fast path
/// (`Bind` / `Cmp` over `Slot | Lit`) never touches it.
#[derive(Clone, Copy)]
pub(crate) struct ExecCx<'a> {
    pub(crate) sym: &'a crate::runtime::SymbolTable,
    pub(crate) span: &'a Span,
    pub(crate) names: &'a [crate::rete::expr_ir::SlotName],
}

/// An `ExecCx` for tests that drive `exec_ops` directly with hand-built ops.
///
/// Those tests never contain an `Op::Eval`, so the sym/span/names are never read — but the type
/// requires them, which is the point: a future test that DOES build an `Op::Eval` gets a working
/// context for free instead of a reason to weaken the signature.
#[cfg(test)]
pub(crate) fn test_exec_cx() -> ExecCx<'static> {
    use std::sync::OnceLock;
    static SPAN: OnceLock<Span> = OnceLock::new();
    ExecCx {
        sym: test_sym(),
        span: SPAN.get_or_init(|| crate::rust_caller_span!()),
        names: &[],
    }
}

pub(crate) fn exec_ops(
    ops: &[Op],
    slots: &mut [Option<Value>],
    fact_fields: &[Value],
    skip_seed: bool,
    cx: ExecCx<'_>,
) -> bool {
    for op in ops {
        if skip_seed && matches!(op, Op::SeedCmp { .. }) {
            continue;
        }
        if !exec_op(op, slots, fact_fields, skip_seed, cx) {
            return false;
        }
    }
    true
}

fn exec_op(
    op: &Op,
    slots: &mut [Option<Value>],
    fact_fields: &[Value],
    skip_seed: bool,
    cx: ExecCx<'_>,
) -> bool {
    match op {
        Op::Fail => false,
        // FIX-LIST F — run a computed operand and materialise it, exactly as `Bind` materialises a
        // field. A raise fails the clause rather than propagating: every rete row is TOTAL by the
        // wall, so reaching the Err arm means an engine bug, and failing closed here matches what
        // an unresolvable operand has always done.
        Op::Eval { expr, slot } => {
            match crate::rete::expr_ir::exec(expr, slots, cx.names, cx.sym, cx.span) {
                Ok(v) => match slots.get_mut(*slot) {
                    Some(dst) => {
                        *dst = Some(v);
                        true
                    }
                    None => false,
                },
                Err(_) => false,
            }
        }
        Op::Bind { field_idx, slot } => match (fact_fields.get(*field_idx), slots.get_mut(*slot)) {
            (Some(v), Some(dst)) => {
                *dst = Some(v.clone());
                true
            }
            _ => false,
        },
        Op::BindCheck { field_idx, slot } => match fact_fields.get(*field_idx) {
            Some(v) => match slots.get_mut(*slot) {
                Some(slot_cell) => {
                    if slot_cell.is_none() {
                        // Should not occur (a BindCheck's slot was, by construction, already written
                        // by the Bind that first introduced it) — fall back to a fresh bind rather
                        // than fail outright, matching eval_clause's own "None => fresh insert" arm.
                        *slot_cell = Some(v.clone());
                        true
                    } else {
                        slot_cell.as_ref() == Some(v)
                    }
                }
                None => false,
            },
            None => false,
        },
        Op::Cmp { op, lhs, rhs } | Op::SeedCmp { op, lhs, rhs } => {
            match (eval_cmp_operand(lhs, slots), eval_cmp_operand(rhs, slots)) {
                (Some(a), Some(b)) => eval_cmp(*op, a, b),
                _ => false,
            }
        }
        Op::Or(branches) => {
            // ONE frame for the whole disjunction, refilled per branch, instead of one
            // allocation per branch (T7). The COPY is the semantics — a failed branch must not
            // leak its bindings into the next, nor a succeeding one into the parent — and
            // `clear` + `extend_from_slice` preserves that exactly: every branch still starts
            // from pristine `slots`, and `clone` is still discarded either way. What goes is
            // only the repeated malloc, which was invariant work inside the loop.
            //
            // NOT a measured win, and it must not be quoted as one: no grid axis reaches these
            // arms (see the module doc), so there is no before/after to cite. It is taken as
            // arithmetic — N allocations become 1 — not as a speedup.
            let mut clone: SlotFrame = Vec::with_capacity(slots.len());
            for branch in branches {
                clone.clear();
                clone.extend_from_slice(slots);
                if exec_ops(branch, &mut clone, fact_fields, skip_seed, cx) {
                    return true;
                }
            }
            false
        }
        Op::Not(sub) => {
            // Single-shot: there is no loop here, so there is nothing to hoist. Driving this
            // last allocation to zero would need a nesting-aware frame arena (the shape
            // `EXEC_ARENA` in `expr_ir.rs` already implements for the where-executor, since
            // `or`/`not` nest arbitrarily and one shared scratch cannot serve two depths).
            //
            // AFFIRMATIVELY CUT, not deferred (T7's close, 2026-08-25): it would stand up a
            // SECOND arena mechanism beside an existing one — the duplication `solvere` exists
            // to catch — to buy a saving nothing in the corpus can measure, on arms no grid axis
            // reaches. If an axis is ever built for the intra-condition `or`/`not` shape, this
            // is the first place to look, and the arena is then worth the second mechanism.
            let mut clone: SlotFrame = slots.to_vec();
            !exec_ops(sub, &mut clone, fact_fields, skip_seed, cx)
        }
    }
}

fn eval_cmp_operand<'a>(operand: &'a Expr, slots: &'a [Option<Value>]) -> Option<&'a Value> {
    match operand {
        Expr::Lit(v) => Some(v),
        Expr::Slot(i) => slots.get(*i as usize).and_then(|o| o.as_ref()),
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
        let ast = crate::parse_one!("(:wjl::Wind (?w <- :kph) (:wat::rete::i64::> ?w ?c))")
            .expect("parse leftover cond");
        let fields = vec!["kph".to_string()];
        let compiled = compile_condition_local(&ast, &fields, test_sym()).expect("compile leftover-as-seed");
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
        let mut pool = Vec::new();
        let mut keys = Vec::new();
        let mut vals = Vec::new();
        let mut ids = ValIntern::default();
        let mut intern = BindIntern {
            keys: &mut keys,
            vals: &mut vals,
            ids: &mut ids,
            pool: &mut pool,
        };
        assert!(
            exec_compiled(test_sym(), &compiled, &fact, &mut scratch, &mut intern, &Value::i64(20),).is_some(),
            "populate skips SeedCmp so the fact enters alpha"
        );

        let seed_ok = PMap::from_pairs([(qvar("?c"), Value::i64(10))]);
        let rematch_ok = exec_compiled_under(test_sym(), &compiled, &fact, &mut scratch, &seed_ok);
        assert!(rematch_ok.is_some(), "20 > 10 holds under seed");

        let seed_fail = PMap::from_pairs([(qvar("?c"), Value::i64(30))]);
        assert!(
            exec_compiled_under(test_sym(), &compiled, &fact, &mut scratch, &seed_fail).is_none(),
            "20 > 30 fails under seed"
        );
        assert!(
            exec_compiled_under(test_sym(), &compiled, &fact, &mut scratch, &PMap::new()).is_none(),
            "unbound leftover seed is no match"
        );

        let seed_pairs = vec![(qvar("?c"), Value::i64(10))];
        let interp = alpha_match_inner_seeded(Some(test_sym()), &ast, "wjl::Wind", &fact, &fields, &seed_pairs);
        assert_eq!(
            interp.is_some(),
            rematch_ok.is_some(),
            "compiled rematch verdict must match the interpreter oracle"
        );
    }

    #[test]
    fn leftover_strict_compile_is_still_fail() {
        let ast = crate::parse_one!("(:wjl::Wind (?w <- :kph) (:wat::rete::i64::> ?w ?c))")
            .expect("parse leftover cond");
        let fields = vec!["kph".to_string()];
        let compiled = compile_alpha_ops(&ast, &fields, test_sym()).expect("strict compile");
        assert!(
            compiled.seed_reads.is_empty(),
            "strict compile must not seed leftover ?vars"
        );
        let mut scratch = Vec::new();
        let mut pool = Vec::new();
        let mut keys = Vec::new();
        let mut vals = Vec::new();
        let mut ids = ValIntern::default();
        let mut intern = BindIntern {
            keys: &mut keys,
            vals: &mut vals,
            ids: &mut ids,
            pool: &mut pool,
        };
        assert!(
            exec_compiled(
                test_sym(),
                &compiled,
                &[Value::i64(20)],
                &mut scratch,
                &mut intern,
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
            "(:wjl::Wind (?w <- :kph) (:wat::rete::i64::> ?w 30))",
            "(:wjl::Wind (?w <- :kph) (:wat::rete::i64::> ?w ?c))",
            "(?p <- :wjl::Wind (?w <- :kph))",
        ];
        for src in cases {
            let ast = crate::parse_one!(src).unwrap_or_else(|_| panic!("parse {src}"));
            assert!(
                crate::rete::matcher::alpha_pattern(&ast).is_some(),
                "alpha_pattern must hold for {src}"
            );
            assert!(
                compile_condition_local(&ast, &["kph".to_string()], test_sym()).is_some(),
                "compile_condition_local must not return None for {src}"
            );
        }
    }

    /// Freeze next to `Op`: a new variant that does not compile is a red build.
    /// Driver = slot population (`Bind`). Everything else is the expression core.
    #[derive(Debug, PartialEq, Eq)]
    enum Lands {
        Core,
        Driver,
    }

    fn lands(op: &Op) -> Lands {
        match op {
            // `Eval` is `Bind`'s sibling and lands the same way: both POPULATE a slot, and that is
            // what Driver means here. They differ only in where the value comes from — `Bind` from
            // a fact field, `Eval` from an expression. The expression it runs is of course the
            // core, but the op itself is the driver moving a value in, which is precisely why
            // `Cmp`'s operands could stay `Slot | Lit` when computed operands arrived.
            Op::Bind { .. } | Op::Eval { .. } => Lands::Driver,
            Op::BindCheck { .. }
            | Op::Cmp { .. }
            | Op::SeedCmp { .. }
            | Op::Or(_)
            | Op::Not(_)
            | Op::Fail => Lands::Core,
        }
    }

    #[test]
    fn every_op_variant_lands_in_core_or_driver() {
        let lit = crate::rete::expr_ir::Expr::Lit(Value::i64(0));
        let cmp = crate::rete::clause::CmpKind::Gt;
        let variants = [
            Op::Bind {
                field_idx: 0,
                slot: 0,
            },
            Op::BindCheck {
                field_idx: 0,
                slot: 0,
            },
            Op::Cmp {
                op: cmp,
                lhs: lit.clone(),
                rhs: lit.clone(),
            },
            Op::SeedCmp {
                op: cmp,
                lhs: lit.clone(),
                rhs: lit.clone(),
            },
            Op::Or(vec![]),
            Op::Not(vec![]),
            Op::Fail,
        ];
        let driver: Vec<_> = variants
            .iter()
            .filter(|op| lands(op) == Lands::Driver)
            .collect();
        let core: Vec<_> = variants
            .iter()
            .filter(|op| lands(op) == Lands::Core)
            .collect();
        assert!(
            core.len() >= 4,
            "only {} of {} variants reach the shared core",
            core.len(),
            variants.len()
        );
        assert_eq!(
            driver.len(),
            1,
            "driver must be exactly Bind, got {driver:?}"
        );
        assert!(matches!(driver[0], Op::Bind { .. }));
    }
}
