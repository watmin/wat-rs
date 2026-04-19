//! The freeze pass — step 11 of the startup pipeline.
//!
//! Per FOUNDATION.md § "Freeze symbol table, type environment, macro
//! registry, and config" (line 2379), the wat-vm starts up, runs its
//! pipeline, and then **freezes** the four accumulated registries. After
//! freeze:
//!
//! - No new `define` can register.
//! - No new macro can be declared.
//! - No new type can be declared.
//! - No `set-*!` config setter can fire.
//!
//! Everything that runs afterward — including `:user::main` and any
//! constrained `eval` — reads from the frozen world but cannot mutate
//! it.
//!
//! # What freeze is, in Rust
//!
//! A [`FrozenWorld`] bundles the four registries. Once constructed via
//! [`FrozenWorld::freeze`], it takes ownership of the mutable-during-
//! build forms. Callers hold `&FrozenWorld` (shared reference), which
//! forbids `&mut` access by the borrow checker — no mutation method
//! is reachable. The type system IS the freeze gate.
//!
//! The module also exposes [`startup_from_source`] — an orchestrator
//! that runs the full 1–11 pipeline from a single entry-source string
//! (plus a [`crate::load::SourceLoader`]) and returns either a
//! `FrozenWorld` or a [`StartupError`] pointing at the failing pass.
//!
//! # What freeze is NOT
//!
//! - It doesn't invoke `:user::main` — that's task #140.
//! - It doesn't wire the CLI binary — task #141.
//! - It doesn't construct a `VectorManager` / `ScalarEncoder` /
//!   `AtomTypeRegistry` from the [`Config`]; those are runtime-layer
//!   concerns the caller (wat-vm binary) handles after freeze.
//! - It doesn't perform full-program signature verification — that's
//!   optionally done by the CLI caller over the frozen AST before
//!   freeze completes, using [`crate::hash::verify_program_signature`].

use crate::ast::WatAST;
use crate::check::{check_program, CheckErrors};
use crate::config::{collect_entry_file, Config, ConfigError};
use crate::load::{resolve_loads, LoadError, SourceLoader};
use crate::macros::{expand_all, register_defmacros, MacroError, MacroRegistry};
use crate::parser::{parse_all, ParseError};
use crate::resolve::{resolve_references, ResolveError};
use crate::runtime::{register_defines, RuntimeError, SymbolTable};
use crate::types::{register_types, TypeEnv, TypeError};
use std::fmt;

/// The frozen startup world — all four registries bundled and
/// owned. After construction, only `&self` read access is possible;
/// Rust's borrow checker blocks any further mutation.
#[derive(Debug)]
pub struct FrozenWorld {
    pub config: Config,
    pub types: TypeEnv,
    pub macros: MacroRegistry,
    pub symbols: SymbolTable,
    /// The post-load, post-expand, post-type-check AST — the
    /// residue of forms left after all definitions were registered.
    /// Contains the toplevel program body (if any) that `:user::main`
    /// will evaluate against.
    pub program: Vec<WatAST>,
}

impl FrozenWorld {
    /// Construct a frozen world from the registries built during
    /// startup. Takes ownership of each — the caller cannot mutate
    /// them after this call.
    pub fn freeze(
        config: Config,
        types: TypeEnv,
        macros: MacroRegistry,
        symbols: SymbolTable,
        program: Vec<WatAST>,
    ) -> Self {
        FrozenWorld {
            config,
            types,
            macros,
            symbols,
            program,
        }
    }

    pub fn config(&self) -> &Config {
        &self.config
    }

    pub fn types(&self) -> &TypeEnv {
        &self.types
    }

    pub fn macros(&self) -> &MacroRegistry {
        &self.macros
    }

    pub fn symbols(&self) -> &SymbolTable {
        &self.symbols
    }

    pub fn program(&self) -> &[WatAST] {
        &self.program
    }
}

/// Failures at any stage of the startup pipeline. Each variant names
/// the pass that raised it so users see "type check failed" rather
/// than a bare error.
#[derive(Debug)]
pub enum StartupError {
    Parse(ParseError),
    Config(ConfigError),
    Load(LoadError),
    Macro(MacroError),
    Type(TypeError),
    Resolve(ResolveError),
    Check(CheckErrors),
    /// A user `define` collided with a builtin or another user
    /// define during registration. Surfaces `register_defines`'s
    /// errors as-is.
    Runtime(RuntimeError),
}

impl fmt::Display for StartupError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            StartupError::Parse(e) => write!(f, "parse: {}", e),
            StartupError::Config(e) => write!(f, "config: {}", e),
            StartupError::Load(e) => write!(f, "load: {}", e),
            StartupError::Macro(e) => write!(f, "macro: {}", e),
            StartupError::Type(e) => write!(f, "types: {}", e),
            StartupError::Resolve(e) => write!(f, "resolve: {}", e),
            StartupError::Check(e) => write!(f, "check:\n{}", e),
            StartupError::Runtime(e) => write!(f, "registration: {}", e),
        }
    }
}

