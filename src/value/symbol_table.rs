//! Stone 251.2d — SymbolTable lifted from `src/runtime.rs` into the value home.
//! PURE STRUCTURAL MOVE — no behavior change.

use std::collections::HashMap;
use std::sync::Arc;

use crate::ast::WatAST;
use crate::load::loader::SourceLoader;
use crate::macros::MacroRegistry;
use crate::value::{EnumValue, Value};
use crate::holon::sigma::SigmaFn;
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
    functions: HashMap<String, Arc<Function>>,
    // TRANSFORMS — clojure-ination (keyword-keyed)
    /// Arc 048 — pre-built [`EnumValue`]s for each registered
    /// unit-variant enum constructor. Populated by
    /// [`register_enum_methods`] at freeze time. Keyed by full
    /// keyword path (e.g. `:trading::types::PhaseLabel::Valley`).
    /// Consulted in `eval`'s keyword arm before the function-lookup
    /// fallback so a bare keyword evaluates directly to its
    /// variant value (mirrors the `:None` shortcut).
    unit_variants: HashMap<String, EnumValue>,
    pub encoding_ctx: Option<Arc<EncodingCtx>>,
    pub source_loader: Option<Arc<dyn SourceLoader>>,
    macro_registry: Option<Arc<MacroRegistry>>,
    /// Ambient presence-sigma function — `:fn(:i64) -> :i64`. Takes
    /// dim, returns σ count. Used by `presence?` to compute the
    /// per-d floor (`σ(d) / sqrt(d)`). Built-in default is
    /// [`crate::holon::sigma::DefaultPresenceSigma`]; user override via
    /// `set-presence-sigma!`.
    pub presence_sigma_fn: Option<Arc<dyn SigmaFn>>,
    /// Ambient coincident-sigma function — `:fn(:i64) -> :i64`.
    /// Built-in default is [`crate::holon::sigma::DefaultCoincidentSigma`];
    /// user override via `set-coincident-sigma!`.
    pub coincident_sigma_fn: Option<Arc<dyn SigmaFn>>,
    /// Frozen type registry — every struct / enum / newtype / alias
    /// declared in user source plus the built-ins. Attached at freeze
    /// time so `#[wat_dispatch]` shims can reflect on type
    /// declarations (variant fields, struct fields, alias targets) —
    /// e.g. to walk a consumer's entry-enum decl and synthesize
    /// schemas + INSERT statements without consumer code (arc 085).
    types: Option<Arc<TypeEnv>>,
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
    runtime_def_values: HashMap<String, Value>,
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
    /// Arc 170 stdio-as-defservice — the three PRIMED stdio defservices' client-dial `Address'` values.
    /// Set once per `invoke_user_main` by the freeze bootstrap after starting
    /// `:wat::kernel::{stdin,stdout,stderr}-svc` on the real fds; propagates to spawned threads via
    /// `Clone`. When set, the `:wat::kernel::spawn-thread` arm gives each spawned thread a fresh
    /// [`crate::services::ThreadIO`] so its `(println ...)` / `(eprintln ...)` / `(readln)` calls can
    /// `connect'` + cache a client peer. `None` when no orchestrator is active (bare test worlds; the
    /// service threads themselves bootstrap before it is set, so their spawn-thread calls skip ThreadIO
    /// — the lazy pattern). Capability-carrier pattern next to `encoding_ctx` / `source_loader` /
    /// `macro_registry` (memory `feedback_capability_carrier.md`).
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
    /// expansion) via `(:wat::string::declare-acronyms :ns ["ACL"])` forms.
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

