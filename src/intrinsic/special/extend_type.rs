//! Special-form doc entry for `:wat::core::extend-type` — arc 255 Stone 1c-d, one of "the
//! declaration three." 157 corpus call sites — the largest of the three by far, and the one
//! whose declare-time recognizer (`parse_extend_type_form`) is genuinely reused at BOTH the
//! `role = declare` and `role = check` call sites (see that fn's own doc for why one function
//! carries both annotations rather than two).

use wat_macros::wat_special_form;

/// Register that type `:T` implements protocol `:P`: `(:wat::core::extend-type :T :P (method-1
/// [self ...] body) ...)`. Each impl clause is parsed as a `defclause` clause body (argspec
/// without type annotations on `self`/the other binders, then body); `register_extend_type_
/// methods` (and siblings, `src/declare/register.rs`) register each impl as a real `Function`
/// in `sym.functions` under `<T>/<method>`, and `env.register_subtype(&T, &P, ...)` records the
/// `(T, P)` edge in the type lattice.
///
/// **Category ground — ★★★ argued, not assumed, per Stone 1a-δ's own lesson.** `:Declaration`'s
/// prose ("registers a program-level entity ... visible to everything after it") is not just a
/// face-value fit here — it is confirmed by what the form measurably does to BOTH the type
/// lattice and the symbol table: `env.register_subtype` (`src/types.rs:3918`) writes the `(T,
/// P)` edge that `is_subtype`'s exact-string lookup and `transport_edge_keys`/
/// `transport_satisfier_heads` (`check.rs`) consult for the rest of the program, and
/// `register_extend_type_surface_impls`/`register_extend_type_methods`
/// (`src/declare/register.rs`) register every method impl into `sym.functions`, visible to
/// every call after it — a strictly LARGER registration footprint than `def`'s single name.
///
/// The contrast that rules OUT the other three candidates, measured rather than assumed:
/// - NOT `:Splice` — Stone 1a-δ's own test (does the form replace itself with N forms and not
///   survive?) fails here: `splice_type_decls`'s `:wat::core::extend-type` arm
///   (`src/types.rs:3918`) ends `Ok(WatAST::List(items, span))` — the form is KEPT verbatim in
///   the form stream (downstream `check.rs`'s `infer_list` arm and `collect_splice_defs_ctx`
///   both still need to see it), unlike a loader, which vanishes and is replaced. This is also
///   NOT the `classify_type_decl` family's shape (`structtype`/`defenum`/…), whose matched forms
///   ARE stripped from the residue once registered — `extend-type` registers AND persists, a
///   third shape distinct from both.
/// - NOT `:CheckGate` — `:CheckGate`'s own prose is "refuses a call site at check time ... the
///   runtime body is identity or otherwise incidental." `extend-type` does the opposite: its
///   whole purpose IS the registration (the edge + N methods); nothing about it merely refuses.
/// - NOT `:Ambient` — what is registered is a NAME-addressed pair (the canonical key
///   `"extend:<P>:<T>"`, and each method under `<T>/<method>`), looked up by name, not a bare
///   process-global flag no value addresses (`:wat::config::set-redef!`'s own contrast case).
/// `Declaration`.
///
/// **Purity ground —** measured directly, same method as `def`'s row: `:wat::core::extend-type`
/// has NO entry in `dispatch_keyword_head`/`dispatch_keyword_head_value`'s match and no
/// `NativeHandler` — the DESIGN's own table confirms "NONE" for its dispatch arm. All four
/// consumers of `@Purity` ask a RUNTIME question this form has no runtime to answer: `Pure`
/// would demand a runnable `@example` of a verb that cannot be run; `Effectful` would claim an
/// effect there is no call to have; `Preserving` would claim sub-forms evaluated BY this form
/// itself — the method bodies are stored into `Clause`s and registered as `Function`s, run
/// later per call, never evaluated by `extend-type` at its own processing. `Unevaluated`.
///
/// **Determinism ground —** the same `extend-type` form, parsed against the same preceding
/// type/method declarations, always produces the identical `ExtendDef` (type name, protocol
/// name, impl clauses) — no clock, no entropy, no gensym anywhere on
/// `parse_extend_type_form`'s path. `Deterministic`.
///
/// **Totality ground —** `parse_extend_type_form` is measured NOT defined on every input: fewer
/// than 3 items, a target/protocol slot that is neither a keyword nor a parametric type form,
/// and other shape mismatches all raise `RuntimeErrorKind::MalformedForm` instead of returning
/// an `ExtendDef`; downstream, `env.register_subtype` can also raise `CyclicSubtype`
/// (`types.rs`'s own comment on the sibling `derive` arm) — a raise the freeze pipeline
/// propagates as a hard failure, never a value a caller matches on. Same reasoning
/// `:wat::i64::/`'s own `@Totality Partial` was ruled on
/// (`RULING-a-raise-is-not-an-outcome-so-a-raising-verb-is-partial.md`). `Partial`.
///
/// **Expand-time ground —** `extend-type` has no runtime call site at all (`role = declare`
/// emits no shim); its registration runs from `register_stdlib_types`/`register_types`
/// (`src/types.rs`, called from `src/freeze/env.rs`'s `build_env`, step 5) and from
/// `collect_splice_defs_ctx` inside `check_program` (step 8) — both strictly AFTER `expand_all`
/// (step 4) has produced the forms they walk. Also absent from `macros/eval.rs`'s expand-time
/// pure-total allow-list (measured — no `:wat::core::extend-type` arm there), so an
/// `extend-type` nested inside a macro body cannot be eagerly evaluated during that expansion —
/// the identical fact `defsurface`'s row measured for its own `env`-dependent state.
/// `RuntimeOnly`.
///
/// @added 1.0.0
/// @Category Declaration
/// @Purity Unevaluated
/// @Determinism Deterministic
/// @Totality Partial
/// @ExpandTime RuntimeOnly
/// @syntax (:wat::core::extend-type :T :P (method-1 [self ...] body) ...)
/// @ret :wat::core::nil no runtime value — the form is consumed entirely at registration (the type-lattice edge and each method impl) and never reaches evaluation; encountered in expression position it raises `DeclarationInExpressionPosition` instead of producing one
/// @example-norun (:wat::core::extend-type :probe::Robot :probe::Greeter (greet [self loudness] "beep")) #=> registers the (Robot, Greeter) subtype edge and the greet method; no runtime value
#[wat_special_form(":wat::core::extend-type")]
pub(crate) struct ExtendType;
