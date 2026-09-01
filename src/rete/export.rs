//! `#wat.rete/Export` — the compiled program as one EDN value.
//!
//! Not a Session. No facts, no memories, no source forms. Native fire only.
//! One tag; interior is packed vectors (kind + integers + literals).
//!
//! ## The three laws of this codec
//!
//! Sixty-one functions sit below, and they are not sixty-one separate decisions — they are ten
//! `pack_*`/`unpack_*` pairs plus the walls that guard the seam. Read these three laws and
//! every function's signature tells you the rest.
//!
//! **1. `pack` is total; `unpack` is partial — and the return type says so.** Every `pack_*` that
//! packs a tagged FORM returns a bare `Value` (`pack_children` is the one exception: it emits a
//! flat tail of child ids as an iterator, not a form): it consumes a structure this process already built and
//! type-checked, so there is nothing left to refuse. Every `unpack_*` returns
//! `Result<_, EvalBreak>`: it consumes bytes some *other* process wrote, and every one of them
//! can be a lie. The asymmetry holds across all ten pairs (`cmp`, `pat`, `expr`, `prog`,
//! `cond_op`, `compiled_cond`, `driver`, `fold`, `rhs_op`, `rhs`) — and across the two the list
//! above omits, `node` and `deps`, which are pairs too. Twelve in all. The asymmetry is the
//! fastest way to see which side of the trust boundary a function is standing on.
//!
//! ⛔ **Round-trip is SEMANTIC, not literal, and FOUR things are dropped — not one.** An earlier
//! version of this header said "exactly one field … nothing else is lossy". `intueri` falsified
//! it the same day. What actually does not survive:
//!
//! - `RhsOp::Bind`'s `Debug`-rendered `WatAST` (`pack_rhs_op`) — reconstructed from the key.
//! - The alpha's condition AST — `unpack_node` writes `empty_pv()` into the `tests` slot.
//! - `TestNode`'s `expr` — `unpack_node` writes `dummy_ast(span)`.
//! - `AccumulateNode`'s `acc-form` — likewise `dummy_ast(span)`.
//!
//! ⛔ **AND THE LAST THREE ARE LOAD-BEARING, WHICH IS WHY THIS MATTERS.** They are exactly the
//! fields `arm.rs` reads to build an arm FROM A NETWORK: `alpha_cond_from_node`,
//! `node_named_ast("expr")`, `node_named_ast("acc-form")`. An imported session runs only because
//! `import_export` interns a PREBUILT arm (`rete_arm_intern`, at the end of this file). Put an
//! imported network through `build_rete_arm` instead and `build_alpha_index` finds no readable
//! pattern, skips every alpha, and yields an arm with an EMPTY alpha index — no refusal, no
//! diagnostic, just a network that matches nothing.
//!
//! So: **an imported network may be FIRED, but it may not be RE-ARMED from its own nodes.** If
//! you ever need that, these three fields have to travel.
//!
//! **2. The wire shape is a tagged vector — `[:tag operand …]` — and an unknown tag is a
//! refusal, never a default.** The leading keyword is the discriminant, and every reader refuses
//! one it does not know — in one of two shapes: a multi-tag reader ends its `match` in an
//! `other =>` arm raising `malformed`, while a reader with exactly ONE legal tag
//! (`unpack_prog`, `unpack_compiled_cond`) refuses with an explicit `!= ":prog"` / `!= ":cond"`
//! check up front. Neither ever falls through to a default: a codec that defaults on an
//! unrecognised tag imports a *different program* than the one exported and reports success.
//!
//! The two directions are guarded differently and it is worth knowing which is protecting you.
//! Every `pack_*` is a bare `match` over a Rust enum with no catch-all, so a new `Pat`/`Expr`/
//! `Op`/`RhsOp`/`NodeKind` variant is a COMPILE ERROR here — you cannot forget to pack it. The
//! unpack side has no such help, because its input is a keyword rather than a type: a tag you
//! forgot to read is a located raise at run time, not a build failure. So when you add a
//! variant, the compiler will find the pack arm for you and nothing will find the unpack arm.
//!
//! **3. Unpacking a value is not trusting it. Six independent walls stand between the wire and
//! the evaluator**, and each catches what the one before it cannot:
//!
//! - **Range refusal, at the read** — `expect_u16` / `expect_op` / `expect_idx` refuse
//!   wrap-into-range rather than writing `n as u16`. A slot index that wraps does not fail; it
//!   silently addresses the wrong slot. `expect_op` additionally refuses `>= RETE_OPS.len()`,
//!   because a wrapped opcode dispatches a real-but-wrong `OpExec`.
//! - **Slot bounds, as a post-pass** — `check_program_slots` / `check_cond_ops` /
//!   `check_expr_slots` / `check_pat_slots` walk an already-unpacked structure and prove every
//!   slot it references lies inside its own declared `frame_len` / `n_slots`. Structure-level,
//!   not read-level: a slot can be a perfectly valid `u16` and still point past the frame it
//!   will run in.
//! - **Three compat gates, in `import_export`** — the format version (`v != FORMAT_V`), the ABI
//!   fingerprint (`abi_of` recomputed and compared), and *then* the host `TypeEnv` field order.
//!   The third is not redundant, and the reason is worth carrying: the fingerprint is computed
//!   from the classes and fields the export *itself* declares, so **a packed ABI can agree with
//!   itself and still disagree with this process's records.** Gate two proves the export is
//!   internally consistent; only gate three proves it fits *here*.
//! - **The graph shape, in `check_node_graph`** — the first three walls are each about a VALUE;
//!   this one is about the SHAPE OF THE GRAPH. Over the already-unpacked node map it proves every
//!   child id names a node, every reference-field alpha id names a node that is an `Alpha`, and
//!   every child id exceeds its parent's — the ascending-id topological order the alpha /
//!   root-join / hash-join passes require, which the compile path gets from minting ids increasing
//!   and the wire path gets from nobody. It refuses; it never repairs.
//! - **Nesting depth, threaded through the descent** — `deeper` / `MAX_IMPORT_DEPTH`. The four
//!   walls above are all about what a value *says*; this one is about how deep the reader goes to
//!   find out. Without it `import` had no depth criterion at all: what it accepted was whatever
//!   the importing THREAD's remaining stack allowed, so the same bytes were a valid network on a
//!   256 MiB thread and `fatal runtime error: stack overflow, aborting` on a 2 MiB one. A stack
//!   guard abort is not a panic and no `catch_unwind` reaches it, so refusing BEFORE the recursion
//!   is the only available cure. One budget is shared across every mutually recursive `unpack_*`
//!   — see `deeper` for why a per-function counter is walked past by the `:user` ↔ `:prog` cycle.
//! - **Node count, before a single node is unpacked** — `MAX_IMPORT_NODES`. The five walls above
//!   are each about the CONTENT of the wire: what a value says, and how deep the reader descends
//!   to find out. This one is about HOW MUCH the door will build. `import_export` assembles the
//!   network through `PMap::from_pairs`, whose accumulator scans everything already accumulated
//!   once per pair, so the build is quadratic in the node count — measured, 1.05 µs/pair at 500
//!   pairs and 4.87 µs/pair at 4 000 — and nothing bounded that count. What `import` accepted was
//!   therefore whatever the caller was willing to WAIT for, the same unstated-criterion shape the
//!   depth wall answers for the stack. The constant carries the corpus maximum it clears and the
//!   worst-case milliseconds it costs at its own limit; a cap without that arithmetic would move
//!   the unstated criterion rather than remove it.
//!
//! **4. The import door OPENS A SESSION, and what it allocates is charged to that session.**
//! `arm-session` is the door a compiled session is born through and it marks the session's byte
//! origin there (`alloc_counter::mark_session_origin`). `import` is the OTHER birth door and used
//! to mark nothing — which is worse than uncounted, because `alloc_counter::session_bytes` files
//! an unmarked session's origin at its FIRST CHECK: every byte the import allocated became
//! retroactively free and the ceiling began after the network already existed. Driven, on the same
//! 2 MB: marked-at-birth reads `2097268`, never-marked reads `0`.
//!
//! The cure is split in two because the ORIGIN CANNOT BE KEYED UNTIL THE THING IT KEYS EXISTS: the
//! key is the built network `PMap`'s identity, so `import_export` reads `thread_bytes()` as its
//! first statement, builds, and files THAT captured reading under the new identity through
//! `alloc_counter::mark_session_origin_at`. Reading at the moment the key appears — the natural
//! placement, and the one this law exists to forbid — excludes the whole build and reproduces the
//! defect while an origin is visibly filed. The filing does not clobber, so a `PMap` identity that
//! already carries an origin keeps the older one.

use std::collections::{HashMap, HashSet};
use std::sync::{Arc, OnceLock};

use crate::ast::WatAST;
use crate::rete::alpha_tree::AlphaTree;
use crate::rete::compiled_cond::{CompiledCond, Op};
use crate::rete::compiled_rhs::{CompiledRhs, CompiledRhsByRule, RhsOp};
use crate::rete::expr_ir::{Expr, Pat, Program};
use crate::rete::kernel::{
    alpha_cond_from_node, class_field_names, get_node, kind_of, network_identity, node_children,
    node_named_field, node_named_i64, node_named_string, node_ref_alpha_id, invert_feeding_alpha,
    kind_id_lists, rete_arm_get_or_build, rete_arm_intern,
    agg_named_field, session_named_field, session_network, session_names, sorted_node_ids, AccFold, AlphasByType,
    CondDriver, InternedNetwork, NodeKind, RuleDep,
};
use crate::rete::clause::CmpKind;
use crate::rete::matcher::alpha_pattern;
use crate::rete::vocabulary::RETE_OPS;
use crate::runtime::{
    EvalBreak, RuntimeError, RuntimeErrorKind, SymbolTable, Value, ValueSnapshot,
};
use crate::span::Span;
use crate::types::Nature;
use crate::value::pmap::PMap;
use crate::value::value::AggregateValue;

const OP: &str = ":wat::rete::export";
const IMPORT_OP: &str = ":wat::rete::import";
const FORMAT_V: i64 = 1;

::wat_source_derive::wat_field_names_from!(EXPORT_FIELDS, "wat/rete.wat", ":wat::rete::Export");
::wat_source_derive::wat_field_names_from!(ALPHA_FIELDS, "wat/rete.wat", ":wat::rete::AlphaNode");
::wat_source_derive::wat_field_names_from!(
    ROOT_FIELDS,
    "wat/rete.wat",
    ":wat::rete::RootJoinNode"
);
::wat_source_derive::wat_field_names_from!(
    HASH_FIELDS,
    "wat/rete.wat",
    ":wat::rete::HashJoinNode"
);
::wat_source_derive::wat_field_names_from!(
    PROD_FIELDS,
    "wat/rete.wat",
    ":wat::rete::ProductionNode"
);
::wat_source_derive::wat_field_names_from!(TEST_FIELDS, "wat/rete.wat", ":wat::rete::TestNode");
::wat_source_derive::wat_field_names_from!(
    NEG_FIELDS,
    "wat/rete.wat",
    ":wat::rete::NegationNode"
);
::wat_source_derive::wat_field_names_from!(
    EXISTS_FIELDS,
    "wat/rete.wat",
    ":wat::rete::ExistsNode"
);
::wat_source_derive::wat_field_names_from!(
    ACC_FIELDS,
    "wat/rete.wat",
    ":wat::rete::AccumulateNode"
);
::wat_source_derive::wat_field_names_from!(QUERY_FIELDS, "wat/rete.wat", ":wat::rete::QueryNode");

fn names(fields: &'static [&'static str]) -> crate::rete::kernel::FieldNames {
    crate::value::value::names_arc_from_static(fields)
}

fn export_names() -> crate::rete::kernel::FieldNames {
    static N: OnceLock<crate::rete::kernel::FieldNames> = OnceLock::new();
    N.get_or_init(|| names(EXPORT_FIELDS)).clone()
}

fn kw(name: &str) -> Value {
    Value::wat__core__keyword(Arc::new(name.to_string()))
}

fn pv(items: impl IntoIterator<Item = Value>) -> Value {
    Value::Vec(Arc::new(items.into_iter().collect()))
}

fn empty_pv() -> Value {
    Value::wat__core__PersistentVector(crate::value::pvec::PVec::new())
}

fn empty_pm() -> Value {
    Value::wat__core__PersistentMap(PMap::new())
}

fn dummy_ast(span: &Span) -> Value {
    Value::wat__WatAST(Arc::new(WatAST::List(Vec::new(), span.clone())))
}

fn malformed(span: &Span, op: &str, reason: impl Into<String>) -> EvalBreak {
    RuntimeError::new(
        span.clone(),
        RuntimeErrorKind::MalformedForm {
            head: op.into(),
            reason: reason.into(),
        },
    )
    .into()
}

fn expect_at<'a>(items: &'a [Value], i: usize, span: &Span, what: &str) -> Result<&'a Value, EvalBreak> {
    items
        .get(i)
        .ok_or_else(|| malformed(span, IMPORT_OP, format!("{what} missing slot {i}")))
}

fn export_named<'a>(export: &'a Value, name: &'static str, span: &Span) -> Result<&'a Value, EvalBreak> {
    agg_named_field(export, name)
        .ok_or_else(|| malformed(span, IMPORT_OP, format!("Export missing field `{name}`")))
}

