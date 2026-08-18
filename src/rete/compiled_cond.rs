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
//! **Slots internally; the public `Arc<[(Value, Value)]>` is materialized ONCE, on SUCCESS
//! ONLY.** The executor threads a `Vec<Option<Value>>` scratch buffer (reused call to call —
//! see [`exec_compiled`]'s `scratch` parameter) indexed by slot; only when every [`Op`] in the
//! compiled sequence has held does it zip the pre-built `slot_keys` with the bound slot values
//! into the same `Arc<[(Value, Value)]>` shape `Element`/`Token` see today. A failing call never
//! allocates a key, never allocates the output array, and — on the straight-line Bind/Constraint
//! path every live grid condition actually uses — never allocates at all (the scratch buffer's
//! own backing storage is allocated once at setup, not per call; see `kernel.rs`'s `match_scratch`).
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

use crate::ast::WatAST;
use crate::rete::matcher::{classify_constraint_head, classify_rete_clause, compare_values, CmpKind, ReteClauseShape};
use crate::runtime::Value;

// ─── The compiled program ──────────────────────────────────────────────────────

// The comparison operator is `matcher::CmpKind` — the SAME enum the grammar and the interpreter
// read, decided once here at compile time instead of re-matched on every call. This file used to
// declare its own identical `CmpKind`; two enums meant two places a new comparison had to be added,
// which is the duplication the ONE DOOR exists to delete.

/// A resolved operand: exactly the three shapes `resolve_operand` distinguishes at runtime
/// (`?var` from bindings, `:field` from the fact, or a bare literal) — but resolved to an index
/// or a built `Value` once, at compile time, instead of re-classified from the `WatAST` and
/// (for `?var`) re-allocated as a `Value::String` key on every call.
#[derive(Clone, Debug)]
pub(crate) enum Operand {
    /// `?var`, already bound earlier in this condition — read scratch slot `usize`.
    Slot(usize),
    /// `:field` — read the fact's field at this declaration-order index.
    Field(usize),
    /// A bare literal, built once.
    Lit(Value),
}

/// One instruction. Execution is a straight walk over a `[Op]` slice with short-circuit AND
/// (the first `Op` that fails to hold ends the whole match), mirroring `eval_clauses`'s
/// left-to-right fold exactly — `Or`/`Not` are the only ops that recurse into a sub-sequence.
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
    /// `(:wat::core::<op> a b)` — both operands resolved, per `resolve_operand`'s rules.
    Cmp { op: CmpKind, lhs: Operand, rhs: Operand },
    /// `(:wat::rete::or c1 c2 …)` — each branch is its OWN op sequence, tried against a scratch
    /// clone of the current slots (never the live slots): mirrors `eval_clause`'s `Or`, which
    /// always returns the pre-`or` bindings unchanged even on a successful branch.
    Or(Vec<Vec<Op>>),
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

/// A condition compiled once, at setup, from the immutable network — the pre-resolved dual of
/// `alpha_match_inner`. Built by [`compile_condition`]; run by [`exec_compiled`].
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
    /// `or`/`not` branch binds its own scratch-only vars). The caller's reusable scratch buffer
    /// must be at least this long.
    n_slots: usize,
}

impl CompiledCond {
    /// The scratch-buffer length `exec_compiled` needs for this program.
    pub(crate) fn n_slots(&self) -> usize {
        self.n_slots
    }
}

// ─── The compiler ───────────────────────────────────────────────────────────────

struct Ctx<'a> {
    field_names: &'a [String],
    next_slot: usize,
    defer_unbound: bool,
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
pub(crate) fn compile_condition(cond: &WatAST, field_names: &[String]) -> Option<CompiledCond> {
    compile_condition_opts(cond, field_names, false)
}

/// Same as [`compile_condition`], but a constraint whose `?var` is not bound
/// in this condition is skipped (deferred to beta), not compiled as `Op::Fail`.
/// Used only for `:exists` / `:not` alphas that mention a left-bound var.
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
    };
    let mut scope: HashMap<String, usize> = HashMap::new();
    let mut order: Vec<(String, usize)> = Vec::new();
    let mut ops: Vec<Op> = Vec::new();
    compile_seq(clauses, &mut ctx, &mut scope, &mut order, &mut ops);

    let slot_keys: Arc<[Value]> = order
        .iter()
        .map(|(name, _)| Value::String(Arc::new(name.clone())))
        .collect::<Vec<_>>()
        .into();
    let output_slots: Arc<[usize]> = order.iter().map(|(_, slot)| *slot).collect::<Vec<_>>().into();

    Some(CompiledCond { ops, slot_keys, output_slots, n_slots: ctx.next_slot })
}

