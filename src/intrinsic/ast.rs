//! `:wat::core::{ast-*, symbol-node, keyword-node, read-string, fresh-symbol}` intrinsics —
//! arc 255 Stone HOME-12, the AST surface's REGISTRY home.
//!
//! BRIEF: `docs/arc/2026/06/255-builtin-registry/BRIEF-STONE-HOME-12-the-AST-surface-gets-a-registry-home.md`.
//! PRIOR ART / SAME SHAPE: `src/intrinsic/edn.rs` (HOME-11, one commit older, same producer
//! question over the same source file).
//!
//! Ten verbs — the homoiconic read/write spine (`read-string`, `ast->source`), the AST↔walkable
//! bridge (`ast->children`), node recognition (`ast-kind`, `ast-name`, `ast-span`,
//! `ast-end-span`), node construction (`symbol-node`, `keyword-node`), and capture-proof gensym
//! (`fresh-symbol`) — from `runtime.rs` literal dispatch arms into `#[wat_intrinsic]` handlers
//! here. **Nothing is renamed** — `:wat::core::` is already the final spelling (pure
//! re-registration, same shape as HOME-8/10/11). No codemod, no `RetirementEntry` row, no `.wat`
//! corpus touch.
//!
//! ## The one contract decision — ALL TEN ARE PRODUCERS
//!
//! Measured: every one of the ten stamps `Provenance::RuntimeBuilt { producer, .. }` in its body
//! (one construction site apiece, all in `src/edn/render.rs`) — they mint AST values (or, for
//! `read-string`, a `ReadOutcome` wrapping one) and record which verb made them. Every handler
//! below therefore returns `Result<TrackedValue, EvalBreak>` un-rewrapped, exactly the shape
//! `src/intrinsic/keyword.rs` established and arc 255 Stone G's `sniff_return` requires to keep a
//! registry-routed producer's own stamp alive instead of the shim's default
//! `Provenance::Unknown` rewrap.
//!
//! This is a SHARPER version of HOME-11's risk: there 3 of 13 were producers, with ten plain
//! shims to fall back on. Here it is 10 of 10 — there is no non-producer half, and a bare-`Value`
//! return on any one of these would silently degrade its stamp to `Provenance::SymbolBound` with
//! every test green, exactly what Stone E-iv did to four keyword verbs before Stone G made the
//! `TrackedValue`-returning shape expressible.
//!
//! ## Two homes, same split HOME-5/HOME-11 established
//!
//! This file is the REGISTRY home — dispatch shim + `///` doc preamble only. The algorithms
//! these handlers call (`crate::edn::render::eval_read_string`, `eval_ast_to_source`,
//! `eval_ast_children`, `eval_ast_kind`, `eval_ast_name`, `eval_ast_span`, `eval_ast_end_span`,
//! `eval_symbol_node`, `eval_fresh_symbol`, `eval_keyword_node`) are UNTOUCHED by this stone;
//! they already live in `src/edn/render.rs` and stay there. **This is itself a known misfiling**
//! (all ten handlers sit in the EDN renderer, beside 7 genuine EDN handlers, and there is no
//! `src/ast/`) — a FILE-domain carve, reported but deliberately not acted on (STOP-5; see the
//! rider's report for HOME-12).
//!
//! ## Six neighbours that are NOT this stone's territory
//!
//! `macroexpand`, `macroexpand-1`, `quasiquote`, `struct->form`, `forms`, `ann-form` are
//! AST-shaped and sit in the same dispatch region, but they are registered SPECIAL FORMS
//! (`src/special_forms.rs`'s `REGISTRY: HashMap<String, SpecialFormDef>`, each with an arity
//! signature) — a different contract (`#[wat_special_form]`), not an intrinsic. Not carved here.

use wat_macros::wat_intrinsic;

use crate::ast::WatAST;
use crate::span::Span;
use crate::value::{Environment, EvalBreak, SymbolTable, TrackedValue};

// ─── parse: the homoiconic read side (1 producer) ──────────────────────────

