//! `:wat::keyword::*` intrinsics — arc 255 Stone E-iv (`keyword` gets its home), REGISTRY half
//! of the two-home split `intrinsic/string.rs` established.
//!
//! The 5 Rust-implemented `:wat::keyword::*` verbs, off the `:wat::core::` junk-drawer
//! (`:wat::core::keyword/*`) onto their own top-level namespace — from the DISPATCH TABLE, not
//! corpus usage (E-ii's lesson: a migration census cannot see a verb nobody calls in the corpus,
//! and an unseen verb strands at retirement).
//!
//! ★ WHY `keyword` TAKES THE PLAIN, UNMARKED `:wat::keyword::` NAME — `keyword` is a SCALAR
//! type, and it is the LAST scalar without a home: `bigint · bytes · char · f64 · i64 ·
//! rational · regex · string · time · uuid` all already live at their own plain top-level
//! namespace. There is no marked/unmarked question here (contrast `hashset`/`linkedlist`,
//! E-iii) — there is only ONE flavor of `keyword`, so nothing is reserved against the plain
//! name. See
//! `docs/arc/2026/06/255-builtin-registry/BRIEF-STONE-E-iv-keyword-gets-its-home.md`.
//!
//! **Two homes** (same split as the string carve): this file is the REGISTRY home — dispatch
//! shim + `///` preamble only. The algorithms these handlers call
//! (`crate::runtime::eval_keyword_to_string`, `crate::runtime::eval_keyword_from_string`,
//! `crate::edn::render::eval_keyword_to_symbol`, `crate::edn::render::eval_keyword_to_type_form`,
//! `crate::edn::render::eval_keyword_to_type_form_colon`) are UNTOUCHED by this stone — they
//! already lived in `runtime.rs`/`edn/render.rs`, and stay there (name-only rename of the
//! DISPATCH ROUTE, not the implementation).
//!
//! ★ FOUR of the five (`from-string`, `to-symbol`, `to-type-form`, `to-type-form-colon`) are
//! PRODUCERS: their handlers below return `Result<TrackedValue, EvalBreak>` directly (not
//! `Result<Value, EvalBreak>`), forwarding the `TrackedValue` their algorithm fn already builds
//! — carrying `Provenance::RuntimeBuilt { producer, call_span }` — un-rewrapped. Arc 255 Stone
//! G gave `NativeHandler` a `TrackedValue`-returning signature with a sniff (mirroring the
//! macro's existing `SniffedArgs` on the argument side) precisely so a registry-routed producer
//! could keep stamping its own provenance instead of being downgraded to `Provenance::Unknown`
//! by the shim's default arm — restoring what Stone E-iv recorded as an open regression.
//! `to-string` (the fifth verb) is a plain Probe, not a producer, and keeps the bare-`Value`
//! shape — the shim wraps it as `Provenance::Unknown`, same as any other non-producer handler.
//!
//! Both the old `:wat::core::keyword/*` spelling and this new one are LIVE during Phase 1/2 of
//! this stone (register, then move the corpus by codemod); Phase 3 retires the old spelling,
//! leaving this file as the ONLY dispatch path for `to-string` (reached via
//! `crate::intrinsic::registry().lookup`, consulted BEFORE `runtime.rs`'s literal table,
//! `DESIGN-STONE-255.1c-guard-hoist.md`) and the ONLY producer path for the other four (reached
//! the same way, since their old `dispatch_keyword_head` producer arms are deleted in Phase 3).
//!
//! ⚠ The bare TYPE `:wat::core::keyword` does NOT move (STOP-3 — arc 251's `wat.type/keyword`):
//! only the 5 slash-verbs below are this stone's territory. The trailing `/` is the whole
//! discrimination.

use wat_macros::wat_intrinsic;

use crate::ast::WatAST;
use crate::span::Span;
use crate::value::{Environment, EvalBreak, SymbolTable, TrackedValue, Value};

// ─── the 5 verbs ────────────────────────────────────────────────────────────

/// `(:wat::keyword::to-string k)` → the text of keyword `k`, without its leading colon sigil.
///
/// @added         1.0.0
/// @Purity        Pure
/// @Determinism   Deterministic
/// @Category      Probe
/// @arg     k :wat::core::keyword the keyword probed
/// @ret     :wat::core::String the text of `k`, without the leading colon
/// @example (:wat::keyword::to-string :foo) #=> "foo"
/// @example (:wat::keyword::to-string :wat::core::i64) #=> "wat::core::i64"
/// @see     :wat::keyword::from-string
#[wat_intrinsic(":wat::keyword::to-string")]
pub(crate) fn eval_keyword_to_string_home(
    k: &WatAST,
    env: &Environment,
    sym: &SymbolTable,
    span: &Span,
) -> Result<Value, EvalBreak> {
    crate::runtime::eval_keyword_to_string(std::slice::from_ref(k), span, env, sym)
}

