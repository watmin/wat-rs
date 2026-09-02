//! Special-form doc entry for `:wat::core::defmacro` — arc 255 Stone 1a-β-ii, one of the
//! last three names in `freeze::is_liftable_declaration_head`'s domain to register.

use wat_macros::wat_special_form;

/// Declare a named macro: `:name::path`, a mandatory `[<param> <- :wat::WatAST ...]`
/// argspec (every fixed param binds a whole FORM, never a value — a macro runs at
/// expand-time, before values exist), a mandatory `-> :wat::WatAST` return type, and a
/// body that constructs the replacement form (typically via `quasiquote`/`unquote`).
/// `register_defmacros` collects every `defmacro` at the top of a file into the
/// `MacroRegistry` BEFORE `expand_all` walks the rest of the program, so a macro may be
/// invoked anywhere after its own `defmacro`, in the same file or one that loads it.
///
/// **Category ground —** same as `defsurface`'s: `defmacro` registers `:name::path` into
/// the `MacroRegistry` — visible to every form after it (in expansion order), not scoped
/// to a body — exactly `Declaration`'s own variant prose ("registers a program-level
/// entity … visible to everything after it"). `Declaration`.
///
/// **Purity ground —** measured directly: `:wat::core::defmacro` appears in
/// `src/runtime.rs` exactly ONCE, inside `is_mutation_head` — a hand-list guarding
/// `eval-ast!`, not a dispatch arm — and nowhere in `dispatch_keyword_head_value`,
/// `eval_tail`, or `step_list`. No `handler`, no eval arm, no tail arm. Same reasoning as
/// `defsurface`'s row: all four consumers of `@Purity` ask a RUNTIME question, and
/// `defmacro` has no runtime to ask it about — `Pure` would demand a runnable `@example`
/// of a verb that cannot be run, `Effectful` would claim an effect there is no call to
/// have, `Preserving` would claim sub-forms that are never evaluated (the argspec and
/// body are stored whole into the `MacroDef`, expanded later at EACH call site, never
/// evaluated by `defmacro` itself). `Unevaluated`.
///
/// **Determinism ground —** the same `defmacro` form always parses into the identical
/// `MacroDef` (name, argspec, return type, body) — no clock, no entropy, no gensym
/// anywhere on `parse_defmacro_form`'s path (the `fresh-symbol` gensym a macro's OWN BODY
/// may call happens later, per expansion, not here). `Deterministic`.
///
/// **Totality ground —** `parse_defmacro_form` is measured NOT defined on every input: the
/// retired 3-item paren-pair shape, a non-keyword name, a non-Vector argspec, a fixed
/// param whose declared type is not `:wat::WatAST`, a rest param whose type is not
/// `(:wat::core::Vector :- [:wat::WatAST])`, or a return type that is not `:wat::WatAST`
/// all raise `MacroErrorKind::MalformedDefmacro` instead of returning a `MacroDef` — a
/// raise the expand pipeline propagates as a hard failure, never a value a caller matches
/// on. Same reasoning `:wat::i64::/`'s own `@Totality Partial` was ruled on
/// (`RULING-a-raise-is-not-an-outcome-so-a-raising-verb-is-partial.md`). `Partial`.
///
/// **Expand-time ground —** `defmacro` has no runtime call site at all (`role = declare`
/// emits no shim) — the form is consumed whole by `register_defmacros`/
/// `register_stdlib_defmacros`, which the startup pipeline runs BEFORE `expand_all` walks
/// the rest of the program (`src/freeze/env.rs`'s `build_env` doc, step 4), so the state
/// `parse_defmacro_form` needs (the in-progress `MacroRegistry` itself) categorically does
/// not exist mid-expansion of some OTHER macro's body. `defmacro` is also absent from
/// `macros/eval.rs`'s expand-time pure-total allow-list (measured — no
/// `:wat::core::defmacro` arm there; its only mention is `refuse_expand_only_in_program`'s
/// UNWALKED-whole-form skip, a different question), so a `defmacro` nested inside another
/// macro's body cannot be eagerly evaluated during that expansion — the identical fact
/// `defsurface`'s row measured for `synthesize_surface_protocol`'s `env` dependency.
/// `RuntimeOnly`.
///
/// @added 1.0.0
/// @Category Declaration
/// @Purity Unevaluated
/// @Determinism Deterministic
/// @Totality Partial
/// @ExpandTime RuntimeOnly
/// @syntax (:wat::core::defmacro :name [<param> <- :wat::WatAST ...] -> :wat::WatAST <body>)
/// @ret :wat::core::nil no runtime value — the form is consumed entirely at macro-registration time (before `expand_all`) and never reaches evaluation; its effect is the `MacroDef` it leaves in the `MacroRegistry`
/// @example-norun (:wat::core::defmacro :probe::ident [x <- :wat::WatAST] -> :wat::WatAST `~x) #=> registers :probe::ident into the MacroRegistry; no runtime value
#[wat_special_form(":wat::core::defmacro")]
pub(crate) struct Defmacro;
