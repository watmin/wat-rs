//! The freeze pass — step 11 of the startup pipeline.
//!
//! Per FOUNDATION.md § "Freeze symbol table, type environment, macro
//! registry, and config" (line 2379), the wat starts up, runs its
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
//! - It doesn't invoke `:user::main` — that's the wat binary's job.
//! - It doesn't perform signature verification at the whole-program
//!   level. Signature verification is per-form — inside
//!   `(:wat::signed-load! ...)` at startup and
//!   `(:wat::eval-signed! ...)` at runtime. Each form carries
//!   its own `sig` / `pubkey` payloads and verifies its own SHA-256
//!   of canonical-EDN via [`crate::hash::verify_program_signature`].
//!   There is no CLI flag for a "full-program" signature; a program's
//!   verification surface is its collection of signed-* forms. See
//!   FOUNDATION's cryptographic-provenance section.
//!
//! What freeze DOES construct, beyond the four registries:
//!
//! - An [`EncodingCtx`] (`VectorManager` + `ScalarEncoder` +
//!   `AtomTypeRegistry` with a `WatAST` canonicalizer registered)
//!   built from the committed [`Config`] and attached to the
//!   [`SymbolTable`]. Runtime primitives that need to project holons
//!   into their vectors (`:wat::holon::cosine`,
//!   `:wat::config::noise-floor`) reach it via dispatch.

