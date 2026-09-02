//! Arc 144 slice 2 — special-form registry.
//!
//! Special forms are syntactic constructs the type checker + runtime
//! handle directly (not via dispatch through Function or TypeScheme).
//! Examples: `:wat::core::if`, `let`, `fn`, `define`, `match`,
//! `quasiquote`, `try`, retired-but-poisoned heads like
//! `:wat::kernel::spawn`.
//!
//! This registry lets `:wat::runtime::lookup-form` (arc 144 slice 1)
//! return `Binding::SpecialForm` for each known form, exposing a
//! synthesized signature sketch the consumer (e.g., a future `(help
//! :if)` form) can render.
//!
//! Each entry carries the form's full keyword name + a synthesized
//! `HolonAST::Bundle` showing the syntax shape + a placeholder `None`
//! doc_string (arc 141 will populate it).
//!
//! # Sketch format
//!
//! Each `signature` is a `HolonAST::Bundle` whose first child is the
//! form's head as a Keyword (`HolonAST::keyword(":wat::core::if")`);
//! remaining children are bare-symbol placeholders for the syntactic
//! slots (`HolonAST::symbol("<cond>")`). Repeating slots use `<name>+`
//! (one or more) or `<name>*` (zero or more). The format is honest
//! about structure-not-types: each slot is a symbol naming the slot's
//! role, not a type. Consumers render this to a help string or AST.
//!
//! # Audit
//!
//! The registry's coverage was audited against:
//!   - `src/check.rs:2950-3430` — primary special-form dispatch in
//!     `infer_list` (heads that get non-scheme handling).
//!   - `src/runtime.rs:2400-2425` — runtime dispatch for the
//!     evaluation-side equivalents of those forms.
//!   - `src/freeze.rs:825-840` — definitional special forms handled
//!     at freeze (top-level only).
//!
//! Forms registered as TypeScheme primitives (e.g., `:wat::core::Vector`,
//! `:wat::kernel::spawn-thread`, `:wat::kernel::send`) do NOT appear
//! here — they are reachable through `lookup_form`'s Primitive branch
//! (slice 3 territory) instead. User-defined wat helpers like
//! `:wat::kernel::run-sandboxed-ast` (defined in `wat/kernel/sandbox.wat`)
//! reach through the UserFunction branch.

use holon::HolonAST;
use std::collections::HashMap;
use std::sync::OnceLock;

/// One special-form entry. Owned data — cloned out at lookup time.
pub struct SpecialFormDef {
    pub name: String,
    pub signature: HolonAST,
    pub doc_string: Option<String>,
}

static REGISTRY: OnceLock<HashMap<String, SpecialFormDef>> = OnceLock::new();

/// Lookup by full keyword name. Returns `Some(&SpecialFormDef)` for
/// every known special form; `None` otherwise.
///
/// The first call lazily initializes the registry; subsequent calls
/// share the same `&'static HashMap` (no Mutex/RwLock — `OnceLock`
/// initialization is the substrate's permitted concurrency primitive
/// per `docs/ZERO-MUTEX.md`).
pub fn lookup_special_form(name: &str) -> Option<&'static SpecialFormDef> {
    REGISTRY.get_or_init(build_registry).get(name)
}

/// Build a `HolonAST::Bundle` whose first child is `head` as a
/// Keyword leaf and remaining children are `slots` as bare Symbol
/// leaves (each slot's name is a literal placeholder string like
/// `"<cond>"` or `"<body>+"`).
fn sketch(head: &str, slots: &[&str]) -> HolonAST {
    let mut children = Vec::with_capacity(1 + slots.len());
    children.push(HolonAST::keyword(head));
    for s in slots {
        children.push(HolonAST::symbol(*s));
    }
    HolonAST::bundle(children)
}

/// Insert one form into the registry. The signature head MUST equal
/// the lookup name; the helper enforces this by reusing `name` in
/// both positions.
fn insert(m: &mut HashMap<String, SpecialFormDef>, name: &str, slots: &[&str]) {
    let signature = sketch(name, slots);
    m.insert(
        name.to_string(),
        SpecialFormDef {
            name: name.to_string(),
            signature,
            doc_string: None,
        },
    );
}