/// `(:wat::core::read-string s)` → `:wat::core::ReadOutcome`. The homoiconic `read`: wat SOURCE
/// text → forms-as-data (a `:wat::WatAST` List of top-level forms), WITHOUT eval. Distinct from
/// `:wat::edn::read` (the EDN parser) — this runs wat's OWN source parser, the read side of the
/// wat-to-wat fixer's read→transform→write cycle. TOTAL — a parse failure is the matchable
/// `ReadOutcome::Malformed[cause]`, never a raise: wat has no try/catch, so a raise here would be
/// unsurvivable by construction, and at a REPL one stray control byte used to end the session.
///
/// @added         1.0.0
/// @Purity        Pure
/// @Determinism   Deterministic
/// @Total         Unreviewed
/// @Category      Transform
/// @arg     s :wat::core::String the wat source text parsed
/// @ret     :wat::core::ReadOutcome `Forms[ast]` on success, `Malformed[cause]` otherwise
/// @example (:wat::core::match (:wat::core::read-string "foo") ((:wat::core::ReadOutcome::Forms ast) (:wat::core::ast-name (:wat::core::first (:wat::core::ast->children ast)))) ((:wat::core::ReadOutcome::Malformed _) "parse failed")) #=> "foo"
/// @see     :wat::core::ast->source
/// @see     :wat::core::ast->children
#[wat_intrinsic(":wat::core::read-string")]
pub(crate) fn eval_read_string_home(
    s: &WatAST,
    env: &Environment,
    sym: &SymbolTable,
    span: &Span,
) -> Result<TrackedValue, EvalBreak> {
    crate::edn::render::eval_read_string(std::slice::from_ref(s), span, env, sym).map_err(Into::into)
}

// ─── the AST↔walkable bridge + the write side (2 producers) ────────────────

/// `(:wat::core::ast->source ast)` → `:wat::core::String`. The sift Predicate's enabling
/// primitive (arc 278 Stone 1): serializes a `:wat::WatAST` node back to VERBATIM wat source —
/// every `::` keyword/symbol printed UNTOUCHED. Deliberately NOT `write-forms` (which dials
/// `::` → `.`) — `(read-string (ast->source form))` reproduces the SAME form, `::` notation
/// surviving round-trip untranslated.
///
/// @added         1.0.0
/// @Purity        Pure
/// @Determinism   Deterministic
/// @Total         Unreviewed
/// @Category      Transform
/// @arg     ast :wat::WatAST the node serialized
/// @ret     :wat::core::String the verbatim `::`-faithful source text
/// @example (:wat::core::ast->source (:wat::core::symbol-node "foo")) #=> "foo"
/// @see     :wat::core::read-string
/// @see     :wat::core::ast->children
#[wat_intrinsic(":wat::core::ast->source")]
pub(crate) fn eval_ast_to_source_home(
    ast: &WatAST,
    env: &Environment,
    sym: &SymbolTable,
    span: &Span,
) -> Result<TrackedValue, EvalBreak> {
    crate::edn::render::eval_ast_to_source(std::slice::from_ref(ast), span, env, sym)
        .map_err(Into::into)
}

/// `(:wat::core::ast->children ast)` → `(:wat::core::Vector :- [:wat::WatAST])`. The AST↔walkable
/// bridge (arc 251.5a-iii): decomposes a node into a Vector of its children — the SAME walkable
/// shape `:wat::core::forms` produces, so the existing `first`/`rest`/`map` collection vocab
/// applies for free. A List/Vector/Set node yields its items; a Map yields its keys and values
/// interleaved; a leaf (Symbol/Keyword/literal) yields the empty vector.
///
/// @added         1.0.0
/// @Purity        Pure
/// @Determinism   Deterministic
/// @Total         Unreviewed
/// @Category      Transform
/// @arg     ast :wat::WatAST the node decomposed
/// @ret     (:wat::core::Vector :- [:wat::WatAST]) `ast`'s children, in order (empty for a leaf)
/// @example (:wat::core::ast->children (:wat::core::symbol-node "foo")) #=> (:wat::core::Vector :- [:wat::WatAST])
/// @see     :wat::core::ast->source
/// @see     :wat::core::ast-kind
#[wat_intrinsic(":wat::core::ast->children")]
pub(crate) fn eval_ast_children_home(
    ast: &WatAST,
    env: &Environment,
    sym: &SymbolTable,
    span: &Span,
) -> Result<TrackedValue, EvalBreak> {
    crate::edn::render::eval_ast_children(std::slice::from_ref(ast), span, env, sym)
        .map_err(Into::into)
}

// ─── node recognition (4 producers) ────────────────────────────────────────