fn expect_kw<'a>(v: &'a Value, op: &str, span: &Span) -> Result<&'a str, EvalBreak> {
    match v {
        Value::wat__core__keyword(s) => Ok(s.as_str()),
        other => Err(RuntimeError::new(
            span.clone(),
            RuntimeErrorKind::TypeMismatch {
                op: op.into(),
                expected: "keyword tag",
                got: Box::new(ValueSnapshot::of(other)),
            },
        )
        .into()),
    }
}

fn expect_i64(v: &Value, op: &str, span: &Span) -> Result<i64, EvalBreak> {
    match v {
        Value::i64(n) => Ok(*n),
        other => Err(RuntimeError::new(
            span.clone(),
            RuntimeErrorKind::TypeMismatch {
                op: op.into(),
                expected: "i64",
                got: Box::new(ValueSnapshot::of(other)),
            },
        )
        .into()),
    }
}

/// Slot / frame / param: refuse wrap-into-range (`n as u16`).
fn expect_u16(v: &Value, span: &Span, what: &str) -> Result<u16, EvalBreak> {
    let n = expect_i64(v, IMPORT_OP, span)?;
    u16::try_from(n).map_err(|_| {
        malformed(span, IMPORT_OP, format!("{what} {n} does not fit u16"))
    })
}

/// Opcode: refuse wrap-into-range AND refuse `>= RETE_OPS.len()` (apply_op would
/// either dispatch the wrong OpExec or MalformedForm after a wrap).
fn expect_op(v: &Value, span: &Span) -> Result<u16, EvalBreak> {
    let n = expect_i64(v, IMPORT_OP, span)?;
    if n < 0 || (n as u64) >= RETE_OPS.len() as u64 {
        return Err(malformed(
            span,
            IMPORT_OP,
            format!("op index {n} is outside RETE_OPS"),
        ));
    }
    Ok(n as u16)
}

/// Compiled-cond slot / n_slots / field_idx: non-negative and ≤ u16::MAX.
fn expect_idx(v: &Value, span: &Span, what: &str) -> Result<usize, EvalBreak> {
    let n = expect_i64(v, IMPORT_OP, span)?;
    if n < 0 || n > i64::from(u16::MAX) {
        return Err(malformed(
            span,
            IMPORT_OP,
            format!("{what} {n} does not fit a slot index"),
        ));
    }
    Ok(n as usize)
}

fn check_slot(slot: u16, frame_len: u16, span: &Span, what: &str) -> Result<(), EvalBreak> {
    if slot >= frame_len {
        return Err(malformed(
            span,
            IMPORT_OP,
            format!("{what} slot {slot} >= frame_len {frame_len}"),
        ));
    }
    Ok(())
}

/// WALL 6 — the largest network `import` will BUILD, and the only thing bounding the quadratic.
///
/// **MEASURED, not chosen for roundness**, the same way [`MAX_IMPORT_DEPTH`] is and for a finding
/// of the same shape: `import` had no SIZE criterion at all, so what it accepted was whatever the
/// caller happened to be willing to wait for. `import_export` assembles the network with
/// `PMap::from_pairs`, whose accumulator does `acc.iter_mut().find(...)` once per pair
/// (`value/pmap.rs`) — a linear scan of everything accumulated so far — so the build is QUADRATIC
/// in the node count, on bytes some other process wrote. Driven 2026-08-31, six samples per point,
/// minimum taken:
///
/// | pairs | total | per pair |
/// |---:|---:|---:|
/// | 500 | 523 µs | 1.05 µs |
/// | 1 000 | 1 954 µs | 1.95 µs |
/// | 2 000 | 5 143 µs | 2.57 µs |
/// | 4 000 | 19 473 µs | 4.87 µs |
///
/// Per-pair cost DOUBLES as N doubles — the quadratic signature. Fitting the 4 000 point gives
/// `t(N) ≈ 1.217e-3 · N² µs`, and the two numbers behind this constant are:
///
/// * **63** — the largest node count the corpus actually produces, measured by logging
///   `nodes.len()` at this door and running the whole `wat::rete` binary (434 tests, of which 34
///   reach an import). It is the datamancer program (`tests/rete/datamancer.rete.edn`, reached by
///   `probe_arc278_rete_edn`); every other import in the tree is 6–28 nodes. The corpus is a floor
///   on what is real and not a ceiling — which is exactly why the headroom below is two orders of
///   magnitude.
/// * **~122 ms** — what 10 000 costs on that curve (`1.217e-3 × 10 000² µs`): the worst case a
///   single `import` call may spend inside `from_pairs`. The next round numbers are 50 000 → ~3.0 s
///   and 100 000 → ~12 s, and neither is a bound worth calling one.
///
/// 10 000 is ~158× the measured maximum and the largest round cap whose worst case stays inside a
/// fifth of a second. Raise it only with a new measurement, and state what the new number COSTS.
///
/// ⚠ **THE CAP IS WHAT MAKES THE QUADRATIC SAFE; it is not a stand-in for a linear `from_pairs`.**
/// Making that accumulator linear is a `value/pmap.rs` change whose blast radius is every `PMap`
/// in the tree, and the table above is its recorded before-curve. Bounding N bounds the worst case
/// without touching it.
///
/// It is checked against the DECLARED length of the `nodes` vector, before a single node is
/// unpacked — refusing a claim is the only refusal that costs nothing.
const MAX_IMPORT_NODES: usize = 10_000;

/// WALL 5 — the recursive descent's ONE depth budget.
///
/// **MEASURED, not chosen for roundness.** The whole finding this bound answers is that
/// `import` had no depth criterion *at all*: the same 20,000-deep Export was ACCEPTED on a
/// 256 MiB thread and killed a 2 MiB one with `fatal runtime error: stack overflow, aborting`
/// — an abort, not a panic, so nothing catches it. Acceptance was a property of the importing
/// THREAD. Replacing that with an unmeasured constant would swap one unstated criterion for
/// another, so both numbers behind this one are written down:
///
/// * **3** — the deepest nesting the corpus actually produces, measured by instrumenting
///   [`deeper`] to record its running maximum and running the whole `wat::rete` binary
///   (423 tests, 26 of them importing). Every packed program in the corpus bottoms out at
///   `unpack_prog` → `unpack_expr` → one operand. The corpus is a floor on what is real, not a
///   ceiling: it is thin, and that is exactly why the headroom below is two orders of magnitude.
/// * **3,000–5,000** — the window in which the smallest stack observed here (a 2 MiB test
///   thread) aborts. The bound must sit far below the low end to be honest on the smallest
///   thread that will ever import.
///
/// 300 is `3 × 100` headroom over the measured maximum and one tenth of the low end of the
/// abort window. Raise it only with a new measurement; the 3,000 ceiling is the hard constraint.
const MAX_IMPORT_DEPTH: u32 = 300;

/// Descend one level, or refuse. Every mutually recursive `unpack_*` on the import path calls
/// this as its first statement and shadows its own `depth` with the result, so ONE budget is
/// shared across `unpack_expr`, `unpack_prog`, `unpack_pat`, `unpack_cond_op` and
/// `unpack_driver` rather than each counting its own.
///
/// That sharing is the contract, not an implementation detail. `unpack_expr`'s `:user` arm
/// calls `unpack_prog`, whose root calls `unpack_expr` again — a tower of `:user` nodes
/// alternates between the two and is walked past by any counter that only one of them
/// increments. `unpack_pat` (through `:match`) and `unpack_driver` / `unpack_cond_op` (through
/// their own composite arms) are three more towers at the same door. The probes named
/// `*_tower_past_the_depth_bound` in `tests/rete/probe_arc278_export.rs` drive one each, and
/// the `:user` one is sized so that an expr-only budget would still ACCEPT it.
fn deeper(depth: u32, span: &Span) -> Result<u32, EvalBreak> {
    let d = depth + 1;
    if d > MAX_IMPORT_DEPTH {
        return Err(malformed(
            span,
            IMPORT_OP,
            format!("import nesting depth {d} exceeds MAX_IMPORT_DEPTH {MAX_IMPORT_DEPTH}"),
        ));
    }
    Ok(d)
}

/// Wall 2 for patterns — every slot a `Pat` binds into must lie inside the frame it will run
/// in. `Lit` and `Wild` bind nothing and are trivially in bounds; the rest recurse.
fn check_pat_slots(pat: &Pat, frame_len: u16, span: &Span) -> Result<(), EvalBreak> {
    match pat {
        Pat::Lit(_) | Pat::Wild => Ok(()),
        Pat::Bind(s) => check_slot(*s, frame_len, span, "pbind"),
        Pat::Variant { payload, .. } => match payload {
            Some(inner) => check_pat_slots(inner, frame_len, span),
            None => Ok(()),
        },
        // Every bound slot is checked, not just the first — a hash-destructure binds N of them
        // and an out-of-frame slot in ANY position is the same wire defect.
        Pat::Fields(binds) => {
            for (_, slot) in binds.iter() {
                check_slot(*slot, frame_len, span, "pfields")?;
            }
            Ok(())
        }
    }
}

/// Wall 2 for expressions — the recursive half. Walks every `Expr` variant and proves each
/// slot READ and each slot BOUND by a `let`/`match` arm lies inside `frame_len`.
///
/// This is the wall that has to be exhaustive rather than clever: an expression tree reaches
/// slots through a dozen different variants, and a single unwalked arm is a hole that
/// `expect_idx` cannot cover — a slot index can be a perfectly valid `u16` and still address
/// past the end of the frame it executes in.
fn check_expr_slots(e: &Expr, frame_len: u16, span: &Span) -> Result<(), EvalBreak> {
    match e {
        Expr::Lit(_) => Ok(()),
        Expr::Slot(s) => check_slot(*s, frame_len, span, "expr"),
        Expr::Call { args, .. } | Expr::And(args) | Expr::Or(args) => {
            for a in args.iter() {
                check_expr_slots(a, frame_len, span)?;
            }
            Ok(())
        }
        Expr::CallFallback {
            args, fallback, ..
        } => {
            for a in args.iter() {
                check_expr_slots(a, frame_len, span)?;
            }
            check_expr_slots(fallback, frame_len, span)
        }
        Expr::CallUser { args, .. } => {
            for a in args.iter() {
                check_expr_slots(a, frame_len, span)?;
            }
            Ok(())
        }
        Expr::Field { recv, .. } => check_expr_slots(recv, frame_len, span),
        Expr::Construct { fields, .. } | Expr::Variant { fields, .. } => {
            for f in fields.iter() {
                check_expr_slots(f, frame_len, span)?;
            }
            Ok(())
        }
        Expr::If {
            cond,
            then_,
            else_,
        } => {
            check_expr_slots(cond, frame_len, span)?;
            check_expr_slots(then_, frame_len, span)?;
            check_expr_slots(else_, frame_len, span)
        }
        Expr::Let { binds, body } => {
            for (s, e) in binds.iter() {
                check_slot(*s, frame_len, span, "let")?;
                check_expr_slots(e, frame_len, span)?;
            }
            check_expr_slots(body, frame_len, span)
        }
        Expr::Match { scrutinee, arms } => {
            check_expr_slots(scrutinee, frame_len, span)?;
            for (pat, body) in arms.iter() {
                check_pat_slots(pat, frame_len, span)?;
                check_expr_slots(body, frame_len, span)?;
            }
            Ok(())
        }
    }
}

/// Wall 2 for a whole `Program` — params, reads, then the root expression, all against the
/// program's own declared `frame_len`.
fn check_program_slots(p: &Program, span: &Span) -> Result<(), EvalBreak> {
    for s in p.params.iter() {
        check_slot(*s, p.frame_len, span, "param")?;
    }
    for (_, s) in p.reads.iter() {
        check_slot(*s, p.frame_len, span, "read")?;
    }
    check_expr_slots(&p.root, p.frame_len, span)
}

/// Wall 2 for a compiled condition — every op that WRITES a slot (`Bind`, `BindCheck`, `Eval`)
/// is bounds-checked against `n_slots`, and every op that READS through an expression hands that
/// expression to `check_expr_slots`.
///
/// `Or` and `Not` recurse: a nested branch is checked against the SAME `n_slots`, because the
/// frame is the condition's, not the branch's. An op that neither reads nor writes a slot
/// (`Fail`) needs no check, and saying so in an explicit arm is what keeps this exhaustive —
/// a catch-all here would silently admit the next slot-writing variant somebody adds.
fn check_cond_ops(ops: &[Op], n_slots: usize, span: &Span) -> Result<(), EvalBreak> {
    let frame_len = n_slots as u16;
    for op in ops {
        match op {
            Op::Bind { slot, .. } | Op::BindCheck { slot, .. } => {
                if *slot >= n_slots {
                    return Err(malformed(
                        span,
                        IMPORT_OP,
                        format!("cond slot {slot} >= n_slots {n_slots}"),
                    ));
                }
            }
            // FIX-LIST F — a COMPUTED operand materialised into a slot. Same two checks as any
            // other slot writer plus the expression's own reads.
            Op::Eval { expr, slot } => {
                if *slot >= n_slots {
                    return Err(malformed(
                        span,
                        IMPORT_OP,
                        format!("cond eval slot {slot} >= n_slots {n_slots}"),
                    ));
                }
                check_expr_slots(expr, frame_len, span)?;
            }
            Op::Cmp { lhs, rhs, .. } | Op::SeedCmp { lhs, rhs, .. } => {
                check_expr_slots(lhs, frame_len, span)?;
                check_expr_slots(rhs, frame_len, span)?;
            }
            Op::Or(branches) => {
                for b in branches.iter() {
                    check_cond_ops(b, n_slots, span)?;
                }
            }
            Op::Not(inner) => check_cond_ops(inner, n_slots, span)?,
            Op::Fail => {}
        }
    }
    Ok(())
}

