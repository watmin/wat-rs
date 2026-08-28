//! Arc 278 #55 (S3b+S4), slice one — the ONE table of rete-namespaced vocabulary ops.
//!
//! `docs/arc/2026/06/278-rules-engine/DESIGN-STONE-slice-one-rete-vocabulary.md`'s whole
//! contract in one sentence: **a rete op is named ONCE, here, and the three sites that used to
//! need a hand-written entry each (a `TypeScheme` in `check.rs`, a dispatch arm in `runtime.rs`,
//! a whitelist row in `rete/purity.rs`) instead ITERATE this table.** Adding op #5 is one row in
//! this file, not three edits across three files (STOP-2: an op named in more than one place is
//! the stone failing, not a detail).
//!
//! ## The four mechanism classes (grounded — see the design stone's "class table, corrected
//! twice by grounding", plus #57 Redispatch)
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
//! - **`Redispatch`** — an ordinary fn whose type cannot be stated as a rank-1 `TypeScheme`
//!   (polymorphic over the container constructor). Named for HOW the type is answered
//!   (checker re-dispatches to core's own bespoke inference). No `TypeScheme`; runtime is
//!   the same generic re-dispatch `Alias`/`Form` already use. See `OpClass::Redispatch`.
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
//! `vocabulary-admitted?` is NOT the fence. The fence is four conjuncts
//! (`pure?` ∧ `deterministic?` ∧ `total?` ∧ `primitive?`). This module's admission
//! test is built and unit-tested in isolation (`tests/rete/`), plus consulted as a
//! consideration inside `head_ok` (`rete/purity.rs`) — additive, never a replacement
//! for the four fence axes. The fence's Law A check is `primitive?`, not this predicate.
//!
//! ## The naming rule (BRIEF-one-naming-rule-then-first-nth-to-string.md, 2026-08-05)
//!
//! **`rete_name` = `core_name` with `rete::` inserted immediately after `wat::`.** One rule, no
//! hand-maintained module list to drift: `RETE_MODULES` collapses to `core::`/`holon::` because
//! every `core_name` is already rooted at `:wat::core::` or `:wat::holon::`, so the rule PUTS every
//! new row inside an admitted module BY CONSTRUCTION. Before this rename the table carried three
//! *different* rules at once (bare insert keeping `core::`; `core::` → `rete::` replacement;
//! straight prefixing) — a module list standing in for a rule already silently missed 17 of 57
//! rows (`String/*`, `PersistentVector/*`, the five bare HOFs, `bool::`/`keyword::`).
//!
//! **Measured exception, six rows:** `string::{=,not=}` / `bool::{=,not=}` / `keyword::{=,not=}`
//! point at the GENERIC `:wat::core::=` / `:wat::core::not=` — core has no per-type equality for
//! these three types and does not need one (minting one would be the tail wagging the dog). The
//! rule as stated is **impossible to apply literally** to these six: `core_name.replacen(":wat::",
//! ":wat::rete::", 1)` produces the IDENTICAL string `:wat::rete::core::=` (resp. `not=`) for all
//! three types at once, which is not a naming quirk but a proven regression — `check.rs`'s
//! registration loop below feeds `op.rete_name` into `CheckEnv::register`, which is a raw
//! `HashMap::insert` (`check/env.rs:284`, doc'd "ungated ON PURPOSE... there is no predecessor a
//! registration could disagree with" — an assumption of distinct names this collision would
//! violate): three rows sharing one name means only the LAST-registered `TypeScheme` survives, so
//! two of the three types silently lose their monomorphic gate. Confirmed against a live call site
//! (`wat-scripts/scratch-pad/probe-brief-f64-surface-is-a-stub.wat`'s PRE-RENAME
//! `(:wat::rete::string::= "abc" "abc")`, now `:wat::rete::core::string::=`): applying the rule
//! verbatim makes that call fail `--check` under whichever scheme won the collision — a real
//! floor regression, not a theoretical one. So these six keep their per-type qualifier, nested
//! under `core::` exactly like their
//! `i64::=`/`f64::=` siblings (already distinct, since core HAS `i64::=`/`f64::=`): the family
//! reads `core::{i64,f64,string,bool,keyword}::{=,not=}`, uniform, still inside the closed module
//! set, still one row per row. The naming unit tests below encode this exception explicitly
//! (a finite, named allowlist) rather than silently special-casing it away.
//!
//! **A second instance of the SAME pattern, found by this file's own tests:** Phase 2's `first`
//! trio (`PersistentVector/first` / `Vector/first` / `List/first`) all point at the ONE
//! polymorphic `:wat::core::first` — minting them with the literal rule initially collapsed all
//! three onto `:wat::rete::core::first`, caught immediately by
//! `rete_name_is_core_name_with_rete_inserted_after_wat` going red (three rows, one name — the
//! exact class the equality trio already needed an exception for). Same fix, same reasoning:
//! these three also keep their per-container qualifier. Nine rows total in
//! `NAMING_RULE_EXCEPTIONS`, not six — "one core verb serving several rete rows" is not a
//! one-off, it recurs whenever a core op is polymorphic across something the rete surface wants
//! to monomorphise per-leaf (per-type for equality, per-container for `first`).

