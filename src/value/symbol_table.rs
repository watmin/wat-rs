//! Stone 251.2d — SymbolTable lifted from `src/runtime.rs` into the value home.
//! PURE STRUCTURAL MOVE — no behavior change.

use std::collections::HashMap;
use std::sync::Arc;

use crate::ast::WatAST;
use crate::load::SourceLoader;
use crate::macros::MacroRegistry;
use crate::value::{EnumValue, Value};
use crate::sigma::SigmaFn;
use crate::services::RuntimeServices;
use crate::types::TypeEnv;
use crate::value::{EncodingCtx, Function};

/// Per-binding metadata: FQDN -> metadata-key -> raw AST value.
pub(crate) type BindingMetadata = HashMap<String, HashMap<String, WatAST>>;

/// Keyword-path ↦ Function registry + runtime capabilities.
///
/// The `encoding_ctx` and `source_loader` fields are populated at
/// freeze time by the startup pipeline. Test harnesses
/// (`SymbolTable::new()`) leave them `None`; primitives that require
/// the capability (presence / encode for ctx, `:wat::eval-file!` and
/// the `-file-path` verified eval variants for loader) error cleanly
/// if invoked without one attached.
///
/// Runtime-capability attachment follows the pattern established by
/// Rust's compiler `Session`, Common Lisp special variables,
/// Clojure dynamic vars, and Haskell `ReaderT`. See arc 007 DESIGN.md.
#[derive(Clone)]
#[derive(Default)]
pub struct SymbolTable {
    pub functions: HashMap<String, Arc<Function>>,
    // TRANSFORMS — clojure-ination (keyword-keyed)
    /// Arc 048 — pre-built [`EnumValue`]s for each registered
    /// unit-variant enum constructor. Populated by
    /// [`register_enum_methods`] at freeze time. Keyed by full
    /// keyword path (e.g. `:trading::types::PhaseLabel::Valley`).
    /// Consulted in `eval`'s keyword arm before the function-lookup
    /// fallback so a bare keyword evaluates directly to its
    /// variant value (mirrors the `:None` shortcut).
    pub unit_variants: HashMap<String, EnumValue>,
    pub encoding_ctx: Option<Arc<EncodingCtx>>,
    pub source_loader: Option<Arc<dyn SourceLoader>>,
    pub macro_registry: Option<Arc<MacroRegistry>>,
    /// Ambient presence-sigma function — `:fn(:i64) -> :i64`. Takes
    /// dim, returns σ count. Used by `presence?` to compute the
    /// per-d floor (`σ(d) / sqrt(d)`). Built-in default is
    /// [`crate::sigma::DefaultPresenceSigma`]; user override via
    /// `set-presence-sigma!`.
    pub presence_sigma_fn: Option<Arc<dyn SigmaFn>>,
    /// Ambient coincident-sigma function — `:fn(:i64) -> :i64`.
    /// Built-in default is [`crate::sigma::DefaultCoincidentSigma`];
    /// user override via `set-coincident-sigma!`.
    pub coincident_sigma_fn: Option<Arc<dyn SigmaFn>>,
    /// Frozen type registry — every struct / enum / newtype / alias
    /// declared in user source plus the built-ins. Attached at freeze
    /// time so `#[wat_dispatch]` shims can reflect on type
    /// declarations (variant fields, struct fields, alias targets) —
    /// e.g. to walk a consumer's entry-enum decl and synthesize
    /// schemas + INSERT statements without consumer code (arc 085).
    pub types: Option<Arc<TypeEnv>>,
    /// Arc 140 slice 1 — when this SymbolTable belongs to a sub-
    /// program (one started via `:wat::kernel::run-sandboxed-ast` /
    /// `run-sandboxed-hermetic-ast`), this field carries an
    /// Arc to the OUTER scope's frozen SymbolTable. Used by the
    /// runtime's UnknownFunction site to detect sandbox-scope leaks:
    /// when a call head doesn't resolve in the inner scope but DOES
    /// resolve in `outer_symbols`, fire `RuntimeError::SandboxScopeLeak`
    /// with a teaching diagnostic. Sandbox isolation is preserved —
    /// `outer_symbols` is read-only and only consulted on the failure
    /// path; nothing in the success path consults it.
    ///
    /// `None` for the entry program (no outer scope) and for test
    /// harnesses that build a SymbolTable directly. Set by the spawn
    /// driver (`spawn::eval_kernel_spawn_program_ast` and siblings)
    /// after the sub-program's freeze completes.
    pub outer_symbols: Option<Arc<SymbolTable>>,
    // Stone A0 — `defined_values: HashMap<String, (TypeExpr, Span)>` field
    // DELETED. Its doc comment claimed "Populated by `register_defs` during
    // the startup pipeline (step 6c...)" but no such writer ever existed
    // anywhere in the tree (grepped exhaustively) — dead field since
    // introduction, permanently empty. Check-time `def`-bound-name
    // redef-tracking lives entirely in `CheckEnv.defined_values`
    // (`check/env.rs`, a DIFFERENT, live, per-file map built incrementally
    // by `check_program`); the runtime value of a `def` binding lives in
    // `runtime_def_values` below (live, populated by
    // `register_stdlib_runtime_defs` / `register_runtime_defs`). A stdlib
    // scalar value-const's TYPE for cross-file resolution is now derived
    // straight from its `runtime_def_values` entry — see
    // `CheckEnv::from_symbols`'s `corpus_values` seed (Stone A0).
    // Stone 241.14 — `defined_value_restrictions` field DELETED.
    // Restriction whitelists now live in `binding_metadata` under the
    // `:restricted-to` key. The `walk_for_restricted_call` walker in
    // `check.rs` reads from `CheckEnv.binding_metadata` (mirrored from
    // `SymbolTable.binding_metadata`) instead of the old parallel map.
    /// Arc 157 slice 1a-ii — runtime values bound via `:wat::core::def`.
    /// Maps name → `Value` produced when the top-level `def` form's
    /// expression was evaluated during `FrozenWorld::freeze` (step 9.5).
    /// Consulted in `eval`'s keyword arm after `unit_variants` so that
    /// a bare keyword reference (`:pi`, `:get-config`, etc.) resolves to
    /// the bound value at runtime without re-evaluating the expression.
    ///
    /// Parallel to `defined_values` (type-check side) — cleaner
    /// separation: check-time carries `(TypeExpr, Span)`; runtime carries
    /// `Value`. Populated by `register_runtime_defs` in `FrozenWorld::freeze`
    /// after all capability carriers are installed.
    pub runtime_def_values: HashMap<String, Value>,
    /// Arc 157 slice 1a-ii — controls compile-time / load-time `def` redef.
    /// Default `false` (opt-in). Toggled via
    /// `(:wat::config::set-redef! true)`. Type-stability check applies
    /// whenever redef happens, regardless of flag value (enforced at
    /// check time via `CheckEnv.redef_allowed`).
    pub redef_allowed: bool,
    /// Arc 157 slice 1a-ii — controls eval-time `def` redef (interactive
    /// `eval-ast!` flow). Default `false` (opt-in). Toggled via
    /// `(:wat::config::set-eval-redef! true)`. Type-stability check applies.
    /// NOTE: eval-time `def` binding is not yet wired (eval arm returns
    /// `Value::Unit`); this flag is write-only scaffolding — config-parsed,
    /// read by no eval path.
    // rune:purgare(future-fixture) — eval-time def-redef scaffolding: config-parsed into this field but no eval path reads it (the read-side gate is unbuilt); write-only by present construction.
    pub eval_redef_allowed: bool,
    /// Arc 170 slice 1f-γ — runtime services carrier. When set, the
    /// `:wat::kernel::spawn-thread` arm registers each spawned thread
    /// with the three stdio services (StdIn / StdOut / StdErr) so the
    /// thread's `(:wat::kernel::println ...)` / `(eprintln ...)` /
    /// `(readln)` calls have a populated [`crate::services::ThreadIO`].
    /// `None` when no orchestrator is active (test harnesses + the
    /// service threads themselves bootstrap before the carrier is set,
    /// so their internal spawn-thread calls see `None` and skip
    /// registration — the lazy-registration pattern). Carrier choice B
    /// per BRIEF § honest-delta — capability-carrier pattern next to
    /// `encoding_ctx`, `source_loader`, `macro_registry`. Memory
    /// `feedback_capability_carrier.md`.
    pub runtime_services: Option<Arc<RuntimeServices>>,
    /// Arc 170 stdio-as-defservice (PHASE 1) — the three PRIMED stdio defservices' client-dial
    /// `Address'` values. Set once per `invoke_user_main` by the freeze bootstrap after starting
    /// `:wat::kernel::{stdin,stdout,stderr}-svc'` on the real fds; propagates to spawned threads via
    /// `Clone`. COEXISTS with `runtime_services` (the hand-rolled path) — Phase 1 flips no verb, so
    /// nothing reads this yet; Strike 2 (verb flip) has each thread `connect'` these addresses.
    /// `None` when no orchestrator is active (bare test worlds, service-thread bootstrap).
    pub primed_stdio: Option<Arc<crate::services::PrimedStdio>>,
    /// Stone 241.6 — binding-level metadata attached via the optional
    /// `{...}` metadata-map clause on `def` / `defn`. Maps binding name
    /// (full FQDN keyword string, e.g. `:my::ns::my-fn`) to the inner
    /// metadata map (key keyword string → raw WatAST value). Generic
    /// storage: the substrate does NOT enforce or validate specific keys;
    /// downstream consumers (Stone 241.7 reflection verb; Stone 241.10
    /// HARD CUT of def-restricted) project to their typed needs.
    /// Populated by `register_defines` / `register_runtime_defs_form`
    /// when a `def` form carries a metadata-map at items[2].
    pub binding_metadata: BindingMetadata,
    /// Arc 265 — namespace-scoped acronym registry.
    /// Maps namespace keyword string (e.g. `":my::aws"`) to the list of
    /// canonical acronyms declared for that namespace (e.g. `["ACL", "HTTP"]`).
    /// Populated by `preregister_acronyms` (freeze step 6.96, before macro
    /// expansion) via `(:wat::core::string::declare-acronyms :ns ["ACL"])` forms.
    /// Consulted by `pascal->kebab-in` and `kebab->pascal-in` at expand time.
    /// No entry for a namespace → plain `pascal->kebab` / `kebab->pascal` behavior.
    pub acronym_registry: HashMap<String, Vec<String>>,
}