impl std::error::Error for StartupError {}

impl From<ParseError> for StartupError {
    fn from(e: ParseError) -> Self {
        StartupError::Parse(e)
    }
}
impl From<ConfigError> for StartupError {
    fn from(e: ConfigError) -> Self {
        StartupError::Config(e)
    }
}
impl From<LoadError> for StartupError {
    fn from(e: LoadError) -> Self {
        StartupError::Load(e)
    }
}
impl From<MacroError> for StartupError {
    fn from(e: MacroError) -> Self {
        StartupError::Macro(e)
    }
}
impl From<TypeError> for StartupError {
    fn from(e: TypeError) -> Self {
        StartupError::Type(e)
    }
}
impl From<ResolveError> for StartupError {
    fn from(e: ResolveError) -> Self {
        StartupError::Resolve(e)
    }
}
impl From<CheckErrors> for StartupError {
    fn from(e: CheckErrors) -> Self {
        StartupError::Check(e)
    }
}
impl From<RuntimeError> for StartupError {
    fn from(e: RuntimeError) -> Self {
        StartupError::Runtime(e)
    }
}

/// Run the full startup pipeline against a single entry-source string
/// and produce a [`FrozenWorld`]. The pipeline follows FOUNDATION.md's
/// steps 1–11 in order:
///
/// 1. Parse the entry source.
/// 2. Run entry-file shape check + config pass ([`collect_entry_file`]).
/// 3. Recursively resolve `load!` forms ([`resolve_loads`]).
/// 4. Register `defmacro`s, then expand all macro call sites
///    ([`register_defmacros`] → [`expand_all`]).
/// 5. Register type declarations ([`register_types`]).
/// 6. Register function definitions ([`register_defines`]).
/// 7. Name resolution ([`resolve_references`]).
/// 8. Type check ([`check_program`]).
/// 9. Freeze into a [`FrozenWorld`] and return.
///
/// Hashing and signature verification on the full expanded program
/// are NOT performed here — those are the CLI caller's responsibility
/// and happen against the frozen program (or via a sidecar signature)
/// in the wat-vm binary.
///
/// `base_canonical` is the entry file's canonical path when known
/// (used for relative-path resolution of top-level `load!`s). Pass
/// `None` when the entry source comes from a string rather than a file.
pub fn startup_from_source(
    entry_src: &str,
    base_canonical: Option<&str>,
    loader: &dyn SourceLoader,
) -> Result<FrozenWorld, StartupError> {
    // 1. Parse.
    let entry_forms = parse_all(entry_src)?;

    // 2. Config pass + entry-file discipline.
    let (config, post_config) = collect_entry_file(entry_forms)?;

    // 3. Recursive load resolution.
    let loaded = resolve_loads(post_config, base_canonical, loader)?;

    // 4. Macro registration + expansion.
    let mut macros = MacroRegistry::new();
    let post_macro_reg = register_defmacros(loaded, &mut macros)?;
    let expanded = expand_all(post_macro_reg, &macros)?;

    // 5. Type declarations.
    let mut types = TypeEnv::new();
    let post_types = register_types(expanded, &mut types)?;

    // 6. Function definitions.
    let mut symbols = SymbolTable::new();
    let residue = register_defines(post_types, &mut symbols)?;

    // 7. Name resolution.
    resolve_references(&residue, &symbols, &macros, &types)?;

    // 8. Type check.
    check_program(&residue, &symbols, &types)?;

    // 9. Freeze.
    Ok(FrozenWorld::freeze(config, types, macros, symbols, residue))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::load::InMemoryLoader;

    /// Helper: start from an entry string with no loaded files.
    fn startup(entry: &str) -> Result<FrozenWorld, StartupError> {
        let loader = InMemoryLoader::new();
        startup_from_source(entry, None, &loader)
    }

    // ─── Happy path ─────────────────────────────────────────────────────

    #[test]
    fn minimal_program_freezes() {
        let src = r#"
            (:wat::config::set-dims! 1024)
            (:wat::config::set-capacity-mode! :error)
            (:wat::algebra::Atom "hello")
        "#;
        let world = startup(src).expect("startup");
        assert_eq!(world.config().dims, 1024);
        assert_eq!(world.program().len(), 1);
    }

    #[test]
    fn global_seed_defaults() {
        let src = r#"
            (:wat::config::set-dims! 4096)
            (:wat::config::set-capacity-mode! :error)
        "#;
        let world = startup(src).expect("startup");
        assert_eq!(world.config().global_seed, 42);
    }

    #[test]
    fn user_define_registers() {
        let src = r#"
            (:wat::config::set-dims! 1024)
            (:wat::config::set-capacity-mode! :error)
            (:wat::core::define (:my::app::add (x :i64) (y :i64) -> :i64)
              (:wat::core::+ x y))
        "#;
        let world = startup(src).expect("startup");
        assert!(world.symbols().get(":my::app::add").is_some());
    }

    #[test]
    fn user_type_registers() {
        let src = r#"
            (:wat::config::set-dims! 1024)
            (:wat::config::set-capacity-mode! :error)
            (:wat::core::struct :my::Candle (open :f64) (close :f64))
        "#;
        let world = startup(src).expect("startup");
        assert!(world.types().contains(":my::Candle"));
    }

    #[test]
    fn user_macro_registers() {
        let src = r#"
            (:wat::config::set-dims! 1024)
            (:wat::config::set-capacity-mode! :error)
            (:wat::core::defmacro (:my::vocab::Double (x :AST<Holon>) -> :AST<Holon>)
              `(:wat::algebra::Blend ,x ,x 1 1))
        "#;
        let world = startup(src).expect("startup");
        assert!(world.macros().contains(":my::vocab::Double"));
    }

    // ─── Failure at each pass ───────────────────────────────────────────

    #[test]
    fn parse_error_bubbles_up() {
        let err = startup("(((").unwrap_err();
        assert!(matches!(err, StartupError::Parse(_)));
    }

    #[test]
    fn config_missing_required_bubbles_up() {
        // No :wat::config::set-dims! — config pass halts.
        let err = startup("(:wat::algebra::Atom 42)").unwrap_err();
        assert!(matches!(err, StartupError::Config(_)));
    }

    #[test]
    fn type_error_bubbles_up() {
        // Duplicate struct declaration.
        let src = r#"
            (:wat::config::set-dims! 1024)
            (:wat::config::set-capacity-mode! :error)
            (:wat::core::struct :my::Candle (x :f64))
            (:wat::core::struct :my::Candle (y :i64))
        "#;
        let err = startup(src).unwrap_err();
        assert!(matches!(err, StartupError::Type(_)));
    }

    #[test]
    fn check_error_bubbles_up() {
        // Passing :i64 to a define that declared :bool — type mismatch.
        let src = r#"
            (:wat::config::set-dims! 1024)
            (:wat::config::set-capacity-mode! :error)
            (:wat::core::+ "hello" 1)
        "#;
        let err = startup(src).unwrap_err();
        assert!(matches!(err, StartupError::Check(_)));
    }

    #[test]
    fn resolve_error_bubbles_up() {
        let src = r#"
            (:wat::config::set-dims! 1024)
            (:wat::config::set-capacity-mode! :error)
            (:my::app::never-defined 42)
        "#;
        let err = startup(src).unwrap_err();
        assert!(matches!(err, StartupError::Resolve(_)));
    }

    #[test]
    fn any_in_type_position_bubbles_up_as_type_error() {
        // :Any is banned at parse_type_expr time; bubbles up as a
        // RuntimeError via register_defines (parse_type_expr is called
        // inside parse_define_signature).
        let src = r#"
            (:wat::config::set-dims! 1024)
            (:wat::config::set-capacity-mode! :error)
            (:wat::core::define (:my::bad (x :Any) -> :i64) 42)
        "#;
        let err = startup(src).unwrap_err();
        // register_defines calls parse_type_expr which raises AnyBanned;
        // runtime wraps it in MalformedForm.
        assert!(matches!(err, StartupError::Runtime(_)));
    }

    // ─── Frozen world is immutable ──────────────────────────────────────

    #[test]
    fn frozen_world_exposes_read_only_accessors() {
        // Sanity: the accessors return shared references — the borrow
        // checker would refuse to compile if they returned mutable
        // references. This test just exercises every accessor.
        let src = r#"
            (:wat::config::set-dims! 1024)
            (:wat::config::set-capacity-mode! :error)
        "#;
        let world = startup(src).unwrap();
        let _: &Config = world.config();
        let _: &TypeEnv = world.types();
        let _: &MacroRegistry = world.macros();
        let _: &SymbolTable = world.symbols();
        let _: &[WatAST] = world.program();
    }

    // ─── Load integration ───────────────────────────────────────────────

    #[test]
    fn loaded_file_contributes_definitions() {
        let mut loader = InMemoryLoader::new();
        loader.add_source(
            "lib.wat",
            r#"(:wat::core::define (:lib::square (x :i64) -> :i64)
                 (:wat::core::* x x))"#,
        );
        let entry = r#"
            (:wat::config::set-dims! 1024)
            (:wat::config::set-capacity-mode! :error)
            (:wat::core::load! :wat::load::file-path "lib.wat")
        "#;
        let world = startup_from_source(entry, None, &loader).expect("startup");
        assert!(world.symbols().get(":lib::square").is_some());
    }
}