/// Compile a clause LIST in the caller's own scope — the top-level condition body, or an `and`'s
/// sub-list (flattened directly into the caller's `ops`/`scope`/`order`, since `and` shares the
/// enclosing sequential scope exactly).
fn compile_seq(
    clauses: &[WatAST],
    ctx: &mut Ctx,
    scope: &mut HashMap<String, usize>,
    order: &mut Vec<(String, usize)>,
    ops: &mut Vec<Op>,
) {
    for clause in clauses {
        compile_one(clause, ctx, scope, order, ops);
    }
}

/// Compile one clause, appending to `ops`. `scope`/`order` are the CALLER's — for `and` (and the
/// top-level list) that is the real, surviving scope; for `or`/`not` branches the caller passes
/// a throwaway clone/scratch (see below), matching `eval_clause`'s discard of branch-local binds.
fn compile_one(
    clause: &WatAST,
    ctx: &mut Ctx,
    scope: &mut HashMap<String, usize>,
    order: &mut Vec<(String, usize)>,
    ops: &mut Vec<Op>,
) {
    match classify_rete_clause(clause) {
        ReteClauseShape::Bind { var, field } => match ctx.field_names.iter().position(|n| n == field) {
            // Field not declared on this class: read_fact_field would return None on every
            // fact of this class — a compile-time-provable, permanent failure.
            None => ops.push(Op::Fail),
            Some(field_idx) => {
                if let Some(&slot) = scope.get(var) {
                    ops.push(Op::BindCheck { field_idx, slot });
                } else {
                    let slot = ctx.next_slot;
                    ctx.next_slot += 1;
                    scope.insert(var.to_string(), slot);
                    order.push((var.to_string(), slot));
                    ops.push(Op::Bind { field_idx, slot });
                }
            }
        },
        ReteClauseShape::Constraint { op, lhs, rhs } => {
            // The ONE DOOR (`matcher::classify_constraint_head`) — the constraint vocabulary is
            // written down once, in `matcher.rs`, and read here rather than re-listed. A `None`
            // means this file and the grammar disagree, which is our bug, not the caller's.
            let (cmp, _spelling) = classify_constraint_head(op)
                .unwrap_or_else(|| unreachable!("classify_rete_clause admitted a Constraint head the ONE DOOR rejects: {op}"));
            // An operand that cannot be resolved AT ALL (an unbound `?var` — statically proven
            // unbound by this point in the scope, since nothing earlier in this exact walk
            // recorded it; or a `:field` this class does not declare) makes `resolve_operand`
            // return None on every call, which makes the whole Constraint (and hence the whole
            // match) fail. Compile that permanently, at build time, instead of re-discovering it
            // on every fact.
            match (
                compile_operand(lhs, scope, ctx.field_names),
                compile_operand(rhs, scope, ctx.field_names),
            ) {
                (Some(lhs), Some(rhs)) => ops.push(Op::Cmp { op: cmp, lhs, rhs }),
                _ if ctx.defer_unbound
                    && (operand_is_qvar(lhs) || operand_is_qvar(rhs)) => {}
                _ => ops.push(Op::Fail),
            }
        }
        ReteClauseShape::And(subs) => compile_seq(subs, ctx, scope, order, ops),
        ReteClauseShape::Or(subs) => {
            // Each branch compiles against its OWN clone of the current scope (so a bind made by
            // one branch is never visible to a sibling), and its binds are recorded into a
            // throwaway `order` (discarded — an Or's successful branch's bindings never survive
            // the Or, exactly like `eval_clause`'s `Or` arm returning the pre-`or` `entry`).
            let branches: Vec<Vec<Op>> = subs
                .iter()
                .map(|sub| {
                    let mut sub_scope = scope.clone();
                    let mut scratch_order = Vec::new();
                    let mut sub_ops = Vec::new();
                    compile_one(sub, ctx, &mut sub_scope, &mut scratch_order, &mut sub_ops);
                    sub_ops
                })
                .collect();
            ops.push(Op::Or(branches));
        }
        ReteClauseShape::Not(sub) => {
            let mut sub_scope = scope.clone();
            let mut scratch_order = Vec::new();
            let mut sub_ops = Vec::new();
            compile_one(sub, ctx, &mut sub_scope, &mut scratch_order, &mut sub_ops);
            ops.push(Op::Not(sub_ops));
        }
        ReteClauseShape::Where(_)
        | ReteClauseShape::Exists(_)
        | ReteClauseShape::Accumulate { .. }
        | ReteClauseShape::FactBind { .. }
        | ReteClauseShape::Unrecognized => ops.push(Op::Fail),
    }
}

