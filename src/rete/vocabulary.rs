//! Arc 278 #55 (S3b+S4), slice one — the ONE table of rete-namespaced vocabulary ops.
//!
//! `docs/arc/2026/06/278-rules-engine/DESIGN-STONE-slice-one-rete-vocabulary.md`'s whole
//! contract in one sentence: **a rete op is named ONCE, here, and the three sites that used to
//! need a hand-written entry each (a `TypeScheme` in `check.rs`, a dispatch arm in `runtime.rs`,
//! a whitelist row in `rete/purity.rs`) instead ITERATE this table.** Adding op #5 is one row in
//! this file, not three edits across three files (STOP-2: an op named in more than one place is
//! the stone failing, not a detail).
//!
//! ## The three mechanism classes (grounded — see the design stone's "class table, corrected
//! twice by grounding")
//!
//! - **`Alias`** — a plain strict fn: rete name, same routine as `core_name`, zero new logic.
//!   Gets a `TypeScheme` (fed from `params`/`ret` below) AND a dispatch arm — but the dispatch
//!   arm is GENERIC (`runtime.rs`'s `dispatch_rete_op`), reached by re-invoking
//!   `dispatch_keyword_head_value` on `core_name`. Never a second implementation.
//! - **`Form`** — lazy / short-circuiting (`and`/`or`/`if`/`let`/`do`/`when`'s family). No
//!   `TypeScheme` — the checker gets a dedicated inference arm (`check.rs`'s
//!   `infer_boolean_shortcircuit`, mirrored generically via this table, never a hardcoded second
//!   FQDN literal). The runtime side is the SAME generic re-dispatch as `Alias`.
//! - **`Fallback`** — an alias PLUS a second terminal handler: the caller supplies a mandatory
//!   `:undefined` fallback value (4th positional arg, preceded by the literal keyword
//!   `:undefined` as a marker in the 3rd slot) that is substituted for the raise the alias's
//!   own routine would otherwise produce. See `runtime.rs`'s `dispatch_rete_op`, `OpClass::Fallback`
//!   arm, for the exact mechanism and why it needed NEITHER the `:9753` substrate kernel NOR a
//!   `:4829` refactor (STOP-3 did not fire — see that arm's doc).
//!
//! ## The admission test — module-set, NOT a bare prefix (STOP-1)
//!
//! `:wat::rete::` is already the rete ENGINE's own API (`fire-rules`, `insert`, `compile`,
//! `Session`, `AlphaNode`, `activate-fact`…) — a bare `starts_with(":wat::rete::")` would wrongly
//! admit those into "the vocabulary a `where` may call." [`RETE_MODULES`] is the real boundary:
//! the declared vocabulary SUB-namespaces. [`rete_vocabulary_admitted`] tests membership in that
//! set; [`rete_op_for`] additionally requires the head be an actual minted row (admission is
//! necessary, not sufficient — an admitted namespace with an unminted verb still default-denies
//! wherever it is consulted).
//!
//! UNARMED, same as `pure?`/`deterministic?`/`total?`: nothing in this slice wires the admission
//! test into `compile-condition` (`wat/rete.wat:661` stays `(and is-pure is-det)`). It is built
//! and unit-tested in isolation (`tests/rete/`), plus consulted as a fourth consideration inside
//! `head_ok` (`rete/purity.rs`) — additive, never a replacement for the three existing ones.

use crate::ast::WatAST;
use crate::runtime::{
    EvalBreak, Environment, RuntimeError, RuntimeErrorKind, SymbolTable, Value, ValueSnapshot,
};
use crate::span::Span;
use crate::types::TypeExpr;

use super::purity::OpMeta;

/// The mechanism class a rete-surface op belongs to — pins how the three sites treat the row.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum OpClass {
    /// Plain strict fn: rete name, same routine as `core_name`, zero new logic.
    Alias,
    /// Lazy / short-circuiting special form (mirrors `core_name`'s checker + eval arm; no
    /// `TypeScheme`).
    Form,
    /// An alias PLUS a second terminal handler substituting a mandatory `:undefined` value.
    Fallback,
}

/// A parameter/return type shape simple enough to live in a `const` table (no heap allocation —
/// `TypeExpr::Path` needs a `String`, so the table stores this instead and converts on
/// registration via [`ParamType::to_type_expr`]).
#[derive(Clone, Copy)]
pub(crate) enum ParamType {
    I64,
    Bool,
    Keyword,
}

impl ParamType {
    pub(crate) fn to_type_expr(self) -> TypeExpr {
        match self {
            ParamType::I64 => TypeExpr::Path(":wat::core::i64".into()),
            ParamType::Bool => TypeExpr::Path(":wat::core::bool".into()),
            ParamType::Keyword => TypeExpr::Path(":wat::core::keyword".into()),
        }
    }
}