fn expect_str<'a>(v: &'a Value, op: &str, span: &Span) -> Result<&'a str, EvalBreak> {
    match v {
        Value::String(s) => Ok(s.as_str()),
        other => Err(RuntimeError::new(
            span.clone(),
            RuntimeErrorKind::TypeMismatch {
                op: op.into(),
                expected: "string",
                got: Box::new(ValueSnapshot::of(other)),
            },
        )
        .into()),
    }
}

/// Accepts BOTH vector representations, because the two sides of the wire produce different
/// ones: this process packs with `pv` (`Value::Vec`), while a value that has been through an
/// EDN read-back arrives as a `PersistentVector`. Refusing either would make the codec's own
/// round-trip depend on which door the value came in by.
fn expect_seq(v: &Value, op: &str, span: &Span) -> Result<Vec<Value>, EvalBreak> {
    match v {
        Value::Vec(xs) => Ok((**xs).clone()),
        Value::wat__core__PersistentVector(pv) => Ok(pv.iter().cloned().collect()),
        other => Err(RuntimeError::new(
            span.clone(),
            RuntimeErrorKind::TypeMismatch {
                op: op.into(),
                expected: "vector",
                got: Box::new(ValueSnapshot::of(other)),
            },
        )
        .into()),
    }
}

fn record(class: &str, field_names: &'static [&'static str], fields: Vec<Value>) -> Value {
    Value::Aggregate(Arc::new(AggregateValue::record(
        class.into(),
        names(field_names),
        Arc::new(fields),
    )))
}

// ── ABI ──────────────────────────────────────────────────────────────────────

/// FNV-1a, 64-bit. Not a security hash and not required to be one — its only job is to make
/// `abi_of`'s fingerprint short enough to carry in the export and collide-resistant enough that
/// two genuinely different ABIs do not agree by accident.
fn fnv1a(s: &str) -> u64 {
    let mut h = 0xcbf29ce484222325;
    for b in s.as_bytes() {
        h ^= u64::from(*b);
        h = h.wrapping_mul(0x100000001b3);
    }
    h
}

/// The compatibility fingerprint: format version, every packed class with its field names IN
/// ORDER, and every `RETE_OPS` name IN ORDER, hashed to `v<N>:<16 hex>`.
///
/// Order is part of the identity on purpose — the wire addresses fields and ops BY INDEX, so two
/// processes that declare the same names in a different order are not compatible, and a
/// set-based fingerprint would call them equal. Note what this can and cannot see: `classes` and
/// `fields` are supplied by the caller, so when `import_export` recomputes it from the *export's
/// own* declaration, agreement proves the export is internally consistent — not that it fits
/// this process. That is why a third gate follows it. (Module header, law 3.)
fn abi_of(classes: &[String], fields: &[Vec<String>]) -> String {
    let mut s = format!("v{FORMAT_V}");
    for (c, fs) in classes.iter().zip(fields.iter()) {
        s.push(';');
        s.push_str(c);
        s.push('[');
        s.push_str(&fs.join(","));
        s.push(']');
    }
    s.push_str(";ops:");
    for (i, op) in RETE_OPS.iter().enumerate() {
        if i > 0 {
            s.push(',');
        }
        s.push_str(op.rete_name);
    }
    format!("v{FORMAT_V}:{:016x}", fnv1a(&s))
}

// ── Expr / Program ───────────────────────────────────────────────────────────

fn pack_cmp(k: CmpKind) -> Value {
    kw(match k {
        CmpKind::Eq => ":eq",
        CmpKind::NotEq => ":neq",
        CmpKind::Lt => ":lt",
        CmpKind::Gt => ":gt",
        CmpKind::Le => ":le",
        CmpKind::Ge => ":ge",
    })
}

fn unpack_cmp(v: &Value, span: &Span) -> Result<CmpKind, EvalBreak> {
    match expect_kw(v, IMPORT_OP, span)? {
        ":eq" => Ok(CmpKind::Eq),
        ":neq" => Ok(CmpKind::NotEq),
        ":lt" => Ok(CmpKind::Lt),
        ":gt" => Ok(CmpKind::Gt),
        ":le" => Ok(CmpKind::Le),
        ":ge" => Ok(CmpKind::Ge),
        other => Err(malformed(span, IMPORT_OP, format!("unknown cmp {other}"))),
    }
}

/// `Pat` → `[:plit v]` · `[:wild]` · `[:pbind slot]` · `[:pvar "Name" pat?]` ·
/// `[:pfields "field" slot …]`.
///
/// Two of these carry an OPTIONAL tail, and the codec encodes optionality two different ways —
/// know which you are looking at. Here it is ARITY: `:pvar` has three items when the variant
/// carries a payload and two when it does not. Inside a fixed-arity vector (`pack_prog`,
/// `pack_compiled_cond`) absence is instead `Value::Unit` in the slot. Both are read back
/// faithfully; neither is inferable from the other.
fn pack_pat(p: &Pat) -> Value {
    match p {
        Pat::Lit(v) => pv([kw(":plit"), v.clone()]),
        Pat::Wild => pv([kw(":wild")]),
        Pat::Bind(s) => pv([kw(":pbind"), Value::i64(*s as i64)]),
        Pat::Variant { name, payload } => {
            let mut xs = vec![kw(":pvar"), Value::String(Arc::new(name.clone()))];
            if let Some(inner) = payload {
                xs.push(pack_pat(inner));
            }
            pv(xs)
        }
        // `[:pfields "field" slot "field2" slot2 …]` — flat pairs, so the arity check on the
        // other side is one `% 2` rather than a nested sequence per binding.
        Pat::Fields(binds) => {
            let mut xs = vec![kw(":pfields")];
            for (field, slot) in binds.iter() {
                xs.push(Value::String(Arc::new(field.to_string())));
                xs.push(Value::i64(*slot as i64));
            }
            pv(xs)
        }
    }
}

/// Inverse of `pack_pat`. `:pfields` is the one arm with an arity law rather than a fixed
/// shape — the tail is flat `"field" slot` pairs, so its check is one `% 2` (see `pack_pat`,
/// which flattens them for exactly that reason) rather than a nested sequence per binding.
fn unpack_pat(v: &Value, span: &Span, depth: u32) -> Result<Pat, EvalBreak> {
    let depth = deeper(depth, span)?;
    let items = expect_seq(v, IMPORT_OP, span)?;
    let tag = items.first().ok_or_else(|| malformed(span, IMPORT_OP, "empty pat"))?;
    match expect_kw(tag, IMPORT_OP, span)? {
        ":plit" => {
            let lit = items
                .get(1)
                .ok_or_else(|| malformed(span, IMPORT_OP, "plit missing value"))?
                .clone();
            Ok(Pat::Lit(lit))
        }
        ":wild" => Ok(Pat::Wild),
        ":pbind" => {
            let n = expect_u16(
                items
                    .get(1)
                    .ok_or_else(|| malformed(span, IMPORT_OP, "pbind missing slot"))?,
                span,
                "pbind",
            )?;
            Ok(Pat::Bind(n))
        }
        ":pfields" => {
            let rest = &items[1..];
            if rest.is_empty() || rest.len() % 2 != 0 {
                return Err(malformed(
                    span,
                    IMPORT_OP,
                    format!("pfields takes non-empty (field, slot) PAIRS; got {} item(s)", rest.len()),
                ));
            }
            let mut binds: Vec<(std::sync::Arc<str>, u16)> = Vec::with_capacity(rest.len() / 2);
            for pair in rest.chunks_exact(2) {
                let field = expect_str(&pair[0], IMPORT_OP, span)?;
                let slot = expect_u16(&pair[1], span, "pfields")?;
                binds.push((std::sync::Arc::from(field), slot));
            }
            Ok(Pat::Fields(binds.into_boxed_slice()))
        }
        ":pvar" => {
            let name = expect_str(
                items
                    .get(1)
                    .ok_or_else(|| malformed(span, IMPORT_OP, "pvar missing name"))?,
                IMPORT_OP,
                span,
            )?
            .to_string();
            let payload = match items.get(2) {
                Some(inner) => Some(Box::new(unpack_pat(inner, span, depth)?)),
                None => None,
            };
            Ok(Pat::Variant { name, payload })
        }
        other => Err(malformed(span, IMPORT_OP, format!("unknown pat {other}"))),
    }
}

/// `Expr` → a tagged vector per variant: `:lit` `:slot` `:field` `:call` `:user` `:ctor`
/// `:variant` `:if` `:let` `:and` `:or` `:match`.
///
/// The largest packer, and the one whose exhaustiveness the compiler enforces for you: it is a
/// bare `match` over `Expr` with no catch-all, so a new variant fails to compile HERE rather
/// than round-tripping into something else. Its inverse cannot get that guarantee — see
/// `unpack_expr`.
fn pack_expr(e: &Expr) -> Value {
    match e {
        Expr::Lit(v) => pv([kw(":lit"), v.clone()]),
        Expr::Slot(s) => pv([kw(":slot"), Value::i64(*s as i64)]),
        Expr::Call { op, args } => {
            let mut xs = vec![kw(":call"), Value::i64(*op as i64)];
            xs.extend(args.iter().map(pack_expr));
            pv(xs)
        }
        Expr::CallFallback { op, args, fallback } => {
            let mut xs = vec![kw(":call-fb"), Value::i64(*op as i64), pack_expr(fallback)];
            xs.extend(args.iter().map(pack_expr));
            pv(xs)
        }
        Expr::CallUser { program, args } => {
            let mut xs = vec![kw(":user"), pack_prog(program)];
            xs.extend(args.iter().map(pack_expr));
            pv(xs)
        }
        Expr::Field { recv, idx } => {
            pv([kw(":field"), pack_expr(recv), Value::i64(*idx as i64)])
        }
        Expr::Construct {
            class,
            names,
            fields,
        } => {
            let mut xs = vec![
                kw(":ctor"),
                Value::String(Arc::new(class.clone())),
                pv(names.iter().map(|n| Value::String(Arc::new(n.clone())))),
            ];
            xs.extend(fields.iter().map(pack_expr));
            pv(xs)
        }
        Expr::Variant {
            type_path,
            variant_name,
            names,
            fields,
        } => {
            let mut xs = vec![
                kw(":variant"),
                Value::String(Arc::new(type_path.clone())),
                Value::String(Arc::new(variant_name.clone())),
                pv(names.iter().map(|n| Value::String(Arc::new(n.clone())))),
            ];
            xs.extend(fields.iter().map(pack_expr));
            pv(xs)
        }
        Expr::If { cond, then_, else_ } => {
            pv([kw(":if"), pack_expr(cond), pack_expr(then_), pack_expr(else_)])
        }
        Expr::And(xs) => {
            let mut out = vec![kw(":and")];
            out.extend(xs.iter().map(pack_expr));
            pv(out)
        }
        Expr::Or(xs) => {
            let mut out = vec![kw(":or")];
            out.extend(xs.iter().map(pack_expr));
            pv(out)
        }
        Expr::Let { binds, body } => {
            let pairs = binds.iter().map(|(s, e)| {
                pv([Value::i64(*s as i64), pack_expr(e)])
            });
            pv([kw(":let"), pv(pairs), pack_expr(body)])
        }
        Expr::Match { scrutinee, arms } => {
            let packed_arms = arms.iter().map(|(p, e)| pv([pack_pat(p), pack_expr(e)]));
            pv([kw(":match"), pack_expr(scrutinee), pv(packed_arms)])
        }
    }
}

