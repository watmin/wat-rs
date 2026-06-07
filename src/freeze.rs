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
use crate::config::{collect_entry_file, collect_entry_file_with_inherit, Config, ConfigError};
use crate::load::{resolve_loads, LoadError, SourceLoader};
use crate::macros::{
    expand_all, register_defmacros, register_stdlib_defmacros, MacroError, MacroRegistry,
};
use crate::parser::{parse_all_with_file, ParseError};
use crate::stdlib::{stdlib_forms, StdlibError};
use crate::resolve::{resolve_references, ResolveError};
use crate::runtime::{
    apply_function, register_defines, register_stdlib_defines, EvalBreak, Environment,
    FunctionBody, RuntimeError, RuntimeErrorKind, SymbolTable, TrackedValue, Value,
};
use crate::value::EncodingCtx;
use crate::span::Span;
use crate::thread_io::ThreadId;
use crate::types::{register_stdlib_types, register_types, TypeEnv, TypeError, TypeExpr};
use std::collections::HashMap;
use std::fmt;
use std::sync::Arc;

// ─── Runtime bootstrap — substrate-owned process startup ──────────────────

/// Arguments to bootstrap a wat-vm runtime process.
///
/// Today: just the FrozenWorld. Future stones may add fields
/// (e.g., custom stdio injection, argv override) — the struct shape
/// keeps additions backwards-compatible.
pub struct BootstrapArgs<'a> {
    pub frozen: &'a FrozenWorld,
}

/// A running wat-vm process context: services running, ThreadIO installed
/// for the calling thread, SymbolTable carrying RuntimeServices.
///
/// Hold this for the duration of wat code execution — call
/// `apply_function` with `runtime.symbols()` while this is alive.
/// Drop when the wat-vm process is done; Drop runs cleanup in the
/// exact order the original orchestrator used:
///
/// 1. Deregister the calling thread from services (sends Deregister events).
/// 2. Uninstall the calling thread's ThreadIO cell.
/// 3. Drop `sym_with_services` (releases the Arc<RuntimeServices> the
///    carrier held).
/// 4. Drop `services` (the local Arc).
/// 5. Join the stdin service peer (Rust JoinHandle). A panicked stdin
///    loop — EOF fired via assertion-failed! — joins Err; the arm logs
///    and continues (we're in teardown).
/// 6. Join the stdout service peer (Rust JoinHandle).
/// 7. Join the stderr service peer (Rust JoinHandle).
///
/// Join errors in Drop are logged to stderr via `eprintln!` and do not
/// propagate (Drop cannot return Result). They are diagnostic noise on
/// the shutdown path — the process is already tearing down.
pub struct ProcessRuntime {
    sym_with_services: SymbolTable,
    /// Wrapped in Option so Drop can `take()` it to explicitly release
    /// the Arc reference INSIDE the drop body (before joins in steps 5-7).
    /// Always Some until Drop runs.
    services: Option<Arc<crate::thread_io::RuntimeServices>>,
    main_thread_id: ThreadId,
    /// Arc 214 Stone 8.2 — JoinHandle for the universe-resident stdin
    /// read service peer loop. Joined in Drop step 5.
    /// A PANICKED stdin join (EOF-cascade via assertion-failed!) logs Err
    /// — the existing log-and-continue arm is correct (we're in teardown).
    stdin_service_join: Option<std::thread::JoinHandle<()>>,
    /// Arc 214 Stone 8.1 — JoinHandle for the universe-resident stdout
    /// write service peer loop. Joined in Drop step 6.
    stdout_service_join: Option<std::thread::JoinHandle<()>>,
    /// Arc 214 Stone 8.1b — JoinHandle for the universe-resident stderr
    /// write service peer loop. Joined in Drop step 7.
    stderr_service_join: Option<std::thread::JoinHandle<()>>,
}

impl ProcessRuntime {
    /// The augmented SymbolTable carrying RuntimeServices.
    /// Use for `apply_function(fn, args, runtime.symbols(), ...)`.
    pub fn symbols(&self) -> &SymbolTable {
        &self.sym_with_services
    }
}

impl Drop for ProcessRuntime {
    fn drop(&mut self) {
        // Cleanup steps must run in this EXACT order:
        //
        //   1. Deregister the calling thread from services (Remove events).
        //   2. Uninstall the calling thread's ThreadIO cell.
        //   3. Drop sym_with_services (releases the Arc<RuntimeServices>
        //      the carrier held).
        //   4. Drop services local Arc.
        //   5. Join stdin service thread (wat Thread value).
        //   6. Join stdout service peer (Rust JoinHandle).
        //   7. Join stderr service peer (Rust JoinHandle).
        //
        // Steps 3 and 4 must happen BEFORE the joins (5-7) so the service
        // threads see their control-rx disconnect and exit — otherwise the
        // joins deadlock (service is still running, waiting for more control
        // messages from the now-dead caller).
        //
        // Rust's autogenerated field drop runs AFTER the drop() body, in
        // declaration order. To force steps 3 and 4 inside the body (before
        // the joins), we use std::mem::take on sym_with_services (SymbolTable
        // implements Default) and Arc::clone + explicit drop on services.

        // Step 1. services is Some until Drop; unwrap is safe here.
        if let Some(ref svc) = self.services {
            crate::thread_io::deregister_thread_from_services(self.main_thread_id, svc);
        }

        // Step 2.
        let _ = crate::thread_io::uninstall_thread_io();

        // Step 3: take sym_with_services, replacing with Default (empty
        // SymbolTable). The taken value drops at end of this block — it
        // releases the Arc<RuntimeServices> clone the carrier held.
        let sym = std::mem::take(&mut self.sym_with_services);
        drop(sym);

        // Step 4: take() the services Arc out of the Option, releasing this
        // caller-side reference BEFORE the joins (5-7). Services exit when
        // ALL Arc<RuntimeServices> clones drop; sym_with_services dropped
        // its clone in step 3; this drops the last local clone. After this
        // point the only surviving clones are those held by still-running
        // user-spawned threads (if any) — the service driver loops exit
        // when their control-rx disconnects (all ControlTx senders gone).
        if let Some(svc) = self.services.take() {
            drop(svc);
        }

        // Steps 5-7: Join service threads/peers. Services exit when their
        // control-rx disconnects; that happens when all Arc<RuntimeServices>
        // clones drop (sym done in step 3, local done in step 4). Join
        // errors are logged and continue — we're in teardown.

        // Step 5: Join the universe-resident stdin service peer (Rust thread).
        // A PANICKED stdin loop — EOF fired via assertion-failed! — joins Err;
        // the log-and-continue arm is correct (we're in teardown).
        if let Some(join) = self.stdin_service_join.take() {
            if let Err(e) = join.join() {
                eprintln!(
                    "[wat substrate] stdin service peer join error during ProcessRuntime::drop: {:?}",
                    e
                );
            }
        }
        // Step 6: Join the universe-resident stdout service peer (Rust thread).
        if let Some(join) = self.stdout_service_join.take() {
            if let Err(e) = join.join() {
                eprintln!(
                    "[wat substrate] stdout service peer join error during ProcessRuntime::drop: {:?}",
                    e
                );
            }
        }
        // Step 7: Join the universe-resident stderr service peer (Rust thread).
        if let Some(join) = self.stderr_service_join.take() {
            if let Err(e) = join.join() {
                eprintln!(
                    "[wat substrate] stderr service peer join error during ProcessRuntime::drop: {:?}",
                    e
                );
            }
        }
    }
}