use std::collections::HashMap;
use std::sync::OnceLock;

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
    /// Arc 278 #57 round 1b — an ordinary fn whose type cannot be stated as a rank-1
    /// `TypeScheme` at all (polymorphic over the *container constructor*, not just the
    /// element — `foldl : (Acc,T)->Acc × Acc × C<T> -> Acc` where `C` ranges over `Vector`,
    /// `PersistentVector`, `List`, `Stream`). Named for HOW the type is answered (checker
    /// re-dispatches to core's own bespoke inference), never for WHAT the op is — unlike
    /// `Form`, whose members are genuinely special forms, a `Redispatch` row is a plain
    /// function. No `TypeScheme` (same as `Form`); the runtime side is the SAME generic
    /// re-dispatch `Alias`/`Form` already use (`dispatch_rete_op`'s `core_name` re-invoke) —
    /// this class changes the CHECKER's routing only.
    Redispatch,
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
    /// Arc 278 #57 round 1b — a bare declared type variable (e.g. the `T` in
    /// `PersistentVector/contains? : (PV<T>, T) -> bool`'s second param). Spelled the same
    /// way the hand-written `env.register` call sites already do for parametric core rows
    /// (`t_var()` = `TypeExpr::Path(":T")`) — a lexically-scoped type var is a `Path` at
    /// this layer, matched against the scheme's own `type_params` list, never `TypeExpr::Var`
    /// (that variant is a FRESH UNIFICATION var the checker mints internally, never authored).
    Var(&'static str),
    /// Arc 278 #57 round 1b — `PersistentVector<T>` for a named type variable `T`. The
    /// PV trio's container param.
    PersistentVectorOf(&'static str),
    /// Arc 278 #57 — `PersistentMap<K, V>` for named type variables `K` and `V`. Two
    /// parameters, unlike every sibling above: a map is keyed, so its container type
    /// cannot be spelled by the single-var `PersistentVectorOf` shape.
    PersistentMapOf(&'static str, &'static str),
    /// BRIEF-one-naming-rule-then-first-nth-to-string.md (2026-08-05) — `Vector<T>` for a named
    /// type variable `T`. Added the same way round 1a added `String`/`F64`: `first`'s per-
    /// container rows need one leaf per container, and `Vector` had none yet.
    VectorOf(&'static str),
    /// BRIEF-one-naming-rule-then-first-nth-to-string.md (2026-08-05) — `List<T>` for a named
    /// type variable `T`. `first`'s third container leaf.
    ListOf(&'static str),
    // BRIEF-get-is-total-by-fallback.md (2026-08-05) — the `OptionOf(&'static str)` leaf that
    // lived here (minted #57 round 1b for `PersistentVector/get`'s then-`Alias` return type) was
    // REMOVED: `get` converted to `Fallback` (ret: `Var("T")`, the hole unwrapped by
    // `dispatch_rete_op`'s new `Value::Option` arm, never surfaced to the rete caller), and
    // nothing else in this table ever constructed it — `cargo clippy` flagged it dead the moment
    // that conversion landed. STOP-4: core still returns `Option<T>` for ordinary wat
    // code — only the rete surface's spelling of `get` changed shape.
    /// Arc 278 the VSA seam opens — a holon AST value. Spells the exact `TypeExpr` `check.rs`'s
    /// own `holon_ty` closure (`check.rs:14974`, `TypeExpr::Path(":wat::holon::HolonAST")`)
    /// already builds for the hand-written holon intrinsic signatures — needed here to declare
    /// `cosine`/`dot`/`presence?`'s `Holon, Holon -> …` rete rows. Naming it narrows core's
    /// `HolonAST | Vector` polymorphism (arc 052/061) to `HolonAST`-only at the rete surface —
    /// deliberate, per the design stone's "the rete surface is per-type, period" ruling, not an
    /// omission.
    Holon,
}

impl ParamType {
    pub(crate) fn to_type_expr(self) -> TypeExpr {
        match self {
            ParamType::I64 => TypeExpr::Path(":wat::core::i64".into()),
            ParamType::Bool => TypeExpr::Path(":wat::core::bool".into()),
            ParamType::Keyword => TypeExpr::Path(":wat::core::keyword".into()),
            ParamType::String => TypeExpr::Path(":wat::core::String".into()),
            ParamType::F64 => TypeExpr::Path(":wat::core::f64".into()),
            ParamType::Var(name) => TypeExpr::Path(format!(":{name}")),
            ParamType::PersistentVectorOf(name) => TypeExpr::Parametric {
                head: "wat::core::PersistentVector".into(),
                args: vec![TypeExpr::Path(format!(":{name}"))],
            },
            ParamType::PersistentMapOf(k, v) => TypeExpr::Parametric {
                head: "wat::core::PersistentMap".into(),
                args: vec![TypeExpr::Path(format!(":{k}")), TypeExpr::Path(format!(":{v}"))],
            },
            ParamType::VectorOf(name) => TypeExpr::Parametric {
                head: "wat::core::Vector".into(),
                args: vec![TypeExpr::Path(format!(":{name}"))],
            },
            ParamType::ListOf(name) => TypeExpr::Parametric {
                head: "wat::core::List".into(),
                args: vec![TypeExpr::Path(format!(":{name}"))],
            },
            ParamType::Holon => TypeExpr::Path(":wat::holon::HolonAST".into()),
        }
    }
}

/// One rete-surface op. THE single place any rete op is named (STOP-2).
pub(crate) struct ReteOp {
    /// The rete-surface FQDN, e.g. `":wat::rete::core::i64::>"`.
    pub(crate) rete_name: &'static str,
    /// The core routine this surfaces, e.g. `":wat::core::i64::>"`. For a `Form` this is the
    /// core form whose checker/eval arm is mirrored generically (re-dispatch, never a duplicate
    /// implementation).
    pub(crate) core_name: &'static str,
    pub(crate) class: OpClass,
    /// `Alias`/`Fallback` only — the params `check.rs` registers a `TypeScheme` from. Empty for
    /// `Form`/`Redispatch` (no `TypeScheme`; the checker consults a dedicated inference arm
    /// instead).
    pub(crate) params: &'static [ParamType],
    /// `Alias`/`Fallback` only — unused for `Form`/`Redispatch`.
    pub(crate) ret: ParamType,
    /// Arc 278 #57 round 1b — the row's OWN type-parameter names, e.g. `&["T"]` for the PV
    /// trio. `&[]` on every row that does not need one (all pre-existing rows, plus the five
    /// `Redispatch` rows, which carry no scheme at all). Fed into `check.rs`'s registration
    /// loop's `TypeScheme.type_params` — previously hardcoded `vec![]` there, which is
    /// EXACTLY what blocked a parametric row from being stated (see the design stone's "the
    /// mechanism is already there and merely unreachable").
    pub(crate) type_params: &'static [&'static str],
    /// The whitelist row — what the fence's three axes (pure/deterministic/total) answer for
    /// this head. Reused type (`rete::purity::OpMeta`) per the brief's own sketch.
    pub(crate) meta: OpMeta,
}

/// THE ONE TABLE. Every rete-vocabulary verb a `where` / `:then` / user accum fold may
/// call. A row with `total: false` is a red build (`every_rete_row_is_total`).
pub(crate) const RETE_OPS: &[ReteOp] = &[
    // ── Alias — the cheap path, and the table's baseline row. `total: true` mirrors
    // `:wat::core::i64::>`'s own hand-list entry (`purity.rs`'s `total` match, `i64::{> < >= <=}`
    // row): an i64-i64 comparison never raises on any input pair — it is genuinely total, not a
    // default-deny placeholder.
    ReteOp {
        type_params: &[],
        rete_name: ":wat::rete::core::i64::>",
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
    // it never raises. Call shape: `(:wat::rete::core::i64::+ a b :undefined fallback)` — 4 positional
    // args; the literal keyword `:undefined` in slot 3 is a mandatory marker (see
    // `runtime.rs`'s `dispatch_rete_op`, `OpClass::Fallback` arm).
    ReteOp {
        type_params: &[],
        rete_name: ":wat::rete::core::i64::+",
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
        type_params: &[],
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
        type_params: &[],
        rete_name: ":wat::rete::core::i64::<",
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
        type_params: &[],
        rete_name: ":wat::rete::core::i64::>=",
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
        type_params: &[],
        rete_name: ":wat::rete::core::i64::<=",
        core_name: ":wat::core::i64::<=",
        class: OpClass::Alias,
        params: &[ParamType::I64, ParamType::I64],
        ret: ParamType::Bool,
        meta: OpMeta { pure: true, deterministic: true, total: true },
    },
    // ── BRIEF-the-f64-surface-is-a-stub.md Part C (2026-08-05) — the four f64 comparator
    // rows, mirroring the i64 comparator quartet immediately above exactly. `total: true`
    // matches `purity.rs:515`'s `total` list (Part A of this brief adds the three siblings of
    // `f64::>`, already present there): each is a comparison whose OUTPUT is a bool, never
    // itself the undefined value, and `eval_f64_compare` is NaN-correct — no raise on any
    // input pair. Core targets `:wat::core::f64::{>,<,>=,<=}` are already registered
    // (`check.rs:15875-15889`) and dispatched (`runtime.rs:5223-5226`) — zero new core logic,
    // a rete-surface alias only.
    ReteOp {
        type_params: &[],
        rete_name: ":wat::rete::core::f64::>",
        core_name: ":wat::core::f64::>",
        class: OpClass::Alias,
        params: &[ParamType::F64, ParamType::F64],
        ret: ParamType::Bool,
        meta: OpMeta { pure: true, deterministic: true, total: true },
    },
    ReteOp {
        type_params: &[],
        rete_name: ":wat::rete::core::f64::<",
        core_name: ":wat::core::f64::<",
        class: OpClass::Alias,
        params: &[ParamType::F64, ParamType::F64],
        ret: ParamType::Bool,
        meta: OpMeta { pure: true, deterministic: true, total: true },
    },
    ReteOp {
        type_params: &[],
        rete_name: ":wat::rete::core::f64::>=",
        core_name: ":wat::core::f64::>=",
        class: OpClass::Alias,
        params: &[ParamType::F64, ParamType::F64],
        ret: ParamType::Bool,
        meta: OpMeta { pure: true, deterministic: true, total: true },
    },
    ReteOp {
        type_params: &[],
        rete_name: ":wat::rete::core::f64::<=",
        core_name: ":wat::core::f64::<=",
        class: OpClass::Alias,
        params: &[ParamType::F64, ParamType::F64],
        ret: ParamType::Bool,
        meta: OpMeta { pure: true, deterministic: true, total: true },
    },
    // Fallback-carrying — `:wat::core::i64::-` overflows at the i64 boundary. Total BY CONSTRUCTION: the caller's `:undefined` value covers
    // the undefined point, and `dispatch_rete_op` faces both i64 domain failures.
    ReteOp {
        type_params: &[],
        rete_name: ":wat::rete::core::i64::-",
        core_name: ":wat::core::i64::-",
        class: OpClass::Fallback,
        params: &[ParamType::I64, ParamType::I64, ParamType::Keyword, ParamType::I64],
        ret: ParamType::I64,
        meta: OpMeta { pure: true, deterministic: true, total: true },
    },
    // Fallback-carrying — `:wat::core::i64::*` overflows at the i64 boundary. Total BY CONSTRUCTION: the caller's `:undefined` value covers
    // the undefined point, and `dispatch_rete_op` faces both i64 domain failures.
    ReteOp {
        type_params: &[],
        rete_name: ":wat::rete::core::i64::*",
        core_name: ":wat::core::i64::*",
        class: OpClass::Fallback,
        params: &[ParamType::I64, ParamType::I64, ParamType::Keyword, ParamType::I64],
        ret: ParamType::I64,
        meta: OpMeta { pure: true, deterministic: true, total: true },
    },
    // Fallback-carrying — `:wat::core::i64::/` is undefined at a zero divisor, and overflows at MIN/-1. Total BY CONSTRUCTION: the caller's `:undefined` value covers
    // the undefined point, and `dispatch_rete_op` faces both i64 domain failures.
    ReteOp {
        type_params: &[],
        rete_name: ":wat::rete::core::i64::/",
        core_name: ":wat::core::i64::/",
        class: OpClass::Fallback,
        params: &[ParamType::I64, ParamType::I64, ParamType::Keyword, ParamType::I64],
        ret: ParamType::I64,
        meta: OpMeta { pure: true, deterministic: true, total: true },
    },
    // Fallback-carrying — `:wat::core::i64::mod` is undefined at a zero divisor (floored; sign of the divisor). Total BY CONSTRUCTION: the caller's `:undefined` value covers
    // the undefined point, and `dispatch_rete_op` faces both i64 domain failures.
    ReteOp {
        type_params: &[],
        rete_name: ":wat::rete::core::i64::mod",
        core_name: ":wat::core::i64::mod",
        class: OpClass::Fallback,
        params: &[ParamType::I64, ParamType::I64, ParamType::Keyword, ParamType::I64],
        ret: ParamType::I64,
        meta: OpMeta { pure: true, deterministic: true, total: true },
    },
    // Fallback-carrying — `:wat::core::i64::rem` is undefined at a zero divisor (sign of the dividend). Total BY CONSTRUCTION: the caller's `:undefined` value covers
    // the undefined point, and `dispatch_rete_op` faces both i64 domain failures.
    ReteOp {
        type_params: &[],
        rete_name: ":wat::rete::core::i64::rem",
        core_name: ":wat::core::i64::rem",
        class: OpClass::Fallback,
        params: &[ParamType::I64, ParamType::I64, ParamType::Keyword, ParamType::I64],
        ret: ParamType::I64,
        meta: OpMeta { pure: true, deterministic: true, total: true },
    },
    // Fallback-carrying — `:wat::core::i64::quot` is undefined at a zero divisor (truncates toward zero). Total BY CONSTRUCTION: the caller's `:undefined` value covers
    // the undefined point, and `dispatch_rete_op` faces both i64 domain failures.
    ReteOp {
        type_params: &[],
        rete_name: ":wat::rete::core::i64::quot",
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
        type_params: &[],
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
        type_params: &[],
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
        type_params: &[],
        rete_name: ":wat::rete::core::if",
        core_name: ":wat::core::if",
        class: OpClass::Form,
        params: &[],
        ret: ParamType::Bool,
        meta: OpMeta { pure: true, deterministic: true, total: true },
    },
    ReteOp {
        type_params: &[],
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
    // the literal core keyword and this row's `rete_name` (via `core_name`).
    // `classify_expr` decides match purity structurally (arm bodies), not via
    // this row's `meta`. `meta` is kept accurate for STOP-2 completeness.
    //
    // ⛔ CORRECTED 2026-08-05 (task #80): was `total: false`, justified as "a non-exhaustive match
    // raises `NoMatchingArm` — genuinely partial". REFUTED TWICE, by run:
    //  1. A non-exhaustive match does not raise, it does not COMPILE — "missing arm(s) for
    //     variant(s): C". The exhaustiveness checker gets there first.
    //  2. `NoMatchingArm` DOES NOT EXIST in this codebase (`grep -rn` returns only this comment
    //     and its sibling). The justification cited an error that was never implemented.
    // Nor can a non-enum match be non-exhaustive: a pattern must be a KEYWORD, SYMBOL or LIST
    // (an int literal is rejected outright) — keywords/lists are enum variants, a bare symbol is
    // an irrefutable binding. No match that compiles can fail to match.
    ReteOp {
        type_params: &[],
        rete_name: ":wat::rete::core::match",
        core_name: ":wat::core::match",
        class: OpClass::Form,
        params: &[],
        ret: ParamType::Bool,
        meta: OpMeta { pure: true, deterministic: true, total: true },
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
    // anyway, for STOP-2 completeness: merely CONSTRUCTING a well-typed `fn` literal never
    // raises, so `total: true` — the same as `if`/`let`/`match`'s own hand-list entries.
    ReteOp {
        type_params: &[],
        rete_name: ":wat::rete::core::fn",
        core_name: ":wat::core::fn",
        class: OpClass::Form,
        params: &[],
        ret: ParamType::Bool,
        meta: OpMeta { pure: true, deterministic: true, total: true },
    },
    // ── BRIEF-rete-cond-is-its-own-macro.md (2026-08-05) — `cond`, the FIRST MACRO-BACKED rete
    // row, and unlike `match`/`fn` (both genuine runtime special forms with their own
    // `dispatch_keyword_head_value` eval arm), `cond` has ZERO runtime arm. Builder's ruling:
    // "i think we need rete's cond to just be a macro itself that expands into rete's if?".
    //
    // `:wat::rete::core::cond` is now its OWN `defmacro` (`wat/rete/syntax.wat`, right after the
    // `query` macro) — a copy of core's `cond` template (`wat/core.wat:1237`) with the emitted
    // head keywords moved to the rete namespace: every backtick-quoted `if` it emits is
    // `:wat::rete::core::if`, and every recursive call is `:wat::rete::core::cond`. This
    // REPLACES an earlier attempt (a `freeze::env::build_env` loop that cloned core's
    // registered `MacroDef` and re-registered it under the rete name): a clone carries core's
    // TEMPLATE regardless of which name invoked it, so it expanded to `:wat::core::if` /
    // `:wat::core::cond` — a second door laundering straight back through core's spelling
    // (measured by `macroexpand`, not reasoned about — that is exactly what the earlier
    // attempt skipped). This row now exists purely as the `RETE_OPS` admission entry the fence
    // and the naming-rule tests need; it does no expansion work itself — the wat-source
    // `defmacro` does.
    //
    // Present: the expander does not descend into `:wat::core::quote` in general, but
    // `make-rule`'s `:when` is a classified boundary (`src/resolve/boundary.rs`
    // `Boundary::MakeRule`; `src/macros/expand.rs` `expand_make_rule` /
    // `expand_make_rule_when`) that expands each `where` body. A `cond` written inside
    // a `where` is legal and expands to rete `if` (`wat-scripts/scratch-pad/probe-cond-rete-where.wat`
    // is the positive control). Outside a `where` (ordinary macro-expanded code — the
    // tier-ladder shape the brief's scorecard row 2 exercises), `cond` works identically
    // to its core twin, fully rete-spelled all the way down.
    //
    // `total: true` mirrors `if`'s own hand-list entry: `cond` expands to nested
    // `:wat::rete::core::if` (already `total` in `purity.rs`'s list) and introduces nothing
    // else; a `cond` with no terminal `:else` is a macro-EXPANSION-time `macro-error`
    // (`StartupError::Macro`), not a runtime domain hole, so it cannot reach the fence at all.
    ReteOp {
        type_params: &[],
        rete_name: ":wat::rete::core::cond",
        core_name: ":wat::core::cond",
        class: OpClass::Form,
        params: &[],
        ret: ParamType::Bool,
        meta: OpMeta { pure: true, deterministic: true, total: true },
    },
    // ── `do` — CUT, not minted (BRIEF-cond-the-first-macro-backed-rete-row.md Part B, builder
    // 2026-08-05: "i think do can go... i almost only ever use do for a stdout write or something
    // similar"). `do` evaluates every non-final form and DISCARDS its value (`eval_do_tail`,
    // `let _ = eval_inner(arg, ...)`), returning only the last. In a `where` — which the fence
    // guarantees pure ∧ deterministic ∧ total — a discarded PURE value cannot affect anything
    // reachable from the result, so `(do a b)` ≡ `b`, ALWAYS: it is not merely *unused*, it is
    // **incapable of meaning** under the fence. Its other real role (def-position splicing,
    // `register_runtime_defs_form`) needs a definition context, and a `where`/rete expression
    // position is not one. Cut on this derivation, deliberately — NOT because the corpus lacks a
    // `do` example (R60: a cut is earned by the fence's semantics, never by absence of demand).
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
        type_params: &[],
        rete_name: ":wat::rete::core::String/concat",
        core_name: ":wat::core::String/concat",
        class: OpClass::Alias,
        params: &[ParamType::String, ParamType::String],
        ret: ParamType::String,
        meta: OpMeta { pure: true, deterministic: true, total: true },
    },
    ReteOp {
        type_params: &[],
        rete_name: ":wat::rete::core::String/starts-with?",
        core_name: ":wat::core::String/starts-with?",
        class: OpClass::Alias,
        params: &[ParamType::String, ParamType::String],
        ret: ParamType::Bool,
        meta: OpMeta { pure: true, deterministic: true, total: true },
    },
    ReteOp {
        type_params: &[],
        rete_name: ":wat::rete::core::String/ends-with?",
        core_name: ":wat::core::String/ends-with?",
        class: OpClass::Alias,
        params: &[ParamType::String, ParamType::String],
        ret: ParamType::Bool,
        meta: OpMeta { pure: true, deterministic: true, total: true },
    },
    ReteOp {
        type_params: &[],
        rete_name: ":wat::rete::core::String/contains?",
        core_name: ":wat::core::String/contains?",
        class: OpClass::Alias,
        params: &[ParamType::String, ParamType::String],
        ret: ParamType::Bool,
        meta: OpMeta { pure: true, deterministic: true, total: true },
    },
    ReteOp {
        type_params: &[],
        rete_name: ":wat::rete::core::String/empty?",
        core_name: ":wat::core::String/empty?",
        class: OpClass::Alias,
        params: &[ParamType::String],
        ret: ParamType::Bool,
        meta: OpMeta { pure: true, deterministic: true, total: true },
    },
    ReteOp {
        type_params: &[],
        rete_name: ":wat::rete::core::string::length",
        core_name: ":wat::core::string::length",
        class: OpClass::Alias,
        params: &[ParamType::String],
        ret: ParamType::I64,
        meta: OpMeta { pure: true, deterministic: true, total: true },
    },
    ReteOp {
        type_params: &[],
        rete_name: ":wat::rete::core::string::trim",
        core_name: ":wat::core::string::trim",
        class: OpClass::Alias,
        params: &[ParamType::String],
        ret: ParamType::String,
        meta: OpMeta { pure: true, deterministic: true, total: true },
    },
    ReteOp {
        type_params: &[],
        rete_name: ":wat::rete::core::string::to-lowercase",
        core_name: ":wat::core::string::to-lowercase",
        class: OpClass::Alias,
        params: &[ParamType::String],
        ret: ParamType::String,
        meta: OpMeta { pure: true, deterministic: true, total: true },
    },
    ReteOp {
        type_params: &[],
        rete_name: ":wat::rete::core::i64::to-f64",
        core_name: ":wat::core::i64::to-f64",
        class: OpClass::Alias,
        params: &[ParamType::I64],
        ret: ParamType::F64,
        meta: OpMeta { pure: true, deterministic: true, total: true },
    },
    // ── #57 round 1b, the parametric pair (`Alias`, with the FIRST non-empty `type_params`
    // in this table). `PersistentVector/{length,contains?}` name exactly one container, so —
    // unlike the five HOFs below — their whole truth IS a rank-1 scheme:
    // `PersistentVector/length : PV<T> -> i64`. Verified against the real implementations
    // (`collection/eval.rs`'s `persistentvector_{length,contains_q}_inner`), NOT assumed
    // from this round's own planning doc. `meta` TRANSCRIBED from `rete/purity.rs`: both
    // sit in its pure∧det hand-list (`:336-339`) AND its total hand-list (`:525-527`) —
    // "always defined". `/get` originally joined this pair as an `Alias`; BRIEF-get-is-total-
    // by-fallback.md (2026-08-05) converted it to `Fallback` below — see that row's own comment.
    ReteOp {
        type_params: &["T"],
        rete_name: ":wat::rete::core::PersistentVector/length",
        core_name: ":wat::core::PersistentVector/length",
        class: OpClass::Alias,
        params: &[ParamType::PersistentVectorOf("T")],
        ret: ParamType::I64,
        meta: OpMeta { pure: true, deterministic: true, total: true },
    },
    // ─── THE CONTAINER CONSTRUCTORS (task #81, 2026-08-05) ────────────────────────────────────
    //
    // ⛔ THE GAP THESE CLOSE, and it was invisible to every instrument we had. Until now the rete
    // vocabulary was ALL ACCESSORS AND NO CONSTRUCTORS: `PersistentVector/{get,first,length,
    // contains?}`, `Vector/{get,first}`, `List/{get,first}`, `PersistentMap/contains-key?` — not
    // one row could BUILD a collection.
    //
    // That only became visible once `get` became Ruby's `fetch` (`BRIEF-get-is-total-by-fallback`):
    // its rete surface returns `T`, not `Option<T>`, so the mandatory `:undefined <value>` must BE
    // a value of the element type. When that element type is itself a collection, there was NO
    // WRITABLE FALLBACK. A rider hit exactly this at `where-collection.wat:157` and had to reach
    // for a BOUND VARIABLE (`:undefined ?t`) because `[]` had no form — a mandatory parameter whose
    // only expressible argument is a coincidence, which makes a fallback surface merely LOOK total.
    //
    // ★ AND THE CENSUS COULD NEVER HAVE FOUND IT. The corpus walker measures which heads APPEAR;
    // a missing constructor is invisible to it precisely BECAUSE the corpus routed around it —
    // the rider wrote `?t` instead of `[]`. Only a real consumer surfaces this class
    // (300 ALIVS ARGVIT), and the same blindness is R62's `NOMINATO INSTRVMENTO` in miniature.
    //
    // WHY `Redispatch` AND NOT A SCHEME: a constructor is VARIADIC and PARAMETRIC — "N arguments,
    // all of T, yielding C<T>" — which a rank-1 `TypeScheme` cannot state, the identical reason the
    // five HOFs re-dispatch. Each already has a bespoke inference arm (`check.rs:3109` for
    // `PersistentVector`), so re-dispatch by head-substitution keeps that arm the ONE place the
    // inference lives. Never a second implementation; STOP-5 (no scheme) untouched.
    //
    // TOTAL BY CONSTRUCTION: building a literal collection has no domain hole — there is no input
    // on which it is undefined — so these need no `:undefined` of their own. `params`/`ret` are
    // inert for `Redispatch` (the checker routes before reading them), matching the HOF rows.
    //
    // ⛔ DELIBERATELY ABSENT — `Stream` is in `seq_container.rs`'s registry and is NOT minted: a
    // lazy sequence inside a `where` is a termination hazard, and the fence exists in part to keep
    // it out. `HashSet`/`WatAstList` likewise await a ruling that they are legal rule-condition
    // values at all. Growth is by DEMAND; absence here is a decision, not an oversight.
    ReteOp {
        type_params: &["T"],
        rete_name: ":wat::rete::core::PersistentVector",
        core_name: ":wat::core::PersistentVector",
        class: OpClass::Redispatch,
        params: &[],
        ret: ParamType::Bool,
        meta: OpMeta { pure: true, deterministic: true, total: true },
    },
    ReteOp {
        type_params: &["T"],
        rete_name: ":wat::rete::core::Vector",
        core_name: ":wat::core::Vector",
        class: OpClass::Redispatch,
        params: &[],
        ret: ParamType::Bool,
        meta: OpMeta { pure: true, deterministic: true, total: true },
    },
    ReteOp {
        type_params: &["T"],
        rete_name: ":wat::rete::core::List",
        core_name: ":wat::core::List",
        class: OpClass::Redispatch,
        params: &[],
        ret: ParamType::Bool,
        meta: OpMeta { pure: true, deterministic: true, total: true },
    },
    ReteOp {
        type_params: &["K", "V"],
        rete_name: ":wat::rete::core::PersistentMap",
        core_name: ":wat::core::PersistentMap",
        class: OpClass::Redispatch,
        params: &[],
        ret: ParamType::Bool,
        meta: OpMeta { pure: true, deterministic: true, total: true },
    },
    ReteOp {
        type_params: &[],
        rete_name: ":wat::rete::core::Tuple",
        core_name: ":wat::core::Tuple",
        class: OpClass::Redispatch,
        params: &[],
        ret: ParamType::Bool,
        meta: OpMeta { pure: true, deterministic: true, total: true },
    },
    ReteOp {
        type_params: &["T"],
        rete_name: ":wat::rete::core::PersistentVector/contains?",
        core_name: ":wat::core::PersistentVector/contains?",
        class: OpClass::Alias,
        params: &[ParamType::PersistentVectorOf("T"), ParamType::Var("T")],
        ret: ParamType::Bool,
        meta: OpMeta { pure: true, deterministic: true, total: true },
    },
    // Arc 278 #57 — the LAST UNSURE-bucket straggler, resolved by AUDIT (the seam's own
    // instruction: "audit, do not guess"), not by analogy. `persistentmap_contains_key_q_inner`
    // (`collection/eval.rs:959`) has exactly two exits and they differ in KIND:
    //
    //   1. an UNHASHABLE key  -> `Ok(Value::bool(false))`. No raise. This is the PREDICATE
    //      ruling of DESIGN-STONE-where-admits-only-rete-ops, not a sentinel: the question
    //      asked is "is this key in the map?", and a value that cannot be a key is not in it.
    //      `false` is the honest answer, the way `coincident?` answers `false` on a degenerate
    //      operand. Nothing is absorbed that the caller needed.
    //   2. a WRONG RECEIVER -> `TypeMismatch` raise. Must-never-happen: this row DECLARES the
    //      receiver as `PersistentMap<K,V>`, so the checker refuses a non-map before runtime.
    //
    // The differential that settles it is the sibling directly above. `PersistentVector/contains?`
    // was already ruled `total: true` and its impl (`persistentvector_contains_q_inner`) carries
    // the SAME receiver raise with NO key-hashability guard at all — so this verb is strictly
    // MORE total than one already ruled total. Refusing it would have been the tighter guard
    // making the honest path non-compliant.
    //
    // Already pure ∧ deterministic (`rete/purity.rs:345`); the audit adds only `total`.
    ReteOp {
        type_params: &["K", "V"],
        rete_name: ":wat::rete::core::PersistentMap/contains-key?",
        core_name: ":wat::core::PersistentMap/contains-key?",
        class: OpClass::Alias,
        params: &[ParamType::PersistentMapOf("K", "V"), ParamType::Var("K")],
        ret: ParamType::Bool,
        meta: OpMeta { pure: true, deterministic: true, total: true },
    },
    // BRIEF-get-is-total-by-fallback.md (2026-08-05) — `PersistentVector/get` CONVERTED from
    // `Alias` to `Fallback` (builder's ruling: "fallback — that's the UX… if it isn't [in the
    // vec] the result is undefined by nature… there's no meaningful value there so it mandates
    // a user supplied value in such cases"). This is the FOURTH failure-mode shape
    // `dispatch_rete_op`'s `Fallback` arm faces (see that arm's own doc): the core op signals
    // its hole by returning `None`, never by raising, a non-finite scalar, or an outcome enum —
    // so it needs its own generic `Value::Option` arm, not a fit into any of the other three.
    // `Vector/get` and `List/get` join as new siblings (verified against their own
    // `vector_get_inner`/`list_get_inner` — each wraps its result in `Value::Option`, same
    // shape as `persistentvector_get_inner`). Same naming-rule note as the PV trio above: each
    // row's `core_name` is per-container (`PersistentVector/get` / `Vector/get` / `List/get`),
    // never shared, so the rete_name = core_name-with-`rete::`-inserted rule applies with NO
    // exception needed (unlike `first`, whose trio shares one polymorphic core_name).
    //
    // ⚠ The cost, recorded per the brief: a rule author loses the ability to distinguish
    // "absent" from "present and equal to the caller's own default" inside a `where` — the
    // deliberate trade the ruling makes. The `Option`-returning form remains available in core
    // for ordinary wat code (STOP-4); only the rete spelling changes shape.
    ReteOp {
        type_params: &["T"],
        rete_name: ":wat::rete::core::PersistentVector/get",
        core_name: ":wat::core::PersistentVector/get",
        class: OpClass::Fallback,
        params: &[ParamType::PersistentVectorOf("T"), ParamType::I64, ParamType::Keyword, ParamType::Var("T")],
        ret: ParamType::Var("T"),
        meta: OpMeta { pure: true, deterministic: true, total: true },
    },
    ReteOp {
        type_params: &["T"],
        rete_name: ":wat::rete::core::Vector/get",
        core_name: ":wat::core::Vector/get",
        class: OpClass::Fallback,
        params: &[ParamType::VectorOf("T"), ParamType::I64, ParamType::Keyword, ParamType::Var("T")],
        ret: ParamType::Var("T"),
        meta: OpMeta { pure: true, deterministic: true, total: true },
    },
    ReteOp {
        type_params: &["T"],
        rete_name: ":wat::rete::core::List/get",
        core_name: ":wat::core::List/get",
        class: OpClass::Fallback,
        params: &[ParamType::ListOf("T"), ParamType::I64, ParamType::Keyword, ParamType::Var("T")],
        ret: ParamType::Var("T"),
        meta: OpMeta { pure: true, deterministic: true, total: true },
    },
    // ── #57 round 1b, originally the five higher-order combinators (`Redispatch` — this
    // table's first use of the class); arc 118.B6b retired `foldr`'s row — it was
    // `reverse`+`foldl` wearing a name borrowed from Haskell, where the verb is distinct only
    // because it is LAZY, a property strict wat cannot have — leaving four. `foldl` is
    // polymorphic over the CONTAINER CONSTRUCTOR (Vector, PersistentVector, List, Stream),
    // which no rank-1 `TypeScheme` can say — STOP-5: no scheme is minted here, ever, for this
    // reason. `params`/`ret` are unused for this class exactly as they are for `Form`
    // (mirrored, not consulted — `check.rs`'s registration loop skips `Redispatch` the same
    // way it skips `Form`); `ret: ParamType::Bool` is a placeholder, matching the `Form` rows'
    // own convention above. `meta` TRANSCRIBED from `rete/purity.rs`, never decided: all four
    // sit in its pure∧det hand-list (`:371-375`, "CONDITIONALLY pure∧det: the combinator
    // itself is referentially transparent + effect-free; its purity/determinism falls out of
    // the arg-recursion over its fn-argument" — `classify_expr`'s unconditional per-argument
    // recursion, already generic, needs no widening for these four). `total`: all four HOF
    // rete heads are `total: true` (the rows below; `every_rete_row_is_total` makes a false
    // row a red build). `classify_expr`'s general-list arm recurses into the fn-literal
    // argument — the same mechanism already built for pure/det.
    ReteOp {
        type_params: &[],
        rete_name: ":wat::rete::core::foldl",
        core_name: ":wat::core::foldl",
        class: OpClass::Redispatch,
        params: &[],
        ret: ParamType::Bool,
        meta: OpMeta { pure: true, deterministic: true, total: true },
    },
    // ── 2026-08-28: `map`/`filter` -> `mapv`/`filterv`, and the reason is the whole point of the
    // § 4.1 reachability ledger. Both rows were `:wat::core::map`/`:wat::core::filter`, which
    // return a LAZY `Stream` (`transform.rs`: `Value::wat__stream__Stream(lazy_map_stream(..))`).
    // A compiled `where` fence has no stream machinery and nothing in a fence can CONSUME a
    // Stream, so those rows were unreachable in every position — admitted, total, arity- and
    // type-checked, and unusable. The ledger drove them and they raised `unbound symbol`.
    //
    // The fix is NOT an eager compiled arm for the lazy heads: that would make
    // `:wat::rete::core::map` mean something different from `:wat::core::map`, silently, when the
    // `Redispatch` contract is "the same routine as `core_name`". wat already ships the eager
    // materializers under their clojure names — `wat/seq.wat`: *"mapv / filterv — the eager forms:
    // force `map`/`filter`'s lazy Stream result to a Vector"* — so rete takes THOSE. No invented
    // semantics, no divergence, and the naming rule derives both rete names unchanged.
    ReteOp {
        type_params: &[],
        rete_name: ":wat::rete::core::mapv",
        core_name: ":wat::core::mapv",
        class: OpClass::Redispatch,
        params: &[],
        ret: ParamType::Bool,
        meta: OpMeta { pure: true, deterministic: true, total: true },
    },
    ReteOp {
        type_params: &[],
        rete_name: ":wat::rete::core::filterv",
        core_name: ":wat::core::filterv",
        class: OpClass::Redispatch,
        params: &[],
        ret: ParamType::Bool,
        meta: OpMeta { pure: true, deterministic: true, total: true },
    },
    // `reduce` is a wat-level `defclause` (`wat/seq.wat`), NOT a checker special form like
    // its four siblings above — verified: no `infer_reduce` exists anywhere in
    // `collection::infer` (searched; the other four's arms are real fns there). Its
    // `infer_rete_form` arm (`check.rs`) therefore cannot call a matching bespoke inference
    // fn the way `infer_if`'s shape does for the other four — it re-dispatches by
    // reconstructing the call with `core_name` as head and recursing into `infer_list`,
    // which reaches the SAME defclause-dispatch machinery (`env.get_defclause_clauses`) a
    // core-spelled `(:wat::core::reduce ...)` call already takes. Genuinely the most literal
    // reading of "re-dispatch to core's existing inference" available for a defclause-backed
    // head — not a second implementation, and not a scheme (STOP-5 untouched).
    // ── 2026-08-28 — THE TUPLE ACCESSORS. Found by the § 4.1 ledger reporting
    // `:wat::rete::core::Tuple` unrunnable, and by the builder refusing the conclusion I drew
    // from it ("no row reads a Tuple, so maybe the row should not exist"). That was the CORPUS
    // FALLACY this table's own totality gate already refuted: absence of a caller is not evidence
    // of absence of need.
    //
    // Measuring says core serves tuples perfectly well — `first`/`second`/`third` project one
    // (verified live: a 3-tuple yields 7 / 99 / 512), which is the right idiom for a fixed-arity
    // heterogeneous product. `get`/`nth` refuse, `Tuple/get` does not exist, and top-level `match`
    // cannot see a Tuple at all (`MatchShape` carries no Tuple), but none of that matters once the
    // trio exists. **The gap was RETE'S**: this table had the Tuple CONSTRUCTOR and no accessor
    // admitting a Tuple, its first-family rows were per-container (`PersistentVector/first` and
    // siblings), and there was no `second`/`third` row at all — for any container. So a rule could
    // BUILD a tuple in a fence and never read one element, which is why `Tuple` is one of the three
    // rows appearing nowhere in the 1569-file corpus. Not neglect: never usable, since genesis.
    //
    // `Redispatch`, not `Alias`: a tuple accessor's type is polymorphic over the tuple's ARITY and
    // its per-position element types, which is precisely what a rank-1 `TypeScheme` cannot state —
    // the class's own definition. `total: true` is honest because arity is enforced at CHECK time:
    // `third` on a 2-tuple is a `TypeMismatch` reading "expects tuple with >= 3 element(s)", so no
    // out-of-range access survives to runtime.
    //
    // Per-type names (`Tuple/first`, not a generic `first`) follow BOTH the arc's "the rete surface
    // is per-type, period" ruling and the existing first-trio precedent; they join the naming-rule
    // exception list for the same reason those three did — one `core_name` serving several rows.
    ReteOp {
        type_params: &[],
        rete_name: ":wat::rete::core::Tuple/first",
        core_name: ":wat::core::first",
        class: OpClass::Redispatch,
        params: &[],
        ret: ParamType::Bool,
        meta: OpMeta { pure: true, deterministic: true, total: true },
    },
    ReteOp {
        type_params: &[],
        rete_name: ":wat::rete::core::Tuple/second",
        core_name: ":wat::core::second",
        class: OpClass::Redispatch,
        params: &[],
        ret: ParamType::Bool,
        meta: OpMeta { pure: true, deterministic: true, total: true },
    },
    ReteOp {
        type_params: &[],
        rete_name: ":wat::rete::core::Tuple/third",
        core_name: ":wat::core::third",
        class: OpClass::Redispatch,
        params: &[],
        ret: ParamType::Bool,
        meta: OpMeta { pure: true, deterministic: true, total: true },
    },
    ReteOp {
        type_params: &[],
        rete_name: ":wat::rete::core::reduce",
        core_name: ":wat::core::reduce",
        class: OpClass::Redispatch,
        params: &[],
        ret: ParamType::Bool,
        meta: OpMeta { pure: true, deterministic: true, total: true },
    },
    // ── #57 round 1c, the ten per-type equality/inequality aliases (`=`/`not=` across the five
    // `ParamType` leaves). `class: Alias` throughout — same shape as round 1a, one round larger.
    // `total: true` per row: TOTALITY IS DELIVERED BY THE SIGNATURE, not by the routine
    // underneath — a row declaring `[T, T] -> Bool` makes an incomparable pair (e.g.
    // `(:wat::rete::core::string::= "a" 1)`) a TYPE ERROR before anything runs, which is the entire
    // domain hole a per-type surface exists to delete (DESIGN-STONE-where-admits-only-rete-ops.md,
    // "★★ RULED — THE RETE SURFACE IS PER-TYPE, PERIOD"). `meta` TRANSCRIBED, not decided, from
    // generic `=`/`not=` (`rete/purity.rs:307-308` pure∧det, `:511-512` total) for every one of
    // the ten rows — including the four below whose `core_name` is the generic op (see next
    // paragraph), so this is the correct source regardless of routing.
    //
    // `String`/`bool`/`keyword` point at the GENERIC `:wat::core::=` / `:wat::core::not=` per the
    // brief — core has no per-type `String::=` and does not need one; minting one would be the
    // tail wagging the dog (STOP-3).
    //
    // BRIEF-one-naming-rule-then-first-nth-to-string.md (2026-08-05) — naming-rule EXCEPTION,
    // documented at the module doc's "The naming rule": because these six rows' `core_name` is
    // the SAME generic op across all three types, the literal insert rule
    // (`core_name.replacen(":wat::", ":wat::rete::", 1)`) would collapse `string::=`/`bool::=`/
    // `keyword::=` onto the IDENTICAL name `:wat::rete::core::=` (and `not=`'s siblings likewise)
    // — not a cosmetic collision: `check.rs`'s registration loop below keys a `HashMap` by
    // `rete_name`, so three same-named rows would silently leave only the LAST-registered
    // `TypeScheme` reachable, deleting two of the three types' monomorphic gate. So these six
    // rows keep the per-type qualifier the rest of the family already carries, nested under
    // `core::` like `i64::=`/`f64::=` below (which need no exception — core genuinely has
    // `i64::=`/`f64::=`) rather than being derived from the shared generic `core_name`.
    //
    // `i64`/`f64` are RE-POINTED at the per-type doors — BRIEF-the-f64-surface-is-a-stub.md
    // Part E (2026-08-05). `c59b2dca` (DESIGN-STONE-per-type-equality-restored.md) restored
    // `:wat::core::{i64,f64}::{=,not=}`: they are registered (`check.rs:15809-15828`,
    // `:15875-15889`) and dispatched (`runtime.rs:5220-5221`, `:5230-5231`) again, reversing
    // 237.8d. The paragraph this replaces said those spellings "do not exist" — that was true
    // when written and is false now; a stale comment is a lie the next reader inherits.
    ReteOp {
        type_params: &[],
        rete_name: ":wat::rete::core::i64::=",
        core_name: ":wat::core::i64::=",
        class: OpClass::Alias,
        params: &[ParamType::I64, ParamType::I64],
        ret: ParamType::Bool,
        meta: OpMeta { pure: true, deterministic: true, total: true },
    },
    ReteOp {
        type_params: &[],
        rete_name: ":wat::rete::core::i64::not=",
        core_name: ":wat::core::i64::not=",
        class: OpClass::Alias,
        params: &[ParamType::I64, ParamType::I64],
        ret: ParamType::Bool,
        meta: OpMeta { pure: true, deterministic: true, total: true },
    },
    ReteOp {
        type_params: &[],
        rete_name: ":wat::rete::core::f64::=",
        core_name: ":wat::core::f64::=",
        class: OpClass::Alias,
        params: &[ParamType::F64, ParamType::F64],
        ret: ParamType::Bool,
        meta: OpMeta { pure: true, deterministic: true, total: true },
    },
    ReteOp {
        type_params: &[],
        rete_name: ":wat::rete::core::f64::not=",
        core_name: ":wat::core::f64::not=",
        class: OpClass::Alias,
        params: &[ParamType::F64, ParamType::F64],
        ret: ParamType::Bool,
        meta: OpMeta { pure: true, deterministic: true, total: true },
    },
    // BRIEF-the-f64-surface-is-a-stub.md Part D (2026-08-05) — casing bug fixed. Round 1c
    // (`6d5af2c8`) minted these with a capital-S `String::`, derived from the TYPE instead of
    // the MODULE; every other string row in both surfaces is lowercase
    // (`:wat::core::string::{length,concat,trim,…}`, `:wat::rete::core::string::{length,to-lowercase,
    // trim}`). Renamed `:wat::rete::String::{=,not=}` → `:wat::rete::core::string::{=,not=}`. Zero
    // call sites existed at rename time (`grep -rn 'rete::String::' --include=*.wat
    // --include=*.rs .` found only this file's own three occurrences: the comment above and
    // these two rows) — a rename, not a migration.
    // ── #57 — enum equality. `Form`, NOT `Alias`, and the reason is the whole point of the row.
    //
    // MEASURED: the where-corpus compares a user enum in 2 places
    // (`(= (:arena::Route/method ?route) :arena::Method::POST)` — sift-rules-arena.wat:114,
    // probe-arena-rich-graph.wat:54). The ten minted equality rows cover bool/f64/i64/keyword/
    // string; `:arena::Method` is none of them, and a USER enum can never have a pre-minted row —
    // the row table is closed and user enums are not.
    //
    // ⛔ WHY THIS IS NOT AN `Alias` WITH A TYPE VAR — the trap this row exists to avoid.
    // `TypeScheme` (`check.rs:79`) is `{ type_params: Vec<String>, params, ret, rest_param_type }`
    // — a type param is a BARE NAME with NO BOUNDS FIELD. So a row
    // `type_params: ["E"], params: [Var("E"), Var("E")]` would accept ANY two same-typed operands
    // — i64, String, a record, anything. That is GENERIC `=` WEARING A PER-TYPE NAME: it passes
    // module admission, passes the naming rule, passes the floor, and silently re-opens the exact
    // door "the rete surface is per-type, period" closed. A name drawn too LOOSE makes the
    // dishonest path look compliant.
    //
    // ⇒ `Form`, so the arm lives in `infer_rete_form` (`check.rs`) as RUST, where the enum-ness
    // gate CAN be expressed and `TypeScheme` cannot express it. The arm asserts both operands
    // resolve to `TypeDef::Enum` and only then defers to `infer_equality` — the SAME routine core
    // `=` uses. Never a second implementation; a second terminal handler on the one routine, which
    // is this stone's implementation law. `params: &[]` / `ret: Bool` mirrors the `match` row:
    // a Form row's shape is not a scheme, it is a marker that inference is re-dispatched.
    //
    // Runtime is free: `dispatch_rete_op` sends `Alias | Form | Redispatch` through
    // `dispatch_keyword_head_value(op.core_name, …)` (`runtime.rs:8250`), i.e. head-substitution
    // into core `=`, whose `values_equal` already compares enum values.
    ReteOp {
        type_params: &[],
        rete_name: ":wat::rete::core::enum::=",
        core_name: ":wat::core::=",
        class: OpClass::Form,
        params: &[],
        ret: ParamType::Bool,
        meta: OpMeta { pure: true, deterministic: true, total: true },
    },
    ReteOp {
        type_params: &[],
        rete_name: ":wat::rete::core::enum::not=",
        core_name: ":wat::core::not=",
        class: OpClass::Form,
        params: &[],
        ret: ParamType::Bool,
        meta: OpMeta { pure: true, deterministic: true, total: true },
    },
    ReteOp {
        type_params: &[],
        rete_name: ":wat::rete::core::string::=",
        core_name: ":wat::core::=",
        class: OpClass::Alias,
        params: &[ParamType::String, ParamType::String],
        ret: ParamType::Bool,
        meta: OpMeta { pure: true, deterministic: true, total: true },
    },
    ReteOp {
        type_params: &[],
        rete_name: ":wat::rete::core::string::not=",
        core_name: ":wat::core::not=",
        class: OpClass::Alias,
        params: &[ParamType::String, ParamType::String],
        ret: ParamType::Bool,
        meta: OpMeta { pure: true, deterministic: true, total: true },
    },
    ReteOp {
        type_params: &[],
        rete_name: ":wat::rete::core::bool::=",
        core_name: ":wat::core::=",
        class: OpClass::Alias,
        params: &[ParamType::Bool, ParamType::Bool],
        ret: ParamType::Bool,
        meta: OpMeta { pure: true, deterministic: true, total: true },
    },
    ReteOp {
        type_params: &[],
        rete_name: ":wat::rete::core::bool::not=",
        core_name: ":wat::core::not=",
        class: OpClass::Alias,
        params: &[ParamType::Bool, ParamType::Bool],
        ret: ParamType::Bool,
        meta: OpMeta { pure: true, deterministic: true, total: true },
    },
    ReteOp {
        type_params: &[],
        rete_name: ":wat::rete::core::keyword::=",
        core_name: ":wat::core::=",
        class: OpClass::Alias,
        params: &[ParamType::Keyword, ParamType::Keyword],
        ret: ParamType::Bool,
        meta: OpMeta { pure: true, deterministic: true, total: true },
    },
    ReteOp {
        type_params: &[],
        rete_name: ":wat::rete::core::keyword::not=",
        core_name: ":wat::core::not=",
        class: OpClass::Alias,
        params: &[ParamType::Keyword, ParamType::Keyword],
        ret: ParamType::Bool,
        meta: OpMeta { pure: true, deterministic: true, total: true },
    },
    // ── BRIEF-f64-fallback-rows.md (2026-08-05) — the f64 arithmetic quartet. Builder's
    // ruling: "±Inf and NaN are undefined - mint the fallback rows." Mirrors the i64
    // fallback quartet's shape exactly, but the mechanism it leans on is DIFFERENT: the i64
    // family fails by RAISING (`IntegerOverflow`/`DivisionByZero`), while
    // `:wat::core::f64::{+,-,*,/}` is raw IEEE 754 with no overflow guard (`purity.rs`'s own
    // `total: false` reasoning for these core rows) and never raises on these inputs — a
    // domain failure surfaces as an `Ok` holding NaN or ±Inf instead. `dispatch_rete_op`'s
    // `OpClass::Fallback` arm now faces both paths, keyed off this row's `ret: ParamType::F64`
    // (never a runtime-value sniff). `total: true` is earned by that: for any two well-typed
    // f64 inputs this always returns some f64 (the result, or the fallback) and never raises.
    // Core itself is untouched — `:wat::core::f64::{+,-,*,/}` keep returning raw IEEE values
    // and keep their `total: false` classification; totality is bought here, at the rete row,
    // by carrying a fallback. Call shape unchanged from i64:
    // `(:wat::rete::core::f64::/ hits total :undefined 0.0)`.
    ReteOp {
        type_params: &[],
        rete_name: ":wat::rete::core::f64::+",
        core_name: ":wat::core::f64::+",
        class: OpClass::Fallback,
        params: &[ParamType::F64, ParamType::F64, ParamType::Keyword, ParamType::F64],
        ret: ParamType::F64,
        meta: OpMeta { pure: true, deterministic: true, total: true },
    },
    ReteOp {
        type_params: &[],
        rete_name: ":wat::rete::core::f64::-",
        core_name: ":wat::core::f64::-",
        class: OpClass::Fallback,
        params: &[ParamType::F64, ParamType::F64, ParamType::Keyword, ParamType::F64],
        ret: ParamType::F64,
        meta: OpMeta { pure: true, deterministic: true, total: true },
    },
    ReteOp {
        type_params: &[],
        rete_name: ":wat::rete::core::f64::*",
        core_name: ":wat::core::f64::*",
        class: OpClass::Fallback,
        params: &[ParamType::F64, ParamType::F64, ParamType::Keyword, ParamType::F64],
        ret: ParamType::F64,
        meta: OpMeta { pure: true, deterministic: true, total: true },
    },
    ReteOp {
        type_params: &[],
        rete_name: ":wat::rete::core::f64::/",
        core_name: ":wat::core::f64::/",
        class: OpClass::Fallback,
        params: &[ParamType::F64, ParamType::F64, ParamType::Keyword, ParamType::F64],
        ret: ParamType::F64,
        meta: OpMeta { pure: true, deterministic: true, total: true },
    },
    // ── DESIGN-STONE-the-vsa-seam-opens.md (2026-08-05) — the VSA seam opens: the four
    // `:wat::holon::` verbs the builder ruled pure∧det∧total on 2026-08-01
    // (`purity.rs`'s VSA-seam block) get their `RETE_OPS` rows, arming the seam R4's
    // design named and #55/#57 left unarmed. `cosine`/`dot` are `Fallback`: core returns
    // an outcome ENUM (`CosineOutcome`/`DotOutcome`), a THIRD failure mode
    // `project_holon_rete_fallback` faces beside i64's raise and f64's non-finite
    // scalar — `dispatch_rete_op` AND native `CallFallback` share that projection:
    // happy payload unwraps to the f64 this row's `ret` promises, taking the
    // caller's `:undefined` on every other variant (`Degenerate`, and both enums'
    // `DimensionMismatch`). `presence?` is `Alias` (already returns a plain
    // `bool`, needs no fallback — STOP-1, the builder's 2026-08-02 predicate ruling).
    // `coincident?` is `Redispatch`: it keeps core's `HolonAST | Vector` polymorphism (arc
    // 052/061), so it cannot be spelled as a rank-1 `params`/`ret` scheme — `params`/`ret`
    // below are unused placeholders, same convention the `foldl`/`map`/`reduce` Redispatch
    // rows above use; `check.rs`'s `infer_rete_form` re-dispatches it to the exact
    // `:wat::holon::coincident?` inference arm core's own spelling already uses.
    ReteOp {
        type_params: &[],
        rete_name: ":wat::rete::holon::cosine",
        core_name: ":wat::holon::cosine",
        class: OpClass::Fallback,
        params: &[ParamType::Holon, ParamType::Holon, ParamType::Keyword, ParamType::F64],
        ret: ParamType::F64,
        meta: OpMeta { pure: true, deterministic: true, total: true },
    },
    ReteOp {
        type_params: &[],
        rete_name: ":wat::rete::holon::dot",
        core_name: ":wat::holon::dot",
        class: OpClass::Fallback,
        params: &[ParamType::Holon, ParamType::Holon, ParamType::Keyword, ParamType::F64],
        ret: ParamType::F64,
        meta: OpMeta { pure: true, deterministic: true, total: true },
    },
    ReteOp {
        type_params: &[],
        rete_name: ":wat::rete::holon::coincident?",
        core_name: ":wat::holon::coincident?",
        class: OpClass::Redispatch,
        params: &[],
        ret: ParamType::Bool,
        meta: OpMeta { pure: true, deterministic: true, total: true },
    },
    ReteOp {
        type_params: &[],
        rete_name: ":wat::rete::holon::presence?",
        core_name: ":wat::holon::presence?",
        class: OpClass::Alias,
        params: &[ParamType::Holon, ParamType::Holon],
        ret: ParamType::Bool,
        meta: OpMeta { pure: true, deterministic: true, total: true },
    },
    // ── BRIEF-one-naming-rule-then-first-nth-to-string.md (2026-08-05) — the `to-string` trio.
    // `Alias`, same shape as round 1a: zero new logic, rete name re-dispatches to `core_name`.
    // `total: true` — GROUNDED here (not simply trusted): each is a scalar→String conversion
    // with no domain restriction (`eval_i64_to_string`/`eval_f64_to_string`/`eval_bool_to_string`,
    // `runtime.rs`), the same reasoning `i64::to-f64` already uses. Promoted into
    // `rete/purity.rs`'s `total` hand-list alongside these rows — that list did NOT already
    // contain any of the three before this strike (the brief's own claim that `bool::to-string`
    // was "already in the total list" did not hold; it was in the pure∧det list only). There is
    // no generic `num-to-string` — grounded, zero hits — so per-type is the only spelling, same
    // as the rest of this table's scalar family.
    ReteOp {
        type_params: &[],
        rete_name: ":wat::rete::core::i64::to-string",
        core_name: ":wat::core::i64::to-string",
        class: OpClass::Alias,
        params: &[ParamType::I64],
        ret: ParamType::String,
        meta: OpMeta { pure: true, deterministic: true, total: true },
    },
    ReteOp {
        type_params: &[],
        rete_name: ":wat::rete::core::f64::to-string",
        core_name: ":wat::core::f64::to-string",
        class: OpClass::Alias,
        params: &[ParamType::F64],
        ret: ParamType::String,
        meta: OpMeta { pure: true, deterministic: true, total: true },
    },
    ReteOp {
        type_params: &[],
        rete_name: ":wat::rete::core::bool::to-string",
        core_name: ":wat::core::bool::to-string",
        class: OpClass::Alias,
        params: &[ParamType::Bool],
        ret: ParamType::String,
        meta: OpMeta { pure: true, deterministic: true, total: true },
    },
    // ── BRIEF-one-naming-rule-then-first-nth-to-string.md (2026-08-05) — `first`, per container,
    // `Fallback` class. `:wat::core::first` is PARTIAL — proven by run: an empty sequence raises
    // `MalformedForm` ("sequence has 0 element(s); no element at index 0", `eval_positional_accessor`
    // in `runtime.rs`, called with `index: 0`). Per-type here is NOT the comparators' reason (that
    // per-type form DELETES a domain hole): an empty `PersistentVector` still has no first element
    // regardless of container — per-type is what makes the row SCHEMABLE (a rank-1 `TypeScheme`
    // naming ONE concrete container), which is what makes `Fallback` available at all. `total: true`
    // is EARNED by the fallback: for a well-typed container + `:undefined` fallback, this always
    // returns the first element or the fallback, never raises.
    //
    // ⚠ Call shape needed generalizing `dispatch_rete_op`'s `Fallback` arm (`runtime.rs`), which
    // was hardcoded to a 4-arg / 2-real-arg shape (the i64/f64/holon families all take TWO real
    // args before the `:undefined` marker). `first` takes exactly ONE real arg (the container), so
    // the arm's arity/slice logic is now DERIVED from `op.params.len()` (real-arg count =
    // `params.len() - 2`, marker at `params.len() - 2`, fallback at `params.len() - 1`) —
    // behavior-preserving for every existing row (all of which have `params.len() == 4`, same
    // slice as before), and now correct for a 3-param row too. See that arm's own doc for the new
    // `RuntimeErrorKind::MalformedForm { head, .. } if head == op.core_name` catch this family
    // needed — matched on `head` (which `eval_positional_accessor` sets to the exact `op` string
    // passed in — `:wat::core::first` for every one of these three rows, since core's `first` is
    // one polymorphic accessor), never a substring/message match, and distinguishable from this
    // arm's OWN `:undefined`-marker-shape validation error (whose `head` is the RETE name, not
    // `core_name` — a caller shape bug, which must still propagate, not be absorbed).
    //
    // ⛔ AFFIRMATIVELY CUT, with reasons (do not silently omit — record them here):
    //   `Tuple` — heterogeneous; element-0's type depends on the tuple's own shape, so it cannot
    //     be spelled `C<T> -> T` at all.
    //   `WatAstList` — a `Value::wat__WatAST` wrapping an AST node (R17: this exact member breaks
    //     a container abstraction); not a homogeneous sequence.
    //   `Stream` — laziness in a rule condition is a ruling nobody has made. Not a row until it is.
    //   `HashSet` — `indexable()` is already `false`; no first element by nature.
    ReteOp {
        type_params: &["T"],
        rete_name: ":wat::rete::core::PersistentVector/first",
        core_name: ":wat::core::first",
        class: OpClass::Fallback,
        params: &[ParamType::PersistentVectorOf("T"), ParamType::Keyword, ParamType::Var("T")],
        ret: ParamType::Var("T"),
        meta: OpMeta { pure: true, deterministic: true, total: true },
    },
    ReteOp {
        type_params: &["T"],
        rete_name: ":wat::rete::core::Vector/first",
        core_name: ":wat::core::first",
        class: OpClass::Fallback,
        params: &[ParamType::VectorOf("T"), ParamType::Keyword, ParamType::Var("T")],
        ret: ParamType::Var("T"),
        meta: OpMeta { pure: true, deterministic: true, total: true },
    },
    ReteOp {
        type_params: &["T"],
        rete_name: ":wat::rete::core::List/first",
        core_name: ":wat::core::first",
        class: OpClass::Fallback,
        params: &[ParamType::ListOf("T"), ParamType::Keyword, ParamType::Var("T")],
        ret: ParamType::Var("T"),
        meta: OpMeta { pure: true, deterministic: true, total: true },
    },
    // arc 278 #57 round 2 — `string::subs`, `Fallback` class. `:wat::core::string::subs` is
    // PARTIAL — proven by run: `(subs "hello" 2 99)` raises `MalformedForm` with `head:
    // ":wat::core::string::subs"` and reason "index out of range: start=2, end=99,
    // char-length=5; require 0 <= start <= end <= char-length" (`check.rs`'s registered scheme
    // is `(String, i64, i64) -> String`). This is the FIRST 3-real-arg `Fallback` row (i64/f64/
    // holon take two real args before the marker, `first` takes one); `dispatch_rete_op`'s
    // `Fallback` arm derives `marker_idx`/`fallback_idx` from `op.params.len()` and slices the
    // real args as `&args[0..marker_idx]` — genuinely arity-generic, not hardcoded to `first`'s
    // one-real-arg shape, so no runtime change was needed here. The existing `MalformedForm {
    // head, .. } if head == op.core_name` catch (added for `first`) matches this row's raise
    // exactly, since `head` here is likewise the literal `:wat::core::string::subs` `op` string.
    ReteOp {
        type_params: &[],
        rete_name: ":wat::rete::core::string::subs",
        core_name: ":wat::core::string::subs",
        class: OpClass::Fallback,
        params: &[ParamType::String, ParamType::I64, ParamType::I64, ParamType::Keyword, ParamType::String],
        ret: ParamType::String,
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

/// CLOSED by the naming rule (module doc, "The naming rule"): `rete_name = core_name` with
/// `rete::` inserted after `wat::` means every row is rooted at `:wat::rete::core::` or
/// `:wat::rete::holon::` BY CONSTRUCTION — there is no third root a core verb can have, so no new
/// container, scalar type, or module ever needs an edit here again. Two entries, not five: the
/// pre-rename table needed `i64::`/`f64::`/`string::` as SEPARATE entries only because its three
/// naming rules put those rows directly under `:wat::rete::{i64,f64,string}::` instead of
/// `:wat::rete::core::{i64,f64,string}::` — the rename moved them under `core::`, so those three
/// entries are now redundant with it (measured 2026-08-05: 17 of 57 rows were falling through this
/// list's gaps before the rename made it closed).
pub(crate) const RETE_MODULES: &[&str] = &[
    ":wat::rete::core::",
    ":wat::rete::holon::",
];

/// Look up `head`'s row, if it is a minted rete-vocabulary op. Exact match — never a prefix scan
/// (STOP-1 applies here too: a prefix match would silently "admit" any typo under a real module).
pub(crate) fn rete_op_index(head: &str) -> Option<usize> {
    static BY_NAME: OnceLock<HashMap<&'static str, usize>> = OnceLock::new();
    BY_NAME
        .get_or_init(|| {
            RETE_OPS
                .iter()
                .enumerate()
                .map(|(i, op)| (op.rete_name, i))
                .collect()
        })
        .get(head)
        .copied()
}

pub(crate) fn rete_op_for(head: &str) -> Option<&'static ReteOp> {
    rete_op_index(head).map(|i| &RETE_OPS[i])
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
/// Not consulted by `compile-condition` — the fence's Law A check is `primitive?`.
///
/// Takes a QUOTED keyword (`(:wat::rete::vocabulary-admitted? (:wat::core::quote
/// :wat::rete::core::i64::>))`), mirroring `pure?`/`deterministic?`'s own `:wat::WatAST` argument
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

// ─── Phase 1 (BRIEF-one-naming-rule-then-first-nth-to-string.md) — the naming rule's own ward ──
//
// These two tests are THE POINT of the Phase-1 rename, not a formality: the rename fixes today's
// 46 mis-derived rows; these make the three-rules drift that produced them UNREPRESENTABLE going
// forward. Both iterate `RETE_OPS` directly — never a grep over the table's own source text,
// which could pass on a comment instead of the row's actual field.
#[cfg(test)]
mod naming_rule_tests {
    use super::*;
    use std::collections::HashSet;

    /// The naming rule's own documented exception (module doc, "The naming rule"): these FOURTEEN
    /// rows' `core_name` is shared by MULTIPLE rows — the `=`/`not=` trio shares
    /// `:wat::core::=`/`:wat::core::not=` across three types; `first`'s trio shares the ONE
    /// polymorphic `:wat::core::first` across three containers (found the same way, by running
    /// this very test against the Phase-2 mint: the literal insert rule collapsed
    /// `PersistentVector/first`/`Vector/first`/`List/first` onto the identical
    /// `:wat::rete::core::first`). So the literal insert rule would collapse each trio onto one
    /// shared name — proven to break `CheckEnv`'s registration for the equality trio
    /// (`check/env.rs:284`, a raw `HashMap::insert` assuming distinct names) and, for `first`,
    /// would make `rete_op_for`'s exact-match lookup (`.find()`, first-match-wins) return only
    /// ONE of the three rows' `ParamType`/`OpMeta` for all three container spellings. A finite,
    /// named allowlist, not a silent special case.
    /// ★★ THE WALL — EVERY rete row is TOTAL, and a non-total one is unmintable.
    ///
    /// Ruled by the builder 2026-08-05: *"every rete form MUST be total — that's the entire point
    /// of this endeavor; we're getting all the ground work done such that we can compile a jump
    /// table for rete eval."* A jump table over a partial op is not a thing — there is no opcode
    /// for "raises". Every non-total row is a hole in `compiled_where`'s specification, and the
    /// vocabulary IS that specification.
    ///
    /// ⛔ WHY A GATE AND NOT A CONVENTION. Five rows carried `total: false` for three days and
    /// nothing objected. Read, their reasons were not judgements:
    ///   · `match` — "a non-exhaustive match raises `NoMatchingArm`". REFUTED TWICE: such a match
    ///     does not raise, it fails to COMPILE; and `NoMatchingArm` does not exist in this
    ///     codebase. The justification cited an error that was never implemented.
    ///   · `foldr`/`map`/`filter`/`reduce` — "extremely likely total... but no `where` row in the
    ///     corpus uses them... Flagged, not classified." The corpus fallacy: absence of a caller is
    ///     not evidence of partiality
    ///     (`[[feedback_optimize_for_the_expressivity_surface_not_the_corpus]]`). It split ONE
    ///     family down the middle while `pure`/`deterministic` took the opposite reading in the
    ///     SAME row.
    ///
    /// Minting a non-total row is now a RED BUILD rather than something someone might notice.
    /// It deliberately does NOT freeze a count — a count cannot tell "+1 new, −1 fixed" from
    /// "nothing happened", and its failure text cannot name the offender
    /// (`[[feedback_a_gate_freezes_names_never_a_count]]`). This names every offending row.
    ///
    /// If a genuinely partial op ever needs a rete surface, the answer is NOT to weaken this — it
    /// is `OpClass::Fallback`: a mandatory `:undefined <value>` is precisely how a partial core op
    /// BUYS totality (`i64::/` is partial in core and `total: true` here for exactly that reason).
    #[test]
    fn every_rete_row_is_total() {
        let partial: Vec<&str> =
            RETE_OPS.iter().filter(|op| !op.meta.total).map(|op| op.rete_name).collect();
        assert!(
            partial.is_empty(),
            "these rete rows are NOT total, and a jump table cannot dispatch a partial op — give \
             each a mandatory `:undefined` fallback (OpClass::Fallback) or do not mint it: {partial:#?}",
        );
        // NON-VACUITY: without this the assert would pass just as happily against an emptied table
        // or a renamed field — a filter that can see nothing always finds nothing wrong.
        assert!(RETE_OPS.len() > 60, "RETE_OPS looks empty — this gate would pass vacuously");
    }

    const NAMING_RULE_EXCEPTIONS: &[&str] = &[
        // #57 — enum equality joins the shared-core_name trio, making it a QUARTET (and this
        // list ELEVEN). Same reason as its three siblings: `:wat::core::=` now serves four rete
        // rows, so the literal insert rule would collapse them onto one name.
        ":wat::rete::core::enum::=",
        ":wat::rete::core::enum::not=",
        ":wat::rete::core::string::=",
        ":wat::rete::core::string::not=",
        ":wat::rete::core::bool::=",
        ":wat::rete::core::bool::not=",
        ":wat::rete::core::keyword::=",
        ":wat::rete::core::keyword::not=",
        ":wat::rete::core::PersistentVector/first",
        ":wat::rete::core::Vector/first",
        ":wat::rete::core::List/first",
        // 2026-08-28 — the Tuple accessors. `Tuple/first` makes `:wat::core::first`'s group a
        // QUARTET (same reason as the trio: one core_name, several rows). `Tuple/second` and
        // `Tuple/third` are here for the OTHER half of the rule — a per-type rete name that does
        // not derive literally from its `core_name` — and they are the sole rows on their cores
        // today. A future `Vector/second` would join them rather than displace them.
        ":wat::rete::core::Tuple/first",
        ":wat::rete::core::Tuple/second",
        ":wat::rete::core::Tuple/third",
    ];

    /// ★★ Every row satisfies [`rete_vocabulary_admitted`] over its OWN `rete_name` — the
    /// admission test's permanent ward. Measured 2026-08-05 (pre-rename): 17 of 57 rows failed
    /// this, working only by accident (`head_ok` falls through into `intrinsic_meta`, which
    /// finds them anyway via `rete_op_for`). Arming this as a real assertion over the live table
    /// makes that accident impossible to reintroduce — a future row minted outside an admitted
    /// module goes red HERE, not silently through the fallthrough.
    #[test]
    fn every_row_is_admitted() {
        for op in RETE_OPS {
            assert!(
                rete_vocabulary_admitted(op.rete_name),
                "row {:?} (core_name {:?}) is not admitted by RETE_MODULES {:?} — the naming rule is supposed to make this impossible",
                op.rete_name, op.core_name, RETE_MODULES,
            );
        }
    }

    /// ★ The rule is ENFORCED, not just applied once by hand: for every row outside the
    /// documented exception, `rete_name == core_name` with `rete::` inserted immediately after
    /// `wat::`. This is the actual extirpare rung — with it, the three-rules drift that produced
    /// today's 46 mis-derived names cannot recur, because a row violating the rule fails HERE.
    #[test]
    fn rete_name_is_core_name_with_rete_inserted_after_wat() {
        for op in RETE_OPS {
            let expected = op.core_name.replacen(":wat::", ":wat::rete::", 1);
            if NAMING_RULE_EXCEPTIONS.contains(&op.rete_name) {
                // The documented exception: verify it stays a genuine exception (the naive rule
                // really would collide) rather than a name that quietly stopped needing one.
                assert_ne!(
                    op.rete_name, expected.as_str(),
                    "row {:?} is listed as a naming-rule exception but its rete_name now EQUALS \
                     the literal rule's output — it no longer needs the exception; remove it from \
                     NAMING_RULE_EXCEPTIONS",
                    op.rete_name,
                );
            } else {
                assert_eq!(
                    op.rete_name, expected.as_str(),
                    "row {:?} (core_name {:?}) violates the naming rule: expected {:?}",
                    op.rete_name, op.core_name, expected,
                );
            }
        }
    }

    /// The exception list itself is exactly the eleven rows the module doc names — no more, no
    /// fewer. Catches the exception set silently growing (a real collision nobody explained) or
    /// shrinking without the corresponding row being deleted.
    ///
    /// The exception NAMES are frozen (this list) AND counted. A silent
    /// `+1 new, −1 fixed` fails the equality on the slice, not only the length.
    #[test]
    fn naming_rule_exceptions_are_exactly_the_documented_fourteen() {
        assert_eq!(NAMING_RULE_EXCEPTIONS.len(), 14);
        let mut frozen: Vec<&str> = NAMING_RULE_EXCEPTIONS.to_vec();
        frozen.sort_unstable();
        let mut live: Vec<&str> = NAMING_RULE_EXCEPTIONS
            .iter()
            .copied()
            .filter(|name| RETE_OPS.iter().any(|op| op.rete_name == *name))
            .collect();
        live.sort_unstable();
        assert_eq!(
            frozen, live,
            "exception names must exist in RETE_OPS — stale or missing entry"
        );
        for &name in NAMING_RULE_EXCEPTIONS {
            assert!(
                RETE_OPS.iter().any(|op| op.rete_name == name),
                "exception {name:?} names no row in RETE_OPS — stale entry"
            );
        }
    }

    /// Deeper invariant the naming rule exists to protect: no two rows ever share a `rete_name`.
    /// A collision silently drops a row's `TypeScheme` in `CheckEnv`'s registration (a raw
    /// `HashMap::insert`, `check/env.rs:284`) with no error anywhere — this is the one check that
    /// would have caught the exception rows' collision even without knowing the naming rule.
    #[test]
    fn every_rete_name_is_unique() {
        let mut seen = HashSet::new();
        for op in RETE_OPS {
            assert!(seen.insert(op.rete_name), "duplicate rete_name: {:?}", op.rete_name);
        }
    }
}