/// `(:wat::core::ast-kind ast)` → `:wat::core::String`. Total kind discriminant (arc
/// 251.5a-v): one of `"int"`, `"float"`, `"rational"`, `"bigint"`, `"char"`, `"bool"`,
/// `"string"`, `"nil"`, `"keyword"`, `"symbol"`, `"list"`, `"vector"`, `"set"`, `"map"`.
///
/// @added         1.0.0
/// @Purity        Pure
/// @Determinism   Deterministic
/// @Total         Unreviewed
/// @Category      Probe
/// @arg     ast :wat::WatAST the node probed
/// @ret     :wat::core::String `ast`'s kind discriminant
/// @example (:wat::core::ast-kind (:wat::core::symbol-node "foo")) #=> "symbol"
/// @example (:wat::core::ast-kind (:wat::core::keyword-node ":foo")) #=> "keyword"
/// @see     :wat::core::ast-name
#[wat_intrinsic(":wat::core::ast-kind")]
pub(crate) fn eval_ast_kind_home(
    ast: &WatAST,
    env: &Environment,
    sym: &SymbolTable,
    span: &Span,
) -> Result<TrackedValue, EvalBreak> {
    crate::edn::render::eval_ast_kind(std::slice::from_ref(ast), span, env, sym).map_err(Into::into)
}

/// `(:wat::core::ast-name ast)` → `:wat::core::String`. Verbatim token text of a Symbol/Keyword
/// node (arc 251.5a-v), or the unquoted string VALUE of a StringLit node (arc 279 — the `format`
/// macro needs a template's literal text at expand time). Raises on any other node kind.
///
/// @added         1.0.0
/// @Purity        Pure
/// @Determinism   Deterministic
/// @Total         Unreviewed
/// @Category      Probe
/// @arg     ast :wat::WatAST the Symbol, Keyword, or StringLit node probed
/// @ret     :wat::core::String `ast`'s verbatim name/text
/// @example (:wat::core::ast-name (:wat::core::symbol-node "foo")) #=> "foo"
/// @example (:wat::core::ast-name (:wat::core::keyword-node ":foo")) #=> ":foo"
/// @see     :wat::core::ast-kind
#[wat_intrinsic(":wat::core::ast-name")]
pub(crate) fn eval_ast_name_home(
    ast: &WatAST,
    env: &Environment,
    sym: &SymbolTable,
    span: &Span,
) -> Result<TrackedValue, EvalBreak> {
    crate::edn::render::eval_ast_name(std::slice::from_ref(ast), span, env, sym).map_err(Into::into)
}

/// `(:wat::core::ast-span ast)` → `(:wat::core::HashMap :- [:wat::core::keyword :wat::core::i64])`.
/// Source START location of any node (Stone 251.5 / Slice 4.2a), `{:line N :col N}`. `:file` is
/// intentionally excluded — the single-file codemod consumer holds its own path and threads it
/// directly, not because file is unknowable.
///
/// @added         1.0.0
/// @Purity        Pure
/// @Determinism   Deterministic
/// @Total         Unreviewed
/// @Category      Projection
/// @arg     ast :wat::WatAST the node probed
/// @ret     (:wat::core::HashMap :- [:wat::core::keyword :wat::core::i64]) `{:line N :col N}`, `ast`'s start location
/// @example (:wat::core::ast-span (:wat::core::symbol-node "foo")) #=> (:wat::core::ast-span (:wat::core::symbol-node "bar"))
/// @see     :wat::core::ast-end-span
#[wat_intrinsic(":wat::core::ast-span")]
pub(crate) fn eval_ast_span_home(
    ast: &WatAST,
    env: &Environment,
    sym: &SymbolTable,
    span: &Span,
) -> Result<TrackedValue, EvalBreak> {
    crate::edn::render::eval_ast_span(std::slice::from_ref(ast), span, env, sym).map_err(Into::into)
}

/// `(:wat::core::ast-end-span ast)` → `(:wat::core::HashMap :- [:wat::core::keyword :wat::core::i64])`.
/// Source END location of any node (arc 281) — the position ONE char past the node's last char
/// (for `(a b c)`, col 8, just after the `)`). Symmetric twin of `ast-span`.
///
/// @added         1.0.0
/// @Purity        Pure
/// @Determinism   Deterministic
/// @Total         Unreviewed
/// @Category      Projection
/// @arg     ast :wat::WatAST the node probed
/// @ret     (:wat::core::HashMap :- [:wat::core::keyword :wat::core::i64]) `{:line N :col N}`, `ast`'s end location
/// @example (:wat::core::ast-end-span (:wat::core::symbol-node "foo")) #=> (:wat::core::ast-end-span (:wat::core::symbol-node "bar"))
/// @see     :wat::core::ast-span
#[wat_intrinsic(":wat::core::ast-end-span")]
pub(crate) fn eval_ast_end_span_home(
    ast: &WatAST,
    env: &Environment,
    sym: &SymbolTable,
    span: &Span,
) -> Result<TrackedValue, EvalBreak> {
    crate::edn::render::eval_ast_end_span(std::slice::from_ref(ast), span, env, sym).map_err(Into::into)
}

// ─── node construction (2 producers) ───────────────────────────────────────