impl std::fmt::Debug for SymbolTable {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SymbolTable")
            .field("functions", &self.functions.len())
            .field("unit_variants", &self.unit_variants.len())
            .field("encoding_ctx", &self.encoding_ctx.is_some())
            .field("source_loader", &self.source_loader.is_some())
            .field("macro_registry", &self.macro_registry.is_some())
            .field("presence_sigma_fn", &self.presence_sigma_fn.is_some())
            .field("coincident_sigma_fn", &self.coincident_sigma_fn.is_some())
            .field("types", &self.types.is_some())
            .field("runtime_def_values", &self.runtime_def_values.len())
            .field("redef_allowed", &self.redef_allowed)
            .field("eval_redef_allowed", &self.eval_redef_allowed)
            .field("binding_metadata", &self.binding_metadata.len())
            .finish()
    }
}

impl SymbolTable {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn get(&self, path: &str) -> Option<&Arc<Function>> {
        self.functions.get(path)
    }

    /// Attach an encoding context. Called once at freeze time by
    /// [`crate::freeze::FrozenWorld::freeze`].
    pub fn set_encoding_ctx(&mut self, ctx: Arc<EncodingCtx>) {
        self.encoding_ctx = Some(ctx);
    }

    /// Borrow the encoding context, if one is attached. Runtime
    /// primitives that require encoding (`:wat::holon::cosine`) call
    /// this and raise [`RuntimeError::NoEncodingCtx`] on `None`.
    pub fn encoding_ctx(&self) -> Option<&Arc<EncodingCtx>> {
        self.encoding_ctx.as_ref()
    }