/// One rete-surface op. THE single place any rete op is named (STOP-2).
pub(crate) struct ReteOp {
    /// The rete-surface FQDN, e.g. `":wat::rete::i64::>"`.
    pub(crate) rete_name: &'static str,
    /// The core routine this surfaces, e.g. `":wat::core::i64::>"`. For a `Form` this is the
    /// core form whose checker/eval arm is mirrored generically (re-dispatch, never a duplicate
    /// implementation).
    pub(crate) core_name: &'static str,
    pub(crate) class: OpClass,
    /// `Alias`/`Fallback` only — the params `check.rs` registers a `TypeScheme` from. Empty for
    /// `Form` (no `TypeScheme`; the checker consults a dedicated inference arm instead).
    pub(crate) params: &'static [ParamType],
    /// `Alias`/`Fallback` only — unused for `Form`.
    pub(crate) ret: ParamType,
    /// The whitelist row — what the fence's three axes (pure/deterministic/total) answer for
    /// this head. Reused type (`rete::purity::OpMeta`) per the brief's own sketch.
    pub(crate) meta: OpMeta,
}

/// THE ONE TABLE. Four rows: one of each mechanism class (the hardest-member-pins-the-shape
/// rule from the design stone), plus nothing else — the other ~46 vocabulary names slot in as
/// rows once this table exists (out of scope this slice).
pub(crate) const RETE_OPS: &[ReteOp] = &[
    // ── Alias — the cheap path, and the table's baseline row. `total: true` mirrors
    // `:wat::core::i64::>`'s own hand-list entry (`purity.rs`'s `total` match, `i64::{> < >= <=}`
    // row): an i64-i64 comparison never raises on any input pair — it is genuinely total, not a
    // default-deny placeholder.
    ReteOp {
        rete_name: ":wat::rete::i64::>",
        core_name: ":wat::core::i64::>",
        class: OpClass::Alias,
        params: &[ParamType::I64, ParamType::I64],
        ret: ParamType::Bool,
        meta: OpMeta { pure: true, deterministic: true, total: true },
    },
    // ── Fallback-carrying — the `:undefined` shape, and the only class touching runtime
    // semantics. `total: true`: this is the whole point of a fallback-carrying variant (see
    // `check.rs:19286`'s own comment: "arming the fence needs the `:undefined`-carrying total
    // variants (T2/T3) to exist first, or a refused `first`/`i64::/` has nowhere to go") — for
    // any two well-typed i64 inputs this ALWAYS returns some i64 (the sum, or the fallback), and
    // it never raises. Call shape: `(:wat::rete::i64::+ a b :undefined fallback)` — 4 positional
    // args; the literal keyword `:undefined` in slot 3 is a mandatory marker (see
    // `runtime.rs`'s `dispatch_rete_op`, `OpClass::Fallback` arm).
    ReteOp {
        rete_name: ":wat::rete::i64::+",
        core_name: ":wat::core::i64::+",
        class: OpClass::Fallback,
        params: &[ParamType::I64, ParamType::I64, ParamType::Keyword, ParamType::I64],
        ret: ParamType::I64,
        meta: OpMeta { pure: true, deterministic: true, total: true },
    },
    // ── Form — that a form can be mirrored at all. `total: true` mirrors `:wat::core::and`'s own
    // hand-list entry (bool args never raise). `params`/`ret` are unused for this class (no
    // `TypeScheme`; see `check.rs`'s generic Form-class dispatch, keyed off `class` alone, never
    // a hardcoded second `":wat::rete::core::and"` literal).
    ReteOp {
        rete_name: ":wat::rete::core::and",
        core_name: ":wat::core::and",
        class: OpClass::Form,
        params: &[],
        ret: ParamType::Bool,
        meta: OpMeta { pure: true, deterministic: true, total: true },
    },
    // Comparison alias — `:wat::core::i64::<` is already `total: true` in `intrinsic_meta`'s
    // own list (an i64-i64 comparison never raises on any input pair). Zero new logic: the rete
    // name re-dispatches to the same routine.
    ReteOp {
        rete_name: ":wat::rete::i64::<",
        core_name: ":wat::core::i64::<",
        class: OpClass::Alias,
        params: &[ParamType::I64, ParamType::I64],
        ret: ParamType::Bool,
        meta: OpMeta { pure: true, deterministic: true, total: true },
    },
    // Comparison alias — `:wat::core::i64::>=` is already `total: true` in `intrinsic_meta`'s
    // own list (an i64-i64 comparison never raises on any input pair). Zero new logic: the rete
    // name re-dispatches to the same routine.
    ReteOp {
        rete_name: ":wat::rete::i64::>=",
        core_name: ":wat::core::i64::>=",
        class: OpClass::Alias,
        params: &[ParamType::I64, ParamType::I64],
        ret: ParamType::Bool,
        meta: OpMeta { pure: true, deterministic: true, total: true },
    },
    // Comparison alias — `:wat::core::i64::<=` is already `total: true` in `intrinsic_meta`'s
    // own list (an i64-i64 comparison never raises on any input pair). Zero new logic: the rete
    // name re-dispatches to the same routine.
    ReteOp {
        rete_name: ":wat::rete::i64::<=",
        core_name: ":wat::core::i64::<=",
        class: OpClass::Alias,
        params: &[ParamType::I64, ParamType::I64],
        ret: ParamType::Bool,
        meta: OpMeta { pure: true, deterministic: true, total: true },
    },
    // Fallback-carrying — `:wat::core::i64::-` overflows at the i64 boundary. Total BY CONSTRUCTION: the caller's `:undefined` value covers
    // the undefined point, and `dispatch_rete_op` faces both i64 domain failures.
    ReteOp {
        rete_name: ":wat::rete::i64::-",
        core_name: ":wat::core::i64::-",
        class: OpClass::Fallback,
        params: &[ParamType::I64, ParamType::I64, ParamType::Keyword, ParamType::I64],
        ret: ParamType::I64,
        meta: OpMeta { pure: true, deterministic: true, total: true },
    },
    // Fallback-carrying — `:wat::core::i64::*` overflows at the i64 boundary. Total BY CONSTRUCTION: the caller's `:undefined` value covers
    // the undefined point, and `dispatch_rete_op` faces both i64 domain failures.
    ReteOp {
        rete_name: ":wat::rete::i64::*",
        core_name: ":wat::core::i64::*",
        class: OpClass::Fallback,
        params: &[ParamType::I64, ParamType::I64, ParamType::Keyword, ParamType::I64],
        ret: ParamType::I64,
        meta: OpMeta { pure: true, deterministic: true, total: true },
    },
    // Fallback-carrying — `:wat::core::i64::/` is undefined at a zero divisor, and overflows at MIN/-1. Total BY CONSTRUCTION: the caller's `:undefined` value covers
    // the undefined point, and `dispatch_rete_op` faces both i64 domain failures.
    ReteOp {
        rete_name: ":wat::rete::i64::/",
        core_name: ":wat::core::i64::/",
        class: OpClass::Fallback,
        params: &[ParamType::I64, ParamType::I64, ParamType::Keyword, ParamType::I64],
        ret: ParamType::I64,
        meta: OpMeta { pure: true, deterministic: true, total: true },
    },
    // Fallback-carrying — `:wat::core::i64::mod` is undefined at a zero divisor (floored; sign of the divisor). Total BY CONSTRUCTION: the caller's `:undefined` value covers
    // the undefined point, and `dispatch_rete_op` faces both i64 domain failures.
    ReteOp {
        rete_name: ":wat::rete::i64::mod",
        core_name: ":wat::core::i64::mod",
        class: OpClass::Fallback,
        params: &[ParamType::I64, ParamType::I64, ParamType::Keyword, ParamType::I64],
        ret: ParamType::I64,
        meta: OpMeta { pure: true, deterministic: true, total: true },
    },
    // Fallback-carrying — `:wat::core::i64::rem` is undefined at a zero divisor (sign of the dividend). Total BY CONSTRUCTION: the caller's `:undefined` value covers
    // the undefined point, and `dispatch_rete_op` faces both i64 domain failures.
    ReteOp {
        rete_name: ":wat::rete::i64::rem",
        core_name: ":wat::core::i64::rem",
        class: OpClass::Fallback,
        params: &[ParamType::I64, ParamType::I64, ParamType::Keyword, ParamType::I64],
        ret: ParamType::I64,
        meta: OpMeta { pure: true, deterministic: true, total: true },
    },
    // Fallback-carrying — `:wat::core::i64::quot` is undefined at a zero divisor (truncates toward zero). Total BY CONSTRUCTION: the caller's `:undefined` value covers
    // the undefined point, and `dispatch_rete_op` faces both i64 domain failures.
    ReteOp {
        rete_name: ":wat::rete::i64::quot",
        core_name: ":wat::core::i64::quot",
        class: OpClass::Fallback,
        params: &[ParamType::I64, ParamType::I64, ParamType::Keyword, ParamType::I64],
        ret: ParamType::I64,
        meta: OpMeta { pure: true, deterministic: true, total: true },
    },
];

