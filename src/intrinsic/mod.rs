//! Intrinsic registry — arc 255. The home where wat **intrinsics** (callables
//! implemented in Rust, exposed under a `:wat::…` FQDN — `runtime.rs:23931`:
//! "intrinsics are custom Rust by definition") become registered, queryable
//! entities. The `#[wat_intrinsic]` preamble (255.1b-ii) lives over each handler
//! in this home.
//!
//! ## Accretion discipline (satisfy a forcing-signal by USE, never silence it)
//!
//! Most fields are added in the SAME strike that builds their reader:
//!   - `name` / `handler` → 255.1b-i/ii: the dispatch route (`lookup`).
//!   - `arity`            → sniffed from the `#[wat_intrinsic]` fixed-arg signature;
//!                          255.1b-iii: consumed by `metadata-of`'s intrinsic branch.
//!   - `prose` / `added` / `ret` → parsed from the `///` via `wat-doc` by the macro
//!                          (255.1b-iv-b1); consumed by `metadata-of`'s intrinsic branch.
//!   - `purity` / `determinism` → DERIVED at the reflection site (namespace deriver
//!                          via `is_effectful_op` + a small nondeterministic-set),
//!                          not stored on the entry.
//!
//! **The one bounded exception (builder-sanctioned, 2026-06-21):** `args` /
//! `examples` / `deprecated` / `see` are parsed + carried by the iv-b1 macro but
//! their reader — iv-b2's `verify-examples` reflection seam — lands one strike
//! later. They are NOT deleted (they are about to be used, not unneeded) and NOT
//! hidden behind a pub-leak (the cheat an earlier draft used — making the module
//! `pub` to FAKE external use and silence the signal — reverted). Each carries
//! `#[expect(dead_code)]` (NOT `#[allow]`): silent while genuinely dead, but the
//! compiler emits an unfulfilled-expectation warning the instant iv-b2's seam
//! references one — SELF-RETIRING, compiler-enforced removal, not a comment-clause
//! the next hand might forget (arc 277's expect-dead idea, applied to the live
//! instance). When iv-b2 reads these, the compiler tells us to take the `#[expect]`s off.
use std::sync::Arc;
use crate::ast::WatAST;
use crate::span::Span;
use crate::value::{EnumValue, Environment, SymbolTable, Value, EvalBreak};

// ─── Arc 255.1b-iv-c: Closed-domain enum mirrors ─────────────────────────────
// Three Rust enums that mirror the three `defenum`s in `wat/runtime-meta.wat`.
// Each carries a `to_enum_value()` method that builds the corresponding
// `Value::Enum` so the derivation site in `eval_metadata_of` (runtime.rs
// ~10119-10122) is typo-proof at the call site.
//
// Invariant: the `type_path` and `variant_name` strings here MUST match the
// `defenum` declarations in `wat/runtime-meta.wat` EXACTLY (checked by the
// iv-c nursery probe).

/// Kind — what kind of callable is this?
/// Mirrors `(:wat::core::defenum :wat::runtime::Kind :Macro :Fn :Intrinsic)`.
pub(crate) enum Kind {
    #[expect(dead_code)] // reader lands at user-form branch parity (iv-c future) → keep
    Macro,
    #[expect(dead_code)] // reader lands at user-form branch parity (iv-c future) → keep
    Fn,
    Intrinsic,
}

impl Kind {
    pub(crate) fn to_enum_value(&self) -> Value {
        let variant_name = match self {
            Kind::Macro => "Macro",
            Kind::Fn => "Fn",
            Kind::Intrinsic => "Intrinsic",
        };
        Value::Enum(Arc::new(EnumValue {
            type_path: ":wat::runtime::Kind".into(),
            variant_name: variant_name.into(),
            fields: vec![],
        }))
    }
}

/// DefinedIn — implementation language.
/// Mirrors `(:wat::core::defenum :wat::runtime::DefinedIn :Wat :Rust)`.
pub(crate) enum DefinedIn {
    #[expect(dead_code)] // reader lands at user-form branch parity (iv-c future) → keep
    Wat,
    Rust,
}

impl DefinedIn {
    pub(crate) fn to_enum_value(&self) -> Value {
        let variant_name = match self {
            DefinedIn::Wat => "Wat",
            DefinedIn::Rust => "Rust",
        };
        Value::Enum(Arc::new(EnumValue {
            type_path: ":wat::runtime::DefinedIn".into(),
            variant_name: variant_name.into(),
            fields: vec![],
        }))
    }
}

/// Layer — where in the system stack does this live?
/// Mirrors `(:wat::core::defenum :wat::runtime::Layer :Substrate :Userland)`.
pub(crate) enum Layer {
    Substrate,
    #[expect(dead_code)] // reader lands at user-form branch parity (iv-c future) → keep
    Userland,
}

impl Layer {
    pub(crate) fn to_enum_value(&self) -> Value {
        let variant_name = match self {
            Layer::Substrate => "Substrate",
            Layer::Userland => "Userland",
        };
        Value::Enum(Arc::new(EnumValue {
            type_path: ":wat::runtime::Layer".into(),
            variant_name: variant_name.into(),
            fields: vec![],
        }))
    }
}

/// The native dispatch handler — matches the eval-fn signature exactly.
pub(crate) type NativeHandler =
    fn(&[WatAST], &Span, &Environment, &SymbolTable) -> Result<Value, EvalBreak>;

/// One `@example` / `@example-norun` entry carried on the registry — the
/// structured form of `wat_doc::DocExample`, lowered to `'static` literals
/// by the `#[wat_intrinsic]` macro.
///
/// Fields are read by the iv-b2 `verify-examples` reflection seam
/// (`src/intrinsic/reflect.rs`). The `#[expect(dead_code)]` has been removed
/// because the seam now satisfies the use.
pub(crate) struct ExampleSubmission {
    pub expr: &'static str,
    pub expected: Option<&'static str>,
    pub run: bool,
}