    /// Attach a source loader. Called once at freeze time by
    /// [`crate::freeze::FrozenWorld::freeze`], mirrors
    /// [`SymbolTable::set_encoding_ctx`].
    pub fn set_source_loader(&mut self, loader: Arc<dyn SourceLoader>) {
        self.source_loader = Some(loader);
    }

    /// Borrow the source loader, if one is attached. Runtime primitives
    /// that read files (`:wat::eval-file!`, file-path variants of the
    /// verified eval/load forms, `:wat::verify::file-path` payloads)
    /// call this and raise an error on `None` — a host that didn't
    /// attach a loader doesn't have the capability.
    pub fn source_loader(&self) -> Option<&Arc<dyn SourceLoader>> {
        self.source_loader.as_ref()
    }

    /// Attach the runtime-services carrier. Called once per
    /// `invoke_user_main` invocation by the orchestrator (after
    /// spawning the three stdio services). The carrier propagates to
    /// spawned threads through SymbolTable's `Clone`; child threads'
    /// `eval_kernel_spawn_thread` sites read it to decide whether to
    /// register the new thread with the services. Arc 170 slice 1f-γ.
    pub fn set_runtime_services(
        &mut self,
        services: Arc<RuntimeServices>,
    ) {
        self.runtime_services = Some(services);
    }