use crate::ast::WatAST;
use crate::check::{check_program, CheckErrors};
use crate::config::{collect_entry_file, Config, ConfigError};
use crate::load::{resolve_loads, LoadError, SourceLoader};
use crate::macros::{
    expand_all, register_defmacros, register_stdlib_defmacros, MacroError, MacroRegistry,
};
use crate::parser::{parse_all_with_file, ParseError};
use crate::stdlib::{stdlib_forms, StdlibError};
use crate::resolve::{resolve_references, ResolveError};
use crate::runtime::{
    apply_function, register_defines, register_stdlib_defines, EncodingCtx, Environment,
    RuntimeError, SymbolTable, Value,
};
use crate::types::{register_stdlib_types, register_types, TypeEnv, TypeError, TypeExpr};
use std::fmt;
use std::sync::Arc;

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
    ///
    /// Also constructs an [`EncodingCtx`] from `config` and attaches it
    /// to `symbols`, so runtime primitives that project holons into
    /// their vectors (`:wat::holon::cosine`, `:wat::config::noise-floor`)
    /// have access at dispatch. Per FOUNDATION 1718, presence is the
    /// retrieval primitive; it is only reachable once freeze has
    /// committed `dims` / `global_seed` / `noise_floor` and built the
    /// `VectorManager` + `ScalarEncoder` + `AtomTypeRegistry`.
    pub fn freeze(
        config: Config,
        types: TypeEnv,
        macros: MacroRegistry,
        mut symbols: SymbolTable,
        program: Vec<WatAST>,
        loader: Arc<dyn crate::load::SourceLoader>,
    ) -> Self {
        let ctx = Arc::new(EncodingCtx::from_config(&config));
        symbols.set_encoding_ctx(ctx);
        symbols.set_source_loader(loader);
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
    /// A baked stdlib source failed to parse. Should never fire in
    /// production — the stdlib is authored in-repo and its parsing is
    /// validated by `cargo test` — but surfaces cleanly if someone
    /// ships a malformed stdlib file.
    Stdlib(StdlibError),
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
            StartupError::Stdlib(e) => write!(f, "stdlib: {}", e),
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
impl From<StdlibError> for StartupError {
    fn from(e: StdlibError) -> Self {
        StartupError::Stdlib(e)
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
/// in the wat binary.
///
/// `base_canonical` is the entry file's canonical path when known
/// (used for relative-path resolution of top-level `load!`s). Pass
/// `None` when the entry source comes from a string rather than a file.
pub fn startup_from_source(
    entry_src: &str,
    base_canonical: Option<&str>,
    loader: Arc<dyn SourceLoader>,
) -> Result<FrozenWorld, StartupError> {
    // 1. Parse. Post-parse the pipeline is shared with
    //    `startup_from_forms` — callers that already hold AST (macros
    //    expanding to sandboxed programs, dynamically-generated
    //    tests, compiler passes) skip the parse + re-serialize round
    //    trip by entering there directly.
    //
    // Span file label: use the canonical path when known; fall back
    // to `<entry>` for in-memory / test sources. Arc 016 slice 1.
    let file_label = base_canonical.unwrap_or("<entry>");
    let entry_forms = parse_all_with_file(entry_src, file_label)?;
    startup_from_forms(entry_forms, base_canonical, loader)
}

// startup_from_source_with_deps retired in arc 015 slice 3a.
// Dep sources now install globally via `wat::source::install_dep_sources`
// before any freezing; `stdlib_forms()` concatenates baked + installed
// so every freeze pass — including `:wat::kernel::run-sandboxed-ast`
// and `:wat::kernel::fork-with-forms` children — sees dep surface
// transparently. Callers build the composition through `compose_and_run`
// / `Harness::from_source_with_deps` / `test_runner::run_tests_from_dir`,
// each of which installs then calls `startup_from_source`.

/// Post-parse entry to the startup pipeline: accepts already-parsed
/// `Vec<WatAST>` forms and runs steps 2–9 (config → load → macros →
/// types → defines → resolve → check → freeze).
///
/// Arc 007 slice 3b splits this out so `:wat::kernel::run-sandboxed-ast`
/// can freeze a program the caller built as AST (e.g., the expansion
/// of a `deftest` macro) without serializing back to source and
/// re-parsing. Same pipeline, one boundary exposed — the
/// source-text path now composes `parse_all` with this function
/// rather than carrying the steps inline.
pub fn startup_from_forms(
    entry_forms: Vec<WatAST>,
    base_canonical: Option<&str>,
    loader: Arc<dyn SourceLoader>,
) -> Result<FrozenWorld, StartupError> {
    // 2. Config pass + entry-file discipline.
    let (config, post_config) = collect_entry_file(entry_forms)?;

    // 3. Recursive load resolution. The loader survives into the
    //    runtime as well — see step 9 — so `resolve_loads` borrows
    //    via `&*loader` (Arc deref) rather than owning.
    let loaded = resolve_loads(post_config, base_canonical, &*loader)?;

    // 3a. Baked stdlib. Registered ahead of user code so any
    //     `(:wat::holon::Subtract …)` / `(:wat::holon::Amplify …)` call
    //     in user source resolves during step 4's macro expansion
    //     without an explicit `load!`. Per FOUNDATION § "Where Each
    //     Lives" (line 2088), `wat/std/*.wat` files ship one form
    //     each whose keyword path matches the file path.
    let stdlib = stdlib_forms()?;

    // 4. Macro registration + expansion. Stdlib defmacros register
    //    first; user defmacros layer on top and can shadow (subject
    //    to the reserved-prefix gate) or reference stdlib forms.
    let mut macros = MacroRegistry::new();
    let stdlib_post_macros = register_stdlib_defmacros(stdlib, &mut macros)?;
    let post_macro_reg = register_defmacros(loaded, &mut macros)?;
    // Expand BOTH stdlib non-defmacro residue and user forms against
    // the combined macro registry. Stdlib functions are authored
    // against stdlib defmacros too — e.g., :wat::std::service::Console's
    // body uses :wat::holon::Subtract / list helpers / etc.
    let expanded_stdlib = expand_all(stdlib_post_macros, &macros)?;
    let expanded_user = expand_all(post_macro_reg, &macros)?;

    // 5. Type declarations. Seeded with wat-rs's own :wat::*
    //    built-in types (e.g., :wat::holon::CapacityExceeded)
    //    before stdlib and user source land; those declarations
    //    cannot be re-declared by user code (the reserved-prefix
    //    gate blocks at `TypeEnv::register`).
    let mut types = TypeEnv::with_builtins();
    let stdlib_post_types = register_stdlib_types(expanded_stdlib, &mut types)?;
    let post_types = register_types(expanded_user, &mut types)?;

    // 6. Function definitions. Stdlib defines bypass the reserved-
    //    prefix gate (they live under :wat::std::* by design); user
    //    defines still go through register_defines where the gate
    //    blocks mis-namespaced user source.
    let mut symbols = SymbolTable::new();
    let _stdlib_function_residue = register_stdlib_defines(stdlib_post_types, &mut symbols)?;
    let residue = register_defines(post_types, &mut symbols)?;

    // 6a. Struct auto-methods. For every `(:wat::core::struct ...)`
    //     declaration (built-in + user), synthesize its `/new`
    //     constructor and one `/<field>` accessor per field, all as
    //     ordinary `Function` entries in the symbol table. Runs
    //     after user defines so collisions with user-authored names
    //     surface as `DuplicateDefine`.
    crate::runtime::register_struct_methods(&types, &mut symbols)?;

    // 7. Name resolution.
    resolve_references(&residue, &symbols, &macros)?;

    // 8. Type check.
    check_program(&residue, &symbols, &types)?;

    // 9. Freeze. The loader moves into the frozen world's
    //    SymbolTable so runtime primitives (`:wat::eval-file!` and
    //    the file-path variants of the verified eval/load forms,
    //    `:wat::verify::file-path` payloads) can route through the
    //    same capability that handled startup loads.
    Ok(FrozenWorld::freeze(
        config,
        types,
        macros,
        symbols,
        residue,
        loader,
    ))
}

// ─── :user::main invocation ─────────────────────────────────────────────

/// Canonical path for the user's entry-point slot. Per FOUNDATION.md
/// (line 1072): `:user::main` is kernel-REQUIRED (user provides;
/// kernel invokes). Zero or more-than-one declarations halt.
pub const USER_MAIN_PATH: &str = ":user::main";

/// Look up `:user::main` in the frozen world and apply it to the
/// provided argument values.
///
/// Per FOUNDATION § "The kernel invokes `:user::main` with four
/// parameters" (line 1041), the kernel hands the user's entry point
/// four channel values — `stdin`, `stdout`, `stderr`, `signals` —
/// plus any additional typed state the deployment signature declares.
/// This function is agnostic to the number / type of arguments; the
/// caller (the wat CLI binary in `src/bin/wat.rs`) constructs the channel
/// [`Value`]s and passes them in. Arity mismatch is caught by
/// [`apply_function`] and surfaces as `ArityMismatch`.
pub fn invoke_user_main(
    frozen: &FrozenWorld,
    args: Vec<Value>,
) -> Result<Value, RuntimeError> {
    let main_func = frozen
        .symbols()
        .get(USER_MAIN_PATH)
        .ok_or(RuntimeError::UserMainMissing)?
        .clone();
    apply_function(main_func, args, frozen.symbols(), crate::rust_caller_span!())
}

// ─── :user::main signature enforcement ──────────────────────────────────
//
// Moved here from `bin/wat.rs` in arc 007 slice 2a so
// `:wat::kernel::run-sandboxed` can reuse the same validator. The CLI
// and the sandbox primitive enforce the same contract.

/// The exact signature `:user::main` must declare. Startup halts if
/// the program's `:user::main` doesn't match.
///
/// Arc 008 (2026-04-21): stdio is passed as abstract IO values —
/// `:wat::io::IOReader` for stdin, `:wat::io::IOWriter` for stdout and
/// stderr. Production (the CLI) wraps real `std::io::Stdin` / `Stdout`
/// / `Stderr` in IO-trait-objects; tests (`run-sandboxed`) pass
/// `StringIo` stand-ins that look identical at the wat surface. Same
/// wat source runs in both. Ruby StringIO model made operational.
pub fn expected_user_main_signature() -> (Vec<TypeExpr>, TypeExpr) {
    let params = vec![
        TypeExpr::Path(":wat::io::IOReader".into()),
        TypeExpr::Path(":wat::io::IOWriter".into()),
        TypeExpr::Path(":wat::io::IOWriter".into()),
    ];
    let ret = TypeExpr::Tuple(vec![]);
    (params, ret)
}

/// Check that a frozen world's `:user::main` declares the expected
/// three-IO signature. Returns `Err(message)` with a reader-friendly
/// diagnostic naming the offending parameter or return type.
pub fn validate_user_main_signature(frozen: &FrozenWorld) -> Result<(), String> {
    let func = frozen.symbols().get(":user::main").ok_or_else(|| {
        ":user::main not defined — a wat program needs an entry point".to_string()
    })?;
    let (expected_params, expected_ret) = expected_user_main_signature();
    if func.param_types.len() != expected_params.len() {
        return Err(format!(
            ":user::main must take exactly {} parameters; got {}",
            expected_params.len(),
            func.param_types.len()
        ));
    }
    for (i, (got, want)) in func
        .param_types
        .iter()
        .zip(expected_params.iter())
        .enumerate()
    {
        if got != want {
            let slot = match i {
                0 => "stdin",
                1 => "stdout",
                2 => "stderr",
                _ => "extra",
            };
            return Err(format!(
                ":user::main parameter #{} ({}) expected {}, got {}",
                i + 1,
                slot,
                format_type_expr(want),
                format_type_expr(got)
            ));
        }
    }
    if func.ret_type != expected_ret {
        return Err(format!(
            ":user::main return type expected :(), got {}",
            format_type_expr(&func.ret_type)
        ));
    }
    Ok(())
}

/// Reader-friendly rendering of a [`TypeExpr`] for diagnostic messages.
/// Matches the surface form users write in wat source — same grammar
/// the parser accepts.
pub fn format_type_expr(t: &TypeExpr) -> String {
    match t {
        TypeExpr::Path(p) => p.clone(),
        TypeExpr::Parametric { head, args } => {
            let inner: Vec<_> = args.iter().map(format_type_expr_inner).collect();
            format!(":{}<{}>", head, inner.join(","))
        }
        TypeExpr::Fn { args, ret } => {
            let in_parts: Vec<_> = args.iter().map(format_type_expr_inner).collect();
            format!(
                ":fn({})->{}",
                in_parts.join(","),
                format_type_expr_inner(ret)
            )
        }
        TypeExpr::Tuple(elements) => {
            let inner: Vec<_> = elements.iter().map(format_type_expr_inner).collect();
            if elements.len() == 1 {
                format!(":({},)", inner[0])
            } else {
                format!(":({})", inner.join(","))
            }
        }
        TypeExpr::Var(id) => format!(":?{}", id),
    }
}

fn format_type_expr_inner(t: &TypeExpr) -> String {
    let raw = format_type_expr(t);
    raw.strip_prefix(':').unwrap_or(&raw).to_string()
}

// ─── Constrained eval ───────────────────────────────────────────────────

/// Constrained `eval` — the wat `(:wat::core::eval! ...)` form.
/// Runs an AST against the frozen world and refuses any form that
/// would mutate the startup registries.
///
/// Per FOUNDATION § "constrained eval at runtime" (line 658):
///
/// > 1. Every function called must be in the static symbol table.
/// > 2. Every type used must be in the static type universe.
/// > 3. Every argument's type must match the called function's signature.
/// > 4. Eval cannot register or replace any definition.
///
/// The fourth rule is enforced by pre-walking the AST and refusing
/// any of the ten mutation-inducing heads before evaluation starts.
/// The other three rules are enforced by the existing runtime,
/// resolve, and check passes (which already ran at startup) — once
/// the AST is confirmed mutation-free, the standard
/// [`crate::runtime::eval`] handles function lookup and argument
/// dispatch against the frozen symbol table.
///
/// Use this for: dynamic holon composition, rule-like pattern-match
/// systems, received holon-programs over the network. An attacker
/// who supplies a malicious AST cannot invoke arbitrary code — only
/// functions the operator explicitly loaded at startup.
pub fn eval_in_frozen(
    ast: &WatAST,
    frozen: &FrozenWorld,
    env: &Environment,
) -> Result<Value, RuntimeError> {
    refuse_mutation_forms(ast)?;
    crate::runtime::eval(ast, env, frozen.symbols())
}

/// Digest-verified eval — the wat `(:wat::eval-digest! ...)`
/// form. Mirrors `(:wat::digest-load! ...)`: verify the hash
/// of the canonical-EDN of the AST before any execution.
///
/// The verification target is `hash_canonical_ast(ast)` — the same
/// sha256 used for content-addressed caching / identity. Mismatch
/// produces [`RuntimeError::EvalVerificationFailed`] and NO code
/// runs. Successful verification is followed by the same mutation-
/// form refusal + delegate-to-eval path as [`eval_in_frozen`].
///
/// `algo` names the hash algorithm (e.g., `"sha256"`); `hex` is the
/// hex-encoded expected digest. Algorithm dispatch matches
/// [`crate::hash::verify_source_hash`] — other algos return
/// `UnsupportedAlgorithm`.
pub fn eval_digest_in_frozen(
    ast: &WatAST,
    frozen: &FrozenWorld,
    env: &Environment,
    algo: &str,
    expected_hex: &str,
) -> Result<Value, RuntimeError> {
    // Compute the canonical-EDN bytes and verify against expected.
    let bytes = crate::hash::canonical_edn_wat(ast);
    crate::hash::verify_source_hash(&bytes, algo, expected_hex).map_err(|err| {
        RuntimeError::EvalVerificationFailed { err }
    })?;
    eval_in_frozen(ast, frozen, env)
}

/// Signature-verified eval — the wat `(:wat::eval-signed! ...)`
/// form. Mirrors `(:wat::signed-load! ...)`: verify an Ed25519
/// (or other registered algorithm) signature over the SHA-256 of the
/// canonical-EDN of the AST before any execution.
///
/// Same signing target as `signed-load!` — this is the load-time
/// integrity story extended to runtime-received ASTs. Typical use:
/// a distributed node receives a signed holon-program over the
/// network, verifies the signature against its pinned public key,
/// evals against its frozen symbol table. Failed verification
/// produces [`RuntimeError::EvalVerificationFailed`] and NO code
/// runs.
///
/// `algo` names the signature algorithm (e.g., `"ed25519"`);
/// `sig_b64` and `pubkey_b64` are base64-encoded per the same
/// discipline as `:wat::verify::signed-ed25519` in load forms.
pub fn eval_signed_in_frozen(
    ast: &WatAST,
    frozen: &FrozenWorld,
    env: &Environment,
    algo: &str,
    sig_b64: &str,
    pubkey_b64: &str,
) -> Result<Value, RuntimeError> {
    crate::hash::verify_ast_signature(ast, algo, sig_b64, pubkey_b64).map_err(
        |err| RuntimeError::EvalVerificationFailed { err },
    )?;
    eval_in_frozen(ast, frozen, env)
}

/// Walk an AST and raise [`RuntimeError::EvalForbidsMutationForm`]
/// if any mutation-inducing head appears at any depth. The forbidden
/// set is exactly the forms that register into or modify startup
/// registries: `define`, `defmacro`, `struct`, `enum`, `newtype`,
/// `typealias`, the three `load!` variants, and any
/// `:wat::config::set-*!` setter.
fn refuse_mutation_forms(ast: &WatAST) -> Result<(), RuntimeError> {
    if let WatAST::List(items, _) = ast {
        if let Some(WatAST::Keyword(head, _)) = items.first() {
            if is_mutation_form(head) {
                return Err(RuntimeError::EvalForbidsMutationForm {
                    head: head.clone(),
                });
            }
        }
        for child in items {
            refuse_mutation_forms(child)?;
        }
    }
    Ok(())
}

fn is_mutation_form(head: &str) -> bool {
    matches!(
        head,
        ":wat::core::define"
            | ":wat::core::defmacro"
            | ":wat::core::struct"
            | ":wat::core::enum"
            | ":wat::core::newtype"
            | ":wat::core::typealias"
            | ":wat::load-file!"
            | ":wat::digest-load!"
            | ":wat::signed-load!"
    ) || head.starts_with(":wat::config::set-")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::load::InMemoryLoader;

    /// Helper: start from an entry string with no loaded files.
    fn startup(entry: &str) -> Result<FrozenWorld, StartupError> {
        startup_from_source(entry, None, Arc::new(InMemoryLoader::new()))
    }

    // ─── Happy path ─────────────────────────────────────────────────────

    #[test]
    fn minimal_program_freezes() {
        let src = r#"
            (:wat::config::set-capacity-mode! :error)
            (:wat::config::set-dims! 1024)
            (:wat::holon::Atom "hello")
        "#;
        let world = startup(src).expect("startup");
        assert_eq!(world.config().dims, 1024);
        assert_eq!(world.program().len(), 1);
    }

    #[test]
    fn global_seed_defaults() {
        let src = r#"
            (:wat::config::set-capacity-mode! :error)
            (:wat::config::set-dims! 4096)
        "#;
        let world = startup(src).expect("startup");
        assert_eq!(world.config().global_seed, 42);
    }

    #[test]
    fn user_define_registers() {
        let src = r#"
            (:wat::config::set-capacity-mode! :error)
            (:wat::config::set-dims! 1024)
            (:wat::core::define (:my::app::add (x :i64) (y :i64) -> :i64)
              (:wat::core::i64::+ x y))
        "#;
        let world = startup(src).expect("startup");
        assert!(world.symbols().get(":my::app::add").is_some());
    }

    #[test]
    fn user_type_registers() {
        let src = r#"
            (:wat::config::set-capacity-mode! :error)
            (:wat::config::set-dims! 1024)
            (:wat::core::struct :my::Candle (open :f64) (close :f64))
        "#;
        let world = startup(src).expect("startup");
        assert!(world.types().contains(":my::Candle"));
    }

    #[test]
    fn user_macro_registers() {
        let src = r#"
            (:wat::config::set-capacity-mode! :error)
            (:wat::config::set-dims! 1024)
            (:wat::core::defmacro (:my::vocab::Double (x :AST<wat::holon::HolonAST>) -> :AST<wat::holon::HolonAST>)
              `(:wat::holon::Blend ,x ,x 1 1))
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
        let err = startup("(:wat::holon::Atom 42)").unwrap_err();
        assert!(matches!(err, StartupError::Config(_)));
    }

    #[test]
    fn type_error_bubbles_up() {
        // Duplicate struct declaration.
        let src = r#"
            (:wat::config::set-capacity-mode! :error)
            (:wat::config::set-dims! 1024)
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
            (:wat::config::set-capacity-mode! :error)
            (:wat::config::set-dims! 1024)
            (:wat::core::i64::+ "hello" 1)
        "#;
        let err = startup(src).unwrap_err();
        assert!(matches!(err, StartupError::Check(_)));
    }

    #[test]
    fn resolve_error_bubbles_up() {
        let src = r#"
            (:wat::config::set-capacity-mode! :error)
            (:wat::config::set-dims! 1024)
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
            (:wat::config::set-capacity-mode! :error)
            (:wat::config::set-dims! 1024)
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
            (:wat::config::set-capacity-mode! :error)
            (:wat::config::set-dims! 1024)
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
                 (:wat::core::i64::* x x))"#,
        );
        let entry = r#"
            (:wat::config::set-capacity-mode! :error)
            (:wat::config::set-dims! 1024)
            (:wat::load-file! "lib.wat")
        "#;
        let world = startup_from_source(entry, None, Arc::new(loader)).expect("startup");
        assert!(world.symbols().get(":lib::square").is_some());
    }

    // ─── :user::main invocation ─────────────────────────────────────────

    #[test]
    fn invoke_main_happy_path() {
        // :user::main takes no arguments and returns an Int.
        let src = r#"
            (:wat::config::set-capacity-mode! :error)
            (:wat::config::set-dims! 1024)
            (:wat::core::define (:user::main -> :i64)
              (:wat::core::i64::+ 21 21))
        "#;
        let world = startup(src).expect("startup");
        let result = invoke_user_main(&world, Vec::new()).expect("main runs");
        assert!(matches!(result, Value::i64(42)));
    }

    #[test]
    fn invoke_main_calls_user_define() {
        // :user::main delegates to a user-defined helper.
        let src = r#"
            (:wat::config::set-capacity-mode! :error)
            (:wat::config::set-dims! 1024)
            (:wat::core::define (:my::app::double (x :i64) -> :i64)
              (:wat::core::i64::* x 2))
            (:wat::core::define (:user::main -> :i64)
              (:my::app::double 21))
        "#;
        let world = startup(src).expect("startup");
        let result = invoke_user_main(&world, Vec::new()).expect("main runs");
        assert!(matches!(result, Value::i64(42)));
    }

    #[test]
    fn invoke_main_missing_is_error() {
        let src = r#"
            (:wat::config::set-capacity-mode! :error)
            (:wat::config::set-dims! 1024)
        "#;
        let world = startup(src).expect("startup");
        let err = invoke_user_main(&world, Vec::new()).unwrap_err();
        assert!(matches!(err, RuntimeError::UserMainMissing));
    }

    #[test]
    fn invoke_main_wrong_arity_is_error() {
        // :user::main declared with one parameter; invoke with zero.
        let src = r#"
            (:wat::config::set-capacity-mode! :error)
            (:wat::config::set-dims! 1024)
            (:wat::core::define (:user::main (x :i64) -> :i64) x)
        "#;
        let world = startup(src).expect("startup");
        let err = invoke_user_main(&world, Vec::new()).unwrap_err();
        assert!(matches!(err, RuntimeError::ArityMismatch { expected: 1, got: 0, .. }));
    }

    // LocalCache stdlib-composition test retired in arc 013 slice 4b
    // — the wat-lru sibling crate owns that surface now. End-to-end
    // composition coverage lives in crates/wat-lru/tests/.

    #[test]
    fn invoke_main_passes_channel_value_through() {
        // :user::main takes one argument; we pass an Int as an opaque
        // stand-in for a channel value. The runtime doesn't inspect
        // the arg type — it passes through to the body.
        let src = r#"
            (:wat::config::set-capacity-mode! :error)
            (:wat::config::set-dims! 1024)
            (:wat::core::define (:user::main (x :i64) -> :i64)
              (:wat::core::i64::+ x 1))
        "#;
        let world = startup(src).expect("startup");
        let result = invoke_user_main(&world, vec![Value::i64(41)]).expect("main runs");
        assert!(matches!(result, Value::i64(42)));
    }

    // ─── Constrained eval ───────────────────────────────────────────────

    fn frozen_with(src: &str) -> FrozenWorld {
        startup(src).expect("startup")
    }

    #[test]
    fn eval_can_invoke_registered_function() {
        let world = frozen_with(
            r#"
            (:wat::config::set-capacity-mode! :error)
            (:wat::config::set-dims! 1024)
            (:wat::core::define (:my::app::triple (x :i64) -> :i64)
              (:wat::core::i64::* x 3))
        "#,
        );
        let ast = crate::parser::parse_one("(:my::app::triple 7)").unwrap();
        let env = Environment::new();
        let result = eval_in_frozen(&ast, &world, &env).expect("eval ok");
        assert!(matches!(result, Value::i64(21)));
    }

    #[test]
    fn eval_can_compose_holon_dynamically() {
        let world = frozen_with(
            r#"
            (:wat::config::set-capacity-mode! :error)
            (:wat::config::set-dims! 1024)
        "#,
        );
        let ast = crate::parser::parse_one(
            r#"(:wat::holon::Bind (:wat::holon::Atom "role") (:wat::holon::Atom "filler"))"#,
        )
        .unwrap();
        let env = Environment::new();
        let result = eval_in_frozen(&ast, &world, &env).expect("eval ok");
        assert!(matches!(result, Value::holon__HolonAST(_)));
    }

    #[test]
    fn eval_refuses_define() {
        let world = frozen_with(
            r#"
            (:wat::config::set-capacity-mode! :error)
            (:wat::config::set-dims! 1024)
        "#,
        );
        let ast = crate::parser::parse_one(
            r#"(:wat::core::define (:evil::backdoor (x :i64) -> :i64) x)"#,
        )
        .unwrap();
        let env = Environment::new();
        let err = eval_in_frozen(&ast, &world, &env).unwrap_err();
        match err {
            RuntimeError::EvalForbidsMutationForm { head } => {
                assert_eq!(head, ":wat::core::define");
            }
            other => panic!("expected EvalForbidsMutationForm, got {:?}", other),
        }
    }

    #[test]
    fn eval_refuses_defmacro() {
        let world = frozen_with(
            r#"
            (:wat::config::set-capacity-mode! :error)
            (:wat::config::set-dims! 1024)
        "#,
        );
        let ast = crate::parser::parse_one(
            r#"(:wat::core::defmacro (:evil::M (x :AST<wat::holon::HolonAST>) -> :AST<wat::holon::HolonAST>) x)"#,
        )
        .unwrap();
        let err = eval_in_frozen(&ast, &world, &Environment::new()).unwrap_err();
        assert!(matches!(err, RuntimeError::EvalForbidsMutationForm { .. }));
    }

    #[test]
    fn eval_refuses_struct() {
        let world = frozen_with(
            r#"
            (:wat::config::set-capacity-mode! :error)
            (:wat::config::set-dims! 1024)
        "#,
        );
        let ast = crate::parser::parse_one(
            r#"(:wat::core::struct :evil::T (x :i64))"#,
        )
        .unwrap();
        let err = eval_in_frozen(&ast, &world, &Environment::new()).unwrap_err();
        assert!(matches!(err, RuntimeError::EvalForbidsMutationForm { .. }));
    }

    #[test]
    fn eval_refuses_enum() {
        let world = frozen_with(
            r#"
            (:wat::config::set-capacity-mode! :error)
            (:wat::config::set-dims! 1024)
        "#,
        );
        let ast =
            crate::parser::parse_one(r#"(:wat::core::enum :evil::E :A :B)"#).unwrap();
        let err = eval_in_frozen(&ast, &world, &Environment::new()).unwrap_err();
        assert!(matches!(err, RuntimeError::EvalForbidsMutationForm { .. }));
    }

    #[test]
    fn eval_refuses_newtype() {
        let world = frozen_with(
            r#"
            (:wat::config::set-capacity-mode! :error)
            (:wat::config::set-dims! 1024)
        "#,
        );
        let ast =
            crate::parser::parse_one(r#"(:wat::core::newtype :evil::N :i64)"#).unwrap();
        let err = eval_in_frozen(&ast, &world, &Environment::new()).unwrap_err();
        assert!(matches!(err, RuntimeError::EvalForbidsMutationForm { .. }));
    }

    #[test]
    fn eval_refuses_typealias() {
        let world = frozen_with(
            r#"
            (:wat::config::set-capacity-mode! :error)
            (:wat::config::set-dims! 1024)
        "#,
        );
        let ast =
            crate::parser::parse_one(r#"(:wat::core::typealias :evil::A :i64)"#).unwrap();
        let err = eval_in_frozen(&ast, &world, &Environment::new()).unwrap_err();
        assert!(matches!(err, RuntimeError::EvalForbidsMutationForm { .. }));
    }

    #[test]
    fn eval_refuses_load() {
        let world = frozen_with(
            r#"
            (:wat::config::set-capacity-mode! :error)
            (:wat::config::set-dims! 1024)
        "#,
        );
        let ast = crate::parser::parse_one(
            r#"(:wat::load-file! "evil.wat")"#,
        )
        .unwrap();
        let err = eval_in_frozen(&ast, &world, &Environment::new()).unwrap_err();
        assert!(matches!(err, RuntimeError::EvalForbidsMutationForm { .. }));
    }

    #[test]
    fn eval_refuses_digest_load() {
        let world = frozen_with(
            r#"
            (:wat::config::set-capacity-mode! :error)
            (:wat::config::set-dims! 1024)
        "#,
        );
        let ast = crate::parser::parse_one(
            r#"(:wat::digest-load! "x" :wat::verify::digest-sha256 :wat::verify::string "hex")"#,
        )
        .unwrap();
        let err = eval_in_frozen(&ast, &world, &Environment::new()).unwrap_err();
        assert!(matches!(err, RuntimeError::EvalForbidsMutationForm { .. }));
    }

    #[test]
    fn eval_refuses_signed_load() {
        let world = frozen_with(
            r#"
            (:wat::config::set-capacity-mode! :error)
            (:wat::config::set-dims! 1024)
        "#,
        );
        let ast = crate::parser::parse_one(
            r#"(:wat::signed-load! "x" :wat::verify::signed-ed25519 :wat::verify::string "sig" :wat::verify::string "pk")"#,
        )
        .unwrap();
        let err = eval_in_frozen(&ast, &world, &Environment::new()).unwrap_err();
        assert!(matches!(err, RuntimeError::EvalForbidsMutationForm { .. }));
    }

    #[test]
    fn eval_refuses_config_setter() {
        let world = frozen_with(
            r#"
            (:wat::config::set-capacity-mode! :error)
            (:wat::config::set-dims! 1024)
        "#,
        );
        let ast =
            crate::parser::parse_one(r#"(:wat::config::set-dims! 8192)"#).unwrap();
        let err = eval_in_frozen(&ast, &world, &Environment::new()).unwrap_err();
        assert!(matches!(err, RuntimeError::EvalForbidsMutationForm { .. }));
    }

    #[test]
    fn eval_refuses_mutation_form_at_any_depth() {
        // A mutation form nested inside otherwise-legal structure is
        // still refused. The walker descends into every child.
        let world = frozen_with(
            r#"
            (:wat::config::set-capacity-mode! :error)
            (:wat::config::set-dims! 1024)
        "#,
        );
        let ast = crate::parser::parse_one(
            r#"(:wat::core::let (((x :i64) 1))
                 (:wat::core::define (:evil (y :i64) -> :i64) y))"#,
        )
        .unwrap();
        let err = eval_in_frozen(&ast, &world, &Environment::new()).unwrap_err();
        assert!(matches!(err, RuntimeError::EvalForbidsMutationForm { .. }));
    }

    // ─── Digest-verified eval ───────────────────────────────────────────

    fn digest_hex_for(ast: &WatAST) -> String {
        let bytes = crate::hash::canonical_edn_wat(ast);
        use sha2::Digest as _;
        let mut hasher = sha2::Sha256::new();
        hasher.update(&bytes);
        crate::hash::hex_encode(&hasher.finalize())
    }

    #[test]
    fn eval_digest_verified_runs() {
        let world = frozen_with(
            r#"
            (:wat::config::set-capacity-mode! :error)
            (:wat::config::set-dims! 1024)
        "#,
        );
        let ast =
            crate::parser::parse_one(r#"(:wat::core::i64::+ 20 22)"#).unwrap();
        let hex = digest_hex_for(&ast);
        let result =
            eval_digest_in_frozen(&ast, &world, &Environment::new(), "sha256", &hex)
                .expect("eval ok");
        assert!(matches!(result, Value::i64(42)));
    }

    #[test]
    fn eval_digest_mismatch_refuses() {
        let world = frozen_with(
            r#"
            (:wat::config::set-capacity-mode! :error)
            (:wat::config::set-dims! 1024)
        "#,
        );
        let ast = crate::parser::parse_one(r#"(:wat::core::i64::+ 1 1)"#).unwrap();
        let wrong =
            "0000000000000000000000000000000000000000000000000000000000000000";
        let err =
            eval_digest_in_frozen(&ast, &world, &Environment::new(), "sha256", wrong)
                .unwrap_err();
        match err {
            RuntimeError::EvalVerificationFailed { err } => {
                assert!(matches!(err, crate::hash::HashError::Mismatch { .. }));
            }
            other => panic!("expected EvalVerificationFailed, got {:?}", other),
        }
    }

    #[test]
    fn eval_digest_unsupported_algo() {
        let world = frozen_with(
            r#"
            (:wat::config::set-capacity-mode! :error)
            (:wat::config::set-dims! 1024)
        "#,
        );
        let ast = crate::parser::parse_one("42").unwrap();
        let err =
            eval_digest_in_frozen(&ast, &world, &Environment::new(), "md5", "abc123")
                .unwrap_err();
        match err {
            RuntimeError::EvalVerificationFailed { err } => {
                assert!(matches!(err, crate::hash::HashError::UnsupportedAlgorithm { .. }));
            }
            other => panic!("expected EvalVerificationFailed, got {:?}", other),
        }
    }

    // ─── Signature-verified eval ────────────────────────────────────────

    fn sign_ast_ed25519(ast: &WatAST) -> (String, String) {
        use base64::engine::general_purpose::STANDARD as B64;
        use base64::Engine;
        use ed25519_dalek::{Signer, SigningKey};
        let sk = SigningKey::from_bytes(&[11u8; 32]);
        let hash = crate::hash::hash_canonical_ast(ast);
        let sig = sk.sign(&hash);
        (
            B64.encode(sig.to_bytes()),
            B64.encode(sk.verifying_key().as_bytes()),
        )
    }

    #[test]
    fn eval_signed_verified_runs() {
        let world = frozen_with(
            r#"
            (:wat::config::set-capacity-mode! :error)
            (:wat::config::set-dims! 1024)
        "#,
        );
        let ast =
            crate::parser::parse_one(r#"(:wat::core::i64::+ 40 2)"#).unwrap();
        let (sig, pk) = sign_ast_ed25519(&ast);
        let result = eval_signed_in_frozen(
            &ast,
            &world,
            &Environment::new(),
            "ed25519",
            &sig,
            &pk,
        )
        .expect("eval ok");
        assert!(matches!(result, Value::i64(42)));
    }

    #[test]
    fn eval_signed_tampered_ast_refuses() {
        let world = frozen_with(
            r#"
            (:wat::config::set-capacity-mode! :error)
            (:wat::config::set-dims! 1024)
        "#,
        );
        let original = crate::parser::parse_one(r#"(:wat::core::i64::+ 1 1)"#).unwrap();
        let tampered = crate::parser::parse_one(r#"(:wat::core::i64::+ 99 99)"#).unwrap();
        let (sig, pk) = sign_ast_ed25519(&original);
        let err = eval_signed_in_frozen(
            &tampered,
            &world,
            &Environment::new(),
            "ed25519",
            &sig,
            &pk,
        )
        .unwrap_err();
        match err {
            RuntimeError::EvalVerificationFailed { err } => {
                assert!(matches!(err, crate::hash::HashError::SignatureMismatch { .. }));
            }
            other => panic!("expected SignatureMismatch, got {:?}", other),
        }
    }

    #[test]
    fn eval_signed_unsupported_algo() {
        let world = frozen_with(
            r#"
            (:wat::config::set-capacity-mode! :error)
            (:wat::config::set-dims! 1024)
        "#,
        );
        let ast = crate::parser::parse_one("42").unwrap();
        let err = eval_signed_in_frozen(
            &ast,
            &world,
            &Environment::new(),
            "rsa",
            "dummy",
            "dummy",
        )
        .unwrap_err();
        match err {
            RuntimeError::EvalVerificationFailed { err } => {
                assert!(matches!(
                    err,
                    crate::hash::HashError::UnsupportedSignatureAlgorithm { .. }
                ));
            }
            other => panic!("expected UnsupportedSignatureAlgorithm, got {:?}", other),
        }
    }

    #[test]
    fn eval_digest_still_refuses_mutation_after_verify() {
        // Even a correctly-signed / correctly-digested AST that
        // contains a mutation form is refused — verification is BEFORE
        // the mutation-form walk, but both guards must pass.
        let world = frozen_with(
            r#"
            (:wat::config::set-capacity-mode! :error)
            (:wat::config::set-dims! 1024)
        "#,
        );
        let ast = crate::parser::parse_one(
            r#"(:wat::core::define (:evil (x :i64) -> :i64) x)"#,
        )
        .unwrap();
        let hex = digest_hex_for(&ast);
        let err =
            eval_digest_in_frozen(&ast, &world, &Environment::new(), "sha256", &hex)
                .unwrap_err();
        assert!(matches!(err, RuntimeError::EvalForbidsMutationForm { .. }));
    }
}