/// Decode one EDN-packed expression back into an `Expr` — the exact inverse of
/// [`pack_expr`], which sits directly above it.
///
/// **The two are a pair and must be edited as one.** Every `Expr` variant needs
/// an arm in `pack_expr` that writes its keyword tag and an arm here that reads
/// it back; a variant added to only one side compiles cleanly and breaks the
/// export/import round-trip at runtime instead. `tests/rete/probe_arc278_export.wat`
/// and `probe_arc278_rete_edn.wat` are what catch that — a new variant belongs in
/// their corpus in the same change.
///
/// Shape: every packed expr is a sequence whose head is the variant's keyword
/// tag and whose tail is that variant's fields, in declaration order. Unknown
/// tags and missing fields raise `MalformedForm` under `IMPORT_OP` rather than
/// defaulting, because a silently-defaulted field would import a DIFFERENT rule
/// than the one exported.
fn unpack_expr(v: &Value, span: &Span, depth: u32) -> Result<Expr, EvalBreak> {
    let depth = deeper(depth, span)?;
    let items = expect_seq(v, IMPORT_OP, span)?;
    let tag = items.first().ok_or_else(|| malformed(span, IMPORT_OP, "empty expr"))?;
    match expect_kw(tag, IMPORT_OP, span)? {
        ":lit" => Ok(Expr::Lit(
            items
                .get(1)
                .ok_or_else(|| malformed(span, IMPORT_OP, "lit missing value"))?
                .clone(),
        )),
        ":slot" => {
            let n = expect_u16(
                items
                    .get(1)
                    .ok_or_else(|| malformed(span, IMPORT_OP, "slot missing n"))?,
                span,
                "slot",
            )?;
            Ok(Expr::Slot(n))
        }
        ":call" => {
            let op = expect_op(
                items
                    .get(1)
                    .ok_or_else(|| malformed(span, IMPORT_OP, "call missing op"))?,
                span,
            )?;
            let mut args = Vec::new();
            for x in items.iter().skip(2) {
                args.push(unpack_expr(x, span, depth)?);
            }
            Ok(Expr::Call {
                op,
                args: args.into_boxed_slice(),
            })
        }
        ":call-fb" => {
            let op = expect_op(
                items
                    .get(1)
                    .ok_or_else(|| malformed(span, IMPORT_OP, "call-fb missing op"))?,
                span,
            )?;
            let fallback = Box::new(unpack_expr(items.get(2).ok_or_else(|| {
                malformed(span, IMPORT_OP, "call-fb missing fallback")
            })?, span, depth)?);
            let mut args = Vec::new();
            for x in items.iter().skip(3) {
                args.push(unpack_expr(x, span, depth)?);
            }
            Ok(Expr::CallFallback {
                op,
                args: args.into_boxed_slice(),
                fallback,
            })
        }
        ":user" => {
            let program = Arc::new(unpack_prog(
                items
                    .get(1)
                    .ok_or_else(|| malformed(span, IMPORT_OP, "user missing prog"))?,
                span, depth
            )?);
            let mut args = Vec::new();
            for x in items.iter().skip(2) {
                args.push(unpack_expr(x, span, depth)?);
            }
            Ok(Expr::CallUser {
                program,
                args: args.into_boxed_slice(),
            })
        }
        ":field" => Ok(Expr::Field {
            recv: Box::new(unpack_expr(
                items
                    .get(1)
                    .ok_or_else(|| malformed(span, IMPORT_OP, "field missing recv"))?,
                span, depth
            )?),
            idx: expect_idx(
                items
                    .get(2)
                    .ok_or_else(|| malformed(span, IMPORT_OP, "field missing idx"))?,
                span,
                "field idx",
            )?,
        }),
        ":ctor" => {
            let class = expect_str(
                items
                    .get(1)
                    .ok_or_else(|| malformed(span, IMPORT_OP, "ctor missing class"))?,
                IMPORT_OP,
                span,
            )?
            .to_string();
            let names_pv = expect_seq(
                items
                    .get(2)
                    .ok_or_else(|| malformed(span, IMPORT_OP, "ctor missing names"))?,
                IMPORT_OP,
                span,
            )?;
            let mut ns = Vec::new();
            for n in names_pv.iter() {
                ns.push(expect_str(n, IMPORT_OP, span)?.to_string());
            }
            let mut fields = Vec::new();
            for x in items.iter().skip(3) {
                fields.push(unpack_expr(x, span, depth)?);
            }
            if ns.len() != fields.len() {
                return Err(malformed(
                    span,
                    IMPORT_OP,
                    format!("ctor names length {} != fields length {}", ns.len(), fields.len()),
                ));
            }
            Ok(Expr::Construct {
                class,
                names: Arc::new(ns),
                fields: fields.into_boxed_slice(),
            })
        }
        ":variant" => {
            let type_path = expect_str(
                items
                    .get(1)
                    .ok_or_else(|| malformed(span, IMPORT_OP, "variant missing type"))?,
                IMPORT_OP,
                span,
            )?
            .to_string();
            let variant_name = expect_str(
                items
                    .get(2)
                    .ok_or_else(|| malformed(span, IMPORT_OP, "variant missing name"))?,
                IMPORT_OP,
                span,
            )?
            .to_string();
            let names_pv = expect_seq(
                items
                    .get(3)
                    .ok_or_else(|| malformed(span, IMPORT_OP, "variant missing names"))?,
                IMPORT_OP,
                span,
            )?;
            let mut ns = Vec::new();
            for n in names_pv.iter() {
                ns.push(expect_str(n, IMPORT_OP, span)?.to_string());
            }
            let mut fields = Vec::new();
            for x in items.iter().skip(4) {
                fields.push(unpack_expr(x, span, depth)?);
            }
            if ns.len() != fields.len() {
                return Err(malformed(
                    span,
                    IMPORT_OP,
                    format!(
                        "variant names length {} != fields length {}",
                        ns.len(),
                        fields.len()
                    ),
                ));
            }
            Ok(Expr::Variant {
                type_path,
                variant_name,
                names: Arc::new(ns),
                fields: fields.into_boxed_slice(),
            })
        }
        ":if" => Ok(Expr::If {
            cond: Box::new(unpack_expr(
                items.get(1).ok_or_else(|| malformed(span, IMPORT_OP, "if"))?,
                span, depth
            )?),
            then_: Box::new(unpack_expr(
                items.get(2).ok_or_else(|| malformed(span, IMPORT_OP, "if"))?,
                span, depth
            )?),
            else_: Box::new(unpack_expr(
                items.get(3).ok_or_else(|| malformed(span, IMPORT_OP, "if"))?,
                span, depth
            )?),
        }),
        ":and" => {
            let mut xs = Vec::new();
            for x in items.iter().skip(1) {
                xs.push(unpack_expr(x, span, depth)?);
            }
            Ok(Expr::And(xs.into_boxed_slice()))
        }
        ":or" => {
            let mut xs = Vec::new();
            for x in items.iter().skip(1) {
                xs.push(unpack_expr(x, span, depth)?);
            }
            Ok(Expr::Or(xs.into_boxed_slice()))
        }
        ":let" => {
            let binds_pv = expect_seq(
                items
                    .get(1)
                    .ok_or_else(|| malformed(span, IMPORT_OP, "let missing binds"))?,
                IMPORT_OP,
                span,
            )?;
            let mut binds = Vec::new();
            for pair in binds_pv.iter() {
                let p = expect_seq(pair, IMPORT_OP, span)?;
                let slot = expect_u16(
                    p.first()
                        .ok_or_else(|| malformed(span, IMPORT_OP, "let bind"))?,
                    span,
                    "let bind",
                )?;
                let e = unpack_expr(p.get(1).ok_or_else(|| malformed(span, IMPORT_OP, "let bind"))?, span, depth)?;
                binds.push((slot, e));
            }
            Ok(Expr::Let {
                binds: binds.into_boxed_slice(),
                body: Box::new(unpack_expr(
                    items
                        .get(2)
                        .ok_or_else(|| malformed(span, IMPORT_OP, "let missing body"))?,
                    span, depth
                )?),
            })
        }
        ":match" => {
            let scrutinee = Box::new(unpack_expr(
                items
                    .get(1)
                    .ok_or_else(|| malformed(span, IMPORT_OP, "match missing scrut"))?,
                span, depth
            )?);
            let arms_pv = expect_seq(
                items
                    .get(2)
                    .ok_or_else(|| malformed(span, IMPORT_OP, "match missing arms"))?,
                IMPORT_OP,
                span,
            )?;
            let mut arms = Vec::new();
            for a in arms_pv.iter() {
                let p = expect_seq(a, IMPORT_OP, span)?;
                arms.push((
                    unpack_pat(p.first().ok_or_else(|| malformed(span, IMPORT_OP, "arm"))?, span, depth)?,
                    unpack_expr(p.get(1).ok_or_else(|| malformed(span, IMPORT_OP, "arm"))?, span, depth)?,
                ));
            }
            Ok(Expr::Match {
                scrutinee,
                arms: arms.into_boxed_slice(),
            })
        }
        other => Err(malformed(span, IMPORT_OP, format!("unknown expr {other}"))),
    }
}

/// `Program` → `[:prog frame_len [params…] [names…] [reads…] root]`.
///
/// `names` is the debug-name table and is positionally aligned with slots, so an unnamed slot
/// must occupy its position: it packs as `Value::Unit`, not as an omission. Dropping unnamed
/// entries would shift every later name onto the wrong slot.
fn pack_prog(p: &Program) -> Value {
    let names = p.names.iter().map(|n| match n {
        Some(s) => Value::String(Arc::new(s.to_string())),
        None => Value::Unit,
    });
    let reads = p.reads.iter().map(|(k, s)| pv([k.clone(), Value::i64(*s as i64)]));
    let params = p.params.iter().map(|s| Value::i64(*s as i64));
    pv([
        kw(":prog"),
        Value::i64(p.frame_len as i64),
        pv(params),
        pv(names),
        pv(reads),
        pack_expr(&p.root),
    ])
}

/// Inverse of `pack_prog`, and the caller of wall 2 for programs: the slot bounds cannot be
/// checked until `frame_len` and the root expression have both been read, so
/// `check_program_slots` runs at the end of this function rather than inside the reads.
fn unpack_prog(v: &Value, span: &Span, depth: u32) -> Result<Program, EvalBreak> {
    let depth = deeper(depth, span)?;
    let items = expect_seq(v, IMPORT_OP, span)?;
    if expect_kw(
        items
            .first()
            .ok_or_else(|| malformed(span, IMPORT_OP, "empty prog"))?,
        IMPORT_OP,
        span,
    )? != ":prog"
    {
        return Err(malformed(span, IMPORT_OP, "expected :prog"));
    }
    let frame_len = expect_u16(
        items
            .get(1)
            .ok_or_else(|| malformed(span, IMPORT_OP, "prog frame"))?,
        span,
        "prog frame",
    )?;
    let params_pv = expect_seq(
        items
            .get(2)
            .ok_or_else(|| malformed(span, IMPORT_OP, "prog params"))?,
        IMPORT_OP,
        span,
    )?;
    let mut params = Vec::new();
    for x in params_pv.iter() {
        params.push(expect_u16(x, span, "prog param")?);
    }
    let names_pv = expect_seq(
        items
            .get(3)
            .ok_or_else(|| malformed(span, IMPORT_OP, "prog names"))?, IMPORT_OP, span)?;
    let mut names = Vec::new();
    for x in names_pv.iter() {
        names.push(match x {
            Value::String(s) => Some(Arc::<str>::from(s.as_str())),
            _ => None,
        });
    }
    let reads_pv = expect_seq(
        items
            .get(4)
            .ok_or_else(|| malformed(span, IMPORT_OP, "prog reads"))?,
        IMPORT_OP,
        span,
    )?;
    let mut reads = Vec::new();
    for x in reads_pv.iter() {
        let p = expect_seq(x, IMPORT_OP, span)?;
        reads.push((
            p.first()
                .ok_or_else(|| malformed(span, IMPORT_OP, "read key"))?
                .clone(),
            expect_u16(
                p.get(1)
                    .ok_or_else(|| malformed(span, IMPORT_OP, "read slot"))?,
                span,
                "read slot",
            )?,
        ));
    }
    let root = unpack_expr(
        items
            .get(5)
            .ok_or_else(|| malformed(span, IMPORT_OP, "prog root"))?,
        span, depth
    )?;
    let program = Program {
        frame_len,
        root,
        reads: reads.into(),
        params: params.into_boxed_slice(),
        names: names.into_boxed_slice(),
        span: span.clone(),
    };
    check_program_slots(&program, span)?;
    Ok(program)
}

/// `Op` → `[:bind …]` `[:bchk …]` `[:eval …]` `[:cmp …]` `[:scmp …]` `[:fail]`.
///
/// `Or` and `Not` nest whole op-vectors as their operands, so a condition's shape survives as a
/// tree rather than being flattened into a jump-encoded sequence — which is what lets
/// `check_cond_ops` recurse into branches with the enclosing `n_slots` intact.
fn pack_cond_op(op: &Op) -> Value {
    match op {
        Op::Bind { field_idx, slot } => pv([
            kw(":bind"),
            Value::i64(*field_idx as i64),
            Value::i64(*slot as i64),
        ]),
        Op::BindCheck { field_idx, slot } => pv([
            kw(":bchk"),
            Value::i64(*field_idx as i64),
            Value::i64(*slot as i64),
        ]),
        Op::Eval { expr, slot } => {
            pv([kw(":eval"), pack_expr(expr), Value::i64(*slot as i64)])
        }
        Op::Cmp { op, lhs, rhs } => {
            pv([kw(":cmp"), pack_cmp(*op), pack_expr(lhs), pack_expr(rhs)])
        }
        Op::SeedCmp { op, lhs, rhs } => {
            pv([kw(":scmp"), pack_cmp(*op), pack_expr(lhs), pack_expr(rhs)])
        }
        Op::Or(branches) => {
            let mut xs = vec![kw(":or-c")];
            xs.extend(branches.iter().map(|b| pv(b.iter().map(pack_cond_op))));
            pv(xs)
        }
        Op::Not(inner) => {
            let mut xs = vec![kw(":not-c")];
            xs.extend(inner.iter().map(pack_cond_op));
            pv(xs)
        }
        Op::Fail => pv([kw(":fail")]),
    }
}