    /// Borrow the runtime-services carrier, if one is attached. The
    /// spawn-thread arm calls this to decide whether to allocate a
    /// new ThreadId + register with the services or skip (service-
    /// thread bootstrap path, pre-orchestrator init, etc.). Arc 170
    /// slice 1f-γ.
    pub fn runtime_services(
        &self,
    ) -> Option<&Arc<RuntimeServices>> {
        self.runtime_services.as_ref()
    }

    /// Attach the primed-stdio carrier (arc 170 stdio-as-defservice, PHASE 1). Called once per
    /// `invoke_user_main` by the freeze bootstrap after starting the three primed stdio defservices.
    /// Mirrors [`SymbolTable::set_runtime_services`].
    pub fn set_primed_stdio(&mut self, primed: Arc<crate::services::PrimedStdio>) {
        self.primed_stdio = Some(primed);
    }

    /// Borrow the primed-stdio carrier, if one is attached (arc 170 PHASE 1). The Strike-2 flipped
    /// verbs will call this to reach each stream's client-dial `Address'`. Mirrors
    /// [`SymbolTable::runtime_services`].
    pub fn primed_stdio(&self) -> Option<&Arc<crate::services::PrimedStdio>> {
        self.primed_stdio.as_ref()
    }

    /// Attach the macro registry. Called once at freeze time by
    /// [`crate::freeze::FrozenWorld::freeze`] so runtime primitives
    /// (`:wat::core::macroexpand`, `:wat::core::macroexpand-1`) can
    /// inspect macro expansion at runtime — the standard Lisp
    /// macro-debugging tool. Arc 030.
    pub fn set_macro_registry(&mut self, registry: Arc<MacroRegistry>) {
        self.macro_registry = Some(registry);
    }

    /// Borrow the macro registry, if one is attached. `macroexpand`
    /// and `macroexpand-1` call this and raise `NoMacroRegistry` on
    /// `None` — test harnesses that build a SymbolTable directly
    /// without going through freeze don't have macros attached.
    pub fn macro_registry(&self) -> Option<&Arc<MacroRegistry>> {
        self.macro_registry.as_ref()
    }

    /// Attach the ambient presence-sigma function. Called once at
    /// freeze time with the user's override (from set-presence-sigma!)
    /// or the built-in [`crate::sigma::DefaultPresenceSigma`].
    pub fn set_presence_sigma_fn(
        &mut self,
        f: Arc<dyn SigmaFn>,
    ) {
        self.presence_sigma_fn = Some(f);
    }

    /// Borrow the presence-sigma function. `presence?` calls this.
    pub fn presence_sigma_fn(&self) -> Option<&Arc<dyn SigmaFn>> {
        self.presence_sigma_fn.as_ref()
    }

    /// Attach the ambient coincident-sigma function.
    pub fn set_coincident_sigma_fn(
        &mut self,
        f: Arc<dyn SigmaFn>,
    ) {
        self.coincident_sigma_fn = Some(f);
    }

    /// Attach the frozen type registry. Called once at freeze time by
    /// [`crate::freeze::FrozenWorld::freeze`] so shims that need to
    /// inspect declared types (e.g. walking an enum decl to synthesize
    /// schemas) can reach them through the standard SymbolTable carrier.
    pub fn set_types(&mut self, types: Arc<TypeEnv>) {
        self.types = Some(types);
    }

    /// Borrow the type registry, if one is attached. Shims that need
    /// to reflect on declared types call this and raise an error on
    /// `None` — a host that didn't attach the registry doesn't have
    /// the capability.
    pub fn types(&self) -> Option<&Arc<TypeEnv>> {
        self.types.as_ref()
    }

    /// Borrow the coincident-sigma function. `coincident?` calls this.
    pub fn coincident_sigma_fn(&self) -> Option<&Arc<dyn SigmaFn>> {
        self.coincident_sigma_fn.as_ref()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Lines 153-169: `Debug` impl for `SymbolTable` — format fields without exposing
    /// full contents (counts + booleans only). Asserts real output tokens.
    #[test]
    fn debug_shows_counts_and_option_flags() {
        let sym = SymbolTable::new();
        let dbg = format!("{:?}", sym);
        assert_eq!(
            dbg,
            "SymbolTable { functions: 0, unit_variants: 0, encoding_ctx: false, source_loader: false, macro_registry: false, presence_sigma_fn: false, coincident_sigma_fn: false, types: false, runtime_def_values: 0, redef_allowed: false, eval_redef_allowed: false, binding_metadata: 0 }",
            "Debug output mismatch"
        );
    }
}