/// Bootstrap a wat-vm runtime context. Performs steps 1–4 of the
/// original `invoke_user_main_orchestrated`:
///
/// 1. Source stdio from the per-thread ambient cell (test injection)
///    or synthesize real-fd wrappers (production).
/// 2. Spawn three services (StdInService / StdOutService / StdErrService).
/// 3. Build `Arc<RuntimeServices>` and augment the FrozenWorld's
///    SymbolTable with the carrier.
/// 4. Allocate a fresh ThreadId; register the calling thread with all
///    three services; install ThreadIO for this thread.
///
/// Returns a [`ProcessRuntime`] whose `Drop` runs cleanup in the exact
/// order required: deregister → uninstall ThreadIO → drop sym_with_services
/// → drop services → join stdin → join stdout → join stderr.
///
/// Caller holds the `ProcessRuntime` for the duration of wat code
/// execution. Drop it (or let it go out of scope) when done.
pub fn bootstrap_wat_vm_process(args: BootstrapArgs<'_>) -> Result<ProcessRuntime, RuntimeError> {
    let frozen = args.frozen;

    // Initialize the process-wide shutdown infrastructure. Idempotent
    // across multiple bootstraps; the worker thread + wake pipe are
    // per-process and survive subsequent bootstraps within the same OS
    // process. Slice B+C+D+E wire the cascade to actually do something.
    // Must be called BEFORE the trio service spawning so SHUTDOWN_RX is
    // available when services start (arc 170 Slice A).
    crate::runtime::init_shutdown_signal();

    // Step 1 — Source the IOReader / IOWriter handles.
    let stdio = match crate::thread_io::take_ambient_stdio() {
        Some(s) => s,
        None => crate::process_stdio::lend_ambient(),
    };

    // Step 2 — Spawn the three services.
    //
    // stdin: Arc 214 Stone 8.2 — universe-resident read peer (same shape as write pair).
    // stdout: Arc 214 Stone 8.1 — universe-resident write peer.
    // stderr: Arc 214 Stone 8.1b — universe-resident write peer (same shape).
    let pre_sym = frozen.symbols();

    // stdin: look up the pure handle fn; spawn the Rust read service peer.
    // reply_of extracts Rep field 1 (the line) as a String.
    let stdin_handle_fn = pre_sym
        .get(":wat::kernel::services::StdInService/handle")
        .ok_or_else(|| RuntimeError {
            span: Span::unknown(),
            kind: RuntimeErrorKind::UnknownFunction(
                ":wat::kernel::services::StdInService/handle".into(),
            ),
        })?
        .clone();
    let stdin_peer = crate::thread_io::spawn_service_peer(
        "stdin",
        stdin_handle_fn,
        Value::io__IOReader(stdio.stdin.clone()),
        pre_sym.clone(),
        |rep: &Value| match rep {
            Value::Struct(sv) if sv.fields.len() >= 2 => match &sv.fields[1] {
                Value::String(s) => Ok((**s).clone()),
                _ => Err("StdInService Rep field[1] is not a String".into()),
            },
            _ => Err("StdInService Rep is not a Struct with ≥2 fields".into()),
        },
    );

    // stdout: look up the pure handle fn; spawn the Rust write service peer.
    let stdout_handle_fn = pre_sym
        .get(":wat::kernel::services::StdOutService/handle")
        .ok_or_else(|| RuntimeError {
            span: Span::unknown(),
            kind: RuntimeErrorKind::UnknownFunction(
                ":wat::kernel::services::StdOutService/handle".into(),
            ),
        })?
        .clone();
    let stdout_peer = crate::thread_io::spawn_service_peer(
        "stdout",
        stdout_handle_fn,
        Value::io__IOWriter(stdio.stdout.clone()),
        pre_sym.clone(),
        |_: &Value| Ok(()),
    );
    // stderr: look up the pure handle fn; spawn the Rust write service peer.
    let stderr_handle_fn = pre_sym
        .get(":wat::kernel::services::StdErrService/handle")
        .ok_or_else(|| RuntimeError {
            span: Span::unknown(),
            kind: RuntimeErrorKind::UnknownFunction(
                ":wat::kernel::services::StdErrService/handle".into(),
            ),
        })?
        .clone();
    let stderr_peer = crate::thread_io::spawn_service_peer(
        "stderr",
        stderr_handle_fn,
        Value::io__IOWriter(stdio.stderr.clone()),
        pre_sym.clone(),
        |_: &Value| Ok(()),
    );

    // Step 3 — Build RuntimeServices carrier + augmented SymbolTable.
    let services = Arc::new(crate::thread_io::RuntimeServices {
        stdin_ctrl: stdin_peer.input_tx.clone(),
        stdout_ctrl: stdout_peer.input_tx.clone(),
        stderr_ctrl: stderr_peer.input_tx.clone(),
    });
    let mut sym_with_services = frozen.symbols().clone();
    sym_with_services.set_runtime_services(Arc::clone(&services));

    // Step 4 — Register calling thread + install ThreadIO.
    let main_thread_id = crate::thread_io::next_thread_id();
    let main_io =
        crate::thread_io::register_thread_with_services(main_thread_id, &services)?;
    crate::thread_io::install_thread_io(main_io);

    Ok(ProcessRuntime {
        sym_with_services,
        services: Some(services),
        main_thread_id,
        stdin_service_join: Some(stdin_peer.thread),
        stdout_service_join: Some(stdout_peer.thread),
        stderr_service_join: Some(stderr_peer.thread),
    })
}

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
    ) -> Result<Self, StartupError> {
        let ctx = Arc::new(EncodingCtx::from_config(&config));
        symbols.set_encoding_ctx(ctx);
        symbols.set_source_loader(loader);
        // Arc 030: runtime macroexpand / macroexpand-1 primitives need
        // access to the frozen macro registry.
        symbols.set_macro_registry(Arc::new(macros.clone()));
        // Arc 085: shims that reflect on declared types (auto-spawn
        // walks an enum decl) reach the registry through SymbolTable
        // alongside the other capability carriers.
        symbols.set_types(Arc::new(types.clone()));

        // Arc 077: the dim router is retired. Program-d lives in
        // `EncodingCtx.dim_count` set above; no router to install.

        // Arc 037 slice 6: install built-in default sigma functions.
        // User overrides via set-presence-sigma! / set-coincident-sigma!
        // replace these below.
        symbols.set_presence_sigma_fn(Arc::new(crate::sigma::DefaultPresenceSigma));
        symbols.set_coincident_sigma_fn(Arc::new(crate::sigma::DefaultCoincidentSigma));

        // Install user-supplied presence-sigma function if present.
        if let Some(sigma_ast) = config.presence_sigma_ast.clone() {
            let env = crate::runtime::Environment::new();
            let v = crate::runtime::eval(&sigma_ast, &env, &symbols).map_err(|e| {
                StartupError::SigmaFn(format!(
                    "set-presence-sigma! body failed to evaluate: {}",
                    e
                ))
            })?.value_owned();
            let func = match v {
                crate::runtime::Value::wat__core__fn(f) => f,
                other => {
                    return Err(StartupError::SigmaFn(format!(
                        "set-presence-sigma! expected a function value; got {}",
                        other.type_name()
                    )));
                }
            };
            check_sigma_fn_signature("set-presence-sigma!", &func)?;
            // Stone 255.1a — sigma fns come from user-provided fn values; always Wat.
            let path = match func.name.clone() {
                Some(name) => name,
                None => match &func.body {
                    FunctionBody::Wat(ast) => format!("<fn@{}>", ast.span()),
                    FunctionBody::Native => unreachable!("native builtin fn-applied — dispatched via the runtime match, not fn-apply"),
                },
            };
            symbols.set_presence_sigma_fn(Arc::new(crate::sigma::WatFnSigmaFn {
                path,
                func,
            }));
        }

        // Install user-supplied coincident-sigma function if present.
        if let Some(sigma_ast) = config.coincident_sigma_ast.clone() {
            let env = crate::runtime::Environment::new();
            let v = crate::runtime::eval(&sigma_ast, &env, &symbols).map_err(|e| {
                StartupError::SigmaFn(format!(
                    "set-coincident-sigma! body failed to evaluate: {}",
                    e
                ))
            })?.value_owned();
            let func = match v {
                crate::runtime::Value::wat__core__fn(f) => f,
                other => {
                    return Err(StartupError::SigmaFn(format!(
                        "set-coincident-sigma! expected a function value; got {}",
                        other.type_name()
                    )));
                }
            };
            check_sigma_fn_signature("set-coincident-sigma!", &func)?;
            // Stone 255.1a — sigma fns come from user-provided fn values; always Wat.
            let path = match func.name.clone() {
                Some(name) => name,
                None => match &func.body {
                    FunctionBody::Wat(ast) => format!("<fn@{}>", ast.span()),
                    FunctionBody::Native => unreachable!("native builtin fn-applied — dispatched via the runtime match, not fn-apply"),
                },
            };
            symbols.set_coincident_sigma_fn(Arc::new(crate::sigma::WatFnSigmaFn {
                path,
                func,
            }));
        }

        // Arc 157 slice 1a-ii — evaluate top-level `def` forms in the
        // program residue and populate `symbols.runtime_def_values`. Must
        // run AFTER all capability carriers are installed (encoding_ctx,
        // sigma fns, etc.) so that def expressions which call substrate
        // primitives see a fully-equipped SymbolTable. Parallels the
        // sigma-fn evaluation above — same pattern, broader scope.
        let env = crate::runtime::Environment::new();
        crate::runtime::register_runtime_defs(&program, &env, &mut symbols)
            .map_err(|e| match e {
                EvalBreak::Diagnostic(re) => StartupError::Runtime(Box::new(re)),
                // A Signal at the freeze boundary is an interpreter bug.
                // TryPropagate/OptionPropagate are eliminated BEFORE this
                // path: the checker rejects top-level `?`/option-propagation
                // with "used outside any function body" (check.rs:8406 /
                // :8520), and the check pass runs before register_runtime_defs
                // in the freeze pipeline. TailCall is trampolined inside
                // apply_function and cannot escape. A Signal here means the
                // checker or eval subgraph is mis-wired.
                EvalBreak::Signal(_) => unreachable!(
                    "interpreter bug: eval-loop control signal escaped to freeze layer"
                ),
            })?;

        Ok(FrozenWorld {
            config,
            types,
            macros,
            symbols,
            program,
        })
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
    Runtime(Box<RuntimeError>),
    /// A baked stdlib source failed to parse. Should never fire in
    /// production — the stdlib is authored in-repo and its parsing is
    /// validated by `cargo test` — but surfaces cleanly if someone
    /// ships a malformed stdlib file.
    Stdlib(StdlibError),
    /// `(:wat::config::set-presence-sigma! <expr>)` or
    /// `(:wat::config::set-coincident-sigma! <expr>)` committed an AST
    /// that did not evaluate to a function value at freeze, or whose
    /// signature did not match `:fn(:i64) -> :i64`.
    SigmaFn(String),
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
            StartupError::SigmaFn(msg) => write!(f, "sigma-fn: {}", msg),
        }
    }
}