/// Every registry a name can be registered in — arc 278,
/// `DESIGN-STONE-registry-kind-one-door.md`.
///
/// The registries answer at DIFFERENT PHASES (`Macro` at expand, `Type` at check, the rest at
/// eval), which is why they are separate tables and not one — fusing them would collapse the
/// phase ordering the language depends on. What was missing was a single QUERY surface, so a
/// consumer asking "what is this name?" had to remember all five. `closure_extract` remembered
/// four; the omitted `Macro` is why a synthesized record shipped to a forked child as a type
/// with no callable constructor.
///
/// ⛔ EXHAUSTIVE BY LAW. The `_`-wildcard ban on enum scrutinees
/// (`109/NOTE-full-enum-match-mandatory-no-wildcard-arm.md`) means adding a sixth registry
/// turns every consumer's match RED until it decides what the new kind means. That is the
/// point of the enum: a new registry cannot be silently skipped, because there is no wildcard
/// to swallow it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum RegistryKind {
    /// Expand-time. Macro definitions — including every kwargs constructor.
    Macro,
    /// Check-time. `TypeEnv` entries.
    Type,
    /// Eval-time. Registered functions.
    Function,
    /// Eval-time. Nullary enum variants, keyed by full path.
    UnitVariant,
    /// Eval-time. `def`-bound values.
    DefValue,
}

/// Every facet a name is registered under. MEASURED (the registry census,
/// `tests/reflection/probe_arc278_registry_census.rs`): over a defservice world, 207 of 2489
/// names appear in more than one registry, in exactly two shapes — `[Macro, Type]` (a record's
/// type + its constructor) and `[Function, DefValue]` (every `defn`, since `defn` expands to
/// `(def :n (fn …))`). Nothing appears in three or more, and NO name means two unrelated
/// things. So this is a set of FACETS OF ONE CONCEPT, never rivals — which is why there is no
/// precedence field here and no precedence ruling to make. A caller takes the facet its phase
/// needs.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct RegistrationSet {
    kinds: Vec<RegistryKind>,
}

impl RegistrationSet {
    pub fn is_empty(&self) -> bool {
        self.kinds.is_empty()
    }
    pub fn contains(&self, kind: RegistryKind) -> bool {
        self.kinds.contains(&kind)
    }
    pub fn iter(&self) -> impl Iterator<Item = RegistryKind> + '_ {
        self.kinds.iter().copied()
    }
    fn push(&mut self, kind: RegistryKind) {
        self.kinds.push(kind);
    }
}

impl SymbolTable {
    pub fn new() -> Self {
        Self::default()
    }

    // ─── THE DOOR ───────────────────────────────────────────────────────
    //
    // One call answers "what is registered under this name?" across every
    // registry. Empty ⇒ unregistered (at value position, a keyword literal).
    //
    // Consumers that need only ONE registry use the phase-named narrow
    // accessors below — so a single-registry read is a DELIBERATE, greppable
    // choice, never the default that happens because four were forgotten.

    /// Every facet `name` is registered under, across all five registries.
    pub fn registrations(&self, name: &str) -> RegistrationSet {
        let mut set = RegistrationSet::default();
        // Ordered to mirror the phases: expand → check → eval. The order is
        // presentational, NOT precedence — see `RegistrationSet`.
        if self
            .macro_registry
            .as_ref()
            .is_some_and(|m| m.contains(name))
        {
            set.push(RegistryKind::Macro);
        }
        if self.types.as_ref().is_some_and(|t| t.contains(name)) {
            set.push(RegistryKind::Type);
        }
        if self.functions.contains_key(name) {
            set.push(RegistryKind::Function);
        }
        if self.unit_variants.contains_key(name) {
            set.push(RegistryKind::UnitVariant);
        }
        if self.runtime_def_values.contains_key(name) {
            set.push(RegistryKind::DefValue);
        }
        set
    }

    // ─── NARROW, PHASE-NAMED READS ──────────────────────────────────────

    pub fn get(&self, path: &str) -> Option<&Arc<Function>> {
        self.functions.get(path)
    }

    /// Narrow: is `path` a registered FUNCTION? (Not "is it registered".)
    pub fn has_function(&self, path: &str) -> bool {
        self.functions.contains_key(path)
    }