/// Resolve one operand AST node to an [`Operand`] at compile time — the same three shapes
/// `resolve_operand` distinguishes at runtime (`?var` from the scope-so-far, `:field` from the
/// class's declared fields, or a bare literal). Returns `None` exactly when `resolve_operand`
/// would unconditionally return `None` for every fact of this class: an unbound `?var` (nothing
/// earlier in this scope's walk bound it) or a field name this class does not declare.
fn operand_is_qvar(operand: &WatAST) -> bool {
    matches!(operand, WatAST::Symbol(ident, _) if ident.as_str().starts_with('?'))
}

fn compile_operand(operand: &WatAST, scope: &HashMap<String, usize>, field_names: &[String]) -> Option<Operand> {
    match operand {
        WatAST::Symbol(ident, _) => {
            let name = ident.as_str();
            if name.starts_with('?') {
                scope.get(name).map(|&slot| Operand::Slot(slot))
            } else {
                None
            }
        }
        WatAST::Keyword(k, _) => {
            let field_name = k.strip_prefix(':').unwrap_or(k.as_str());
            field_names.iter().position(|n| n == field_name).map(Operand::Field)
        }
        WatAST::IntLit(n, _) => Some(Operand::Lit(Value::i64(*n))),
        WatAST::FloatLit(x, _) => Some(Operand::Lit(Value::f64(*x))),
        WatAST::BoolLit(b, _) => Some(Operand::Lit(Value::bool(*b))),
        WatAST::StringLit(s, _) => Some(Operand::Lit(Value::String(Arc::new(s.clone())))),
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
/// Returns exactly what `alpha_match_inner(cond, fact_class, fact_fields, field_names)` would
/// for the SAME condition/fact — same `Some`/`None`, and on `Some` the identical
/// `Arc<[(Value, Value)]>`: same keys, same values, same order (STOP-1).
pub(crate) fn exec_compiled(
    compiled: &CompiledCond,
    fact_fields: &[Value],
    scratch: &mut Vec<Option<Value>>,
) -> Option<Arc<[(Value, Value)]>> {
    // Arc 278 DESIGN-STONE-compiled-conditions.md — the compiled path's call counter, parallel to
    // `alpha_match_inner`'s `match:calls`. Since this stone re-points the round loop's step 1 at
    // this function (`kernel.rs`), `match:calls` alone would read zero on a real fire from here
    // on; this is what a diagnostic census reads instead to see the production path is live.
    crate::rete::kernel::census_count("compiled:calls");
    scratch.clear();
    scratch.resize(compiled.n_slots, None);

    if !exec_ops(&compiled.ops, scratch, fact_fields) {
        return None;
    }

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

fn exec_ops(ops: &[Op], slots: &mut [Option<Value>], fact_fields: &[Value]) -> bool {
    for op in ops {
        if !exec_op(op, slots, fact_fields) {
            return false;
        }
    }
    true
}

fn exec_op(op: &Op, slots: &mut [Option<Value>], fact_fields: &[Value]) -> bool {
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
        Op::Cmp { op, lhs, rhs } => {
            match (read_operand(lhs, slots, fact_fields), read_operand(rhs, slots, fact_fields)) {
                (Some(a), Some(b)) => eval_cmp(*op, &a, &b),
                _ => false,
            }
        }
        Op::Or(branches) => {
            for branch in branches {
                let mut clone: Vec<Option<Value>> = slots.to_vec();
                if exec_ops(branch, &mut clone, fact_fields) {
                    return true;
                }
            }
            false
        }
        Op::Not(sub) => {
            let mut clone: Vec<Option<Value>> = slots.to_vec();
            !exec_ops(sub, &mut clone, fact_fields)
        }
    }
}

fn read_operand(operand: &Operand, slots: &[Option<Value>], fact_fields: &[Value]) -> Option<Value> {
    match operand {
        Operand::Slot(i) => slots.get(*i).and_then(|o| o.clone()),
        Operand::Field(i) => fact_fields.get(*i).cloned(),
        Operand::Lit(v) => Some(v.clone()),
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
