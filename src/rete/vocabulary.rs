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
//! - **`Form`** — lazy / short-circuiting, OR otherwise syntactic (not a plain strict fn). No
//!   `TypeScheme` — the checker's `infer_rete_form` (`check.rs`, just above `infer_list`) routes
//!   by `core_name` to the SAME inference helper the mirrored core form uses, never a hardcoded
//!   rete FQDN. The runtime side is the SAME generic re-dispatch as `Alias`
//!   (`dispatch_rete_op`'s `Alias | Form` arm).
//!
//!   **Its history, kept because the mistake is cheap to re-make:** #55 shipped this class with
//!   the Form dispatch routing UNCONDITIONALLY to `infer_boolean_shortcircuit` — correct only for
//!   `and`/`or` (whose core arm in `check.rs` is one shared `":wat::core::and" | ":wat::core::or"`
//!   match). `if` and `let` are lazy too but their inference is NOT that arm (`if` unifies its
//!   branches under a bool condition; `let` opens a binding scope) — as rows under the old
//!   unconditional route they would have silently typed as boolean short-circuits. #56 phase 1
//!   fixed this: `infer_rete_form` matches on `core_name` explicitly, with a LOUD located error
//!   (never a silent fallthrough) for a Form row nobody taught it to route (STOP-1).
//!   `cond`/`match`/`fn` are a further thing again: structural guards matched in
//!   `rete/purity.rs`'s `classify_expr`, which never reach `head_ok` at all — a mirrored `match`
//!   or `fn` is STILL a `Form` row for `infer_rete_form`'s purposes (routes to `infer_match`/
//!   `infer_fn` respectively), but ALSO needs its structural guard's own match-guard widened to
//!   recognise the rete name (a second, independent edit — STOP-4: do not conflate the two).
//!   `fn`'s earlier STOP-3 (#56 phase 1) is RETRACTED (design stone, "CORRECTED 2026-08-02"): it
//!   was a claim about `(def :name (fn …))`, the definition-registration path, and does not touch
//!   an anonymous `fn` value — `fn` is minted below by the exact same recipe `match` took,
//!   nothing more.
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
    /// Arc 278 #57 round 1a — needed to spell the `String/*` / `string::*` monomorphic rows.
    String,
    /// Arc 278 #57 round 1a — needed to spell `i64::to-f64`'s return type.
    F64,
}

