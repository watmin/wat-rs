//! vigilatum: 2026-05-30 @ 22c89e04 — vigilia 8-spell L1+L2=0
//!
//! `CheckEnv<'a>` — the type-check environment.
//!
//! Stone 243.3.1 — borrow redesign. `CheckEnv` previously deep-cloned
//! `SymbolTable`'s immutable inputs (`binding_metadata`, `TypeEnv`) into
//! owned fields. This is the failure-engineering roof: the two immutable
//! inputs are now BORROWED (`&'a TypeEnv`, `Option<&'a HashMap<…>>`),
//! making deep-clone-into-CheckEnv a compile error — the duplication
//! situation is structurally unrepresentable.
//!
//! Field classification (Stone 243.3.1 DESIGN § Field classification):
//! - `types: &'a TypeEnv` — BORROW (was Arc<TypeEnv>, deep-cloned)
//! - `binding_metadata: Option<&'a HashMap<…>>` — BORROW (was Arc<HashMap>, deep-cloned)
//! - All other 6 fields — OWNED (derived, incremental, or mid-pass-mutable)

use crate::ast::WatAST;
use crate::runtime::SymbolTable;
use crate::span::Span;
use crate::types::{TypeEnv, TypeExpr};
use std::collections::HashMap;

pub use super::TypeScheme;
use super::register_builtins;

/// The type-check environment: built-in + user function schemes plus
/// a borrowed handle to the [`TypeEnv`] (user type declarations).
/// Unification consults the type-env to expand typealiases to their
/// structural definitions before the structural match.
///
/// Stone 243.3.1 — `CheckEnv<'a>` borrows its immutable inputs:
/// - `types: &'a TypeEnv` — the registered user type declarations;
///   read-only after the register phase (freeze builds it before check).
/// - `binding_metadata: Option<&'a HashMap<…>>` — the SymbolTable's
///   binding-level metadata; read-only after freeze (never mutated during check).
///
/// The borrow makes deep-clone-into-CheckEnv a compile error. A field
/// of type `&'a TypeEnv` cannot hold an `Arc::new(x.clone())` — the
/// compiler rejects it. This is the failure-engineering roof: the
/// duplication situation is never constructible, not merely avoided.
///
/// Failure-engineering discipline (FAILURE-ENGINEERING.md): eliminate
/// the CLASS by making the wrong shape STRUCTURALLY UNAVAILABLE.
#[derive(Debug)]
pub struct CheckEnv<'a> {
    pub(super) schemes: HashMap<String, TypeScheme>,
    /// Arc 048 — keyword paths for user-enum unit variants mapped to
    /// the enum's type. When `infer` sees one of these as a value-
    /// position keyword (e.g. `:trading::types::PhaseLabel::Valley`),
    /// it returns the enum's type instead of the generic
    /// `:wat::core::keyword`. Mirrors the runtime's
    /// `SymbolTable.unit_variants`. Populated at construction by
    /// walking every `:wat::core::enum` declaration in `types`.
    pub(super) unit_variant_types: HashMap<String, TypeExpr>,
    /// Stone 243.3.1 — BORROW (was Arc<TypeEnv>, deep-cloned at check.rs:2175).
    /// Read-only after the register phase; outlives every use in check_program.
    pub(super) types: &'a TypeEnv,
    /// Arc 157 — names bound via `:wat::core::def` at top-level
    /// position. Maps name → inferred TypeExpr of the binding. Keyword
    /// references to a `def`'d name resolve here instead of falling
    /// through to the generic `:wat::core::keyword` type.
    ///
    /// Populated incrementally by `check_program` as it processes
    /// top-level forms left-to-right: after each `def` form is
    /// type-checked, the bound name + inferred type are inserted so
    /// subsequent forms can reference it. Also tracks the span for
    /// redef diagnostics (via `defined_value_spans`).
    pub(crate) defined_values: HashMap<String, TypeExpr>,
    /// Arc 157 — parallel to `defined_values`: maps name → span of
    /// the binding site. Used to emit `DefRedefForbidden` with the
    /// prior location when a collision is detected.
    pub(crate) defined_value_spans: HashMap<String, Span>,
    /// Stone 241.14 — BORROW of SymbolTable's binding-level metadata
    /// (was Arc<HashMap>, deep-cloned at check.rs:2019). The `:restricted-to`
    /// key carries a Vector of prefix keywords; the walker
    /// `walk_for_restricted_call` reads from this map to enforce
    /// caller-prefix whitelists declared via metadata-map on `def`/`defn`.
    ///
    /// Stone 243.3.1 — `Option<&'a …>` because standalone constructors
    /// (`with_builtins_and_types`) have no SymbolTable to borrow from;
    /// those paths carry `None` (no `:restricted-to` enforcement in pure
    /// builtin-only envs). `from_symbols` carries `Some(&sym.binding_metadata)`.
    ///
    /// Generic storage: the substrate does NOT enforce specific keys; each
    /// downstream consumer projects to its typed needs.
    pub(crate) binding_metadata: Option<&'a HashMap<String, HashMap<String, WatAST>>>,
    /// Arc 157 slice 1a-ii — compile-time redef-allowed flag. Default
    /// `false` (strict default: every redef is an error). Updated
    /// in-line by `check_program` when it encounters a top-level
    /// `(:wat::config::set-redef! <bool>)` form (single-pass
    /// program-order semantics). Consulted by `infer_def` at the
    /// redef-collision site.
    pub(crate) redef_allowed: bool,
    /// Stone 237.2 — per-defclause clause registrations.
    ///
    /// Maps FQDN name → list of `(arg_types, return_type)` per clause,
    /// in declaration order (first-match-wins at call sites).
    /// Populated incrementally by `collect_splice_defs_ctx` when it
    /// encounters a `:wat::core::defclause` top-level form. Consumed
    /// in `infer_list` for call-site dispatch type-checking.
    /// Stone 241.5 — tuple is (fixed_arg_types, return_type, has_rest_binder).
    pub(crate) defclause_registrations: HashMap<String, Vec<(Vec<TypeExpr>, TypeExpr, bool)>>,
}