/// Inverse of `pack_cond_op`. Every slot-bearing arm reads through `expect_idx` (wall 1,
/// no wrap-into-range); the resulting op is bounds-checked as a set by `check_cond_ops`
/// (wall 2) once `n_slots` is known — one read cannot see the frame it will run in.
fn unpack_cond_op(v: &Value, span: &Span, depth: u32) -> Result<Op, EvalBreak> {
    let depth = deeper(depth, span)?;
    let items = expect_seq(v, IMPORT_OP, span)?;
    match expect_kw(
        items
            .first()
            .ok_or_else(|| malformed(span, IMPORT_OP, "empty cond-op"))?,
        IMPORT_OP,
        span,
    )? {
        ":bind" => Ok(Op::Bind {
            field_idx: expect_idx(expect_at(&items, 1, span, "bind field_idx")?, span, "bind field_idx")?,
            slot: expect_idx(expect_at(&items, 2, span, "bind slot")?, span, "bind slot")?,
        }),
        ":bchk" => Ok(Op::BindCheck {
            field_idx: expect_idx(expect_at(&items, 1, span, "bchk field_idx")?, span, "bchk field_idx")?,
            slot: expect_idx(expect_at(&items, 2, span, "bchk slot")?, span, "bchk slot")?,
        }),
        ":eval" => Ok(Op::Eval {
            expr: unpack_expr(expect_at(&items, 1, span, "eval expr")?, span, depth)?,
            slot: expect_idx(expect_at(&items, 2, span, "eval slot")?, span, "eval slot")?,
        }),
        ":cmp" => Ok(Op::Cmp {
            op: unpack_cmp(expect_at(&items, 1, span, "cmp op")?, span)?,
            lhs: unpack_expr(expect_at(&items, 2, span, "cmp lhs")?, span, depth)?,
            rhs: unpack_expr(expect_at(&items, 3, span, "cmp rhs")?, span, depth)?,
        }),
        ":scmp" => Ok(Op::SeedCmp {
            op: unpack_cmp(expect_at(&items, 1, span, "scmp op")?, span)?,
            lhs: unpack_expr(expect_at(&items, 2, span, "scmp lhs")?, span, depth)?,
            rhs: unpack_expr(expect_at(&items, 3, span, "scmp rhs")?, span, depth)?,
        }),
        ":or-c" => {
            let mut branches = Vec::new();
            for b in items.iter().skip(1) {
                let bp = expect_seq(b, IMPORT_OP, span)?;
                let mut ops = Vec::new();
                for x in bp.iter() {
                    ops.push(unpack_cond_op(x, span, depth)?);
                }
                branches.push(ops);
            }
            Ok(Op::Or(branches))
        }
        ":not-c" => {
            let mut inner = Vec::new();
            for x in items.iter().skip(1) {
                inner.push(unpack_cond_op(x, span, depth)?);
            }
            Ok(Op::Not(inner))
        }
        ":fail" => Ok(Op::Fail),
        other => Err(malformed(span, IMPORT_OP, format!("unknown cond-op {other}"))),
    }
}

/// `CompiledCond` → `[:cond n_slots fact_bind [keys…] [out_slots…] [seed_reads…] [ops…]]`.
///
/// `fact_bind` is optional and sits at a FIXED position, so absence is `Value::Unit` rather than
/// a shorter vector (contrast `pack_pat`'s `:pvar`, where absence is arity). Everything after it
/// is a homogeneous sequence, which is why they can be read back without per-item tags.
fn pack_compiled_cond(c: &CompiledCond) -> Value {
    let ops = c.ops().iter().map(pack_cond_op);
    let keys = c.slot_keys().iter().cloned();
    let slots = c.output_slots().iter().map(|s| Value::i64(*s as i64));
    let seeds = c
        .seed_reads()
        .iter()
        .map(|(k, s)| pv([k.clone(), Value::i64(*s as i64)]));
    let bind = match c.fact_bind() {
        Some(v) => v.clone(),
        None => Value::Unit,
    };
    pv([
        kw(":cond"),
        Value::i64(c.n_slots() as i64),
        bind,
        pv(keys),
        pv(slots),
        pv(seeds),
        pv(ops),
    ])
}

/// Inverse of `pack_compiled_cond`. `n_slots` is read FIRST because it is the frame every
/// later field is validated against — the ops cannot be bounds-checked before it is known, and
/// reading it late would mean holding an unvalidated op list in hand.
fn unpack_compiled_cond(v: &Value, span: &Span) -> Result<CompiledCond, EvalBreak> {
    let items = expect_seq(v, IMPORT_OP, span)?;
    if expect_kw(expect_at(&items, 0, span, ":cond tag")?, IMPORT_OP, span)? != ":cond" {
        return Err(malformed(span, IMPORT_OP, "expected :cond"));
    }
    let n_slots = expect_idx(expect_at(&items, 1, span, "n_slots")?, span, "n_slots")?;
    let fact_bind = match expect_at(&items, 2, span, "fact_bind")? {
        Value::String(_) => Some(items[2].clone()),
        _ => None,
    };
    let keys_pv = expect_seq(expect_at(&items, 3, span, "slot_keys")?, IMPORT_OP, span)?;
    let slot_keys: Arc<[Value]> = keys_pv.into();
    let slots_pv = expect_seq(expect_at(&items, 4, span, "output_slots")?, IMPORT_OP, span)?;
    let output_slots: Arc<[usize]> = slots_pv
        .iter()
        .map(|x| expect_idx(x, span, "output slot"))
        .collect::<Result<Vec<_>, _>>()?
        .into();
    let seeds_pv = expect_seq(expect_at(&items, 5, span, "seed_reads")?, IMPORT_OP, span)?;
    let mut seed_reads = Vec::new();
    for x in seeds_pv.iter() {
        let p = expect_seq(x, IMPORT_OP, span)?;
        seed_reads.push((
            expect_at(&p, 0, span, "seed key")?.clone(),
            expect_idx(expect_at(&p, 1, span, "seed slot")?, span, "seed slot")?,
        ));
    }
    let ops_pv = expect_seq(expect_at(&items, 6, span, "ops")?, IMPORT_OP, span)?;
    let mut ops = Vec::new();
    for x in ops_pv.iter() {
        // Top-level entry: a compiled cond is not reachable from inside the descent, so it
        // opens a fresh budget rather than continuing one.
        ops.push(unpack_cond_op(x, span, 0)?);
    }
    for s in output_slots.iter() {
        if *s >= n_slots {
            return Err(malformed(
                span,
                IMPORT_OP,
                format!("output slot {s} >= n_slots {n_slots}"),
            ));
        }
    }
    for (_, s) in seed_reads.iter() {
        if *s >= n_slots {
            return Err(malformed(
                span,
                IMPORT_OP,
                format!("seed slot {s} >= n_slots {n_slots}"),
            ));
        }
    }
    if slot_keys.len() != output_slots.len() {
        return Err(malformed(
            span,
            IMPORT_OP,
            format!(
                "slot_keys length {} != output_slots length {}",
                slot_keys.len(),
                output_slots.len()
            ),
        ));
    }
    check_cond_ops(&ops, n_slots, span)?;
    // An imported program carries no source. The span is the IMPORT's — honest, and the same
    // answer `rules_lack_ast` already gives elsewhere — and slot NAMES are diagnostics that were
    // never on the wire, so an `Op::Eval` raise in an imported cond reports a slot index rather
    // than a name. Deliberate: putting names on the wire would grow the ABI for a message.
    Ok(CompiledCond::from_parts(
        ops,
        slot_keys,
        output_slots,
        n_slots,
        seed_reads.into(),
        fact_bind,
        span.clone(),
        Box::from([]),
    ))
}

/// `CondDriver` → `[:leaf id]` · `[:where prog]` · `[:not d]` · `[:exists d]` · `[:and d…]` ·
/// `[:or d…]`.
///
/// Six arms and no accumulator among them — folds are a SEPARATE side table (`pack_fold`, also
/// keyed by node id; see `eval_export`). `:leaf` carries a bare node id rather than the node, so
/// a driver tree points INTO the network instead of embedding a second copy of it.
fn pack_driver(d: &CondDriver) -> Value {
    match d {
        CondDriver::Leaf(id) => pv([kw(":leaf"), Value::i64(*id)]),
        CondDriver::And(ks) => {
            let mut xs = vec![kw(":and")];
            xs.extend(ks.iter().map(pack_driver));
            pv(xs)
        }
        CondDriver::Or(ks) => {
            let mut xs = vec![kw(":or")];
            xs.extend(ks.iter().map(pack_driver));
            pv(xs)
        }
        CondDriver::Not(inner) => pv([kw(":not"), pack_driver(inner)]),
        CondDriver::Exists(inner) => pv([kw(":exists"), pack_driver(inner)]),
        CondDriver::Where(p) => pv([kw(":where"), pack_prog(p)]),
    }
}

/// Inverse of `pack_driver`. The composite arms (`:and`, `:or`, `:not`, `:exists`) recurse, and
/// `:where` re-enters `unpack_prog`, so this is a second unbounded tower at the import door
/// independent of `unpack_expr`'s.
///
/// ⚠ This doc used to read *"a driver tree of any depth round-trips WITHOUT A DEPTH PARAMETER —
/// the wire's nesting IS the recursion"*, which stated wall 5's absence as a feature. It was
/// true and it was the defect: the wire's nesting being the recursion is precisely how an
/// attacker picks this process's stack depth. It now carries the shared budget (`deeper`).
fn unpack_driver(v: &Value, span: &Span, depth: u32) -> Result<CondDriver, EvalBreak> {
    let depth = deeper(depth, span)?;
    let items = expect_seq(v, IMPORT_OP, span)?;
    match expect_kw(expect_at(&items, 0, span, "driver tag")?, IMPORT_OP, span)? {
        ":leaf" => Ok(CondDriver::Leaf(expect_i64(
            expect_at(&items, 1, span, "leaf id")?,
            IMPORT_OP,
            span,
        )?)),
        ":and" => {
            let mut ks = Vec::new();
            for x in items.iter().skip(1) {
                ks.push(unpack_driver(x, span, depth)?);
            }
            Ok(CondDriver::And(ks))
        }
        ":or" => {
            let mut ks = Vec::new();
            for x in items.iter().skip(1) {
                ks.push(unpack_driver(x, span, depth)?);
            }
            Ok(CondDriver::Or(ks))
        }
        ":not" => Ok(CondDriver::Not(Box::new(unpack_driver(
            expect_at(&items, 1, span, "not inner")?,
            span, depth
        )?))),
        ":exists" => Ok(CondDriver::Exists(Box::new(unpack_driver(
            expect_at(&items, 1, span, "exists inner")?,
            span, depth
        )?))),
        ":where" => Ok(CondDriver::Where(Arc::new(unpack_prog(
            expect_at(&items, 1, span, "where program")?,
            span, depth
        )?))),
        other => Err(malformed(span, IMPORT_OP, format!("unknown driver {other}"))),
    }
}

/// `AccFold` → `[:count]` `[:sum …]` `[:min …]` `[:max …]` `[:mean …]` `[:all …]`
/// `[:distinct …]` `[:group …]` `[:ufold …]`.
///
/// `:ufold` is the user-defined arm and carries a packed `Program`; the rest are built-ins whose
/// operands are slots and keys. Splitting them this way is what keeps a user fold from needing a
/// distinct top-level tag on the wire.
fn pack_fold(f: &AccFold) -> Value {
    match f {
        AccFold::Count => pv([kw(":count")]),
        AccFold::Sum(k) => pv([kw(":sum"), k.clone()]),
        AccFold::Min(k) => pv([kw(":min"), k.clone()]),
        AccFold::Max(k) => pv([kw(":max"), k.clone()]),
        AccFold::Mean(k) => pv([kw(":mean"), k.clone()]),
        AccFold::Distinct(k) => pv([kw(":distinct"), k.clone()]),
        AccFold::All => pv([kw(":all")]),
        AccFold::GroupBy(k) => pv([kw(":group"), k.clone()]),
        AccFold::User { var, program } => pv([kw(":ufold"), var.clone(), pack_prog(program)]),
    }
}

/// Inverse of `pack_fold`. An unknown fold tag is a refusal, not a fallback to `:count` — a
/// silently-substituted fold would produce a plausible number from the wrong aggregation, which
/// is the single worst outcome this codec can produce.
fn unpack_fold(v: &Value, span: &Span) -> Result<AccFold, EvalBreak> {
    let items = expect_seq(v, IMPORT_OP, span)?;
    match expect_kw(expect_at(&items, 0, span, "fold tag")?, IMPORT_OP, span)? {
        ":count" => Ok(AccFold::Count),
        ":sum" => Ok(AccFold::Sum(expect_at(&items, 1, span, "sum key")?.clone())),
        ":min" => Ok(AccFold::Min(expect_at(&items, 1, span, "min key")?.clone())),
        ":max" => Ok(AccFold::Max(expect_at(&items, 1, span, "max key")?.clone())),
        ":mean" => Ok(AccFold::Mean(expect_at(&items, 1, span, "mean key")?.clone())),
        ":distinct" => Ok(AccFold::Distinct(expect_at(&items, 1, span, "distinct key")?.clone())),
        ":all" => Ok(AccFold::All),
        ":group" => Ok(AccFold::GroupBy(expect_at(&items, 1, span, "group key")?.clone())),
        ":ufold" => Ok(AccFold::User {
            var: expect_at(&items, 1, span, "ufold var")?.clone(),
            // Top-level entry — fresh budget (a fold is not reached from inside the descent).
            program: Arc::new(unpack_prog(expect_at(&items, 2, span, "ufold program")?, span, 0)?),
        }),
        other => Err(malformed(span, IMPORT_OP, format!("unknown fold {other}"))),
    }
}

/// `RhsOp` → `[:rbind key]` · `[:rlit v]` · `[:rexpr prog]`.
///
/// ⚠ **One of four places the codec is deliberately lossy** (module header lists them), and this
/// one is not a defect: the
/// second field of `RhsOp::Bind` is a `Debug` rendering of the original `WatAST`, kept only to
/// name the form in a fire-time unbound-variable error. It is SOURCE, not residual — the
/// imported program does not need it to run, and the source it renders does not exist on the
/// importing disk. `unpack_rhs_op` reconstructs a usable stand-in from the key. Round-trip here
/// is semantic, not literal.
fn pack_rhs_op(op: &RhsOp) -> Value {
    match op {
        // Slot name only. The second Bind field is a Debug rendering of
        // WatAST for fire-time unbound errors — source, not residual.
        RhsOp::Bind(k, _, _) => pv([kw(":rbind"), k.clone()]),
        RhsOp::Lit(v) => pv([kw(":rlit"), v.clone()]),
        RhsOp::Expr(p) => pv([kw(":rexpr"), pack_prog(p)]),
    }
}