/// `(:wat::keyword::from-string s)` → a keyword `Value` built from text `s`. `s` MUST NOT start
/// with `:` (the sigil, not part of the payload) — raises a diagnostic naming the offending
/// input otherwise. Round-trips with `to-string`: `(from-string (to-string k)) == k`.
///
/// @added         1.0.0
/// @Purity        Pure
/// @Determinism   Deterministic
/// @Category      Transform
/// @arg     s :wat::core::String the colon-free keyword text
/// @ret     :wat::core::keyword a keyword built from `s`
/// @example (:wat::keyword::from-string "foo") #=> :foo
/// @example (:wat::keyword::from-string "wat::core::i64") #=> :wat::core::i64
/// @see     :wat::keyword::to-string
#[wat_intrinsic(":wat::keyword::from-string")]
pub(crate) fn eval_keyword_from_string_home(
    s: &WatAST,
    env: &Environment,
    sym: &SymbolTable,
    span: &Span,
) -> Result<TrackedValue, EvalBreak> {
    crate::runtime::eval_keyword_from_string(std::slice::from_ref(s), span, env, sym)
}

/// `(:wat::keyword::to-symbol kw-node)` → convert a wat rust-scheme call-head Keyword FORM node
/// into a faithful-Clojure Symbol FORM node (the kind change — Keyword to Symbol — IS the
/// inversion: a call head is a symbol in Clojure, never a keyword). Raises if `kw-node` is not a
/// convertible head/reference keyword (a bare data keyword or a namespace-prefix marker).
///
/// @added         1.0.0
/// @Purity        Pure
/// @Determinism   Deterministic
/// @Category      Transform
/// @arg     kw_node :wat::WatAST the Keyword form node converted
/// @ret     :wat::WatAST a Symbol form node carrying the faithful-Clojure spelling
/// @example (:wat::keyword::to-symbol (:wat::core::keyword-node ":wat::core::Bytes::to-hex")) #=> wat.core.Bytes/to-hex
/// @see     :wat::keyword::to-type-form
#[wat_intrinsic(":wat::keyword::to-symbol")]
pub(crate) fn eval_keyword_to_symbol_home(
    kw_node: &WatAST,
    env: &Environment,
    sym: &SymbolTable,
    span: &Span,
) -> Result<TrackedValue, EvalBreak> {
    crate::edn::render::eval_keyword_to_symbol(std::slice::from_ref(kw_node), span, env, sym)
        .map_err(Into::into)
}

/// `(:wat::keyword::to-type-form kw-node)` → convert an old rust-scheme TYPE keyword
/// (`:wat::core::Vector<wat::core::i64>`) into the faithful-Clojure type FORM
/// (`(wat.type/Vector [wat.type/i64])`).
///
/// @added         1.0.0
/// @Purity        Pure
/// @Determinism   Deterministic
/// @Category      Transform
/// @arg     kw_node :wat::WatAST the type Keyword form node rendered
/// @ret     :wat::WatAST the faithful-Clojure type form
/// @example (:wat::keyword::to-type-form (:wat::core::keyword-node ":wat::core::i64")) #=> wat.type/i64
/// @see     :wat::keyword::to-type-form-colon
#[wat_intrinsic(":wat::keyword::to-type-form")]
pub(crate) fn eval_keyword_to_type_form_home(
    kw_node: &WatAST,
    env: &Environment,
    sym: &SymbolTable,
    span: &Span,
) -> Result<TrackedValue, EvalBreak> {
    crate::edn::render::eval_keyword_to_type_form(std::slice::from_ref(kw_node), span, env, sym)
        .map_err(Into::into)
}

/// `(:wat::keyword::to-type-form-colon kw-node)` — Colon-mode sibling of `to-type-form`: same
/// parse+render pipeline, the rust-ish `:wat::core::` head spelling instead of the Clojure
/// `wat.type/` flip (`:wat::core::Vector<wat::core::i64>` → `(:wat::core::Vector :- [:wat::core::i64])`).
///
/// @added         1.0.0
/// @Purity        Pure
/// @Determinism   Deterministic
/// @Category      Transform
/// @arg     kw_node :wat::WatAST the type Keyword form node rendered
/// @ret     :wat::WatAST the Colon-mode type form
/// @example (:wat::keyword::to-type-form-colon (:wat::core::keyword-node ":wat::core::i64")) #=> :wat.core/i64
/// @see     :wat::keyword::to-type-form
#[wat_intrinsic(":wat::keyword::to-type-form-colon")]
pub(crate) fn eval_keyword_to_type_form_colon_home(
    kw_node: &WatAST,
    env: &Environment,
    sym: &SymbolTable,
    span: &Span,
) -> Result<TrackedValue, EvalBreak> {
    crate::edn::render::eval_keyword_to_type_form_colon(std::slice::from_ref(kw_node), span, env, sym)
        .map_err(Into::into)
}
