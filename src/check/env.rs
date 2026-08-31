//! vigilatum: 2026-06-06T04:56:04Z — vigilia 8-spell L1+L2=0 (first earned 2026-05-30T20:46:58Z; RE-EARNED diff-scoped at the 245 clear: defined_value_asts added [arc-054 byte-equiv idempotent redeclare; span-agnostic WatAST eq verified at span.rs Span::eq-always-true; accessor discipline held — no raw field access outside the home]; gates: idempotent_redeclare 6/6, lib 923/0/1, clippy-in-home empty)
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
//! - All other 7 fields — OWNED (derived, incremental, or mid-pass-mutable)

use crate::ast::WatAST;
use crate::check::error::CheckError;
use crate::runtime::SymbolTable;
use crate::span::Span;
use crate::types::{TypeEnv, TypeExpr};
use std::collections::{HashMap, HashSet};

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
    /// Arc 232 Stone 232.1 — extend-type satisfaction edges.
    ///
    /// Maps `(protocol_fqdn, type_fqdn)` → list of method names that the
    /// implementation provides. The KEY's existence is the satisfaction
    /// signal for 232.2's `assignable(T, :P)`. Method names are stored
    /// for 232.3 dispatch to verify completeness.
    pub(crate) extend_registrations: HashMap<(String, String), Vec<String>>,
    /// Arc 054 — body AST of each `def`-bound value, for byte-equivalence
    /// re-declaration checking. Maps name → the WAT AST of the expression
    /// (the `expr` arg of `(:wat::core::def :name expr)`). Populated in
    /// `register_defined_value_ast`; consulted in `infer_def` when a redef
    /// collision is detected and `!redef_allowed` — if the new body is
    /// structurally identical (span-agnostic), the redef is a no-op
    /// rather than `DefRedefForbidden`.
    pub(crate) defined_value_asts: HashMap<String, WatAST>,
    /// Stone A0 — corpus-wide `def`-bound value types, seeded once from
    /// the live `runtime_def_values` (typed per scalar) in `from_symbols`. RESOLUTION-ONLY:
    /// consulted as a fallback by `get_defined_value_type` when a name
    /// isn't (yet) in the per-file `defined_values` map, so a function
    /// body re-checked cross-file can reference a stdlib value-const by
    /// name (e.g. `:wat::spawn::DEFAULT-MAX-MESSAGE-BYTES`) instead of
    /// requiring the corpus to inline the literal.
    ///
    /// Deliberately NOT read by the redef "first binding" check
    /// (`check.rs`'s `!env.defined_values.contains_key(&name)`, and
    /// `env.rs`'s `register_defclause` guard) — those stay on the raw
    /// per-file `defined_values` map so a file's own first `def` of a
    /// name is still treated as the first binding, never colliding with
    /// this corpus seed. Resolution and redef-tracking are decomplected
    /// on purpose (option iii): this map only ever grows via the one
    /// seed loop in `from_symbols` and is never written to elsewhere.
    pub(crate) corpus_values: HashMap<String, TypeExpr>,
    /// Names present in `SymbolTable::functions` — the same set `sym.has_function`
    /// uses to emit `EvalSignal::TailCall`. Builtins/defclauses that are not
    /// Function entries are absent, matching the runtime.
    pub(crate) registered_functions: HashSet<String>,
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
    pub fn from_symbols(sym: &'a SymbolTable, types: &'a TypeEnv) -> Result<CheckEnv<'a>, Box<CheckError>> {
        let mut env = Self::with_builtins_and_types(types);
        for (path, func) in sym.functions_iter() {
            env.registered_functions.insert(path.clone());
            if let Some(scheme) = super::derive_scheme_from_function(func) {
                // Arc 170 — the OVERLAY lands through the gate, not a bare insert.
                // Privilege::Stdlib here because this loop replays an ALREADY-FROZEN
                // symbol table: reserved-prefix policing for user code happens at
                // define-registration, upstream of the freeze, so re-asserting it here
                // would reject the stdlib's own `:wat::` functions. What this call is
                // for is the Divergent arm — a name meaning two different things.
                //
                // Arc 296 stone I — this used to be `eprintln!("GATE-REJECT...")` and
                // keep going ("Loud on purpose while we learn what the corpus holds").
                // It is now a real, located error: a divergent scheme reaching here was
                // being silently accepted. `?` performs the `Rejection` -> `CheckError`
                // taxonomy conversion.
                env.register_overlay(path.clone(), scheme, crate::resolve::Privilege::Stdlib)?;
            }
        }
        // Stone 237.8b — also load defclauses from runtime_def_values so the checker
        // can dispatch calls to stdlib defclauses (:wat::core::+, -, *, /, <, >, <=, >=).
        // Previously, stdlib ops were Rust intrinsics (infer_arithmetic etc.); now they
        // are defclauses that live in runtime_def_values after register_stdlib_defclauses.
        // Arc 232 Stone 232.1 — also load extend_registrations
        // from the extend-def Values in runtime_def_values.
        for (name, value) in sym.def_values_iter() {
            match value {
                crate::runtime::Value::wat__core__clauses(cs) => {
                    let clauses: Vec<(Vec<TypeExpr>, TypeExpr, bool)> = cs.clauses.iter()
                        .map(|clause| {
                            let arg_types: Vec<TypeExpr> = clause.args.fixed_params.iter()
                                .map(|(_, t)| t.clone())
                                .collect();
                            let has_rest = clause.args.rest_param.is_some();
                            (arg_types, clause.return_type.clone(), has_rest)
                        })
                        .collect();
                    env.defclause_registrations.insert(name.clone(), clauses);
                }
                crate::runtime::Value::wat__core__extend_def(ed) => {
                    let method_names: Vec<String> = ed.impl_clauses.keys().cloned().collect();
                    env.extend_registrations.insert(
                        (ed.protocol_name.clone(), ed.type_name.clone()),
                        method_names,
                    );
                }
                // Stone A0 — seed corpus-wide `def`-bound SCALAR value types
                // (resolution-only; see `corpus_values` field doc) so a
                // function body re-checked cross-file can reference a
                // stdlib value-const by name (e.g.
                // `:wat::spawn::DEFAULT-MAX-MESSAGE-BYTES`) instead of the
                // corpus inlining the literal. `sym.defined_values` (the
                // originally-sketched seed source) has no writer anywhere
                // in the tree — dead field, deleted (see symbol_table.rs).
                // `sym.runtime_def_values` IS live (populated by
                // `register_stdlib_runtime_defs`'s "Arc 255 escape-hatch"
                // arm for scalar stdlib `def` forms), so derive the
                // `TypeExpr` straight from the `Value`'s own scalar shape
                // — exact for scalars, the only consumers here. A value
                // with no obvious scalar type (Vec, Fn, aggregate, …) is
                // skipped — no consumer needs it and guessing is worse
                // than silence.
                other => {
                    let scalar_ty = match other {
                        crate::runtime::Value::i64(_) => Some(":wat::core::i64"),
                        crate::runtime::Value::u8(_) => Some(":wat::core::u8"),
                        crate::runtime::Value::f64(_) => Some(":wat::core::f64"),
                        crate::runtime::Value::bool(_) => Some(":wat::core::bool"),
                        crate::runtime::Value::String(_) => Some(":wat::core::String"),
                        crate::runtime::Value::wat__core__keyword(_) => Some(":wat::core::keyword"),
                        _ => None,
                    };
                    if let Some(ty_path) = scalar_ty {
                        env.corpus_values.insert(name.clone(), TypeExpr::Path(ty_path.into()));
                    }
                }
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
        Ok(env)
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
            defined_value_asts: HashMap::new(),
            extend_registrations: HashMap::new(),
            corpus_values: HashMap::new(),
            registered_functions: HashSet::new(),
        }
    }

    /// True when `path` is a registered Function — the checker's twin of
    /// `SymbolTable::has_function`. A builtin/defclause head is false.
    pub fn has_registered_function(&self, path: &str) -> bool {
        self.registered_functions.contains(path)
    }

    /// Arc 048 — look up the enum type for a unit-variant keyword
    /// path. Returns `None` for non-variant keywords.
    pub fn unit_variant_type(&self, key: &str) -> Option<&TypeExpr> {
        self.unit_variant_types.get(key)
    }

    /// Register a BASE-layer scheme at `name` — the substrate primitive table.
    ///
    /// Ungated ON PURPOSE, and the purpose is narrow: `register_builtins` fills an
    /// EMPTY map with distinct names, so there is no predecessor a registration
    /// could disagree with. Anything layering ON TOP of that base must go through
    /// [`register_overlay`], which asks the gate.
    pub fn register(&mut self, name: String, scheme: TypeScheme) {
        self.schemes.insert(name, scheme);
    }

    /// Register an OVERLAY scheme — a definition landing on top of the base table.
    ///
    /// # Why this is gated and [`register`] is not
    ///
    /// Types have routed every registration through ONE gate since arc 054
    /// (`TypeEnv::register_validated` → `resolve::gate`): a byte-equivalent
    /// redeclaration is a `NoOp`, a DIVERGENT one is a hard located error. Macros
    /// route through the same gate. **Verbs did not** — this path was a bare
    /// `schemes.insert`, so a function silently clobbered whatever held its name,
    /// and the substrate had no way to answer "is this name already taken by
    /// something DIFFERENT?" That asymmetry is the arc-170 `0z` blocker: it is why
    /// renaming a prime could not be proven safe by construction.
    ///
    /// The gate is registry-agnostic and error-taxonomy-neutral by design (its own
    /// doc says so), so wiring this path to it needs no new policy — only the
    /// equivalence relation, which is `TypeScheme`'s derived `PartialEq`.
    ///
    /// Measured before the change: across 60 real freezes, exactly ONE name
    /// registers twice (`:wat::io::read-file`, declared BOTH as a builtin scheme at
    /// `check.rs` and as a `defn` in `wat/io.wat`) and it registers IDENTICALLY —
    /// zero divergent clobbers. So this gate is not expected to reject anything
    /// that exists today; it exists so that the day something divergent appears,
    /// it is a located error instead of a silent last-writer-wins.
    pub fn register_overlay(
        &mut self,
        name: String,
        scheme: TypeScheme,
        privilege: crate::resolve::Privilege,
    ) -> Result<(), crate::resolve::Rejection> {
        let existing = match self.schemes.get(&name) {
            None => crate::resolve::Existing::Absent,
            Some(prev) if prev == &scheme => crate::resolve::Existing::Equivalent,
            Some(_) => crate::resolve::Existing::Divergent,
        };
        // Arc 296 stone I — no form span at this call site (`from_symbols` replays an
        // already-frozen SymbolTable, not source forms); `rust_caller_span!()` is the
        // honest answer.
        let span = crate::rust_caller_span!();
        crate::resolve::register(&name, privilege, existing, &span, || -> Result<(), crate::resolve::Rejection> {
            self.schemes.insert(name.clone(), scheme);
            Ok(())
        })?;
        Ok(())
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
        // Stone A0 — per-file `defined_values` wins; `corpus_values` (seeded
        // once in `from_symbols` from the live `runtime_def_values`) is the
        // fallback for a stdlib value-const this file never `def`s itself.
        self.defined_values.get(name).or_else(|| self.corpus_values.get(name))
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

    /// Arc 054 — store the body AST for byte-equivalence checking on redef.
    /// Called alongside `register_defined_value` at the first registration site.
    pub fn register_defined_value_ast(&mut self, name: &str, ast: WatAST) {
        self.defined_value_asts.insert(name.to_string(), ast);
    }

    /// Arc 054 — look up the stored body AST for a `def`-bound name.
    /// Returns `Some(&WatAST)` when the body was stored at registration time.
    pub fn get_defined_value_ast(&self, name: &str) -> Option<&WatAST> {
        self.defined_value_asts.get(name)
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

    /// Arc 232 Stone 232.1 — register a `(protocol, type)` satisfaction edge.
    /// Called from `collect_splice_defs_ctx` when it encounters a
    /// `:wat::core::extend-type` top-level form.
    /// `method_names` lists the method names the implementation provides.
    pub fn register_extend(
        &mut self,
        protocol_name: String,
        type_name: String,
        method_names: Vec<String>,
    ) {
        self.extend_registrations.insert((protocol_name, type_name), method_names);
    }

    /// Arc 232 Stone 232.1 — look up the method names provided by a
    /// `(protocol, type)` extend-type implementation. The KEY's existence
    /// is the satisfaction signal for 232.2's `assignable(T, :P)`.
    /// Returns `None` if no `extend-type` has declared this `(P, T)` edge.
    pub fn get_extend_methods(
        &self,
        protocol_name: &str,
        type_name: &str,
    ) -> Option<&[String]> {
        self.extend_registrations
            .get(&(protocol_name.to_string(), type_name.to_string()))
            .map(|v| v.as_slice())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::runtime::{SymbolTable, Value};
    use crate::types::{TypeEnv, TypeExpr};
    use crate::value::{ClauseSet, Clause};
    use std::sync::Arc;

    /// Line 146: `from_symbols` inserts defclause registrations from
    /// `runtime_def_values` entries that are `Value::wat__core__clauses`.
    ///
    /// Constructs a `SymbolTable` with one `clauses` entry in `runtime_def_values`
    /// and asserts that `CheckEnv::from_symbols` populates `defclause_registrations`
    /// with the correct clause table for that name.
    #[test]
    fn from_symbols_loads_defclause_from_runtime_def_values() {
        // Build a minimal ClauseSet with one clause: no args, returns :nil.
        let nil_body = crate::ast::WatAST::nil();
        let clause = Clause {
            args: crate::argspec::ArgSpec { fixed_params: vec![], rest_param: None },
            return_type: TypeExpr::Path(":wat::core::nil".into()),
            guard: None,
            ensure_fn: None,
            body: Arc::new(nil_body),
            // Checker-side fixture: never reaches the evaluator, so no compiled Function.
            func: None,
        };
        let cs = Arc::new(ClauseSet {
            name: ":my::op".into(),
            clauses: vec![clause],
            shared_return: None,
            metadata: None,
        });
        let mut sym = SymbolTable::new();
        sym.register_def_value(":my::op".into(), Value::wat__core__clauses(cs));

        let types = TypeEnv::default();
        let env = CheckEnv::from_symbols(&sym, &types).expect("from_symbols ok");

        // The defclause must be in the check env's registration table.
        let clauses = env.get_defclause_clauses(":my::op")
            .expect("defclause must be registered after from_symbols");
        assert_eq!(clauses.len(), 1, "expected 1 clause; got: {}", clauses.len());
        // The clause has no fixed args (has_rest = false).
        let (arg_types, _, has_rest) = &clauses[0];
        assert!(arg_types.is_empty(), "expected zero arg types; got: {:?}", arg_types);
        assert!(!has_rest, "expected has_rest=false for zero-rest clause");
    }

}