impl ParamType {
    pub(crate) fn to_type_expr(self) -> TypeExpr {
        match self {
            ParamType::I64 => TypeExpr::Path(":wat::core::i64".into()),
            ParamType::Bool => TypeExpr::Path(":wat::core::bool".into()),
            ParamType::Keyword => TypeExpr::Path(":wat::core::keyword".into()),
            ParamType::String => TypeExpr::Path(":wat::core::String".into()),
            ParamType::F64 => TypeExpr::Path(":wat::core::f64".into()),
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
    // ── #56, the head-table form mirrors (12 + 8 corpus `where` forms respectively).
    //
    // `not` is an ALIAS, not a form — the design stone corrected the parent stone on exactly this:
    // `and`/`or` must short-circuit and route to `eval_and`/`eval_or`, but `not` is a plain strict
    // fn with an ordinary `TypeScheme` (`check.rs`'s `:wat::core::not` registration) dispatched to
    // `eval_not` (`runtime.rs`). It belongs beside the comparisons, not beside `and`.
    ReteOp {
        rete_name: ":wat::rete::core::not",
        core_name: ":wat::core::not",
        class: OpClass::Alias,
        params: &[ParamType::Bool],
        ret: ParamType::Bool,
        meta: OpMeta { pure: true, deterministic: true, total: true },
    },
    // `or` is `and`'s twin at every site that matters: the checker handles them in ONE arm
    // (`check.rs`'s `":wat::core::and" | ":wat::core::or"` → `infer_boolean_shortcircuit`), which
    // is precisely the arm the `Form` class re-dispatches to, and the runtime routes both to their
    // own lazy eval arms via the generic `core_name` re-dispatch. So this is a row and nothing
    // else. (`if`/`let`, below, needed the two mechanism edits this table's doc note on the
    // `Form` class's history describes — they are NOT "a row and nothing else" the way `or` is.)
    ReteOp {
        rete_name: ":wat::rete::core::or",
        core_name: ":wat::core::or",
        class: OpClass::Form,
        params: &[],
        ret: ParamType::Bool,
        meta: OpMeta { pure: true, deterministic: true, total: true },
    },
    // ── #56 phase 1, the head-table pair. Both were ALREADY admitted structurally by
    // `head_ok`'s plain `matches!` list (`purity.rs:246-249`, the very list `+`/`-`/`*` are
    // admitted through) — mirroring them was never an ADMISSION question, only a ROUTING one,
    // and the routing edit is `infer_rete_form` (`check.rs`, just above `infer_list`): `if`
    // unifies its two branches under a bool condition (`infer_if`); `let` opens a binding scope
    // (`infer_let`) — neither is `infer_boolean_shortcircuit`, the arm the old unconditional Form
    // route would have sent them to. `meta` mirrors `:wat::core::if`/`:wat::core::let`'s own
    // hand-list entries exactly (`purity.rs`'s pure∧det list, `:256`/`:257`; its `total` list,
    // `:453`/`:454`) — same values as the `and`/`or` rows above.
    //
    // The SECOND mechanism edit `if`/`let` needed — gating `eval_tail` (`runtime.rs:3807`) so a
    // rete `if`/`let` in TAIL POSITION reaches the same `eval_if_tail`/`eval_let_tail` its core
    // twin does — is not table-driven at all (nothing here to consult beyond `core_name`); see
    // `eval_tail`'s own doc for the gate and `tests/rete/probe_arc278_55_slice_one_vocabulary.rs`'s
    // TCO gate for the proof it is load-bearing (a rete `if` minted without it is a strictly WORSE
    // `if`: identical semantics, silently no TCO — SIGSEGV at depth instead of a stack-overflow
    // error).
    ReteOp {
        rete_name: ":wat::rete::core::if",
        core_name: ":wat::core::if",
        class: OpClass::Form,
        params: &[],
        ret: ParamType::Bool,
        meta: OpMeta { pure: true, deterministic: true, total: true },
    },
    ReteOp {
        rete_name: ":wat::rete::core::let",
        core_name: ":wat::core::let",
        class: OpClass::Form,
        params: &[],
        ret: ParamType::Bool,
        meta: OpMeta { pure: true, deterministic: true, total: true },
    },
    // ── #56 phase 2, the FIRST of the structural-guard pair. `match` is admitted through a
    // completely different door than `if`/`let`: `head_ok` (the ADMISSION-consulting fn) never
    // even sees a match head, because `rete/purity.rs`'s `classify_expr` intercepts a match FORM
    // structurally (skip the scrutinee's pattern positions, walk only the arm BODIES) before the
    // generic call-shape arm that would call `head_ok` ever runs. That structural guard matched
    // the literal core keyword only; widening it to also recognise this row's `rete_name` (by
    // resolving to `core_name` first — the table's own field, never a duplicated arm body) is a
    // SEPARATE edit from minting this row (STOP-4), and this row alone does nothing until that
    // widening lands.
    //
    // `meta` is therefore closer to VESTIGIAL than load-bearing for this specific row: nothing
    // reads it for an ordinary `(:wat::rete::core::match ...)` expression, because `classify_expr`
    // decides that expression's purity/determinism structurally (recursing into each arm body,
    // never consulting `head_ok`/`RETE_OPS.meta` for the match head itself) — `head_ok`'s
    // admission branch (which DOES read `meta`) is reachable for this head only if the structural
    // guard were somehow bypassed, which the widening below prevents. Kept accurate anyway, for
    // STOP-2 completeness and any future direct consumer: `total: false` because a non-exhaustive
    // match raises `NoMatchingArm` — genuinely partial, unlike `if`/`let` (always total for
    // well-typed args, per their own hand-list entries).
    ReteOp {
        rete_name: ":wat::rete::core::match",
        core_name: ":wat::core::match",
        class: OpClass::Form,
        params: &[],
        ret: ParamType::Bool,
        meta: OpMeta { pure: true, deterministic: true, total: false },
    },
    // ── #56's leftover, closed — `fn`, the second and last of the structural-guard pair.
    // `fn`'s STOP-3 (#56 phase 1) is RETRACTED: it was a claim about `(def :name (fn …))`, the
    // definition-registration path (`try_parse_fn_shape_def` + its variadic sibling, both
    // matching the literal `:wat::core::fn`) — irrelevant to an anonymous `fn` VALUE, which is
    // expressible today free-standing (proven by run: passed straight to `foldl`, never bound).
    // Same shape as `match`: `infer_rete_form` routes to `infer_fn` (a clean helper, exactly like
    // `infer_if`/`infer_let`/`infer_match`), and `rete/purity.rs`'s `fn`-literal structural guard
    // (its OWN arm, distinct from `match`'s) is widened through `resolve_core_name` the same one
    // way (STOP-4: one indirection, never a duplicated arm body).
    //
    // `meta` is vestigial here too, for the same reason as `match`'s row: `classify_expr`
    // intercepts a `fn` literal structurally (walks the body forms only; params/return-type are
    // never evaluated) before `head_ok` — which reads `meta` — is ever reached. Kept accurate
    // anyway, for STOP-2 completeness: unlike `match` (which can raise `NoMatchingArm` on a
    // non-exhaustive scrutinee), merely CONSTRUCTING a well-typed `fn` literal never raises, so
    // `total: true` — the same as `if`/`let`'s own hand-list entries, not `match`'s `false`.
    ReteOp {
        rete_name: ":wat::rete::core::fn",
        core_name: ":wat::core::fn",
        class: OpClass::Form,
        params: &[],
        ret: ParamType::Bool,
        meta: OpMeta { pure: true, deterministic: true, total: true },
    },
    // ── #57 round 1a, the nine monomorphic aliases (DESIGN-STONE-where-admits-only-rete-ops.md,
    // "THE MINT ROUNDS" → round 1's alias class). All `Alias`, all zero new logic: the rete name
    // re-dispatches to the same routine as `core_name` (`runtime.rs`'s generic `dispatch_rete_op`).
    // `meta` is TRANSCRIBED from `rete/purity.rs`'s `intrinsic_meta`, not decided here — every one
    // of these nine core heads is already `pure ∧ deterministic ∧ total = true` on disk: the
    // `String/*` five and `i64::to-f64` are named explicitly in both of `intrinsic_meta`'s
    // pure∧det and total hand-lists (`purity.rs:~410` / `~523-532`); `string::{length,trim,
    // to-lowercase}` are the three the namespace-prefix arm carves out of `total: false` by name
    // (`purity.rs:176-190`, "each verified total by reading its own implementation"). Signatures
    // verified against the CHECKER's own registration (`check.rs`'s `env.register` calls for each
    // core head — the `String/*` family's public type, not `string_ops.rs`'s doc comment, which
    // documents the underlying variadic `eval_string_concat` `String/concat` itself delegates to
    // but the checker constrains to exactly two args); `string::*`/`i64::to-f64` verified against
    // `string_ops.rs`'s own doc comments, which match exactly.
    ReteOp {
        rete_name: ":wat::rete::String/concat",
        core_name: ":wat::core::String/concat",
        class: OpClass::Alias,
        params: &[ParamType::String, ParamType::String],
        ret: ParamType::String,
        meta: OpMeta { pure: true, deterministic: true, total: true },
    },
    ReteOp {
        rete_name: ":wat::rete::String/starts-with?",
        core_name: ":wat::core::String/starts-with?",
        class: OpClass::Alias,
        params: &[ParamType::String, ParamType::String],
        ret: ParamType::Bool,
        meta: OpMeta { pure: true, deterministic: true, total: true },
    },
    ReteOp {
        rete_name: ":wat::rete::String/ends-with?",
        core_name: ":wat::core::String/ends-with?",
        class: OpClass::Alias,
        params: &[ParamType::String, ParamType::String],
        ret: ParamType::Bool,
        meta: OpMeta { pure: true, deterministic: true, total: true },
    },
    ReteOp {
        rete_name: ":wat::rete::String/contains?",
        core_name: ":wat::core::String/contains?",
        class: OpClass::Alias,
        params: &[ParamType::String, ParamType::String],
        ret: ParamType::Bool,
        meta: OpMeta { pure: true, deterministic: true, total: true },
    },
    ReteOp {
        rete_name: ":wat::rete::String/empty?",
        core_name: ":wat::core::String/empty?",
        class: OpClass::Alias,
        params: &[ParamType::String],
        ret: ParamType::Bool,
        meta: OpMeta { pure: true, deterministic: true, total: true },
    },
    ReteOp {
        rete_name: ":wat::rete::string::length",
        core_name: ":wat::core::string::length",
        class: OpClass::Alias,
        params: &[ParamType::String],
        ret: ParamType::I64,
        meta: OpMeta { pure: true, deterministic: true, total: true },
    },
    ReteOp {
        rete_name: ":wat::rete::string::trim",
        core_name: ":wat::core::string::trim",
        class: OpClass::Alias,
        params: &[ParamType::String],
        ret: ParamType::String,
        meta: OpMeta { pure: true, deterministic: true, total: true },
    },
    ReteOp {
        rete_name: ":wat::rete::string::to-lowercase",
        core_name: ":wat::core::string::to-lowercase",
        class: OpClass::Alias,
        params: &[ParamType::String],
        ret: ParamType::String,
        meta: OpMeta { pure: true, deterministic: true, total: true },
    },
    ReteOp {
        rete_name: ":wat::rete::i64::to-f64",
        core_name: ":wat::core::i64::to-f64",
        class: OpClass::Alias,
        params: &[ParamType::I64],
        ret: ParamType::F64,
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

/// Arc 278 #56 phase 2 — resolves a head to its `core_name` if it is a minted rete row,
/// otherwise returns `head` unchanged. THE single discriminator a structural-guard match arm
/// (`rete/purity.rs`'s `classify_expr` — `cond`/`match`/`fn`) consults to recognise a rete-named
/// twin WITHOUT duplicating its arm body (STOP-4's "do not conflate" applies here too: the guard
/// widening is this one indirection, never a second copy of the structural logic keyed on the
/// rete name). A non-rete head (the entire core corpus) round-trips through unchanged — this is
/// a pure lookup, zero behavior change for anything not in [`RETE_OPS`].
pub(crate) fn resolve_core_name(head: &str) -> &str {
    rete_op_for(head).map(|op| op.core_name).unwrap_or(head)
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