/// THE ADMISSION TEST's boundary — declared rete-vocabulary SUB-namespaces. NOT the bare
/// `:wat::rete::` prefix (STOP-1): that prefix is already the engine's own API
/// (`fire-rules`/`insert`/`compile`/`Session`/`AlphaNode`/`activate-fact`…), none of which are
/// (or should ever become) a `RETE_OPS` row.
/// The vocabulary's root prefix — the O(1) gate in front of `rete_op_for`'s scan
/// (`runtime.rs`'s `dispatch_keyword_head_value`). It lives HERE, as a const, rather than as a
/// literal at the call site for a non-obvious reason: `purity.rs`'s completeness gate discovers
/// dispatched verbs by scanning that function's SOURCE TEXT for `":wat::…"` string literals, so a
/// bare prefix literal written inline is harvested as a phantom verb named `:wat::rete::` and the
/// gate goes red on something that does not exist. Referencing a const keeps the literal out of
/// the scanned range. (Found by that gate, 2026-08-02 — the scanner is naive, and this is the
/// cheaper side to fix.)
///
/// NOT the admission test — that is [`RETE_MODULES`], the module SET (STOP-1): this prefix alone
/// would admit the engine's own API (`fire-rules`/`insert`/`compile`/`Session`/…).
pub(crate) const RETE_PREFIX: &str = ":wat::rete::";