/// `(:wat::core::symbol-node s)` → `:wat::WatAST`. Construct a bare Symbol node (arc 251.5a-v)
/// carrying `s`'s text, with an empty scope set (identical in shape to a Symbol the parser would
/// produce from source text `s`). Raises if `s` is an angle-bracketed generic-looking name
/// (`angle_type_head_in_name`) — arc 109 annihilated the angle bracket at the lexer, so no legal
/// program can produce that token; this door refuses to mint one out-of-band.
///
/// @added         1.0.0
/// @Purity        Pure
/// @Determinism   Deterministic
/// @Total         Unreviewed
/// @Category      Transform
/// @arg     s :wat::core::String the bare symbol text
/// @ret     :wat::WatAST a Symbol node carrying `s`
/// @example (:wat::core::ast-kind (:wat::core::symbol-node "foo")) #=> "symbol"
/// @see     :wat::core::keyword-node
/// @see     :wat::core::fresh-symbol
#[wat_intrinsic(":wat::core::symbol-node")]
pub(crate) fn eval_symbol_node_home(
    s: &WatAST,
    env: &Environment,
    sym: &SymbolTable,
    span: &Span,
) -> Result<TrackedValue, EvalBreak> {
    crate::edn::render::eval_symbol_node(std::slice::from_ref(s), span, env, sym).map_err(Into::into)
}

/// `(:wat::core::keyword-node s)` → `:wat::WatAST`. Construct a Keyword node (arc 251.5a-v)
/// carrying `s`'s text; `s` MUST start with `:`. Raises on a missing `:` prefix or on an
/// angle-bracketed generic-looking name (`angle_type_head_in_name`), same door as `symbol-node`.
///
/// @added         1.0.0
/// @Purity        Pure
/// @Determinism   Deterministic
/// @Total         Unreviewed
/// @Category      Transform
/// @arg     s :wat::core::String the `:`-prefixed keyword text
/// @ret     :wat::WatAST a Keyword node carrying `s`
/// @example (:wat::core::ast-kind (:wat::core::keyword-node ":foo")) #=> "keyword"
/// @see     :wat::core::symbol-node
#[wat_intrinsic(":wat::core::keyword-node")]
pub(crate) fn eval_keyword_node_home(
    s: &WatAST,
    env: &Environment,
    sym: &SymbolTable,
    span: &Span,
) -> Result<TrackedValue, EvalBreak> {
    crate::edn::render::eval_keyword_node(std::slice::from_ref(s), span, env, sym).map_err(Into::into)
}

// ─── capture-proof gensym (1 producer, Nondeterministic) ───────────────────

/// `(:wat::core::fresh-symbol base)` → `:wat::WatAST`. Construct a capture-proof Symbol node
/// (arc 274 Stone 274.1). Like `symbol-node` but adds a fresh, globally-unique `ScopeId`
/// (`add_scope(fresh_scope())`, a process-wide atomic counter) to the `Identifier` — the
/// resulting symbol's `env_key` is distinct from any user symbol of the same base name (which
/// carries an empty scope set). A computing macro uses the SAME returned value for both the
/// binder and every reference, so they share the unique scope and resolve to each other, never
/// to a user variable — capture is structurally impossible by construction.
///
/// NONDETERMINISTIC (same shape as `:wat::uuid::v4`): the SAME `base` argument mints a
/// DIFFERENT symbol on every call, because the scope counter advances each time — no I/O, no
/// observable side effect (`@Purity Pure`), but the result cannot be pinned across calls, only
/// its shape can (`ast-kind` = `"symbol"`, `ast-name` = `base`).
///
/// @added         1.0.0
/// @Purity        Pure
/// @Determinism   Nondeterministic
/// @Total         Unreviewed
/// @Category      Transform
/// @arg     base :wat::core::String the base name the fresh symbol's text carries
/// @ret     :wat::WatAST a Symbol node carrying `base`'s text and a fresh, unique scope
/// @example-norun (:wat::core::fresh-symbol "x") #=> a Symbol node whose identifier carries base "x" plus a fresh globally-unique ScopeId — a different, capture-proof symbol on every call, never equal to a bare user "x"
/// @see     :wat::core::symbol-node
#[wat_intrinsic(":wat::core::fresh-symbol")]
pub(crate) fn eval_fresh_symbol_home(
    base: &WatAST,
    env: &Environment,
    sym: &SymbolTable,
    span: &Span,
) -> Result<TrackedValue, EvalBreak> {
    crate::edn::render::eval_fresh_symbol(std::slice::from_ref(base), span, env, sym).map_err(Into::into)
}