impl<'a> CheckEnv<'a> {
    /// Build an env with built-in schemes for `:wat::core::*` and
    /// `:wat::holon::*` forms, then overlay user-define signatures
    /// from `sym`. `types` carries the registered user type
    /// declarations (struct/enum/newtype/typealias) — unification uses
    /// it to expand aliases.
    ///
    /// Stone 243.3.1 — `types` is now borrowed (`&'a TypeEnv`), not
    /// deep-cloned into `Arc<TypeEnv>`. `binding_metadata` is borrowed
    /// directly from `sym` — no `Arc::new(sym.binding_metadata.clone())`.
    pub fn from_symbols(sym: &'a SymbolTable, types: &'a TypeEnv) -> CheckEnv<'a> {
        let mut env = Self::with_builtins_and_types(types);
        for (path, func) in &sym.functions {
            if let Some(scheme) = super::derive_scheme_from_function(func) {
                env.register(path.clone(), scheme);
            }
        }
        // Arc 157 slice 1a-ii — mirror the redef-allowed flag from the
        // SymbolTable carrier (populated from Config at freeze time) so
        // `infer_def` can gate the collision check without needing direct
        // SymbolTable access. Option (b) per the BRIEF: mirror, not direct.
        env.set_redef_allowed(sym.redef_allowed);
        // Read-only after freeze time — binding_metadata is populated before
        // check_program runs; safe to borrow for the pass duration.
        env.binding_metadata = Some(&sym.binding_metadata);
        env
    }

    /// Build an env with built-in schemes + the given `TypeEnv` borrow.
    ///
    /// Stone 243.3.1 — `types` is now `&'a TypeEnv` (borrow), not
    /// `Arc<TypeEnv>`. Caller binds the `TypeEnv` first, then passes a
    /// reference. For standalone use (no SymbolTable): `binding_metadata`
    /// is `None` (no `:restricted-to` enforcement in pure builtin envs).
    pub fn with_builtins_and_types(types: &'a TypeEnv) -> CheckEnv<'a> {
        let mut env = Self::with_types(types);
        register_builtins(&mut env);
        env
    }

    /// Private constructor: builds the base `CheckEnv` with the borrowed
    /// `TypeEnv` and pre-populates `unit_variant_types` from enum declarations.
    fn with_types(types: &'a TypeEnv) -> CheckEnv<'a> {
        // Arc 048 — pre-populate unit-variant keyword types from the
        // declared enums. Structural knowledge of TypeDef::Enum /
        // EnumVariant::Unit belongs to TypeEnv; delegate there.
        let unit_variant_types = types.build_unit_variant_map();
        CheckEnv {
            schemes: HashMap::new(),
            unit_variant_types,
            types,
            defined_values: HashMap::new(),
            defined_value_spans: HashMap::new(),
            binding_metadata: None,
            redef_allowed: false,
            defclause_registrations: HashMap::new(),
        }
    }

    /// Arc 048 — look up the enum type for a unit-variant keyword
    /// path. Returns `None` for non-variant keywords.
    pub fn unit_variant_type(&self, key: &str) -> Option<&TypeExpr> {
        self.unit_variant_types.get(key)
    }

    /// Register a function/builtin type scheme at `name`. Consumed by
    /// `from_symbols` (user functions) and `register_builtins` (substrate primitives).
    pub fn register(&mut self, name: String, scheme: TypeScheme) {
        self.schemes.insert(name, scheme);
    }

    /// Look up a function or builtin scheme by FQDN. For `def`-bound value types
    /// use `get_defined_value_type`; for defclause dispatch use `get_defclause_clauses`.
    pub fn get(&self, name: &str) -> Option<&TypeScheme> {
        self.schemes.get(name)
    }

    /// Handle to the user/builtin type declarations. Used by `unify`
    /// to expand typealiases to their structural form before the
    /// structural match.
    pub fn types(&self) -> &'a TypeEnv {
        self.types
    }

    /// Arc 157 — look up the inferred type for a `def`-bound name.
    /// Returns `Some(&TypeExpr)` when the name was bound via
    /// `:wat::core::def`; `None` otherwise. Consulted in `infer`
    /// before the generic keyword fall-through.
    pub fn get_defined_value_type(&self, name: &str) -> Option<&TypeExpr> {
        self.defined_values.get(name)
    }

    /// Arc 157 — look up the span of a prior `def` binding. Used to
    /// emit `DefRedefForbidden` with the prior location.
    pub fn get_defined_value_span(&self, name: &str) -> Option<&Span> {
        self.defined_value_spans.get(name)
    }

    /// Arc 157 — register a new `def` binding. Called from
    /// `infer_def` when a `def` form passes position + redef checks.
    /// Subsequent forms in `check_program`'s sequential loop will see
    /// this name in `get_defined_value_type`.
    pub fn register_defined_value(&mut self, name: String, ty: TypeExpr, span: Span) {
        self.defined_values.insert(name.clone(), ty);
        self.defined_value_spans.insert(name, span);
    }

    /// Stone 241.14 — look up binding-level metadata for a named binding.
    /// Returns `Some(&HashMap<String, WatAST>)` when the binding carries
    /// metadata (e.g. `:restricted-to`); `None` otherwise.
    /// Consulted by `walk_for_restricted_call` at every call site to
    /// extract the `:restricted-to` prefix whitelist.
    ///
    /// Stone 243.3.1 — reads through the borrow; no Arc dereference needed.
    pub fn get_binding_metadata(&self, name: &str) -> Option<&'a HashMap<String, WatAST>> {
        self.binding_metadata.and_then(|m| m.get(name))
    }

    /// Stone 243.3 — setter for the compile-time redef-allowed flag.
    /// Replaces direct field mutation at call sites; keeps the field
    /// write path explicit and under accessor control.
    pub(crate) fn set_redef_allowed(&mut self, flag: bool) {
        self.redef_allowed = flag;
    }

    /// Register a defclause's clause table, and ensure a value-binding exists
    /// under the same name so value-position keyword references resolve here
    /// instead of failing UnknownCallee.
    ///
    /// Two writes with deliberately different semantics:
    /// - The clause table (`defclause_registrations`) is inserted unconditionally
    ///   — a re-registration replaces the prior clause set.
    /// - The sentinel value-binding (`Var(u64::MAX)` in `defined_values`) is
    ///   written only if no value-binding exists yet — the guard is load-bearing:
    ///   it must not clobber a real value type set by a prior `def` of this name.
    pub fn register_defclause(
        &mut self,
        name: String,
        clauses: Vec<(Vec<TypeExpr>, TypeExpr, bool)>,
        span: Span,
    ) {
        // Also register in defined_values so keyword references to the
        // name in value position (e.g. passing it as :fn argument) see
        // a type. Use a fresh Var(u64::MAX) sentinel — actual call-site
        // dispatch uses defclause_registrations, not defined_values.
        // The sentinel prevents `UnknownCallee` from firing for the name
        // in call-head position when there is no scheme.
        if !self.defined_values.contains_key(&name) {
            self.defined_values
                .insert(name.clone(), TypeExpr::Var(u64::MAX));
            self.defined_value_spans.insert(name.clone(), span);
        }
        self.defclause_registrations.insert(name, clauses);
    }

    /// Stone 237.2 — look up a defclause's clause table by name.
    /// Returns `None` if the name was not bound via defclause.
    pub fn get_defclause_clauses(
        &self,
        name: &str,
    ) -> Option<&[(Vec<TypeExpr>, TypeExpr, bool)]> {
        self.defclause_registrations.get(name).map(|v| v.as_slice())
    }
}