/// A link-time submission of one intrinsic, gathered by `inventory`. The
/// `#[wat_intrinsic("<fqdn>")]` proc-macro emits one `inventory::submit!` of
/// this type per annotated handler; `registry()` builds itself by iterating
/// `inventory::iter::<IntrinsicSubmission>`. All fields are `'static` — the
/// macro emits string-literal fields and a fn-pointer `handler`, all of which
/// outlive the program.
///
/// Arc 255.1b-iv-b1 — the structured doc now rides each submission. `prose`,
/// `added`, and `ret` are CONSUMED by `metadata-of`'s intrinsic branch;
/// `args`, `examples`, and `see` are carried for iv-b2's verifier seam.
pub(crate) struct IntrinsicSubmission {
    pub name: &'static str,
    pub handler: NativeHandler,
    pub arity: usize,
    /// GFM prose body (everything before the first `@`-tag line).
    pub prose: &'static str,
    /// `@added` version string.
    pub added: &'static str,
    /// `@arg` directives: `(name, desc)` pairs, in source order.
    pub args: &'static [(&'static str, &'static str)],
    /// `@ret` description.
    pub ret: &'static str,
    /// `@example` / `@example-norun` directives, in source order (≥1).
    pub examples: &'static [ExampleSubmission],
    /// `@deprecated (since, use_instead)`, if present.
    pub deprecated: Option<(&'static str, &'static str)>,
    /// `@see` FQDNs, in source order.
    pub see: &'static [&'static str],
}

inventory::collect!(IntrinsicSubmission);

/// One registered intrinsic's full baseline. `handler` is consumed by the
/// runtime dispatch route (`lookup`); `name`/`arity`/`prose`/`added`/`ret` are
/// consumed by `metadata-of`'s intrinsic branch (`lookup_entry`); `args`/
/// `examples`/`see` are carried for iv-b2's doctest verifier seam — every
/// field has a reader, so none is dead-code.
pub(crate) struct IntrinsicEntry {
    pub name: &'static str,
    pub handler: NativeHandler,
    pub arity: usize,
    pub prose: &'static str,
    pub added: &'static str,
    pub ret: &'static str,
    // The iv-b2 carry: parsed + carried now, read by the `verify-examples`
    // reflection seam (`src/intrinsic/reflect.rs`). `examples` is now read
    // by the seam (iv-b2-a), so its `#[expect(dead_code)]` has been removed.
    // `args`/`deprecated`/`see` are still unread — their readers land later;
    // keep their `#[expect(dead_code)]` so removal stays compiler-enforced.
    #[expect(dead_code)] // reader lands later (wiki/doc) → keep
    pub args: &'static [(&'static str, &'static str)],
    pub examples: &'static [ExampleSubmission],
    #[expect(dead_code)] // reader lands later → keep
    pub deprecated: Option<(&'static str, &'static str)>,
    #[expect(dead_code)] // reader lands later → keep
    pub see: &'static [&'static str],
}

/// `name → entry`. Built once at startup; the dispatch route reads `handler`
/// via `lookup`, `metadata-of` reads the baseline via `lookup_entry`.
pub(crate) struct IntrinsicRegistry {
    entries: std::collections::HashMap<&'static str, IntrinsicEntry>,
}

impl IntrinsicRegistry {
    fn new() -> Self { IntrinsicRegistry { entries: std::collections::HashMap::new() } }

    /// Register an intrinsic's full baseline. Duplicate registration is a
    /// programmer error (two homes claiming the same FQDN).
    fn register(&mut self, entry: IntrinsicEntry) {
        debug_assert!(!self.entries.contains_key(entry.name), "duplicate intrinsic registration: {}", entry.name);
        self.entries.insert(entry.name, entry);
    }

    /// The dispatch route — the native handler for `name` (255.1b-i/ii).
    /// `None` = not a registered intrinsic.
    pub(crate) fn lookup(&self, name: &str) -> Option<NativeHandler> {
        self.entries.get(name).map(|e| e.handler)
    }

    /// The reflection route — the full baseline entry for `name` (255.1b-iii),
    /// read by `metadata-of`'s intrinsic branch. `None` = not registered.
    pub(crate) fn lookup_entry(&self, name: &str) -> Option<&IntrinsicEntry> {
        self.entries.get(name)
    }

    /// Iterate all registered entries. Read by the iv-b2 `verify-examples`
    /// reflection seam (`src/intrinsic/reflect.rs`) to build the examples vector.
    pub(crate) fn all_entries(&self) -> impl Iterator<Item = &IntrinsicEntry> {
        self.entries.values()
    }
}

/// The process-wide intrinsic registry, built once on first access.
pub(crate) fn registry() -> &'static IntrinsicRegistry {
    static REGISTRY: std::sync::OnceLock<IntrinsicRegistry> = std::sync::OnceLock::new();
    REGISTRY.get_or_init(|| {
        let mut r = IntrinsicRegistry::new();
        // Each `#[wat_intrinsic("<fqdn>")]` handler submits an entry via
        // `inventory`; gather them all into the registry at first access.
        for submission in inventory::iter::<IntrinsicSubmission> {
            r.register(IntrinsicEntry {
                name: submission.name,
                handler: submission.handler,
                arity: submission.arity,
                prose: submission.prose,
                added: submission.added,
                args: submission.args,
                ret: submission.ret,
                examples: submission.examples,
                deprecated: submission.deprecated,
                see: submission.see,
            });
        }
        r
    })
}

mod bytes;
mod reflect;