/// Signature check for `set-presence-sigma!` / `set-coincident-sigma!`.
/// Both expect `:fn(:i64) -> :i64` — takes dim, returns σ count.
/// Fns that lack declared types skip the check (same policy as
/// dim-router).
fn check_sigma_fn_signature(
    setter: &str,
    func: &crate::runtime::Function,
) -> Result<(), StartupError> {
    if func.params.len() != 1 {
        return Err(StartupError::SigmaFn(format!(
            "{} function must take exactly 1 argument (got {})",
            setter,
            func.params.len()
        )));
    }
    if !func.param_types.is_empty() {
        let expected_param = crate::types::TypeExpr::Path(":wat::core::i64".into());
        if func.param_types[0] != expected_param {
            return Err(StartupError::SigmaFn(format!(
                "{} function param must be :i64; got {:?}",
                setter, func.param_types[0]
            )));
        }
        let expected_ret = crate::types::TypeExpr::Path(":wat::core::i64".into());
        if func.ret_type != expected_ret {
            return Err(StartupError::SigmaFn(format!(
                "{} function return type must be :i64; got {:?}",
                setter, func.ret_type
            )));
        }
    }
    Ok(())
}

impl StartupError {
    /// Arc 115 slice 1 — produce structured [`Diagnostic`] records
    /// for this error. The `Check` arm yields one record per
    /// `CheckError`; other arms yield ONE record with `kind` matching
    /// the variant name and a `message` field carrying the inner
    /// error's Display rendering.
    ///
    /// **Data first.** Renderers (`render_edn`, `render_json`,
    /// `Display` itself) consume this — no parsing of text forms.
    pub fn diagnostics(&self) -> Vec<crate::diagnostic::Diagnostic> {
        use crate::diagnostic::Diagnostic;
        match self {
            StartupError::Check(errors) => errors.diagnostics(),
            StartupError::Parse(e) => {
                vec![Diagnostic::new("Parse").field("message", format!("{}", e))]
            }
            StartupError::Config(e) => {
                vec![Diagnostic::new("Config").field("message", format!("{}", e))]
            }
            StartupError::Load(e) => {
                vec![Diagnostic::new("Load").field("message", format!("{}", e))]
            }
            StartupError::Macro(e) => {
                vec![Diagnostic::new("Macro").field("message", format!("{}", e))]
            }
            StartupError::Type(e) => {
                vec![Diagnostic::new("Type").field("message", format!("{}", e))]
            }
            StartupError::Resolve(e) => {
                vec![Diagnostic::new("Resolve").field("message", format!("{}", e))]
            }
            StartupError::Runtime(e) => {
                vec![Diagnostic::new("Runtime").field("message", format!("{}", e))]
            }
            StartupError::Stdlib(e) => {
                vec![Diagnostic::new("Stdlib").field("message", format!("{}", e))]
            }
            StartupError::SigmaFn(msg) => {
                vec![Diagnostic::new("SigmaFn").field("message", msg.as_str())]
            }
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
        StartupError::Runtime(Box::new(e))
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
// and `:wat::kernel::fork-program-ast` children — sees dep surface
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
    startup_from_forms_post_config(config, post_config, base_canonical, loader)
}

/// Sandbox-sibling of [`startup_from_forms`]: seeds the config pass from
/// the caller's `inherit` baseline so sandbox forms that omit
/// `(:wat::config::set-*!)` take the caller's committed values rather
/// than erroring on required-field-missing.
///
/// Called by `:wat::kernel::run-sandboxed-ast`,
/// `:wat::kernel::run-sandboxed-hermetic-ast`, and `:wat::kernel::fork-program-ast`
/// children — each passes the active runtime's [`Config`] as `inherit`.
/// Arc 031.
pub fn startup_from_forms_with_inherit(
    entry_forms: Vec<WatAST>,
    base_canonical: Option<&str>,
    loader: Arc<dyn SourceLoader>,
    inherit: &Config,
) -> Result<FrozenWorld, StartupError> {
    let (config, post_config) = collect_entry_file_with_inherit(entry_forms, inherit)?;
    startup_from_forms_post_config(config, post_config, base_canonical, loader)
}

/// Shared post-config pipeline (steps 3–9). Extracted so
/// [`startup_from_forms`] and [`startup_from_forms_with_inherit`] share
/// every stage after the config pass diverges.
fn startup_from_forms_post_config(
    config: Config,
    post_config: Vec<WatAST>,
    base_canonical: Option<&str>,
    loader: Arc<dyn SourceLoader>,
) -> Result<FrozenWorld, StartupError> {
    // 3. Recursive load resolution. The loader survives into the
    //    runtime as well — see step 9 — so `resolve_loads` borrows
    //    via `&*loader` (Arc deref) rather than owning.
    let loaded = resolve_loads(post_config, base_canonical, &*loader)?;

    // 3a. Baked stdlib. Registered ahead of user code so any
    //     `(:wat::holon::Subtract …)` / `(:wat::holon::Amplify …)` call
    //     in user source resolves during step 4's macro expansion
    //     without an explicit `load!`. Per FOUNDATION § "Where Each
    //     Lives" (line 2088), each `wat/**/*.wat` file ships one form
    //     whose keyword path matches the file path.
    let stdlib = stdlib_forms()?;

    // 4. Macro registration + expansion. Stdlib defmacros register
    //    first; user defmacros layer on top and can shadow (subject
    //    to the reserved-prefix gate) or reference stdlib forms.
    let mut macros = MacroRegistry::new();
    let stdlib_post_macros = register_stdlib_defmacros(stdlib, &mut macros)?;
    let post_macro_reg = register_defmacros(loaded, &mut macros)?;

    // ORDER LOAD-BEARING: macro_eval purity (src/macros/eval.rs) depends on
    // expand_all preceding register_defines. User/stdlib defns are not yet
    // registered at expand time — a reference to one cannot resolve, so only
    // blessed builtins + inline lambdas (body-validated) are reachable. If
    // register_defines is ever moved before this block, the fenced evaluator
    // alone no longer guarantees purity; a gate at apply_function becomes
    // necessary (see eval.rs header "Load-bearing invariant").
    //
    // Expand BOTH stdlib non-defmacro residue and user forms against
    // the combined macro registry. Stdlib functions are authored
    // against stdlib defmacros too — e.g., :wat::stream bodies
    // use :wat::holon::Subtract / list helpers / etc.
    let macro_sym = SymbolTable::default();
    let expanded_stdlib = expand_all(
        stdlib_post_macros,
        &mut macros,
        &Environment::default(),
        &macro_sym,
    )?;
    let expanded_user = expand_all(
        post_macro_reg,
        &mut macros,
        &Environment::default(),
        &macro_sym,
    )?;

    // 4b. Arc 163 slice 3g phase A — bare-legacy walker on raw
    //     post-expansion forms BEFORE register_types/register_defines
    //     consume the structural data (which would hide sig-position
    //     type-keywords like `:i64` in `(define (a :i64) -> :i64)`
    //     from check_program's walker). Walks user forms only;
    //     stdlib forms are substrate-authored and audited via
    //     in-repo discipline. The bare-legacy walker (arc 109 slice
    //     1c) is wired here so the diagnostic stream covers ALL
    //     user-written type-position keywords, not just expression-
    //     position ones. Slice 3g sweeps remaining bare-form sites
    //     based on these diagnostics; slice 3h retires the
    //     canonicalize=true upgrade arms once the sweep is complete.
    {
        let mut bare_errors: Vec<crate::check::CheckError> = Vec::new();
        for form in &expanded_user {
            crate::check::validate_bare_legacy_primitives(form, &mut bare_errors);
        }
        // Arc 170 slice 2 — substrate-as-teacher walker for the
        // spawn-verb consolidation + `:user::main` 4-arg + ExitCode
        // contract change. User-source only (stdlib paths continue
        // to call legacy verbs through the sweep window; slice 4
        // retires both walker bodies + legacy dispatch arms).
        for form in &expanded_user {
            crate::check::validate_arc170_legacy_callsites(form, &mut bare_errors);
        }
        if !bare_errors.is_empty() {
            return Err(StartupError::Check(crate::check::CheckErrors(bare_errors)));
        }
    }

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
    // Stone 237.8b — capture stdlib residue so defclause forms reach register_runtime_defs.
    // The stdlib residue is NOT included in the user `residue` (it hasn't been through
    // resolver step 7). Instead, we:
    //   (a) pre-register defclause stubs so the checker sees them as callable names
    //   (b) extract just the defclause forms for processing by register_runtime_defs
    let stdlib_residue = register_stdlib_defines(stdlib_post_types, &mut symbols)?;
    // (a) Pre-register stubs into sym.functions so checker sees them.
    for form in &stdlib_residue {
        crate::runtime::preregister_stdlib_defclause_stub(form, &mut symbols);
    }
    // (b) Extract only defclause forms from stdlib residue (other stdlib forms
    // already processed; defclauses need runtime registration via runtime_defs).
    let stdlib_defclause_forms: Vec<crate::ast::WatAST> = stdlib_residue.into_iter()
        .filter(|form| {
            if let crate::ast::WatAST::List(items, _) = form {
                matches!(items.first(), Some(crate::ast::WatAST::Keyword(k, _)) if k.as_str() == ":wat::core::defclause")
            } else {
                false
            }
        })
        .collect();
    let residue = register_defines(post_types, &mut symbols)?;

    // 6a. Struct auto-methods. For every `(:wat::core::defstruct ...)`
    //     declaration (built-in + user), synthesize its `/new`
    //     constructor and one `/<field>` accessor per field, all as
    //     ordinary `Function` entries in the symbol table. Runs
    //     after user defines so collisions with user-authored names
    //     surface as `DuplicateDefine`.
    //     (Stone 241.8: renamed from :wat::core::struct to :wat::core::defstruct)
    crate::runtime::register_struct_methods(&types, &mut symbols)?;
    // 6.5. Enum variant constructors — arc 048. Walks enum decls
    //      and synthesizes per-variant constructors (units into
    //      `unit_variants` map; tagged variants as Function entries
    //      whose bodies invoke `:wat::core::enum-new`).
    crate::runtime::register_enum_methods(&types, &mut symbols)?;
    // 6.7. Newtype auto-methods — arc 049. For each `(:wat::core::newtype
    //      :Type :Inner)` decl, synthesize `:Type/new` constructor and
    //      `:Type/0` accessor. Newtype values are represented as
    //      `Value::Struct` of arity 1, reusing the existing struct-new
    //      and struct-field primitives.
    crate::runtime::register_newtype_methods(&types, &mut symbols)?;
    // 6.9. Arc 237 Stone 237.6 — auto-mint `:ns::is-<Name>?` membership predicates.
    //      One pass over the TypeEnv; for every non-Alias TypeDef (Struct / Enum /
    //      Newtype / Union) synthesize a Function whose body is
    //      `(:wat::core::conforms? v :<FQDN>)` — the one mechanism, not a
    //      second computation. Positioned after register_newtype_methods so the
    //      TypeEnv is fully populated before predicate synthesis begins.
    crate::runtime::register_type_predicates(&types, &mut symbols)?;

    // 6.8. Arc 198 slice 2 Stone 1 — drain the `inventory` registry of
    //      Rust-side `RestrictionEntry` declarations into
    //      Stone 241.14 — migrated from `defined_value_restrictions` to
    //      `binding_metadata`. The Rust-side RestrictionEntry inventory
    //      channel (arc 198 slice 2) populates `binding_metadata` here;
    //      the wat-side `def`/`defn` metadata-map path populates it during
    //      `register_defines`. Both feeds land in one `binding_metadata`
    //      map so the walker (`walk_for_restricted_call`) sees a unified
    //      `:restricted-to` whitelist regardless of origin.
    //
    //      Positioned AFTER all `register_*` calls (so the map isn't
    //      stomped) and BEFORE step 7.5 (which propagates other config to
    //      `symbols` for check_program). Subsequent stones that annotate
    //      substrate fns with `#[restricted_to(...)]` plug into this
    //      channel without changing the iteration shape.
    // Stone 241.14 (migrated from arc 198 def-restricted path) — populate
    // `binding_metadata` with `:restricted-to` entries from the Rust-side
    // `RestrictionEntry` inventory channel. Arc 170 Stone B's restrictions
    // on `Thread/join-result` + `Process/join-result` land here unchanged;
    // only the populate-target changes (binding_metadata instead of the
    // deleted defined_value_restrictions).
    //
    // The `:restricted-to` value is a WatAST::List whose first item is the
    // `:wat::core::Vector` head (matching the brace-form parser's encoding
    // of `[p1 p2 ...]`) and whose remaining items are prefix keywords. The
    // walker `walk_for_restricted_call` calls `extract_prefix_list_from_metadata`
    // to unpack this structure at check time.
    // rune:sequi(ambient-context) — inventory::iter::<RestrictionEntry> consumes
    // a compile-time static linker table populated by inventory::submit! entries
    // across the workspace; the registry is link-time fixed binary state, not
    // runtime domain state; threading it as a runtime parameter would impose
    // the registry's compile-time nature into every startup-pipeline call site.
    for entry in inventory::iter::<crate::restriction_entry::RestrictionEntry> {
        let name = entry.wat_name.to_string();
        // Build WatAST::List([Keyword(":wat::core::Vector"), Keyword(p1), ...], Span)
        // matching the brace-form encoding that `extract_prefix_list_from_metadata` expects.
        let mut prefix_items = vec![WatAST::Keyword(":wat::core::Vector".into(), Span::unknown())];
        for p in entry.prefixes {
            prefix_items.push(WatAST::Keyword(p.to_string(), Span::unknown()));
        }
        let restricted_to_ast = WatAST::List(prefix_items, Span::unknown());
        let mut meta: HashMap<String, WatAST> = HashMap::new();
        meta.insert(":restricted-to".to_string(), restricted_to_ast);
        symbols.binding_metadata.entry(name).or_insert_with(HashMap::new).extend(meta);
    }

    // 7. Name resolution.
    resolve_references(&residue, &symbols, &macros)?;

    // 7.5. Arc 157 slice 1a-ii — propagate redef config flags to the
    // SymbolTable carrier BEFORE check_program so that CheckEnv::from_symbols
    // sees the correct redef_allowed flag (check happens at step 8;
    // FrozenWorld::freeze at step 9 would be too late).
    symbols.redef_allowed = config.redef_allowed;
    symbols.eval_redef_allowed = config.eval_redef_allowed;

    // 7.6 Stone 237.8b — register stdlib defclauses into runtime_def_values.
    // Uses a privileged parser that bypasses the reserved-prefix check
    // (`:wat::core::+` etc. live under :wat::core::* which is reserved from
    // user code but legal for the stdlib). These forms have already been
    // through macro-expansion at step 4; they're stdlib-privileged.
    crate::runtime::register_stdlib_defclauses(&stdlib_defclause_forms, &mut symbols)
        .map_err(|e| StartupError::Runtime(Box::new(e)))?;

    // 8. Type check.
    check_program(&residue, &symbols, &types)?;

    // 9. Freeze. The loader moves into the frozen world's
    //    SymbolTable so runtime primitives (`:wat::eval-file!` and
    //    the file-path variants of the verified eval/load forms,
    //    `:wat::verify::file-path` payloads) can route through the
    //    same capability that handled startup loads.
    FrozenWorld::freeze(
        config,
        types,
        macros,
        symbols,
        residue,
        loader,
    )
}

// ─── :user::main invocation ─────────────────────────────────────────────

/// Canonical path for the user's entry-point slot. Per FOUNDATION.md
/// (line 1072): `:user::main` is kernel-REQUIRED (user provides;
/// kernel invokes). Zero or more-than-one declarations halt.
pub const USER_MAIN_PATH: &str = ":user::main";

/// Look up `:user::main` in the frozen world and apply it.
///
/// Per arc 170 REALIZATIONS pass 7 (ambient runtime) + pass 10
/// (`:wat::core::nil` IS the exit code), `:user::main` is now
/// `[] -> :wat::core::nil`. argv moves to the ambient
/// `:wat::runtime::argv` (set via [`crate::runtime::set_argv`]
/// before this function runs); stdio access moves to the three
/// substrate services (slice 1f); the entry point takes no
/// arguments at all. The `args: Vec<Value>` parameter is retained
/// to keep call-site changes minimal during the transition; callers
/// pass `Vec::new()`. Non-empty `args` triggers `ArityMismatch` via
/// [`apply_function`] — the substrate honors the new contract.
///
/// **Arc 170 slice 1f-γ — runtime orchestrator.** Before running
/// `:user::main`, the orchestrator spawns the three stdio services
/// (StdInService / StdOutService / StdErrService), populates the
/// SymbolTable's runtime-services carrier, and registers thread-0
/// with the services so the body's `(:wat::kernel::println ...)` /
/// `(eprintln ...)` / `(readln)` calls land on the per-thread
/// ThreadIO. On return (Ok or Err), the orchestrator deregisters
/// thread-0, uninstalls the ThreadIO, drops the carrier (cascading
/// shutdown to the services via scope-drop), and joins each service
/// thread.
///
/// IO handles for the three services come from the per-thread
/// ambient stdio cell ([`crate::thread_io::AmbientStdio`]); tests
/// install pipe-backed handles via
/// [`crate::thread_io::install_ambient_stdio`] before invoking.
/// Production paths (wat-cli, fork.rs:659/1044) leave the ambient
/// unset and fall through to real fd 0/1/2 via PipeReader /
/// PipeWriter.
pub fn invoke_user_main(
    frozen: &FrozenWorld,
    args: Vec<Value>,
) -> Result<Value, RuntimeError> {
    invoke_user_main_orchestrated(frozen, args)
}

/// The orchestrator body. Delegates to [`bootstrap_wat_vm_process`] for
/// steps 1–4 (service spawn + ThreadIO install); then runs `:user::main`;
/// cleanup runs automatically when `runtime` drops at end of scope.
fn invoke_user_main_orchestrated(
    frozen: &FrozenWorld,
    args: Vec<Value>,
) -> Result<Value, RuntimeError> {
    // Steps 1–4: bootstrap services + ThreadIO (substrate-owned).
    let runtime = bootstrap_wat_vm_process(BootstrapArgs { frozen })?;

    // Step 5: Run `:user::main`. Any error (or the Ok value) is
    // captured in `result`; `runtime` drops after this block,
    // running cleanup regardless of success or failure.
    let main_lookup = runtime.symbols().get(USER_MAIN_PATH).cloned();
    let result = match main_lookup {
        Some(main_func) => apply_function(
            main_func,
            args,
            runtime.symbols(),
            crate::rust_caller_span!(),
        ),
        None => Err(RuntimeError { span: Span::unknown(), kind: RuntimeErrorKind::UserMainMissing }),
    };

    // Steps 6–8: cleanup runs in ProcessRuntime::drop when `runtime`
    // goes out of scope here. Drop order: deregister → uninstall
    // ThreadIO → drop sym_with_services → drop services → join
    // stdin → join stdout → join stderr.
    drop(runtime);

    result
}

// Arc 214 Stone 8.2 — spawn_service and join_service PURGED. stdin is the
// last tenant that used the old (wat Thread value, ControlTx) pair. All three
// services now boot via spawn_service_peer (Rust-resident); joins are plain
// JoinHandle::join calls in ProcessRuntime::drop.

// ─── :user::main signature enforcement ──────────────────────────────────
//
// Moved here from `bin/wat.rs` in arc 007 slice 2a so
// `:wat::kernel::run-sandboxed` can reuse the same validator. The CLI
// and the sandbox primitive enforce the same contract.

/// The exact signature `:user::main` must declare. Startup halts if
/// the program's `:user::main` doesn't match.
///
/// Arc 170 slice 1e (REALIZATIONS pass 7 + pass 10): the canonical
/// shape is `[] -> :wat::core::nil`. argv moves to the ambient
/// `:wat::runtime::argv` (set by wat-cli via
/// [`crate::runtime::set_argv`] before invocation); stdio access
/// moves to the three substrate services (slice 1f's
/// `StdInService` / `StdOutService` / `StdErrService`); the entry
/// point takes no arguments and the success marker is `nil`. The
/// substrate maps clean nil-return to `libc::exit(0)`; panic-cascade
/// maps to `libc::exit(N)` via slice 1i's `StdErrService` epilogue.
/// User code never participates in exit-code arithmetic.
///
/// `:wat::core::nil` canonicalizes to `TypeExpr::Tuple(vec![])` at
/// type-check time (per `src/types.rs:1740`); the validator
/// compares against that internal form so wat source written as
/// `-> :wat::core::nil` flows through unification cleanly.
pub fn expected_user_main_signature() -> (Vec<TypeExpr>, TypeExpr) {
    let params = vec![]; // empty — argv is ambient (REALIZATIONS pass 7)
    let ret = TypeExpr::Tuple(vec![]); // :wat::core::nil canonical (REALIZATIONS pass 10)
    (params, ret)
}

/// Check that a frozen world's `:user::main` declares the canonical
/// `[] -> :wat::core::nil` shape. Returns `Err(message)` with a
/// reader-friendly diagnostic naming the offending shape.
///
/// Per arc 170 REALIZATIONS pass 7 + pass 10: argv is ambient
/// (`:wat::runtime::argv`); stdio access is via the three substrate
/// services (slice 1f); `nil` IS the success exit code. The entry
/// point takes no arguments and returns `:wat::core::nil`.
pub fn validate_user_main_signature(frozen: &FrozenWorld) -> Result<(), String> {
    let func = frozen.symbols().get(":user::main").ok_or_else(|| {
        ":user::main not defined — a wat program needs an entry point".to_string()
    })?;
    let (expected_params, expected_ret) = expected_user_main_signature();
    if func.param_types.len() != expected_params.len() {
        return Err(format!(
            ":user::main must take exactly {} parameters; got {}. \
             Arc 170 slice 1e (REALIZATIONS pass 7) — `:user::main` \
             takes no arguments. argv is ambient via \
             `(:wat::runtime::argv)`; stdio is mediated by the three \
             substrate services (slice 1f's StdInService / StdOutService / \
             StdErrService). The canonical signature is \
             `[] -> :wat::core::nil`.",
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
            return Err(format!(
                ":user::main parameter #{} expected {}, got {}",
                i + 1,
                format_type_expr(want),
                format_type_expr(got)
            ));
        }
    }
    if func.ret_type != expected_ret {
        return Err(format!(
            ":user::main return type expected :wat::core::nil; got {}. \
             Arc 170 slice 1e (REALIZATIONS pass 10) — `nil` IS the \
             success exit code. Clean nil-return maps to libc::exit(0); \
             panic-cascade maps to libc::exit(N) via the StdErrService \
             epilogue (slice 1i). User code never participates in \
             exit-code arithmetic. The canonical signature is \
             `[] -> :wat::core::nil`.",
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
            // Arc 163 follow-up — emit canonical FQDN syntax. Pre-fix
            // this rendered as `:fn(args)->ret` (legacy pre-arc-155).
            // The walker `BareLegacyLowercaseFn` rejects bare `:fn(`
            // input post-walker-rearm, so the renderer must emit the
            // canonical `:wat::core::Fn(...)` to keep round-trip
            // consistency between rendered diagnostics and parser.
            let in_parts: Vec<_> = args.iter().map(format_type_expr_inner).collect();
            format!(
                ":wat::core::Fn({})->{}",
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
) -> Result<TrackedValue, RuntimeError> {
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
) -> Result<TrackedValue, RuntimeError> {
    // Compute the canonical-EDN bytes and verify against expected.
    let bytes = crate::hash::canonical_edn_wat(ast);
    crate::hash::verify_source_hash(&bytes, algo, expected_hex).map_err(|err| {
        RuntimeError { span: ast.span().clone(), kind: RuntimeErrorKind::EvalVerificationFailed { err } }
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
) -> Result<TrackedValue, RuntimeError> {
    crate::hash::verify_ast_signature(ast, algo, sig_b64, pubkey_b64).map_err(
        |err| RuntimeError { span: ast.span().clone(), kind: RuntimeErrorKind::EvalVerificationFailed { err } },
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
    // Walker-specific List-head logic — fire EvalForbidsMutationForm on
    // mutation-form Keyword heads. Mutation form heads always appear
    // in List position; this guard preserves the pre-arc-212 check.
    if let WatAST::List(items, list_span) = ast {
        if let Some(WatAST::Keyword(head, _)) = items.first() {
            if is_mutation_form(head) {
                return Err(RuntimeError { span: list_span.clone(), kind: RuntimeErrorKind::EvalForbidsMutationForm {
                    head: head.clone()
                } });
            }
        }
    }
    // Arc 212 — generic recursion via children() covers List, Vector,
    // and StructPattern uniformly. Pre-arc-212 this walker silently
    // accepted mutation forms buried inside Vector (let-binding-vector
    // RHSes) and StructPattern bracketed shapes — they slipped past
    // freeze-time refusal. children() returns &[] for leaf nodes (no-op).
    for child in ast.children() {
        refuse_mutation_forms(child)?;
    }
    Ok(())
}

fn is_mutation_form(head: &str) -> bool {
    matches!(
        head,
        // Arc 157 — `:wat::core::def` modifies the module-level value
        // env (it's a top-level declaration, not a pure expression).
        // Refused inside `eval_in_frozen` / dynamic-eval paths so that
        // runtime-received code cannot inject module bindings. Legal
        // only at startup (the startup pipeline's `check_program` +
        // `run_program` path; not the frozen `eval_in_frozen` path).
        ":wat::core::def"
            // Stone 241.16 — `:wat::core::define` arm DELETED. HARD CUT is total;
            // define is no longer a recognized mutation form at eval time.
            | ":wat::core::defmacro"
            // Stone 241.8 — defstruct replaces struct (HARD CUT).
            | ":wat::core::defstruct"
            // Stone 241.9 — defenum replaces enum (HARD CUT).
            | ":wat::core::defenum"
            | ":wat::core::newtype"
            | ":wat::core::typealias"
            | ":wat::load-file!"
            | ":wat::digest-load!"
            | ":wat::signed-load!"
    ) || head.starts_with(":wat::config::set-")
}

/// Returns `true` for forms that **declare a name** into the module's type
/// or value registry. This is the narrower subset of [`is_mutation_form`]
/// that covers ONLY the 8 declaration forms — excluding loads
/// (`load-file!` / `digest-load!` / `signed-load!`) and config setters
/// (`config::set-*`), which mutate state but do not introduce new bindings.
///
/// `defn` is intentionally absent: it is a macro that expands to
/// `(:wat::core::def ...)` BEFORE `extract_closure` runs. By the time the
/// prelude-lift or position-validator consults this predicate, `defn` has
/// already been rewritten to its `def` form; `def` is covered here.
///
/// Callers:
/// - `closure_extract::split_body_prelude` — lifts these forms from a fn
///   body's `do`-prefix into the closure's prologue so the child's
///   `startup_from_forms` registers them before the body runs.
/// - `check::validate_def_position_with_wrapper` (Gap I-B, future slice) —
///   extends compile-time position discipline to all 8 declaration forms.
pub fn is_declaration_form(head: &str) -> bool {
    matches!(
        head,
        ":wat::core::def"
            // Stone 241.16 — `:wat::core::define` arm DELETED from is_declaration_form.
            // HARD CUT is total; define is no longer a declaration form.
            | ":wat::core::defmacro"
            // Stone 241.8 — defstruct replaces struct (HARD CUT).
            | ":wat::core::defstruct"
            // Stone 241.9 — defenum replaces enum (HARD CUT).
            | ":wat::core::defenum"
            | ":wat::core::newtype"
            | ":wat::core::typealias"
            // Stone 241.12 — defalias is a declaration form (alias binding).
            | ":wat::core::defalias"
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::load::InMemoryLoader;

    /// Helper: start from an entry string with no loaded files.
    fn startup(entry: &str) -> Result<FrozenWorld, StartupError> {
        startup_from_source(entry, Some(concat!(file!(), ":", line!())), Arc::new(InMemoryLoader::new()))
    }

    // ─── Happy path ─────────────────────────────────────────────────────

    #[test]
    fn minimal_program_freezes() {
        // Arc 225 Stone 225.1: narrow Atom only accepts HolonAST; use
        // to-holon for string literals (the polymorphic UP verb).
        let src = r#"
            (:wat::config::set-capacity-mode! :error)
            (:wat::holon::to-holon "hello")
        "#;
        let world = startup(src).expect("startup");
        assert_eq!(world.config().capacity_mode, crate::config::CapacityMode::Error);
        assert_eq!(world.program().len(), 1);
    }

    #[test]
    fn global_seed_defaults() {
        let src = r#"
            (:wat::config::set-capacity-mode! :error)
        "#;
        let world = startup(src).expect("startup");
        assert_eq!(world.config().global_seed, 42);
    }

    #[test]
    fn user_define_registers() {
        let src = r#"
            (:wat::config::set-capacity-mode! :error)
            (:wat::core::defn :my::app::add [x <- :wat::core::i64 y <- :wat::core::i64] -> :wat::core::i64 (:wat::core::i64::+ x y))
        "#;
        let world = startup(src).expect("startup");
        assert!(world.symbols().get(":my::app::add").is_some());
    }

    #[test]
    fn user_type_registers() {
        // Stone 241.8 — migrated from :wat::core::struct to defstruct.
        let src = r#"
            (:wat::config::set-capacity-mode! :error)
            (:wat::core::defstruct :my::Candle [open <- :wat::core::f64 close <- :wat::core::f64])
        "#;
        let world = startup(src).expect("startup");
        assert!(world.types().contains(":my::Candle"));
    }

    #[test]
    fn user_macro_registers() {
        let src = r#"
            (:wat::config::set-capacity-mode! :error)
            (:wat::core::defmacro :my::vocab::Double
              [x <- :AST<wat::holon::HolonAST>]
              -> :AST<wat::holon::HolonAST>
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
    fn config_error_bubbles_up() {
        // Arc 037 retired required-ness for dims/capacity-mode; any
        // remaining ConfigError — here, wrong type for dims — still
        // propagates through startup as StartupError::Config.
        let err = startup(r#"(:wat::config::set-dims! "oops")"#).unwrap_err();
        assert!(matches!(err, StartupError::Config(_)));
    }

    #[test]
    fn type_error_bubbles_up() {
        // Duplicate defstruct declaration.
        // Stone 241.8 — migrated from :wat::core::struct to defstruct.
        let src = r#"
            (:wat::config::set-capacity-mode! :error)
            (:wat::core::defstruct :my::Candle [x <- :wat::core::f64])
            (:wat::core::defstruct :my::Candle [y <- :wat::core::i64])
        "#;
        let err = startup(src).unwrap_err();
        assert!(matches!(err, StartupError::Type(_)));
    }

    #[test]
    fn check_error_bubbles_up() {
        // Passing :i64 to a define that declared :bool — type mismatch.
        let src = r#"
            (:wat::config::set-capacity-mode! :error)
            (:wat::core::i64::+ "hello" 1)
        "#;
        let err = startup(src).unwrap_err();
        assert!(matches!(err, StartupError::Check(_)));
    }

    #[test]
    fn resolve_error_bubbles_up() {
        let src = r#"
            (:wat::config::set-capacity-mode! :error)
            (:my::app::never-defined 42)
        "#;
        let err = startup(src).unwrap_err();
        assert!(matches!(err, StartupError::Resolve(_)));
    }

    #[test]
    fn any_in_type_position_bubbles_up_as_type_error() {
        // :Any is banned at parse_type_expr time.
        // Stone 241.11 — defn macro-expands to def before register_defines,
        // so the :Any check happens either at register_defines (if
        // try_parse_fn_shape_def returns None, deferring to check_program)
        // or at check_program itself. Either way, startup fails.
        let src = r#"
            (:wat::config::set-capacity-mode! :error)
            (:wat::core::defn :my::bad [x <- :Any] -> :wat::core::i64 42)
        "#;
        let err = startup(src).unwrap_err();
        // Startup must fail — accept Runtime or Check errors.
        assert!(
            matches!(err, StartupError::Runtime(_)) || matches!(err, StartupError::Check(_)),
            "expected startup error, got {:?}", err
        );
    }

    // ─── Frozen world is immutable ──────────────────────────────────────

    #[test]
    fn frozen_world_exposes_read_only_accessors() {
        // Sanity: the accessors return shared references — the borrow
        // checker would refuse to compile if they returned mutable
        // references. This test just exercises every accessor.
        let src = r#"
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
            r#"(:wat::core::defn :lib::square [x <- :wat::core::i64] -> :wat::core::i64 (:wat::core::i64::* x x))"#,
        );
        let entry = r#"
            (:wat::config::set-capacity-mode! :error)
            (:wat::load-file! "lib.wat")
        "#;
        let world = startup_from_source(entry, Some(concat!(file!(), ":", line!())), Arc::new(loader)).expect("startup");
        assert!(world.symbols().get(":lib::square").is_some());
    }

    // ─── :user::main invocation ─────────────────────────────────────────

    #[test]
    fn invoke_main_happy_path() {
        // Arc 170 slice 1e — canonical `[] -> :wat::core::nil` shape
        // (REALIZATIONS pass 7 + pass 10). `:user::main` returns nil;
        // substrate maps to libc::exit(0).
        let src = r#"
            (:wat::config::set-capacity-mode! :error)
            (:wat::core::defn :user::main [] -> :wat::core::nil nil)
        "#;
        let world = startup(src).expect("startup");
        let result = invoke_user_main(&world, Vec::new()).expect("main runs");
        assert!(matches!(result, Value::Unit));
    }

    #[test]
    fn invoke_main_calls_user_define() {
        // :user::main delegates to a user-defined helper that
        // side-effects (or in this minimal case, just produces nil).
        let src = r#"
            (:wat::config::set-capacity-mode! :error)
            (:wat::core::defn :my::app::do-work [] -> :wat::core::nil nil)
            (:wat::core::defn :user::main [] -> :wat::core::nil (:my::app::do-work))
        "#;
        let world = startup(src).expect("startup");
        let result = invoke_user_main(&world, Vec::new()).expect("main runs");
        assert!(matches!(result, Value::Unit));
    }

    #[test]
    fn invoke_main_missing_is_error() {
        let src = r#"
            (:wat::config::set-capacity-mode! :error)
        "#;
        let world = startup(src).expect("startup");
        let err = invoke_user_main(&world, Vec::new()).unwrap_err();
        assert!(matches!(err, RuntimeError { kind: RuntimeErrorKind::UserMainMissing, .. }));
    }

    // LocalCache stdlib-composition test retired in arc 013 slice 4b
    // — the wat-lru sibling crate owns that surface now. End-to-end
    // composition coverage lives in crates/wat-lru/tests/.
    //
    // `invoke_main_wrong_arity_is_error` and
    // `invoke_main_passes_channel_value_through` retired in arc 170
    // slice 1e — both relied on declaring `:user::main` with non-
    // canonical parameters (REALIZATIONS pass 7 made `:user::main`
    // arg-free; ambient runtime carries argv). The arity-mismatch
    // path is exercised by `apply_function` callers other than
    // `invoke_user_main`; the apply mechanic itself is tested at
    // the `Function` layer.

    // ─── Constrained eval ───────────────────────────────────────────────

    fn frozen_with(src: &str) -> FrozenWorld {
        startup(src).expect("startup")
    }

    #[test]
    fn eval_can_invoke_registered_function() {
        let world = frozen_with(
            r#"
            (:wat::config::set-capacity-mode! :error)
            (:wat::core::defn :my::app::triple [x <- :wat::core::i64] -> :wat::core::i64 (:wat::core::i64::* x 3))
        "#,
        );
        let ast = crate::parse_one!("(:my::app::triple 7)").unwrap();
        let env = Environment::new();
        let result = eval_in_frozen(&ast, &world, &env).expect("eval ok");
        assert!(matches!(result.value(), Value::i64(21)));
    }

    #[test]
    fn eval_can_compose_holon_dynamically() {
        let world = frozen_with(
            r#"
            (:wat::config::set-capacity-mode! :error)
        "#,
        );
        // Arc 225 Stone 225.1: narrow Atom only accepts HolonAST; use
        // to-holon for string literals (the polymorphic UP verb).
        let ast = crate::parse_one!(
            r#"(:wat::holon::Bind (:wat::holon::to-holon "role") (:wat::holon::to-holon "filler"))"#,
        )
        .unwrap();
        let env = Environment::new();
        let result = eval_in_frozen(&ast, &world, &env).expect("eval ok");
        assert!(matches!(result.value(), Value::holon__HolonAST(_)));
    }

    #[test]
    fn eval_refuses_define() {
        // Stone 241.16 — migrated from `:wat::core::define` (HARD CUT total; no longer
        // in is_mutation_head) to `:wat::core::defstruct` (still a recognized mutation form).
        // The test bypasses startup (uses parse_one! + eval_in_frozen directly) to verify
        // the eval-time guard independently of the startup-time HARD CUT.
        // Mechanism under test: eval_in_frozen refuses ANY mutation-headed form.
        let world = frozen_with(
            r#"
            (:wat::config::set-capacity-mode! :error)
        "#,
        );
        let ast = crate::parse_one!(
            r#"(:wat::core::defstruct :evil::Backdoor [x <- :wat::core::i64])"#,
        )
        .unwrap();
        let env = Environment::new();
        let err = eval_in_frozen(&ast, &world, &env).unwrap_err();
        match err {
            RuntimeError { kind: RuntimeErrorKind::EvalForbidsMutationForm { head, .. }, .. } => {
                assert_eq!(head, ":wat::core::defstruct");
            }
            other => panic!("expected EvalForbidsMutationForm, got {:?}", other),
        }
    }

    #[test]
    fn eval_refuses_defmacro() {
        let world = frozen_with(
            r#"
            (:wat::config::set-capacity-mode! :error)
        "#,
        );
        let ast = crate::parse_one!(
            r#"(:wat::core::defmacro :evil::M [x <- :AST<wat::holon::HolonAST>] -> :AST<wat::holon::HolonAST> x)"#,
        )
        .unwrap();
        let err = eval_in_frozen(&ast, &world, &Environment::new()).unwrap_err();
        assert!(matches!(err, RuntimeError { kind: RuntimeErrorKind::EvalForbidsMutationForm { .. }, .. }));
    }

    #[test]
    fn eval_refuses_struct() {
        // Stone 241.8 — migrated to defstruct.
        let world = frozen_with(
            r#"
            (:wat::config::set-capacity-mode! :error)
        "#,
        );
        let ast = crate::parse_one!(
            r#"(:wat::core::defstruct :evil::T [x <- :i64])"#,
        )
        .unwrap();
        let err = eval_in_frozen(&ast, &world, &Environment::new()).unwrap_err();
        assert!(matches!(err, RuntimeError { kind: RuntimeErrorKind::EvalForbidsMutationForm { .. }, .. }));
    }

    #[test]
    fn eval_refuses_enum() {
        // Stone 241.9 — migrated to :wat::core::defenum (HARD CUT).
        let world = frozen_with(
            r#"
            (:wat::config::set-capacity-mode! :error)
        "#,
        );
        let ast =
            crate::parse_one!(r#"(:wat::core::defenum :evil::E :A :B)"#).unwrap();
        let err = eval_in_frozen(&ast, &world, &Environment::new()).unwrap_err();
        assert!(matches!(err, RuntimeError { kind: RuntimeErrorKind::EvalForbidsMutationForm { .. }, .. }));
    }

    #[test]
    fn eval_refuses_newtype() {
        let world = frozen_with(
            r#"
            (:wat::config::set-capacity-mode! :error)
        "#,
        );
        let ast =
            crate::parse_one!(r#"(:wat::core::newtype :evil::N :i64)"#).unwrap();
        let err = eval_in_frozen(&ast, &world, &Environment::new()).unwrap_err();
        assert!(matches!(err, RuntimeError { kind: RuntimeErrorKind::EvalForbidsMutationForm { .. }, .. }));
    }

    #[test]
    fn eval_refuses_typealias() {
        let world = frozen_with(
            r#"
            (:wat::config::set-capacity-mode! :error)
        "#,
        );
        let ast =
            crate::parse_one!(r#"(:wat::core::typealias :evil::A :i64)"#).unwrap();
        let err = eval_in_frozen(&ast, &world, &Environment::new()).unwrap_err();
        assert!(matches!(err, RuntimeError { kind: RuntimeErrorKind::EvalForbidsMutationForm { .. }, .. }));
    }

    #[test]
    fn eval_refuses_load() {
        let world = frozen_with(
            r#"
            (:wat::config::set-capacity-mode! :error)
        "#,
        );
        let ast = crate::parse_one!(
            r#"(:wat::load-file! "evil.wat")"#,
        )
        .unwrap();
        let err = eval_in_frozen(&ast, &world, &Environment::new()).unwrap_err();
        assert!(matches!(err, RuntimeError { kind: RuntimeErrorKind::EvalForbidsMutationForm { .. }, .. }));
    }

    #[test]
    fn eval_refuses_digest_load() {
        let world = frozen_with(
            r#"
            (:wat::config::set-capacity-mode! :error)
        "#,
        );
        let ast = crate::parse_one!(
            r#"(:wat::digest-load! "x" :wat::verify::digest-sha256 :wat::verify::string "hex")"#,
        )
        .unwrap();
        let err = eval_in_frozen(&ast, &world, &Environment::new()).unwrap_err();
        assert!(matches!(err, RuntimeError { kind: RuntimeErrorKind::EvalForbidsMutationForm { .. }, .. }));
    }

    #[test]
    fn eval_refuses_signed_load() {
        let world = frozen_with(
            r#"
            (:wat::config::set-capacity-mode! :error)
        "#,
        );
        let ast = crate::parse_one!(
            r#"(:wat::signed-load! "x" :wat::verify::signed-ed25519 :wat::verify::string "sig" :wat::verify::string "pk")"#,
        )
        .unwrap();
        let err = eval_in_frozen(&ast, &world, &Environment::new()).unwrap_err();
        assert!(matches!(err, RuntimeError { kind: RuntimeErrorKind::EvalForbidsMutationForm { .. }, .. }));
    }

    #[test]
    fn eval_refuses_config_setter() {
        let world = frozen_with(
            r#"
            (:wat::config::set-capacity-mode! :error)
        "#,
        );
        let ast =
            crate::parse_one!(r#"(:wat::config::set-dims! 8192)"#).unwrap();
        let err = eval_in_frozen(&ast, &world, &Environment::new()).unwrap_err();
        assert!(matches!(err, RuntimeError { kind: RuntimeErrorKind::EvalForbidsMutationForm { .. }, .. }));
    }

    #[test]
    fn eval_refuses_mutation_form_at_any_depth() {
        // A mutation form nested inside otherwise-legal structure is
        // still refused. The walker descends into every child.
        // Stone 241.16 — migrated from `:wat::core::define` (HARD CUT total; no longer
        // in is_mutation_head) to `:wat::core::defstruct` (still a recognized mutation form).
        // parse_one! bypasses macro expansion; defstruct head preserved as-is.
        // Mechanism under test: refuse_mutation_forms walker catches nested mutation heads.
        let world = frozen_with(
            r#"
            (:wat::config::set-capacity-mode! :error)
        "#,
        );
        let ast = crate::parse_one!(
            r#"(:wat::core::let ((x 1))
                 (:wat::core::defstruct :evil::Inner [y <- :wat::core::i64]))"#,
        )
        .unwrap();
        let err = eval_in_frozen(&ast, &world, &Environment::new()).unwrap_err();
        assert!(matches!(err, RuntimeError { kind: RuntimeErrorKind::EvalForbidsMutationForm { .. }, .. }));
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
        "#,
        );
        let ast =
            crate::parse_one!(r#"(:wat::core::i64::+ 20 22)"#).unwrap();
        let hex = digest_hex_for(&ast);
        let result =
            eval_digest_in_frozen(&ast, &world, &Environment::new(), "sha256", &hex)
                .expect("eval ok");
        assert!(matches!(result.value(), Value::i64(42)));
    }

    #[test]
    fn eval_digest_mismatch_refuses() {
        let world = frozen_with(
            r#"
            (:wat::config::set-capacity-mode! :error)
        "#,
        );
        let ast = crate::parse_one!(r#"(:wat::core::i64::+ 1 1)"#).unwrap();
        let wrong =
            "0000000000000000000000000000000000000000000000000000000000000000";
        let err =
            eval_digest_in_frozen(&ast, &world, &Environment::new(), "sha256", wrong)
                .unwrap_err();
        match err {
            RuntimeError { kind: RuntimeErrorKind::EvalVerificationFailed { err }, .. } => {
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
        "#,
        );
        let ast = crate::parse_one!("42").unwrap();
        let err =
            eval_digest_in_frozen(&ast, &world, &Environment::new(), "md5", "abc123")
                .unwrap_err();
        match err {
            RuntimeError { kind: RuntimeErrorKind::EvalVerificationFailed { err }, .. } => {
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
        "#,
        );
        let ast =
            crate::parse_one!(r#"(:wat::core::i64::+ 40 2)"#).unwrap();
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
        assert!(matches!(result.value(), Value::i64(42)));
    }

    #[test]
    fn eval_signed_tampered_ast_refuses() {
        let world = frozen_with(
            r#"
            (:wat::config::set-capacity-mode! :error)
        "#,
        );
        let original = crate::parse_one!(r#"(:wat::core::i64::+ 1 1)"#).unwrap();
        let tampered = crate::parse_one!(r#"(:wat::core::i64::+ 99 99)"#).unwrap();
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
            RuntimeError { kind: RuntimeErrorKind::EvalVerificationFailed { err }, .. } => {
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
        "#,
        );
        let ast = crate::parse_one!("42").unwrap();
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
            RuntimeError { kind: RuntimeErrorKind::EvalVerificationFailed { err }, .. } => {
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
        // Stone 241.16 — migrated from `:wat::core::define` (HARD CUT total; no longer
        // in is_mutation_head) to `:wat::core::defstruct` (still a recognized mutation form).
        // Mechanism: even a digest-verified AST is refused if it contains a mutation head.
        let world = frozen_with(
            r#"
            (:wat::config::set-capacity-mode! :error)
        "#,
        );
        let ast = crate::parse_one!(
            r#"(:wat::core::defstruct :evil::E [x <- :wat::core::i64])"#,
        )
        .unwrap();
        let hex = digest_hex_for(&ast);
        let err =
            eval_digest_in_frozen(&ast, &world, &Environment::new(), "sha256", &hex)
                .unwrap_err();
        assert!(matches!(err, RuntimeError { kind: RuntimeErrorKind::EvalForbidsMutationForm { .. }, .. }));
    }

    // ─── Phase-order invariant: expand_all precedes register_defines ────────
    //
    // ORDER LOAD-BEARING: macro_eval purity (src/macros/eval.rs) depends on
    // expand_all running BEFORE register_defines in the freeze pipeline. This
    // test goes RED if the order reverses — a user defn referenced inside a
    // macro program-body must NOT be reachable at expand time. The test asserts
    // the invariant structurally: a computed-unquote that tries to call a
    // user-defined function must fail (the defn is not yet registered when
    // expand_all runs). If register_defines ran first the call would succeed
    // and the fenced evaluator's purity guarantee would be vacated.
    #[test]
    fn expand_runs_before_register_defines_phase_order() {
        // A macro whose program body calls a user-defined function.
        // At expand time the defn is NOT yet registered → the call errors.
        // If this test PASSES: order is correct (expand before defn-register).
        // If this test FAILS (startup succeeds): register_defines moved before
        // expand_all — the load-bearing invariant in eval.rs is violated.
        let err = startup(r#"
            (:wat::config::set-capacity-mode! :error)
            (:wat::core::defn :my::helper [] -> :wat::core::i64 42)
            (:wat::core::defmacro :my::uses-helper []
              -> :AST
              (:wat::core::i64::+ (:my::helper) 0))
            (:my::uses-helper)
        "#);
        assert!(
            err.is_err(),
            "a macro program-body calling a user defn must fail at expand time \
             (defn not registered yet); if it succeeded, register_defines ran before \
             expand_all — ORDER LOAD-BEARING invariant violated (src/macros/eval.rs)"
        );
    }
}
