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

pub(crate) mod env;
// `pub`, not `pub(crate)` — unlike `env` (a genuinely internal builder), `validator` is the
// extension point ITSELF: any crate depending on `wat` must be able to name
// `wat::freeze::validator::{FreezeValidator, FreezeValidatorError}` to `inventory::submit!`
// its own freeze-time validator (mirrors `crate::restriction_entry`, which is `pub mod` at
// the crate root for the same reason — see `tests/reflection/wat_arc198_slice2_stone_1_inventory_wiring.rs`
// for the cross-crate submission proof this same shape supports).
pub mod validator;

use crate::ast::WatAST;
use crate::check::{check_program, CheckErrors};
use crate::config::{collect_entry_file, collect_entry_file_with_inherit, Config, ConfigError};
use crate::load::{resolve_loads, LoadError, SourceLoader};
use crate::macros::{MacroError, MacroRegistry};
use crate::parser::{parse_all_with_file, ParseError};
use crate::stdlib::StdlibError;
use crate::resolve::ResolveError;
use crate::runtime::{
    apply_function, EvalBreak, Environment,
    FunctionBody, RuntimeError, RuntimeErrorKind, SymbolTable, TrackedValue, Value,
};
use crate::value::EncodingCtx;
use crate::services::ThreadId;
use crate::types::{TypeEnv, TypeError, TypeExpr};
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
    services: Option<Arc<crate::services::RuntimeServices>>,
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
    /// Arc 170 stdio-as-defservice (PHASE 1) — the three PRIMED stdio defservices' `Handle` values
    /// (`:wat::kernel::{stdin,stdout,stderr}-svc'`), each carrying its admin lineage `Peer'`. Held
    /// here for the process lifetime so the services stay alive (dropping a Handle's admin peer
    /// signals `:Shutdown`). They drop with this struct's field teardown (after the Drop body), which
    /// signals the three idle serve loops to exit. COEXISTS with the hand-rolled path above; Phase 1
    /// flips no verb, so these primes only bootstrap + stay held for Strike 2. Order: stdin, stdout,
    /// stderr.
    ///
    /// Held-for-lifetime (RAII): the value IS the liveness; no field is read in Phase 1. Strike 2
    /// reads the paired addresses via `sym.primed_stdio()`, not this Vec. `allow(dead_code)` marks
    /// the intentional hold.
    #[allow(dead_code)]
    primed_stdio_handles: Vec<Value>,
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
            crate::services::deregister_thread_from_services(self.main_thread_id, svc);
        }

        // Step 2.
        let _ = crate::services::uninstall_thread_io();

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
                eprintln!("#wat.substrate/Diag{{:site \"process-runtime-drop\" :msg \"stdin service peer join error\" :error {:?}}}", format!("{:?}", e));
            }
        }
        // Step 6: Join the universe-resident stdout service peer (Rust thread).
        if let Some(join) = self.stdout_service_join.take() {
            if let Err(e) = join.join() {
                eprintln!("#wat.substrate/Diag{{:site \"process-runtime-drop\" :msg \"stdout service peer join error\" :error {:?}}}", format!("{:?}", e));
            }
        }
        // Step 7: Join the universe-resident stderr service peer (Rust thread).
        if let Some(join) = self.stderr_service_join.take() {
            if let Err(e) = join.join() {
                eprintln!("#wat.substrate/Diag{{:site \"process-runtime-drop\" :msg \"stderr service peer join error\" :error {:?}}}", format!("{:?}", e));
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
    let stdio = match crate::services::take_ambient_stdio() {
        Some(s) => s,
        None => crate::process::lend_ambient(),
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
            span: crate::rust_caller_span!(),
            kind: RuntimeErrorKind::UnknownFunction(
                ":wat::kernel::services::StdInService/handle".into(),
            ),
        })?
        .clone();
    let stdin_peer = crate::services::spawn_service_peer(
        "stdin",
        stdin_handle_fn,
        Value::io__IOReader(stdio.stdin.clone()),
        pre_sym.clone(),
        |rep: &Value| match rep {
            Value::Aggregate(a) if a.fields.len() >= 2 => match &a.fields[1] {
                Value::String(s) => Ok((**s).clone()),
                _ => Err("StdInService Rep field[1] is not a String".into()),
            },
            _ => Err("StdInService Rep is not an Aggregate with ≥2 fields".into()),
        },
    );

    // stdout: look up the pure handle fn; spawn the Rust write service peer.
    let stdout_handle_fn = pre_sym
        .get(":wat::kernel::services::StdOutService/handle")
        .ok_or_else(|| RuntimeError {
            span: crate::rust_caller_span!(),
            kind: RuntimeErrorKind::UnknownFunction(
                ":wat::kernel::services::StdOutService/handle".into(),
            ),
        })?
        .clone();
    let stdout_peer = crate::services::spawn_service_peer(
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
            span: crate::rust_caller_span!(),
            kind: RuntimeErrorKind::UnknownFunction(
                ":wat::kernel::services::StdErrService/handle".into(),
            ),
        })?
        .clone();
    let stderr_peer = crate::services::spawn_service_peer(
        "stderr",
        stderr_handle_fn,
        Value::io__IOWriter(stdio.stderr.clone()),
        pre_sym.clone(),
        |_: &Value| Ok(()),
    );

    // Step 3 — Build RuntimeServices carrier + augmented SymbolTable.
    let services = Arc::new(crate::services::RuntimeServices {
        stdin_ctrl: stdin_peer.input_tx.clone(),
        stdout_ctrl: stdout_peer.input_tx.clone(),
        stderr_ctrl: stderr_peer.input_tx.clone(),
    });
    let mut sym_with_services = frozen.symbols().clone();
    sym_with_services.set_runtime_services(Arc::clone(&services));

    // Step 4 — Register calling thread + install ThreadIO.
    let main_thread_id = crate::services::next_thread_id();
    let main_io =
        // boot-time registration — no user form exists; genuinely spanless-by-domain.
        crate::services::register_thread_with_services(main_thread_id, &services, &crate::rust_caller_span!())?;
    crate::services::install_thread_io(main_io);

    // Step 5 (arc 170 stdio-as-defservice, PHASE 1) — start the three PRIMED stdio defservices on the
    // real fds (0/1/2), COEXISTING with the hand-rolled path (steps 2–4). Driven through the kernel
    // helper `:wat::kernel::start-primed-stdio` (wat/kernel/services/stdio-primes.wat), a plain 3-arg
    // defn whose body calls the three `<svc>/start` kwargs macros — those expand at NORMAL freeze time
    // (inside the defn body), sidestepping the kwargs-defn-via-`eval_in_frozen` macro-eval gap (that
    // path mis-resolves the companion's `$impl` keyword to a live fn). The helper returns a 3-tuple of
    // Handles. The fds are PURE i64 literals (they ride `Admin::Init` clean); each impure
    // IOWriter/IOReader is born INSIDE the service's kernel `::init` via `IOWriter/from-fd`
    // (dup-then-own — the service owns a dup, never the real fd 0/1/2).
    //
    // Applied against `frozen.symbols()` (NOT `sym_with_services`) so the spawned service threads see
    // no `runtime_services` carrier and skip old-path stdio registration (the lazy-registration
    // pattern; symbol_table.rs). Nothing is flipped: the five caller verbs still route to the old
    // path; these primes just bootstrap and are held alive (`primed_stdio_handles`) for Strike 2.
    let start_helper = pre_sym
        .get(":wat::kernel::start-primed-stdio")
        .ok_or_else(|| RuntimeError {
            span: crate::rust_caller_span!(),
            kind: RuntimeErrorKind::UnknownFunction(":wat::kernel::start-primed-stdio".into()),
        })?
        .clone();
    let handles_tuple = apply_function(
        start_helper,
        vec![Value::i64(0), Value::i64(1), Value::i64(2)],
        pre_sym,
        crate::rust_caller_span!(),
    )?;
    let (stdin_handle, stdout_handle, stderr_handle) = match &handles_tuple {
        Value::Tuple(v) if v.len() == 3 => (v[0].clone(), v[1].clone(), v[2].clone()),
        other => {
            return Err(RuntimeError {
                span: crate::rust_caller_span!(),
                kind: RuntimeErrorKind::MalformedForm {
                    head: "arc 170 start-primed-stdio".into(),
                    reason: format!(
                        "expected a 3-tuple of stdio Handles; got {:?}",
                        crate::runtime::ValueSnapshot::of(other)
                    ),
                },
            });
        }
    };

    // Extract each Handle's `addr` field (record field order: [handle <- Peer', addr <- Address'],
    // wat/service.wat) → the client-dial Address' the Strike-2 verbs will `connect'`. Stash the three
    // on a `PrimedStdio` carrier set on the SymbolTable (mirrors `RuntimeServices`).
    let addr_of = |h: &Value, which: &str| -> Result<Value, RuntimeError> {
        match h {
            Value::Aggregate(a) if a.fields.len() >= 2 => Ok(a.fields[1].clone()),
            other => Err(RuntimeError {
                span: crate::rust_caller_span!(),
                kind: RuntimeErrorKind::MalformedForm {
                    head: format!("arc 170 primed {which} Handle"),
                    reason: format!(
                        "expected a ≥2-field Handle aggregate (handle, addr); got {:?}",
                        crate::runtime::ValueSnapshot::of(other)
                    ),
                },
            }),
        }
    };
    let primed = Arc::new(crate::services::PrimedStdio {
        stdin_addr: addr_of(&stdin_handle, "stdin")?,
        stdout_addr: addr_of(&stdout_handle, "stdout")?,
        stderr_addr: addr_of(&stderr_handle, "stderr")?,
    });
    sym_with_services.set_primed_stdio(primed);

    Ok(ProcessRuntime {
        sym_with_services,
        services: Some(services),
        main_thread_id,
        stdin_service_join: Some(stdin_peer.thread),
        stdout_service_join: Some(stdout_peer.thread),
        stderr_service_join: Some(stderr_peer.thread),
        // stdin, stdout, stderr — held alive for the process lifetime (Strike 2 reaches them).
        primed_stdio_handles: vec![stdin_handle, stdout_handle, stderr_handle],
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
                    // Arc 109 — an anonymous fn's identity is the structured
                    // ANON_FN_SYMBOL marker, NOT a `<fn@span>` stringy costume.
                    FunctionBody::Wat(_) => crate::value::ANON_FN_SYMBOL.to_string(),
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
                    // Arc 109 — an anonymous fn's identity is the structured
                    // ANON_FN_SYMBOL marker, NOT a `<fn@span>` stringy costume.
                    FunctionBody::Wat(_) => crate::value::ANON_FN_SYMBOL.to_string(),
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
pub enum StartupError {
    Parse(ParseError),
    Config(ConfigError),
    Load(LoadError),
    Macro(MacroError),
    Type(TypeError),
    Resolve(ResolveError),
    Check(CheckErrors),
    /// A registered [`crate::freeze::validator::FreezeValidator`] (drained at `build_env`
    /// step 7.8) found a problem. Arc 294 item 9a's `defrule` wall
    /// (`crate::rete::validate::validate_rete_rules`) is the first registered consumer — a
    /// malformed `:when` clause / unknown field-ref / RHS arity mismatch surfaces here,
    /// still tagged `#wat.rete/*` (the boxed error's `to_edn()` preserves its concrete
    /// namespace by dynamic dispatch). Any crate depending on `wat` can register its own
    /// freeze-time validator the same way.
    Validator(Box<dyn crate::freeze::validator::FreezeValidatorError>),
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
    /// Arc 170 — the `:user::main` wall, imposed at `startup_from_source`.
    /// Fires when a declared `:user::main` is not exactly `[] -> :wat::core::nil`
    /// ([`validate_user_main_signature`]) or its body is the bare `nil` literal
    /// ([`validate_user_main_not_useless`], UselessMain).
    MainSignature(String),
}

impl fmt::Debug for StartupError {
    // Stone B: Debug emits EDN, not Rust struct layout.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&crate::to_edn::to_wire_edn(self))
    }
}

impl fmt::Display for StartupError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&crate::to_edn::to_wire_edn(self))
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
    /// Arc 296 — produce structured [`wat_edn::OwnedValue`] records
    /// for this error. The `Check` arm yields one record per
    /// `CheckError` (so tooling receives one structured object per
    /// finding); all other arms yield a SINGLE tagged envelope from
    /// `self.to_edn()`.
    ///
    /// **Data first.** Consumers (`--check-output edn|json`;
    /// structured test-runner output) call this instead of
    /// `Display` so field-level data reaches tooling without text
    /// parsing.
    pub fn to_edn_values(&self) -> Vec<wat_edn::OwnedValue> {
        use crate::to_edn::ToEdn;
        match self {
            StartupError::Check(errors) => errors.0.iter().map(|e| e.to_edn()).collect(),
            _ => vec![self.to_edn()],
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
/// 7. Name resolution — normalize namespaced symbol refs ([`normalize_symbol_refs`]),
///    THEN validate all call-head references ([`resolve_references`]). Order matters:
///    the resolver only validates keyword heads, so normalize must precede it.
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
    let world = startup_from_forms(entry_forms, base_canonical, loader)?;
    // Arc 170 — the `:user::main` wall. Imposed HERE (not in
    // `startup_from_forms`) because this is the chokepoint every real
    // entry path routes through (`startup_from_file` / `startup_beside` /
    // `cargo wat` / the wat-scripts load gate); `startup_from_forms` stays
    // usable by internal callers (macro-expanded sandboxes, dynamically
    // generated tests, compiler passes) that legitimately build worlds
    // without a `:user::main`. Conditional on `:user::main` being
    // declared at all — `startup_bare()` (no main) passes cleanly.
    if world.symbols().get(":user::main").is_some() {
        validate_user_main_signature(&world).map_err(StartupError::MainSignature)?;
        validate_user_main_not_useless(&world).map_err(StartupError::MainSignature)?;
    }
    Ok(world)
}

/// Slurp a workspace-relative `.wat` file and start it up — the canonical loader for
/// tests/demos. Assumes the cwd is the crate root (where `cargo test`/`cargo wat` run).
///
/// **The repo filesystem architecture is the contract:** test wat lives in `.wat` files
/// (e.g. `wat-scripts/demos/<name>.wat`), slurped by relative-name literal — NEVER inlined
/// as a Rust string. A `.wat` fixture can be `cargo wat`-run, fix-wat-migrated, and lint-checked
/// as real wat; an inlined string can do none of those. Deviating is more friction, not less.
///
/// Panics with a clear message if the path does not resolve — a fixture path that misses is a
/// test-authoring error, surfaced loudly rather than swallowed.
pub fn startup_from_file(rel_path: &str) -> Result<FrozenWorld, StartupError> {
    let src = std::fs::read_to_string(rel_path)
        .unwrap_or_else(|e| panic!("wat fixture {rel_path:?} must exist (run from crate root): {e}"));
    startup_from_source(&src, Some(rel_path), Arc::new(crate::load::InMemoryLoader::new()))
}

/// Slurp the `.wat` fixture sitting BESIDE a probe — same basename, `.rs`→`.wat`.
///
/// **The co-located test-fixture scheme (intueri-derived):** a probe's fixture inherits the probe's
/// already-structured name and lives at its side (`tests/<group>/<probe>.{rs,wat}`), so no fixture
/// ever names its own context — the atrocious-names failure is dissolved at the root. Call as
/// `startup_beside(file!())`: `file!()` yields the crate-root-relative `<probe>.rs`, the swap yields
/// `<probe>.wat`. Rename-safe — rename the probe and the derived path follows. Use the explicit
/// [`startup_from_file`] only for the rare fixture shared by multiple probes.
///
/// (`wat-scripts/demos/<name>/` is reserved for curated README-bearing showpieces a human reads — NOT
/// probe inputs.)
pub fn startup_beside(caller_rs: &str) -> Result<FrozenWorld, StartupError> {
    let wat = caller_rs
        .strip_suffix(".rs")
        .map(|stem| format!("{stem}.wat"))
        .unwrap_or_else(|| panic!("startup_beside expects a `.rs` caller path (pass file!()); got {caller_rs:?}"));
    startup_from_file(&wat)
}

/// Run a co-located `.wat` fixture's named zero-arg entry fn — the lint-clean idiom.
///
/// [`startup_beside`] + fetch-the-entry-fn + [`apply_function`], fused: freezes the `.wat`
/// beside `caller_rs` (pass `file!()`), fetches the fully-qualified zero-arg entry fn
/// `fn_name` (e.g. `":user::compute"`), applies it, and returns the fn's OWN
/// `Result<Value, RuntimeError>`. The wat LOGIC stays in the co-located `.wat`; this is a
/// pure Rust INVOCATION, so it does NOT trip `no_inlined_wat` (which targets inline wat
/// *logic*, not invocation) — unlike the `parse_one!("(:user::compute)")` stub it replaces.
///
/// `call` (invoke a named fn), deliberately NOT `run` — distinct from `run_beside(expr)`-style
/// helpers that eval an expression STRING inline. The entry fn's own raise is returned as
/// `Err` (so a test can `expect_err` a fixture meant to fail); only a broken/missing fixture
/// panics (a test-authoring bug, not an outcome under test).
pub fn call_beside(caller_rs: &str, fn_name: &str) -> Result<Value, RuntimeError> {
    let world = startup_beside(caller_rs)
        .unwrap_or_else(|e| panic!("call_beside: fixture beside {caller_rs:?} failed to freeze: {e:?}"));
    let func = world
        .symbols()
        .get(fn_name)
        .unwrap_or_else(|| panic!("call_beside: no entry fn {fn_name:?} in the fixture beside {caller_rs:?}"))
        .clone();
    apply_function(func, vec![], world.symbols(), crate::rust_caller_span!())
}

/// A frozen DEFAULT world with no user source — the canonical "I just need a world to evaluate
/// against" entry for tests whose subject is the Rust substrate, not any wat program.
///
/// Prefer this over a hand-rolled `startup_from_source("")` / `:user::main` placeholder: it carries
/// NO inlined wat, so a substrate test states honestly that it has no wat-under-test. (For a test
/// WITH wat-under-test, co-locate the wat and use [`startup_beside`].) The stdlib is loaded; only the
/// user entry is empty.
pub fn startup_bare() -> Result<FrozenWorld, StartupError> {
    startup_from_source("", None, Arc::new(crate::load::InMemoryLoader::new()))
}

// startup_from_source_with_deps retired in arc 015 slice 3a.
// Dep sources now install globally via `wat::source::install_dep_sources`
// before any freezing; `stdlib_forms()` concatenates baked + installed
// so every freeze pass — including `:wat::kernel::run-sandboxed-ast`
// and `:wat::kernel::spawn-process` children — sees dep surface
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
/// `:wat::kernel::run-sandboxed-hermetic-ast`, and `:wat::kernel::spawn-process`
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

    // 3a–7.6. Build the full registered environment (macros + types +
    //         symbols + user residue) via the canonical single pipeline.
    let mut bundle = env::build_env(loaded)?;

    // 7.5. Arc 157 slice 1a-ii — propagate redef config flags to the
    // SymbolTable carrier BEFORE check_program so that CheckEnv::from_symbols
    // sees the correct redef_allowed flag (check happens at step 8;
    // FrozenWorld::freeze at step 9 would be too late).
    bundle.symbols.redef_allowed = config.redef_allowed;
    bundle.symbols.eval_redef_allowed = config.eval_redef_allowed;

    // 8. Type check.
    check_program(&bundle.residue, &bundle.symbols, &bundle.types)?;

    // 9. Freeze. The loader moves into the frozen world's
    //    SymbolTable so runtime primitives (`:wat::eval-file!` and
    //    the file-path variants of the verified eval/load forms,
    //    `:wat::verify::file-path` payloads) can route through the
    //    same capability that handled startup loads.
    FrozenWorld::freeze(
        config,
        bundle.types,
        bundle.macros,
        bundle.symbols,
        bundle.residue,
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
/// ambient stdio cell ([`crate::services::AmbientStdio`]); tests
/// install pipe-backed handles via
/// [`crate::services::install_ambient_stdio`] before invoking.
/// Production paths (wat-cli, fork.rs:659/1044) leave the ambient
/// unset and fall through to real fd 0/1/2 via PipeReader /
/// PipeWriter.
pub fn invoke_user_main(
    frozen: &FrozenWorld,
    args: Vec<Value>,
) -> Result<Value, RuntimeError> {
    invoke_user_main_orchestrated(frozen, args, None)
}

/// Like [`invoke_user_main`], but installs `user_program` as the `user.program` field of the
/// ambient `:wat::program::Env` (instead of the `EmptyEnv` default) before `:user::main` runs.
/// `user_program` must be a `:wat::core::Record` (any subtype). The root (wat-cli `--env`) and process
/// children supply the result of running their env-producing fn here.
pub fn invoke_user_main_with_program(
    frozen: &FrozenWorld,
    args: Vec<Value>,
    user_program: Value,
) -> Result<Value, RuntimeError> {
    invoke_user_main_orchestrated(frozen, args, Some(user_program))
}

/// Resolve a process/CLI env-fn SOURCE STRING into a `user.program` `:wat::core::Record`, evaluated
/// in `world` (the universe that has the type loaded). Dispatches on the eval result:
/// - a 0-arg fn (`Value::wat__core__fn`) → applied (0 args); the result must be a `:wat::core::Record`
/// - a `:wat::core::Record` (any subtype, including holon variants) → used directly
/// - anything else → `RuntimeError` (env-fn must produce a `:wat::core::Record`)
///
/// This is the shared core that `run_user_main_in_child` (process tier) and the CLI `--env`
/// path (arc 213 / 3b-f) both call. Testable in-process against a world that defines the
/// record type — exactly as it runs in the spawned universe.
pub fn resolve_env_program(world: &FrozenWorld, src: &str) -> Result<Value, RuntimeError> {
    let ast = crate::parse_one!(src).map_err(|e| RuntimeError {
        span: crate::rust_caller_span!(),
        kind: RuntimeErrorKind::MalformedForm {
            head: "env-fn".into(),
            reason: format!("arc 209 C0b.3b-e: env-fn parse error: {e:?}"),
        },
    })?;
    let v = eval_in_frozen(&ast, world, &Environment::new())?.value_owned();
    match v {
        Value::wat__core__fn(f) => {
            let r = apply_function(f, vec![], world.symbols(), crate::rust_caller_span!())?;
            match r {
                r @ Value::Aggregate(_) => Ok(r),
                other => Err(RuntimeError {
                    span: crate::rust_caller_span!(),
                    kind: RuntimeErrorKind::MalformedForm {
                        head: "env-fn".into(),
                        reason: format!(
                            "arc 209 C0b.3b-e: env-fn fn returned a non-record; \
                             env-fn must produce a :wat::core::Record (any subtype); got: {:?}",
                            crate::runtime::ValueSnapshot::of(&other)
                        ),
                    },
                }),
            }
        }
        r @ Value::Aggregate(_) => Ok(r),
        other => Err(RuntimeError {
            span: crate::rust_caller_span!(),
            kind: RuntimeErrorKind::MalformedForm {
                head: "env-fn".into(),
                reason: format!(
                    "arc 209 C0b.3b-e: env-fn must produce a :wat::core::Record (any subtype); got: {:?}",
                    crate::runtime::ValueSnapshot::of(&other)
                ),
            },
        }),
    }
}

/// The orchestrator body. Delegates to [`bootstrap_wat_vm_process`] for
/// steps 1–4 (service spawn + ThreadIO install); then runs `:user::main`;
/// cleanup runs automatically when `runtime` drops at end of scope.
fn invoke_user_main_orchestrated(
    frozen: &FrozenWorld,
    args: Vec<Value>,
    user_program: Option<Value>,
) -> Result<Value, RuntimeError> {
    // Steps 1–4: bootstrap services + ThreadIO (substrate-owned).
    let runtime = bootstrap_wat_vm_process(BootstrapArgs { frozen })?;

    // Arc 259 (The Forced Hand) — install the ambient program env BEFORE
    // `:user::main`. The live VM constructs the base env itself (self-hosted):
    // `wat.started-at`      = boot clock (real epoch — primed by wat-cli at its
    //                         earliest point; re-captured across a fork so a
    //                         :process peer measures its own boot). The gap
    //                         started-at → peer-started-at is the real
    //                         boot→entry latency.
    // `wat.peer-started-at` = now at the seam (this frame's entry).
    // `wat.process-id` = OS pid; `wat.os-thread-id` = OS thread id (gettid).
    // `wat.peer-kind` = :process — root main OWNS its address space (builder-locked).
    //   Thread peers stamp :thread; forked process peers stamp :process.
    // `wat.cpu-count` = available_parallelism(), a host constant inherited down the
    //   spawn tree (like wat.started-at). Fallback 1 if the OS refuses to report.
    // The RAII guard is held across main's run on this thread and uninstalls on
    // scope exit.
    //
    // Arc 209 C0b.3b-d — `user.program` injection seam: the 7th field of
    // `:wat::program::Env` is populated from the injected value (if any) or the
    // `EmptyEnv` default (current behavior, preserved). The value is bound as a
    // local in `ctor_env` and referenced by name in the env constructor source —
    // exactly the thread-tier init-fn pattern from spawn.rs:482–498.
    let pid = std::process::id() as i64;
    let tid = unsafe { libc::gettid() } as i64;
    let boot_nanos = crate::time::process_boot_instant().timestamp_nanos_opt().unwrap_or(0);
    // cpu_count = available_parallelism() via host_cpu_count(), a host constant inherited
    // down the spawn tree (like wat.started-at). Fallback 1 if the OS refuses to report.
    let cpu_count = crate::runtime::host_cpu_count();
    let user_program_val = match user_program {
        Some(v) => v,
        None => eval_in_frozen(
            // Arc 294 item 9a — internal boot machinery evals the ctor form directly (no
            // macro expansion), so it targets the positional PRIME `:EmptyEnv'` (the bare
            // name is now the kwargs companion macro, which never runs on this path).
            &crate::parse_one!("(:wat::program::EmptyEnv)")
                .expect("arc 209 C0b.3b-d: EmptyEnv ctor parses"),
            frozen,
            &crate::runtime::Environment::new(),
        )
        .map(|tv| tv.value_owned())?,
    };
    let ctor_env = crate::runtime::Environment::new()
        .child()
        .bind_unknown_span("user-program", crate::value::TrackedValue::from(user_program_val))
        .build();
    let env_src = format!(
        // Arc 294 item 9a — direct-eval boot machinery → positional PRIME `:Env'`.
        "(:wat::program::Env :wat.started-at (:wat::time::at-nanos {boot_nanos}) :wat.peer-started-at (:wat::time::now) :wat.process-id {pid} :wat.os-thread-id {tid} :wat.peer-kind :wat::program::PeerKind::process :wat.cpu-count {cpu_count} :user.program user-program)"
    );
    let env_ast = crate::parse_one!(&env_src)
        .expect("arc 259: the program-env constructor form parses");
    let program_env = eval_in_frozen(&env_ast, frozen, &ctor_env)
        .map(|tv| tv.value_owned())?;
    let _program_env_guard = crate::services::install_program_env(program_env);

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
        None => Err(RuntimeError { span: crate::rust_caller_span!(), kind: RuntimeErrorKind::UserMainMissing }),
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

/// A declared `:user::main` must DO something — its body may not be the bare
/// `nil` literal. "Either give it a real body or omit the main entirely."
/// (arc 170, the UselessMain wall.) Semantic uselessness is undecidable; we
/// wall the one literal form sonnets keep writing:
/// `(:user::main [] -> :wat::core::nil nil)`.
pub fn validate_user_main_not_useless(frozen: &FrozenWorld) -> Result<(), String> {
    let func = match frozen.symbols().get(":user::main") {
        Some(f) => f,
        None => return Ok(()), // no main declared — nothing to check
    };
    if let FunctionBody::Wat(ast) = &func.body {
        if matches!(&**ast, WatAST::NilLit(_)) {
            return Err(
                ":user::main body is the bare `nil` literal (UselessMain). \
                 A declared :user::main must DO something — either give it a real \
                 body, or OMIT the main entirely (not every file needs one; only \
                 programs that RUN). Never write `(:user::main [] -> :wat::core::nil nil)`. \
                 And do NOT cheat by wrapping a no-op to slip past this check — \
                 `(:user::main [] -> :wat::core::nil (:wat::core::let [_ 0] nil))`, \
                 `(... (:wat::core::do nil))`, or any body that computes nothing and \
                 returns nil is the SAME uselessness in disguise and will be rejected on \
                 sight in review. If the file does not need to RUN, delete the main."
                    .to_string(),
            );
        }
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
    // READ→EXPAND→EVAL (arc 294 item 9a — full Lisp). Expand macros against the frozen
    // registry BEFORE eval, so a source-written form (a kwargs construction, a user
    // macro/DSL — even inside a Rust string literal) evaluates correctly here, exactly
    // as it would at startup. Without this, `eval_in_frozen` was READ→EVAL and callers
    // had to reach for the positional prime `:T'`; with it, written source is kwargs and
    // the prime is reserved for GENERATED code. Mutation forms are refused on BOTH the
    // raw and the expanded form (a macro could expand to a def/defmacro).
    let expanded = crate::macros::expand_fully(ast.clone(), &frozen.macros, env, frozen.symbols())
        .map_err(|e| RuntimeError {
            span: ast.span().clone(),
            kind: RuntimeErrorKind::MacroExpansionFailed { op: "eval_in_frozen".into(), cause: Box::new(e) },
        })?;
    refuse_mutation_forms(&expanded)?;
    crate::runtime::eval(&expanded, env, frozen.symbols())
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
    // Map, and Set uniformly. Pre-arc-212 this walker silently accepted
    // mutation forms buried inside bracketed shapes (let-binding vectors,
    // map/set literals) — they slipped past freeze-time refusal.
    // children() returns &[] for leaf nodes (no-op).
    for child in ast.children().iter() {
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
            // Arc 293.2-parity — structtype is the low-level primitive defstruct (macro) expands to.
            | ":wat::core::structtype"
            // Stone 241.9 — defenum replaces enum (HARD CUT).
            | ":wat::core::defenum"
            | ":wat::core::newtype"
            | ":wat::core::typealias"
            // Arc 293.3-core — structural surface.
            | ":wat::core::defsurface"
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
            // Arc 293.2-parity — structtype is the low-level primitive defstruct (macro) expands to.
            | ":wat::core::structtype"
            // Stone 241.9 — defenum replaces enum (HARD CUT).
            | ":wat::core::defenum"
            | ":wat::core::newtype"
            | ":wat::core::typealias"
            // Stone 241.12 — defalias is a declaration form (alias binding).
            | ":wat::core::defalias"
            // Arc 293.3-core — structural surface.
            | ":wat::core::defsurface"
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
        // rune:lint(loose-assert) — property over variable set; type registry holds many built-in types; only user-type membership is the contract
        assert!(world.types().contains(":my::Candle"));
    }

    #[test]
    fn user_macro_registers() {
        let src = r#"
            (:wat::config::set-capacity-mode! :error)
            (:wat::core::defmacro :my::vocab::Double
              [x <- :wat::WatAST]
              -> :wat::WatAST
              `(:wat::holon::Blend ,x ,x 1 1))
        "#;
        let world = startup(src).expect("startup");
        // rune:lint(loose-assert) — property over variable set; macro registry holds built-ins; only user-macro membership is the contract
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
            (:wat::core::defn :user::main [] -> :wat::core::nil (:wat::core::let [_argv (:wat::runtime::argv)] nil))
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
            r#"(:wat::core::defmacro :evil::M [x <- :wat::WatAST] -> :wat::WatAST x)"#,
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
              -> :wat::WatAST
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