/// Inverse of `pack_rhs_op`, and the place the codec's one lossy field is made good.
///
/// `:rbind`'s dropped `Debug` rendering is reconstructed here: from a third element if some
/// future writer supplies one, otherwise from the key itself. And the span is restamped from the
/// IMPORT site rather than faked — an imported rule's original source is not on this disk, so
/// pointing an error at where it was imported is the only location that is true.
fn unpack_rhs_op(v: &Value, span: &Span) -> Result<RhsOp, EvalBreak> {
    let items = expect_seq(v, IMPORT_OP, span)?;
    match expect_kw(expect_at(&items, 0, span, "rhs-op tag")?, IMPORT_OP, span)? {
        ":rbind" => {
            let k = items
                .get(1)
                .ok_or_else(|| malformed(span, IMPORT_OP, "rbind missing key"))?
                .clone();
            let dbg = match items.get(2) {
                Some(v) => expect_str(v, IMPORT_OP, span)?.to_string(),
                None => match &k {
                    Value::String(s) => s.as_ref().clone(),
                    _ => String::new(),
                },
            };
            // Restamped from the import site — the truthful location for an
            // imported rule, whose original source is not on this disk.
            Ok(RhsOp::Bind(k, dbg, span.clone()))
        }
        ":rlit" => Ok(RhsOp::Lit(expect_at(&items, 1, span, "rlit value")?.clone())),
        ":rexpr" => Ok(RhsOp::Expr(Arc::new(unpack_prog(
            expect_at(&items, 1, span, "rexpr prog")?,
            span, 0, // top-level entry — fresh budget
        )?))),
        other => Err(malformed(span, IMPORT_OP, format!("unknown rhs-op {other}"))),
    }
}

/// `CompiledRhs` → `[:rec "Class" [names…] op…]` · `[:rcall prog]`.
///
/// The record arm splices its ops as a FLAT tail rather than nesting them in a sub-vector, so
/// the class and its field names stay at fixed indices 1 and 2 and the ops are simply
/// "everything from 3 on".
fn pack_rhs(r: &CompiledRhs) -> Value {
    match r {
        CompiledRhs::Record { class, names, ops } => {
            let mut xs = vec![
                kw(":rec"),
                Value::String(Arc::new(class.to_string())),
                pv(names.iter().map(|n| Value::String(Arc::new(n.clone())))),
            ];
            xs.extend(ops.iter().map(pack_rhs_op));
            pv(xs)
        }
        CompiledRhs::Call(p) => pv([kw(":rcall"), pack_prog(p)]),
    }
}

/// Inverse of `pack_rhs`. The `:rec` arm reads its ops from index 3 to the end — the flat-tail
/// shape `pack_rhs` chose — so a short read cannot run past the end of the vector. A truncated
/// tail is then REFUSED, not accepted: the names/ops length check below raises
/// `rhs names length {} != ops length {}`. (An earlier version of this doc said a truncated
/// vector "yields a record with fewer ops"; it does not, and the check is fifteen lines down.)
fn unpack_rhs(v: &Value, span: &Span) -> Result<CompiledRhs, EvalBreak> {
    let items = expect_seq(v, IMPORT_OP, span)?;
    match expect_kw(expect_at(&items, 0, span, "tag")?, IMPORT_OP, span)? {
        ":rec" => {
            let class: Arc<str> = expect_str(expect_at(&items, 1, span, "slot 1")?, IMPORT_OP, span)?.into();
            let names_pv = expect_seq(expect_at(&items, 2, span, "slot 2")?, IMPORT_OP, span)?;
            let mut ns = Vec::new();
            for n in names_pv.iter() {
                ns.push(expect_str(n, IMPORT_OP, span)?.to_string());
            }
            let mut ops = Vec::new();
            for x in items.iter().skip(3) {
                ops.push(unpack_rhs_op(x, span)?);
            }
            if ns.len() != ops.len() {
                return Err(malformed(
                    span,
                    IMPORT_OP,
                    format!("rhs names length {} != ops length {}", ns.len(), ops.len()),
                ));
            }
            Ok(CompiledRhs::Record {
                class,
                names: Arc::new(ns),
                ops,
            })
        }
        // Top-level entry — fresh budget.
        ":rcall" => Ok(CompiledRhs::Call(Arc::new(unpack_prog(expect_at(&items, 1, span, "slot 1")?, span, 0)?))),
        other => Err(malformed(span, IMPORT_OP, format!("unknown rhs {other}"))),
    }
}

// ── topology ─────────────────────────────────────────────────────────────────

type ClassFields = Vec<Vec<String>>;

struct ClassIntern {
    names: Vec<String>,
    fields: ClassFields,
    idx: HashMap<String, usize>,
}

impl ClassIntern {
    fn new() -> Self {
        ClassIntern {
            names: Vec::new(),
            fields: Vec::new(),
            idx: HashMap::new(),
        }
    }
    fn intern(&mut self, class: &str, field_names: Vec<String>) -> i64 {
        if let Some(i) = self.idx.get(class) {
            return *i as i64;
        }
        let i = self.names.len();
        self.idx.insert(class.to_string(), i);
        self.names.push(class.to_string());
        self.fields.push(field_names);
        i as i64
    }
}

/// Child ids as a flat tail. Every node tag that has children ends with them, so the reader can
/// treat "everything past this kind's fixed fields" as the child list without a length prefix.
fn pack_children(node: &Value) -> impl Iterator<Item = Value> {
    node_children(node).into_iter().map(Value::i64)
}

/// One network node → a tagged vector, one tag per `NodeKind`:
/// `:a` alpha · `:j` root-join · `:h` hash-join · `:p` production · `:t` test · `:n` negation ·
/// `:e` exists · `:acc` accumulate · `:q` query.
///
/// Tags are terse because this is the highest-cardinality thing on the wire — one per node, for
/// every node in the network — and unlike the `Expr`/`Op` tags they are never read by a human
/// composing a form by hand.
///
/// The alpha arm is the only one that does real work rather than field copying: it recovers the
/// node's CLASS, first from the node's own condition (`alpha_pattern`) and, failing that, from
/// the alpha tree's class index. Both can miss, and `-1` is the honest "unknown" — it is written
/// deliberately rather than defaulted, and `unpack_node` reads it back as unknown rather than as
/// class zero. Classes are interned through `ClassIntern` so a network with many alphas over the
/// same type carries the field-name list once.
fn pack_node(
    node: &Value,
    classes: &mut ClassIntern,
    sym: &SymbolTable,
    tree: &AlphaTree,
) -> Value {
    let id = node_named_i64(node, "id").unwrap_or(-1);
    match kind_of(node) {
        NodeKind::Alpha => {
            let class_idx = alpha_cond_from_node(node)
                .and_then(|ast| alpha_pattern(&ast).map(|p| {
                    let ty = p.type_head.to_string();
                    let fs = class_field_names(sym, &ty);
                    classes.intern(&ty, fs)
                }))
                .or_else(|| {
                    tree.class_for_alpha(id).map(|ty| {
                        let fs = class_field_names(sym, ty);
                        classes.intern(ty, fs)
                    })
                })
                .unwrap_or(-1);
            let mut xs = vec![kw(":a"), Value::i64(id), Value::i64(class_idx)];
            xs.extend(pack_children(node));
            pv(xs)
        }
        NodeKind::RootJoin => {
            let mut xs = vec![kw(":j"), Value::i64(id)];
            xs.extend(pack_children(node));
            pv(xs)
        }
        NodeKind::HashJoin => {
            let mut xs = vec![kw(":h"), Value::i64(id)];
            xs.extend(pack_children(node));
            pv(xs)
        }
        NodeKind::Production => {
            let name = node_named_string(node, "rule-name")
                .unwrap_or("")
                .to_string();
            pv([kw(":p"), Value::i64(id), Value::String(Arc::new(name))])
        }
        NodeKind::Test => {
            let mut xs = vec![kw(":t"), Value::i64(id)];
            xs.extend(pack_children(node));
            pv(xs)
        }
        NodeKind::Negation => {
            let aid = node_named_i64(node, "negated-alpha-id").unwrap_or(-1);
            let mut xs = vec![kw(":n"), Value::i64(id), Value::i64(aid)];
            xs.extend(pack_children(node));
            pv(xs)
        }
        NodeKind::Exists => {
            let aid = node_named_i64(node, "exists-alpha-id").unwrap_or(-1);
            let mut xs = vec![kw(":e"), Value::i64(id), Value::i64(aid)];
            xs.extend(pack_children(node));
            pv(xs)
        }
        NodeKind::Accumulate => {
            let var = node_named_string(node, "result-var")
                .unwrap_or("")
                .to_string();
            let aid = node_named_i64(node, "from-alpha-id").unwrap_or(-1);
            let mut xs = vec![
                kw(":acc"),
                Value::i64(id),
                Value::String(Arc::new(var)),
                Value::i64(aid),
            ];
            xs.extend(pack_children(node));
            pv(xs)
        }
        NodeKind::Query => {
            let name = node_named_string(node, "query-name")
                .unwrap_or("")
                .to_string();
            let params = match node_named_field(node, "param-keys") {
                Some(Value::wat__core__PersistentVector(pv)) => pv.iter().cloned().collect(),
                _ => vec![],
            };
            let mut xs = vec![kw(":q"), Value::i64(id), Value::String(Arc::new(name))];
            xs.extend(params);
            pv(xs)
        }
    }
}

/// Read a flat tail of ids, dropping the `skip` fixed fields ahead of it — the read half of the
/// flat-tail shape `pack_children` writes.
fn unpack_i64s(items: &[Value], skip: usize, span: &Span) -> Result<Vec<i64>, EvalBreak> {
    let mut out = Vec::new();
    for x in items.iter().skip(skip) {
        out.push(expect_i64(x, IMPORT_OP, span)?);
    }
    Ok(out)
}

fn i64_pv(ids: &[i64]) -> Value {
    let mut v = crate::value::pvec::PVec::new();
    for id in ids {
        v.push_back_mut(Value::i64(*id));
    }
    Value::wat__core__PersistentVector(v)
}

type UnpackedNode = (i64, Value, Option<i64>);

/// Inverse of `pack_node`, returning `UnpackedNode` — `(id, node-record, class-index?)`.
///
/// It does NOT return a live node: it rebuilds the node RECORD, and leaves wiring the network to
/// `import_export`, which needs every id in hand before it can resolve any edge.
///
/// The third element is `Some` for exactly one tag, `:a` — it is the alpha's CLASS INDEX into
/// the export's interned class table, and every other kind returns `None` because no other kind
/// has one. This is where `pack_node`'s `-1`-means-unknown convention is converted back into an
/// `Option`: the sentinel exists only on the wire (which has no null), and it is turned into a
/// real absence at the first opportunity rather than being carried inward as a magic number.
fn unpack_node(v: &Value, span: &Span) -> Result<UnpackedNode, EvalBreak> {
    let items = expect_seq(v, IMPORT_OP, span)?;
    let tag = expect_kw(expect_at(&items, 0, span, "tag")?, IMPORT_OP, span)?;
    match tag {
        ":a" => {
            let id = expect_i64(expect_at(&items, 1, span, "slot 1")?, IMPORT_OP, span)?;
            let class_idx = expect_i64(expect_at(&items, 2, span, "slot 2")?, IMPORT_OP, span)?;
            let kids = unpack_i64s(&items, 3, span)?;
            let rec = record(
                "wat::rete::AlphaNode",
                ALPHA_FIELDS,
                vec![Value::i64(id), empty_pv(), i64_pv(&kids)],
            );
            let class = if class_idx >= 0 {
                Some(class_idx)
            } else {
                None
            };
            Ok((id, rec, class))
        }
        ":j" => {
            let id = expect_i64(expect_at(&items, 1, span, "slot 1")?, IMPORT_OP, span)?;
            let kids = unpack_i64s(&items, 2, span)?;
            let rec = record(
                "wat::rete::RootJoinNode",
                ROOT_FIELDS,
                vec![Value::i64(id), i64_pv(&kids)],
            );
            Ok((id, rec, None))
        }
        ":h" => {
            let id = expect_i64(expect_at(&items, 1, span, "slot 1")?, IMPORT_OP, span)?;
            let kids = unpack_i64s(&items, 2, span)?;
            let rec = record(
                "wat::rete::HashJoinNode",
                HASH_FIELDS,
                vec![Value::i64(id), i64_pv(&kids)],
            );
            Ok((id, rec, None))
        }
        ":p" => {
            let id = expect_i64(expect_at(&items, 1, span, "slot 1")?, IMPORT_OP, span)?;
            let name = expect_str(expect_at(&items, 2, span, "slot 2")?, IMPORT_OP, span)?.to_string();
            let rec = record(
                "wat::rete::ProductionNode",
                PROD_FIELDS,
                vec![Value::i64(id), Value::String(Arc::new(name))],
            );
            Ok((id, rec, None))
        }
        ":t" => {
            let id = expect_i64(expect_at(&items, 1, span, "slot 1")?, IMPORT_OP, span)?;
            let kids = unpack_i64s(&items, 2, span)?;
            let rec = record(
                "wat::rete::TestNode",
                TEST_FIELDS,
                vec![Value::i64(id), dummy_ast(span), i64_pv(&kids)],
            );
            Ok((id, rec, None))
        }
        ":n" => {
            let id = expect_i64(expect_at(&items, 1, span, "slot 1")?, IMPORT_OP, span)?;
            let aid = expect_i64(expect_at(&items, 2, span, "slot 2")?, IMPORT_OP, span)?;
            let kids = unpack_i64s(&items, 3, span)?;
            let rec = record(
                "wat::rete::NegationNode",
                NEG_FIELDS,
                vec![Value::i64(id), Value::i64(aid), i64_pv(&kids)],
            );
            Ok((id, rec, None))
        }
        ":e" => {
            let id = expect_i64(expect_at(&items, 1, span, "slot 1")?, IMPORT_OP, span)?;
            let aid = expect_i64(expect_at(&items, 2, span, "slot 2")?, IMPORT_OP, span)?;
            let kids = unpack_i64s(&items, 3, span)?;
            let rec = record(
                "wat::rete::ExistsNode",
                EXISTS_FIELDS,
                vec![Value::i64(id), Value::i64(aid), i64_pv(&kids)],
            );
            Ok((id, rec, None))
        }
        ":acc" => {
            let id = expect_i64(expect_at(&items, 1, span, "slot 1")?, IMPORT_OP, span)?;
            let var = expect_str(expect_at(&items, 2, span, "slot 2")?, IMPORT_OP, span)?.to_string();
            let aid = expect_i64(expect_at(&items, 3, span, "slot 3")?, IMPORT_OP, span)?;
            let kids = unpack_i64s(&items, 4, span)?;
            let rec = record(
                "wat::rete::AccumulateNode",
                ACC_FIELDS,
                vec![
                    Value::i64(id),
                    Value::String(Arc::new(var)),
                    dummy_ast(span),
                    Value::i64(aid),
                    i64_pv(&kids),
                ],
            );
            Ok((id, rec, None))
        }
        ":q" => {
            let id = expect_i64(expect_at(&items, 1, span, "slot 1")?, IMPORT_OP, span)?;
            let name = expect_str(expect_at(&items, 2, span, "slot 2")?, IMPORT_OP, span)?.to_string();
            let mut params = crate::value::pvec::PVec::new();
            for x in items.iter().skip(3) {
                params.push_back_mut(x.clone());
            }
            let rec = record(
                "wat::rete::QueryNode",
                QUERY_FIELDS,
                vec![
                    Value::i64(id),
                    Value::String(Arc::new(name)),
                    Value::wat__core__PersistentVector(params),
                ],
            );
            Ok((id, rec, None))
        }
        other => Err(malformed(span, IMPORT_OP, format!("unknown node {other}"))),
    }
}