    /// Narrow: iterate registered functions. Bulk transfer, not name lookup.
    pub fn functions_iter(&self) -> impl Iterator<Item = (&String, &Arc<Function>)> {
        self.functions.iter()
    }

    pub fn function_values(&self) -> impl Iterator<Item = &Arc<Function>> {
        self.functions.values()
    }

    /// Narrow: the nullary-enum-variant facet.
    pub fn unit_variant(&self, path: &str) -> Option<&EnumValue> {
        self.unit_variants.get(path)
    }

    pub fn has_unit_variant(&self, path: &str) -> bool {
        self.unit_variants.contains_key(path)
    }

    pub fn unit_variants_iter(&self) -> impl Iterator<Item = (&String, &EnumValue)> {
        self.unit_variants.iter()
    }

    /// Narrow: the `def`-bound-value facet.
    pub fn def_value(&self, path: &str) -> Option<&Value> {
        self.runtime_def_values.get(path)
    }

    pub fn has_def_value(&self, path: &str) -> bool {
        self.runtime_def_values.contains_key(path)
    }

    pub fn def_values_iter(&self) -> impl Iterator<Item = (&String, &Value)> {
        self.runtime_def_values.iter()
    }

    // NOTE: the macro facet's narrow accessor already exists further down as
    // `macro_registry()` — kept as the incumbent rather than duplicated here.
    // It had TWO call sites in the whole tree before this stone, which is
    // exactly why omitting the macro registry went unnoticed for two arcs.

    // ─── REGISTRATION (writes) ──────────────────────────────────────────
    //
    // The registration path is NOT a door consumer: it names one registry on
    // purpose. These exist so the fields can stay private.

    pub fn register_function(&mut self, path: String, f: Arc<Function>) {
        self.functions.insert(path, f);
    }

    pub fn remove_function(&mut self, path: &str) -> Option<Arc<Function>> {
        self.functions.remove(path)
    }

    pub fn register_unit_variant(&mut self, path: String, v: EnumValue) {
        self.unit_variants.insert(path, v);
    }

    pub fn register_def_value(&mut self, path: String, v: Value) {
        self.runtime_def_values.insert(path, v);
    }

    pub fn remove_def_value(&mut self, path: &str) -> Option<Value> {
        self.runtime_def_values.remove(path)
    }

    /// Narrow: the type facet, dereferenced. Companion to [`Self::types`]
    /// (which yields `&Arc<TypeEnv>`); several call sites want `&TypeEnv`.
    pub fn types_deref(&self) -> Option<&TypeEnv> {
        self.types.as_deref()
    }

    /// Attach/replace the `TypeEnv`, yielding the stored handle. Mirrors the
    /// `Option::insert` the field previously exposed directly.
    pub fn types_insert(&mut self, t: Arc<TypeEnv>) -> &mut Arc<TypeEnv> {
        self.types.insert(t)
    }

    /// Mutable access to a registered function, for the in-place fixups the
    /// freeze pipeline performs after registration.
    pub fn function_entry(
        &mut self,
        path: String,
    ) -> std::collections::hash_map::Entry<'_, String, Arc<Function>> {
        self.functions.entry(path)
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

    /// Attach the primed-stdio carrier (arc 170 stdio-as-defservice). Called once per
    /// `invoke_user_main` by the freeze bootstrap after starting the three primed stdio defservices.
    /// Propagates to spawned threads through SymbolTable's `Clone`; the spawn-thread arm reads it to
    /// decide whether to give the new thread a ThreadIO.
    pub fn set_primed_stdio(&mut self, primed: Arc<crate::services::PrimedStdio>) {
        self.primed_stdio = Some(primed);
    }

    /// Borrow the primed-stdio carrier, if one is attached (arc 170). The flipped stdio verbs call this
    /// to reach each stream's client-dial `Address'`; the spawn-thread arm uses its presence as the
    /// "stdio is running → give the thread a ThreadIO" signal.
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
    /// or the built-in [`crate::holon::sigma::DefaultPresenceSigma`].
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