fn build_registry() -> HashMap<String, SpecialFormDef> {
    let mut m = HashMap::new();

    // ─── Value binding — top-level ──────────────────────────────────────
    // Arc 157 — `:wat::core::def` is the foundational top-level value-
    // binding special form (Clojure-faithful; strongly-typed Clojure on
    // Rust). Shape: `(:wat::core::def :name expr)`. Legal only at
    // top-level position: direct file form, inside a top-level
    // `(:wat::core::do ...)`, or inside a top-level
    // `(:wat::core::let ...)` body (the splice-eligible positions
    // per DESIGN § "Scope (Q1)"). The bound name's type is the inferred
    // type of `expr`; no type annotation on the form.
    // Dispatch sites: `src/check.rs` (infer_def arm) +
    // `src/runtime.rs` (dispatch_keyword_head arm). Position enforcement
    // via the `validate_def_position` walker called in `check_program`.
    insert(&mut m, ":wat::core::def", &["<name>", "<expr>"]);

    // Stone 241.14 — `:wat::core::def-restricted` HARD CUT (retired).
    // Restrictions now live as {:restricted-to [...]} metadata-map on def/defn:
    //   (:wat::core::def :name {:restricted-to [<prefix-kw>...]} <expr>)
    // Entry removed from active forms; HARD-CUT arm in check.rs fires for
    // any residual caller. See retirement.rs entry for replacement remedy.

    // ─── Redef config setters (arc 157 slice 1a-ii) ─────────────────────
    // Arc 157 slice 1a-ii — compile-time redef opt-in. Default `false`
    // (strict: every redef is an error). Set `true` to permit redef with
    // mandatory type-stability check (type of re-bound name must not change).
    // Shape: `(:wat::config::set-redef! true)` — takes one bool literal.
    // Dispatch sites: `src/check.rs` (infer_config_set_bool arm +
    // extract_redef_setter in check_program loop) +
    // `src/runtime.rs` (register_runtime_defs_form arm + dispatch_keyword_head no-op).
    insert(&mut m, ":wat::config::set-redef!", &["<bool>"]);
    // Arc 157 slice 1a-ii — eval-time redef opt-in. Default `false`.
    // Carrier + setter scaffolding wired; behavior gate is a no-op until
    // eval-time def-binding is implemented (eval arm returns Value::Unit).
    // A future arc opens IFF a caller surfaces wanting eval-time def redef.
    insert(&mut m, ":wat::config::set-eval-redef!", &["<bool>"]);

    // ─── Type ascription ───────────────────────────────────────────────
    // Arc 251 Stone 251.4b — core.typed's `ann-form`: checked, type-erased
    // identity. `(ann-form expr type)` asserts `expr`'s inferred type is
    // assignable to `type` (check time); at runtime evaluates `expr` and
    // returns its value unchanged (type is erased).
    // Dispatch sites: `src/check.rs` (infer_list arm) + `src/runtime.rs`
    // (dispatch_keyword_head_value arm + eval_ann_form).
    insert(&mut m, ":wat::core::ann-form", &["<expr>", "<type>"]);

    // ─── Control / branching ────────────────────────────────────────────
    // Dispatch sites: `src/check.rs:2956-2959` + `src/runtime.rs:2402-2405`.
    insert(&mut m, ":wat::core::if", &["<cond>", "<then>", "<else>"]);
    // Bindings: layout is `(let ((<name> <expr>)*) <body>+)` — the
    // bindings slot is a list of name/expr pairs; the type checker
    // walks it specially (arc 057 et al.). Arc 154 made `:wat::core::let`
    // sequential (Clojure-faithful single-letform vocabulary;
    // `:wat::core::let*` retired into `let`).
    insert(&mut m, ":wat::core::let", &["<bindings>", "<body>+"]);
    // Arc 154 — `:wat::core::let*` retired (single-letform vocabulary).
    // Registry entry removed in arc 170 slice 3 (lambda precedent
    // symmetry: arc 155 slice 2 removed lambda's entry; let*'s entry
    // was the only asymmetry). Source-level use fires BareLegacyLetStar
    // fatally at check time; `(help :wat::core::let*)` now returns
    // "no such form" — matches lambda's behavior post-arc-155-slice-2.
    // Arc 136 slice 1a — Clojure-faithful sequential side-effect chain.
    // `(:wat::core::do f1 f2 ... fN)` — variadic; one or more forms.
    // Non-finals' types are unconstrained (results discarded); the
    // final form's inferred type IS the do form's type. No `-> :T`
    // slot — the substrate's existing inference + recipient unification
    // is the static check (per the FOURTH amendment to arc 136 DESIGN).
    insert(&mut m, ":wat::core::do", &["<form>+"]);
    // Match: `(match <scrutinee> -> <T> <arm>+)`. The `->` and `<T>`
    // are part of the surface form (arc 091 / arc 098 grammar).
    insert(
        &mut m,
        ":wat::core::match",
        &["<scrutinee>", "->", "<T>", "<arm>+"],
    );

    // ─── Functions ────────────────────────────────────────────────────
    // Arc 155 — `:wat::core::fn` is the canonical operator for function
    // values (Clojure-faithful lowercase verb; mirrors arc 154's let
    // retirement recipe). The legacy `:wat::core::lambda` keyword retired in
    // arc 155 slice 2 (Path B full retirement; registry entry +
    // dispatch arms gone; source-level use surfaces standard "unknown
    // form" error). BareLegacyLambda variant + Display retained as
    // orphaned scaffolding (arc 113 precedent).
    insert(&mut m, ":wat::core::fn", &["<params>", "<body>+"]);
    // Stone 241.16 — `:wat::core::define` registry entry DELETED. HARD CUT total.
    // (Stone 241.11 HARD-CUT startup check; Stone 241.16 eval-time residue complete.)
    insert(&mut m, ":wat::core::defmacro", &["<head>", "<template>"]);

    // ─── Type definitions ───────────────────────────────────────────────
    // Dispatch sites: `src/check.rs:3393-3396` (return None at
    // expression position) + `src/freeze.rs:833-836` (top-level
    // mutation forms).
    // Stone 241.8 — defstruct replaces struct (HARD CUT).
    insert(&mut m, ":wat::core::defstruct", &["<name>", "[<field> <- <type>]+"]);
    // Stone 241.9 — defenum replaces enum (HARD CUT).
    insert(&mut m, ":wat::core::defenum", &["<name>", "<variant>+"]);
    insert(&mut m, ":wat::core::newtype", &["<name>", "<target>"]);
    insert(&mut m, ":wat::core::typealias", &["<name>", "<target>"]);

    // ─── Error handling — canonical (post-arc-109 § J) ─────────────────
    // Dispatch sites: `src/check.rs:3000-3019` + `src/runtime.rs:2439-2449`.
    insert(&mut m, ":wat::core::Result/try", &["<expr>"]);
    insert(&mut m, ":wat::core::Option/try", &["<expr>"]);
    insert(
        &mut m,
        ":wat::core::Option/expect",
        &["->", "<T>", "<opt>", "<msg>"],
    );
    insert(
        &mut m,
        ":wat::core::Result/expect",
        &["->", "<T>", "<res>", "<msg>"],
    );

    // ─── Error handling — RETIRED forms deleted by Stone 241.15 ──────────
    // :wat::core::try, :wat::core::option::expect, :wat::core::result::expect
    // are HARD CUT (MalformedForm rejection; RETIREMENT_TABLE entries provide
    // remedy). Reflection entries removed — HARD CUT forms are not reflected.

    // ─── Quote / quasiquote / AST ──────────────────────────────────────
    // Dispatch sites: `src/check.rs:3083-3107, 3401-3413` + `src/runtime.rs:2406-2407, 2421`.
    insert(&mut m, ":wat::core::quote", &["<expr>"]);
    insert(&mut m, ":wat::core::quasiquote", &["<template>"]);
    // Arc 294.b — `#holon <form>` reader tag; desugars to this special form.
    insert(&mut m, ":wat::holon::literal", &["<form>"]);
    // Arc 118 — `lazy-seq` captures its body unevaluated as a thunk (capture-don't-eval,
    // like quote). Dispatch sites: `src/check.rs` (infer_list arm) + `src/runtime.rs`
    // (dispatch_keyword_head_value arm + eval_lazy_seq).
    insert(&mut m, ":wat::stream::lazy", &["<body>"]);
    // `unquote` and `unquote-splicing` are only legal INSIDE a
    // quasiquote template; at the top level they return None from
    // expression-position inference (`src/check.rs:3401-3402`).
    // Registered here for uniform reflection.
    insert(&mut m, ":wat::core::unquote", &["<expr>"]);
    insert(&mut m, ":wat::core::unquote-splicing", &["<expr>"]);
    insert(&mut m, ":wat::core::forms", &["<form>*"]);
    insert(&mut m, ":wat::core::struct->form", &["<struct-value>"]);

    // ─── Boolean shortcircuit ───────────────────────────────────────────
    // Dispatch site: `src/check.rs:3378` (special: returns :bool;
    // walks args without unifying against a fixed scheme so callers
    // can pass any boolean expression).
    //
    // ⚠ Arc 255 Stone 1a-i — `and`/`or` are now ALSO registered in the intrinsic registry
    // (`src/intrinsic/special/and_form.rs` / `or_form.rs`, `#[wat_special_form]`), with real
    // `role = check|eval|tail` implementations wired (`infer_boolean_shortcircuit`,
    // `eval_and_tail`, `eval_or_tail`). Per the stone's brief this row would normally come OUT
    // once registered — measured and left IN instead: `src/reflect/lookup.rs:197`'s
    // `lookup_form` consults `lookup_special_form` (THIS registry) as its only route to a
    // `Binding::SpecialForm`, with no fallback step reading `crate::intrinsic::registry()`.
    // Deleting this row would silently drop `:wat::core::and`/`:wat::core::or` out of
    // `lookup_form` (and whatever it feeds — `:wat::runtime::lookup-form`,
    // `:wat::runtime::lookup-define`) with no existing test catching the loss (confirmed: no
    // other `lookup_form` step — user-defines/macros/`CheckEnv` builtins/types — resolves
    // either name). A stone that adds `reflect/lookup.rs` to its blast radius should retire this
    // row properly, either by having `lookup_form` also consult the intrinsic registry or by
    // some other resolution; until then this duplication is a KNOWN, MEASURED, DELIBERATE
    // holdover, not an oversight.
    insert(&mut m, ":wat::core::and", &["<expr>*"]);
    insert(&mut m, ":wat::core::or", &["<expr>*"]);

    // ─── Macro debug primitives ─────────────────────────────────────────
    // Dispatch site: `src/check.rs:3205` (special: takes :wat::WatAST,
    // returns :wat::WatAST, no scheme registration).
    insert(&mut m, ":wat::core::macroexpand-1", &["<form>"]);
    insert(&mut m, ":wat::core::macroexpand", &["<form>"]);

    // ─── Pattern-matcher entry point (arc 098) ─────────────────────────
    // Dispatch site: `src/check.rs:3269` (substrate-recognized; macros
    // expand before type-check and can't query the struct registry, so
    // matches? gets its own grammar walker).
    insert(
        &mut m,
        ":wat::form::matches?",
        &["<subject>", "<clause>+"],
    );

    // ─── Resolve-pass declaration ───────────────────────────────────────
    // Dispatch site: `src/check.rs:3382` (no-op returning :() —
    // validation happens at the resolve pass, not type inference).
    insert(&mut m, ":wat::core::use!", &["<path>"]);

    // ─── Top-level loaders (freeze-time mutation forms) ────────────────
    // Dispatch sites: `src/check.rs:3398-3400` (return None at
    // expression position) + `src/freeze.rs:837-839` (mutation forms).
    insert(&mut m, ":wat::load-file!", &["<path>"]);
    insert(&mut m, ":wat::digest-load!", &["<path>", "<digest>"]);
    insert(
        &mut m,
        ":wat::signed-load!",
        &["<path>", "<signature>", "<key>"],
    );

    m
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lookup_returns_some_for_if() {
        let def = lookup_special_form(":wat::core::if").expect("if");
        assert_eq!(def.name, ":wat::core::if");
        assert!(def.doc_string.is_none());
        match &def.signature {
            HolonAST::Bundle(children) => {
                // head + 3 slots (cond, then, else)
                assert_eq!(children.len(), 4);
                // Arc 221 Stone 221.3 (holon-rs fa48b39): HolonAST::keyword() now returns
                // HolonAST::Keyword (stripped of leading colon). The sketch() builder at
                // special_forms.rs:75 calls HolonAST::keyword(head) → Keyword("wat::core::if").
                // as_keyword() returns content WITHOUT colon; as_symbol() → None.
                assert_eq!(
                    children[0].as_keyword(),
                    Some("wat::core::if"),
                    "first child should be the keyword head (HolonAST::Keyword after arc 221 Stone 221.3)"
                );
                // Slot children are still Symbol (HolonAST::symbol("<cond>") unchanged).
                assert_eq!(children[1].as_symbol(), Some("<cond>"));
                assert_eq!(children[2].as_symbol(), Some("<then>"));
                assert_eq!(children[3].as_symbol(), Some("<else>"));
            }
            other => panic!("expected Bundle, got {:?}", other),
        }
    }

    #[test]
    fn lookup_returns_none_for_unknown() {
        assert!(lookup_special_form(":wat::core::not-a-special-form").is_none());
    }

    #[test]
    fn registry_covers_audited_forms() {
        // Spot-check one entry per group.
        for name in [
            ":wat::core::def",
            ":wat::core::if",
            ":wat::core::let",
            ":wat::core::fn",
            // Stone 241.16 — `:wat::core::define` REMOVED from audited-forms list.
            // HARD CUT total; define is no longer a registered special form.
            // Stone 241.8 — defstruct replaces struct.
            ":wat::core::defstruct",
            ":wat::core::Result/try",
            // Stone 241.15 — :wat::core::try is HARD CUT; removed from registry.
            ":wat::core::quote",
            ":wat::core::quasiquote",
            // Arc 294.b — holon literal registered as a special form.
            ":wat::holon::literal",
            ":wat::core::and",
            ":wat::core::macroexpand-1",
            ":wat::form::matches?",
            ":wat::core::use!",
            ":wat::load-file!",
        ] {
            assert!(
                lookup_special_form(name).is_some(),
                "expected {} in registry",
                name
            );
        }
    }
}