// ── Session field readers ────────────────────────────────────────────────────

/// Pull `(network, rules)` off a Session, or raise `TypeMismatch` naming
/// `:wat::rete::Session`.
///
/// Both fields are fetched in one match so a value missing EITHER is refused with the same
/// message: from the caller's side "this is not a Session" is one fact, and reporting it as two
/// different partial failures would leak which field happened to be probed first.
fn session_network_rules<'a>(
    session: &'a Value,
    span: &Span,
) -> Result<(&'a Value, &'a Value), EvalBreak> {
    match (session_network(session), session_named_field(session, "rules")) {
        (Some(network), Some(rules)) => Ok((network, rules)),
        _ => Err(RuntimeError::new(
            span.clone(),
            RuntimeErrorKind::TypeMismatch {
                op: OP.into(),
                expected: ":wat::rete::Session",
                got: Box::new(ValueSnapshot::of(session)),
            },
        )
        .into()),
    }
}

/// Pack a node-id-keyed side table as `[[id value] …]`, **keys sorted**.
///
/// The sort is not cosmetic. `HashMap` iteration order is not stable across runs, so packing in
/// map order would make the same network export DIFFERENTLY each time. An export is a durable
/// artifact — `tests/rete/datamancer.rete.edn` is one, checked into the tree and regenerated by
/// a CLI invocation — and a map-ordered export would churn every line of it on every
/// regeneration, leaving a diff that cannot distinguish a real change from reshuffled bytes.
/// Sorting is what makes two exports of the same network comparable at all.
///
/// This shape is why the export is not one tree: the network is a flat node list, and conds,
/// drivers, wheres and folds hang off it in four parallel tables joined by node id. A node stays
/// small and the tables stay independently readable.
fn map_i64<V>(m: &HashMap<i64, V>, mut f: impl FnMut(&V) -> Value) -> Value {
    let mut keys: Vec<i64> = m.keys().copied().collect();
    keys.sort_unstable();
    let mut pairs = Vec::with_capacity(keys.len());
    for k in keys {
        pairs.push(pv([Value::i64(k), f(m.get(&k).expect("sorted key"))]));
    }
    pv(pairs)
}

/// `map_i64`'s twin for the one table keyed by RULE NAME rather than node id — the compiled RHS,
/// which belongs to a rule, not to a network node. Sorted for the same determinism reason.
fn map_str<V>(m: &HashMap<String, V>, mut f: impl FnMut(&V) -> Value) -> Value {
    let mut keys: Vec<&String> = m.keys().collect();
    keys.sort();
    let mut pairs = Vec::with_capacity(keys.len());
    for k in keys {
        pairs.push(pv([Value::String(Arc::new(k.clone())), f(m.get(k).expect("sorted key"))]));
    }
    pv(pairs)
}

// ── public mouths ────────────────────────────────────────────────────────────

/// `(:wat::rete::export <session>) -> :wat::rete::Export`
pub(crate) fn eval_export(
    args: &[WatAST],
    list_span: &Span,
    env: &crate::runtime::Environment,
    sym: &SymbolTable,
) -> Result<Value, EvalBreak> {
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
    let (network, rules) = session_network_rules(&session, list_span)?;
    // Pack door: MISS intern's (`DESIGN-STONE-intern-eviction`); HIT reuses the compile lease.
    let arm = rete_arm_get_or_build(network, rules, sym)?;
    let mut classes = ClassIntern::new();
    let mut nodes = Vec::new();
    for id in sorted_node_ids(network) {
        if let Some(node) = get_node(network, id) {
            nodes.push(pack_node(node, &mut classes, sym, &arm.alpha_tree));
        }
    }
    let conds = map_i64(&arm.compiled_conds, pack_compiled_cond);
    let drivers = map_i64(&arm.compiled_drivers, pack_driver);
    let progs = map_i64(&arm.compiled_wheres, pack_prog);
    let folds = map_i64(&arm.compiled_acc_folds, pack_fold);
    let rhs = map_str(&arm.compiled_rhs, |items| {
        pv(items.iter().map(pack_rhs))
    });
    // Residual stratify schedule lives on the interned arm, not Session.rules
    // (`wat/rete.wat` Export/deps). Import drops source forms; packing from
    // `rule_deps_from_rules(session.rules)` wrote empty deps on re-export.
    let deps = pack_deps(&arm.rule_deps);
    let abi = abi_of(&classes.names, &classes.fields);
    let class_pv = pv(classes
        .names
        .iter()
        .map(|c| Value::String(Arc::new(c.clone()))));
    let fields_pv = pv(classes.fields.iter().map(|fs| {
        pv(fs.iter().map(|f| Value::String(Arc::new(f.clone()))))
    }));
    Ok(Value::Aggregate(Arc::new(AggregateValue::record(
        "wat::rete::Export".into(),
        export_names(),
        Arc::new(vec![
            Value::i64(FORMAT_V),
            Value::String(Arc::new(abi)),
            class_pv,
            fields_pv,
            pv(nodes),
            conds,
            drivers,
            progs,
            folds,
            rhs,
            deps,
        ]),
    ))))
}

/// The residual stratification schedule — packed from the INTERNED ARM, not from
/// `Session.rules`.
///
/// The distinction is load-bearing and was a real defect: import drops source forms, so a
/// re-export that recomputed deps from `rule_deps_from_rules(session.rules)` found nothing to
/// read and wrote EMPTY deps. The schedule survives a round-trip only because it is taken from
/// the arm, which import rebuilds. (`wat/rete.wat`, `Export`/`deps`.)
fn pack_deps(deps: &[RuleDep]) -> Value {
    pv(deps.iter().map(|d| {
        pv([
            Value::String(Arc::new(d.name.clone())),
            pv(d.view.produced.iter().map(|s| Value::String(Arc::new(s.clone())))),
            pv(d.view.negated.iter().map(|s| Value::String(Arc::new(s.clone())))),
            pv(d.view.consumed.iter().map(|s| Value::String(Arc::new(s.clone())))),
            pv(d.view
                .exists_and_from_types
                .iter()
                .map(|s| Value::String(Arc::new(s.clone())))),
        ])
    }))
}

/// A vector of strings, each element type-checked rather than stringified — an `i64` in a
/// string list is a refusal, not a coercion.
fn unpack_string_list(v: &Value, span: &Span) -> Result<Vec<String>, EvalBreak> {
    let xs = expect_seq(v, IMPORT_OP, span)?;
    let mut out = Vec::new();
    for x in xs {
        out.push(expect_str(&x, IMPORT_OP, span)?.to_string());
    }
    Ok(out)
}

/// Inverse of `pack_deps`. See `pack_deps` for why this schedule travels on the wire at all
/// rather than being recomputed on the importing side.
fn unpack_deps(v: &Value, span: &Span) -> Result<Vec<RuleDep>, EvalBreak> {
    let mut out = Vec::new();
    for row in expect_seq(v, IMPORT_OP, span)? {
        let p = expect_seq(&row, IMPORT_OP, span)?;
        if p.len() < 4 {
            return Err(malformed(span, IMPORT_OP, "deps row needs name + 4 lists (5th optional for old rows)"));
        }
        let bag = if p.len() >= 5 {
            unpack_string_list(&p[4], span)?
        } else {
            Vec::new()
        };
        out.push(RuleDep {
            name: expect_str(&p[0], IMPORT_OP, span)?.to_string(),
            view: crate::rete::kernel::StratifyView {
                produced: unpack_string_list(&p[1], span)?,
                negated: unpack_string_list(&p[2], span)?,
                consumed: unpack_string_list(&p[3], span)?,
                exists_and_from_types: bag,
            },
        });
    }
    Ok(out)
}

/// `(:wat::rete::import <export>) -> :wat::rete::Session`
///
/// Slim topology, interned arm, empty facts. Fire does not lower.
/// Stratify schedule is `:deps` (produced / negated / consumed / exists-and-from class names).
pub(crate) fn eval_import(
    args: &[WatAST],
    list_span: &Span,
    env: &crate::runtime::Environment,
    sym: &SymbolTable,
) -> Result<Value, EvalBreak> {
    if args.len() != 1 {
        return Err(RuntimeError::new(
            list_span.clone(),
            RuntimeErrorKind::ArityMismatch {
                op: IMPORT_OP.into(),
                expected: 1,
                got: args.len(),
            },
        )
        .into());
    }
    let export = crate::runtime::eval_inner(&args[0], env, sym)?.value_owned();
    import_export(&export, list_span, sym)
}

/// WALL 4 — the imported graph must be one the fire passes can legally WALK.
///
/// The other three walls (module header, law 3) are each about a VALUE: a range at the read, a
/// slot inside its own frame, a compat fingerprint. This one is about the SHAPE OF THE GRAPH. It
/// runs over the already-unpacked node map, so it reads no bytes, changes no wire format, and
/// costs no version bump — and it proves three things:
///
/// 1. every child id names a node in this import;
/// 2. every reference-field alpha id (Negation `negated-alpha-id`, Exists `exists-alpha-id`,
///    Accumulate `from-alpha-id`) names a node in this import **whose kind is `Alpha`**;
/// 3. every child id EXCEEDS its parent's. `kernel/node.rs` (`sorted_node_ids`) and
///    `kernel/arm.rs` both state that ascending node id IS the topological order the
///    alpha / root-join / hash-join passes require. On the compile path that holds because ids
///    are minted increasing; on the wire path nothing minted anything.
///
/// **It REFUSES; it never repairs.** No dangling edge is dropped, no graph re-sorted, no missing
/// node synthesised. A repaired import's output would depend on the damage rather than on the
/// input, which is exactly the property that makes a wall a wall.
fn check_node_graph(network: &Value, span: &Span) -> Result<(), EvalBreak> {
    let ids = sorted_node_ids(network);
    let known: HashSet<i64> = ids.iter().copied().collect();
    for id in ids {
        let Some(node) = get_node(network, id) else {
            return Err(malformed(
                span,
                IMPORT_OP,
                format!("node graph: id {id} keys the network but names no node"),
            ));
        };
        for kid in node_children(node) {
            if !known.contains(&kid) {
                return Err(malformed(
                    span,
                    IMPORT_OP,
                    format!(
                        "node graph: node {id} has a child edge to {kid}, \
                         which names no node in this import"
                    ),
                ));
            }
            if kid <= id {
                return Err(malformed(
                    span,
                    IMPORT_OP,
                    format!(
                        "node graph: node {id} has a child edge to {kid}, but a child id must \
                         exceed its parent's — ascending node id is the topological order the \
                         alpha / root-join / hash-join passes require"
                    ),
                ));
            }
        }
        if let Some(aid) = node_ref_alpha_id(node) {
            let kind = kind_of(node);
            match get_node(network, aid) {
                None => {
                    return Err(malformed(
                        span,
                        IMPORT_OP,
                        format!(
                            "node graph: {kind:?} node {id} references alpha id {aid}, \
                             which names no node in this import"
                        ),
                    ))
                }
                Some(target) => {
                    let target_kind = kind_of(target);
                    if target_kind != NodeKind::Alpha {
                        return Err(malformed(
                            span,
                            IMPORT_OP,
                            format!(
                                "node graph: {kind:?} node {id} references alpha id {aid}, \
                                 whose kind is {target_kind:?}, not Alpha"
                            ),
                        ));
                    }
                }
            }
        }
    }
    Ok(())
}