pub(crate) const RETE_MODULES: &[&str] = &[
    ":wat::rete::core::",
    ":wat::rete::i64::",
    ":wat::rete::f64::",
    ":wat::rete::string::",
    ":wat::rete::holon::",
];

/// Look up `head`'s row, if it is a minted rete-vocabulary op. Exact match — never a prefix scan
/// (STOP-1 applies here too: a prefix match would silently "admit" any typo under a real module).
pub(crate) fn rete_op_for(head: &str) -> Option<&'static ReteOp> {
    RETE_OPS.iter().find(|op| op.rete_name == head)
}

/// THE ADMISSION TEST. Does `head` fall inside a declared rete-vocabulary sub-namespace? Module-
/// set membership (see [`RETE_MODULES`]'s doc for why this is not a bare prefix). Admission is
/// necessary, not sufficient: an admitted namespace whose specific verb is not yet a `RETE_OPS`
/// row still default-denies wherever a caller additionally requires [`rete_op_for`] to resolve.
pub(crate) fn rete_vocabulary_admitted(head: &str) -> bool {
    RETE_MODULES.iter().any(|module| head.starts_with(module))
}

/// `(:wat::rete::vocabulary-admitted? <head: :wat::WatAST, a QUOTED keyword>) -> :bool` — THE
/// ADMISSION TEST surfaced for wat callers and its own isolated probe (`tests/rete/`), decoupled
/// from `pure?`/`deterministic?`/`total?` (which classify an EXPRESSION; this classifies a HEAD
/// NAME against the module-set boundary alone, independent of whether that head is pure).
/// UNARMED — not consulted by `compile-condition`.
///
/// Takes a QUOTED keyword (`(:wat::rete::vocabulary-admitted? (:wat::core::quote
/// :wat::rete::i64::>))`), mirroring `pure?`/`deterministic?`'s own `:wat::WatAST` argument
/// shape (`eval_axis_predicate`, above) — NOT a bare `:wat::core::keyword` value: a bare keyword
/// literal that names a REGISTERED function resolves at check time to that function's `Fn` type
/// (first-class function reference), not a `:wat::core::keyword` value, so an unquoted head name
/// cannot reach this predicate as data for exactly the heads worth testing.
pub(crate) fn eval_vocabulary_admitted_predicate(
    args: &[WatAST],
    list_span: &Span,
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, EvalBreak> {
    const OP: &str = ":wat::rete::vocabulary-admitted?";
    if args.len() != 1 {
        return Err(RuntimeError::new(
            list_span.clone(),
            RuntimeErrorKind::ArityMismatch { op: OP.into(), expected: 1, got: args.len() },
        )
        .into());
    }
    let val = crate::runtime::eval_inner(&args[0], env, sym)?.value_owned();
    let head = match val {
        Value::wat__WatAST(ref a) => match a.as_ref() {
            WatAST::Keyword(k, _) => k.clone(),
            other => {
                return Err(RuntimeError::new(
                    args[0].span().clone(),
                    RuntimeErrorKind::TypeMismatch {
                        op: OP.into(),
                        expected: ":wat::WatAST holding a Keyword (a quoted head name)",
                        got: Box::new(ValueSnapshot::of(&Value::String(std::sync::Arc::new(format!("{other:?}"))))),
                    },
                )
                .into());
            }
        },
        other => {
            return Err(RuntimeError::new(
                args[0].span().clone(),
                RuntimeErrorKind::TypeMismatch {
                    op: OP.into(),
                    expected: ":wat::WatAST (a quoted keyword from :wat::core::quote)",
                    got: Box::new(ValueSnapshot::of(&other)),
                },
            )
            .into());
        }
    };
    Ok(Value::bool(rete_vocabulary_admitted(&head)))
}