/// The whole import, in phases — the inverse of `eval_export` and the file's one place where
/// untrusted bytes become a runnable network.
///
/// 1. **Type gate** — the value is a `wat::rete::Export` aggregate, or `TypeMismatch`.
/// 2. **Three compat gates, in order** (module header, law 3): format version, then the ABI
///    fingerprint recomputed from the export's own classes/fields, then the HOST `TypeEnv`
///    field order. The third catches what the second structurally cannot — an export that is
///    internally consistent but describes records this process declares differently.
/// 3. **The node cap** (module header, law 3, wall six) — `MAX_IMPORT_NODES`, against the
///    DECLARED length of the `nodes` vector, before anything is unpacked. Phase 4's build is
///    quadratic in this count; the constant carries the curve.
/// 4. **Nodes** — `unpack_node` per entry into a network `PMap`, with each alpha's class index
///    resolved back to a class NAME through the export's interned table.
/// 5. **The graph wall** (module header, law 3, wall four) — `check_node_graph` proves the
///    assembled node map is one the fire passes can legally walk: every child id names a node,
///    every reference-field alpha id names a node that is an `Alpha`, and every child id exceeds
///    its parent's. It runs BEFORE the side tables, because everything after it assumes an edge
///    can be followed and that ascending id is topological. It refuses; it never repairs.
/// 6. **The five side tables** — conds, drivers, progs, folds (node-id keyed) and rhs (rule-name
///    keyed), each re-read into a `HashMap`.
/// 7. **Derived structure is REBUILT, never transported** — `NetworkEdges`, `AlphaTree`,
///    `WhereTree`, `kind_ids`, `joins_fed_by`, `compiled_max_slots` are all recomputed here from
///    the data above. This is why the wire format carries none of them: an index is a function of
///    the network, so shipping one would create a second thing that can disagree with the first,
///    and an importer that trusted it could be handed a stale index over a valid network.
/// 8. **Intern and return** — the assembled `InternedNetwork` is registered via
///    `rete_arm_intern` so the imported program fires through the same arm path a locally
///    compiled one does.
/// 9. **Charge the session this door just opened** (module header, law 4) — the byte origin
///    captured as this function's FIRST statement is filed under the new network's identity, and
///    the session ceiling is then read against it. This is why the reading and the filing are
///    separate calls: the key does not exist until phase 4 has run, and a reading taken here would
///    exclude every phase above.
///
/// Its 194 lines are phase COUNT rather than depth — brace nesting peaks at 3 inside the body,
/// and every level of it is a `for` over one table's pairs. Splitting it would put six one-caller
/// helpers between a gate and the gate that must follow it, which is the ordering the phase list
/// above exists to make legible.
fn import_export(export: &Value, span: &Span, sym: &SymbolTable) -> Result<Value, EvalBreak> {
    // ★ THE SESSION'S ZERO POINT, CAPTURED BEFORE IT CAN BE FILED. See phase 8 below and
    // `alloc_counter::mark_session_origin_at` for why the reading and the filing are split: the
    // key is the built network's identity, which does not exist yet, and reading `thread_bytes()`
    // at the moment the key appears would exclude the entire build from the session it created.
    // This is the FIRST statement at the door on purpose — everything the import allocates, from
    // the compat gates onward, belongs to the session it is about to hand back.
    let session_origin = crate::alloc_counter::thread_bytes();
    let agg = match export {
        Value::Aggregate(a) if a.nature != Nature::Struct && a.class.as_ref() == "wat::rete::Export" => a,
        other => {
            return Err(RuntimeError::new(
                span.clone(),
                RuntimeErrorKind::TypeMismatch {
                    op: IMPORT_OP.into(),
                    expected: ":wat::rete::Export",
                    got: Box::new(ValueSnapshot::of(other)),
                },
            )
            .into());
        }
    };
    let _ = agg;
    let v = expect_i64(export_named(export, "v", span)?, IMPORT_OP, span)?;
    if v != FORMAT_V {
        return Err(malformed(
            span,
            IMPORT_OP,
            format!("unsupported Export version {v}"),
        ));
    }
    let stored_abi = expect_str(export_named(export, "abi", span)?, IMPORT_OP, span)?;
    let classes_pv = expect_seq(export_named(export, "classes", span)?, IMPORT_OP, span)?;
    let mut classes = Vec::new();
    for c in classes_pv.iter() {
        classes.push(expect_str(c, IMPORT_OP, span)?.to_string());
    }
    let fields_pv = expect_seq(export_named(export, "fields", span)?, IMPORT_OP, span)?;
    let mut fields = Vec::new();
    for row in fields_pv.iter() {
        let rp = expect_seq(row, IMPORT_OP, span)?;
        let mut fs = Vec::new();
        for f in rp.iter() {
            fs.push(expect_str(f, IMPORT_OP, span)?.to_string());
        }
        fields.push(fs);
    }
    if classes.len() != fields.len() {
        return Err(malformed(
            span,
            IMPORT_OP,
            format!(
                "classes length {} != fields length {}",
                classes.len(),
                fields.len()
            ),
        ));
    }
    let expect_abi = abi_of(&classes, &fields);
    if stored_abi != expect_abi {
        return Err(malformed(
            span,
            IMPORT_OP,
            "ABI mismatch — export is from a different packed-classes/RETE_OPS",
        ));
    }
    // Host TypeEnv field-order. Packed ABI can agree with itself and still
    // disagree with this process's declared records.
    for (c, packed) in classes.iter().zip(fields.iter()) {
        let host = class_field_names(sym, c);
        if !host.is_empty() && &host != packed {
            return Err(malformed(
                span,
                IMPORT_OP,
                format!("ABI mismatch — host TypeEnv field-order for {c} differs from export"),
            ));
        }
    }

    let nodes_pv = expect_seq(export_named(export, "nodes", span)?, IMPORT_OP, span)?;
    // WALL 6 — how much this door will BUILD, refused on the declared count before any node is
    // unpacked. The build below is quadratic in this number; see `MAX_IMPORT_NODES` for the curve
    // and for what the cap costs at its own limit.
    if nodes_pv.len() > MAX_IMPORT_NODES {
        return Err(malformed(
            span,
            IMPORT_OP,
            format!(
                "import node count {} exceeds MAX_IMPORT_NODES {MAX_IMPORT_NODES} —                  the network build is quadratic in this count",
                nodes_pv.len()
            ),
        ));
    }
    let mut network_pairs = Vec::new();
    let mut alpha_by_type: AlphasByType = HashMap::new();
    let mut max_id = 0i64;
    for n in nodes_pv.iter() {
        let (id, rec, class_hint) = unpack_node(n, span)?;
        if id > max_id {
            max_id = id;
        }
        if let Some(class_idx) = class_hint {
            if let Some(name) = classes.get(class_idx as usize) {
                alpha_by_type.entry(name.clone()).or_default().push(id);
            }
        }
        network_pairs.push((Value::i64(id), rec));
    }
    let network = Value::wat__core__PersistentMap(PMap::from_pairs(network_pairs));

    // WALL 4 — the graph must be one the fire passes can legally walk. Before any side
    // table is read, and before anything downstream assumes ascending id is topological.
    check_node_graph(&network, span)?;

    let mut compiled_conds = HashMap::new();
    for pair in expect_seq(export_named(export, "conds", span)?, IMPORT_OP, span)? {
        let p = expect_seq(&pair, IMPORT_OP, span)?;
        compiled_conds.insert(
            expect_i64(expect_at(&p, 0, span, "cond id")?, IMPORT_OP, span)?,
            unpack_compiled_cond(expect_at(&p, 1, span, "cond")?, span)?,
        );
    }
    let mut compiled_drivers = HashMap::new();
    for pair in expect_seq(export_named(export, "drivers", span)?, IMPORT_OP, span)? {
        let p = expect_seq(&pair, IMPORT_OP, span)?;
        compiled_drivers.insert(
            expect_i64(expect_at(&p, 0, span, "driver id")?, IMPORT_OP, span)?,
            unpack_driver(expect_at(&p, 1, span, "driver")?, span, 0)?,
        );
    }
    let mut compiled_wheres = HashMap::new();
    for pair in expect_seq(export_named(export, "progs", span)?, IMPORT_OP, span)? {
        let p = expect_seq(&pair, IMPORT_OP, span)?;
        compiled_wheres.insert(
            expect_i64(expect_at(&p, 0, span, "prog id")?, IMPORT_OP, span)?,
            unpack_prog(expect_at(&p, 1, span, "prog")?, span, 0)?,
        );
    }
    let mut compiled_acc_folds = HashMap::new();
    for pair in expect_seq(export_named(export, "folds", span)?, IMPORT_OP, span)? {
        let p = expect_seq(&pair, IMPORT_OP, span)?;
        compiled_acc_folds.insert(
            expect_i64(expect_at(&p, 0, span, "fold id")?, IMPORT_OP, span)?,
            unpack_fold(expect_at(&p, 1, span, "fold")?, span)?,
        );
    }
    let mut compiled_rhs: CompiledRhsByRule = HashMap::new();
    for pair in expect_seq(export_named(export, "rhs", span)?, IMPORT_OP, span)? {
        let p = expect_seq(&pair, IMPORT_OP, span)?;
        let name = expect_str(expect_at(&p, 0, span, "rhs name")?, IMPORT_OP, span)?.to_string();
        let items = expect_seq(expect_at(&p, 1, span, "rhs items")?, IMPORT_OP, span)?;
        let mut rs = Vec::new();
        for x in &items {
            rs.push(unpack_rhs(x, span)?);
        }
        compiled_rhs.insert(name, rs);
    }

    let rule_deps = match agg_named_field(export, "deps") {
        Some(d) => unpack_deps(d, span)?,
        None => Vec::new(),
    };

    let node_ids = sorted_node_ids(&network);
    let crate::rete::kernel::NetworkEdges {
        feeding_alpha_of,
        parents_of,
        children_of,
        beta_readers,
    } = crate::rete::kernel::index_network_edges(&network, &node_ids);
    let compiled_max_slots = compiled_conds.values().map(|c| c.n_slots()).max().unwrap_or(0);
    let alpha_tree = AlphaTree::unpruned(&alpha_by_type);
    let where_tree = crate::rete::where_tree::WhereTree::build(&compiled_wheres);
    let kind_ids = kind_id_lists(&network, &node_ids);
    let joins_fed_by = invert_feeding_alpha(&feeding_alpha_of);
    let test_sibs = crate::rete::kernel::build_test_sibs(&network, &node_ids, &parents_of);
    let test_children = crate::rete::kernel::build_test_children(&network, &node_ids);
    let arm = Arc::new(InternedNetwork {
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
    });
    if let Some(id) = network_identity(&network) {
        // Import is an arm-session equivalent: MISS leases=1, HIT increments
        // (`DESIGN-STONE-intern-eviction`). Drop without release-session leaks until thread end.
        rete_arm_intern(id, &arm);
    }

    // ★ THE IMPORT DOOR IS A SESSION'S BIRTH, AND IS CHARGED LIKE ONE.
    //
    // `arm-session` marks its origin the moment the session exists; this door could not, because
    // its key IS the network it had to build first. So the reading was taken at the top of this
    // function and only the FILING happens here — everything above is charged to the session
    // below. Filing the reading taken HERE would zero the whole import, which is the defect:
    // `session_bytes` sets an unmarked session's origin at the FIRST CHECK, so an import that
    // never marked began its ceiling after its network already existed.
    //
    // `network_identity` is `Some` for any `PersistentMap`, which is what phase 4 built.
    // `mark_session_origin_at` does not clobber, so an identity already carrying an origin — the
    // same `PMap` intern arriving twice — keeps the older one, per A4.
    let origin_key = network_identity(&network);
    crate::alloc_counter::mark_session_origin_at(origin_key, session_origin);

    // And now the reading is meaningful: bytes this thread took since the top of this call, which
    // is precisely what the import allocated. It refuses the way its five neighbours refuse —
    // `malformed`, at the door — rather than inventing an outcome shape behind this arc's outcome
    // wall for a door whose every other refusal is a raise.
    //
    // ⚠ THIS MEASURES; IT DOES NOT PREVENT. The bytes are already spent when this runs, because
    // the quantity being judged does not exist until the work is done. That is why WALL 6 is a
    // separate mechanism and not an optimisation of this one: the cap refuses a CLAIM before the
    // build, this refuses a MEASUREMENT after it, and neither substitutes for the other. What this
    // one buys is that the session cannot go on to be used, and — by way of the origin filed two
    // lines up — that every later `insert` and fixpoint round is charged for the network too.
    if let Some(breach) =
        crate::rete::kernel::session::session_ceiling_breach(sym, origin_key)
    {
        return Err(malformed(
            span,
            IMPORT_OP,
            format!(
                "import allocated {} bytes, past max-session-bytes {} — the session an import \
                 opens is charged for the network it builds",
                breach.used, breach.limit
            ),
        ));
    }

    Ok(Value::Aggregate(Arc::new(AggregateValue::record(
        "wat::rete::Session".into(),
        session_names(),
        Arc::new(vec![
            network,
            empty_pv(),
            empty_pm(),
            empty_pm(),
            empty_pm(),
            empty_pv(),
            Value::i64(max_id + 1),
            empty_pm(),
        ]),
    ))))
}
