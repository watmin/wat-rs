//! `wat::distribution` — **published surface for third-party wat
//! distributions.** A distributor writes their own crate, their own
//! `#[wat_dispatch]` extensions, and a small `[[bin]]` that calls
//! [`run`] with their batteries — composed against wat core, without
//! forking it. This is the extension point, not an internal helper;
//! it has no in-tree consumer *by design*, because wat-rs's own
//! batteries (cache, sqlite) were absorbed into core (arc 278 Cache
//! Stone 5) and now register via [`wat::rust_deps::RustDepsBuilder::with_wat_rs_defaults`]
//! unconditionally. A future reader finding [`run`] / [`Battery`]
//! unused in-tree should read this note before assuming dead code.
//!
//! Arc 099 extracted the bare CLI from the substrate crate into the
//! sibling `wat-cli` crate. Arc 100 vended its guts as a public API.
//! Arc 170 folded the crate back into core — the split was leaky in
//! the wrong direction (`fork_program_from_source` was already core's,
//! documented as "used exclusively by wat-cli") — as `wat::distribution`,
//! renamed from `wat_cli` to name the CAPABILITY ("roll their own wat
//! distribution") rather than the implementation detail (argv parsing):
//!
//! ```text
//! // your_crate/src/main.rs
//! fn main() -> std::process::ExitCode {
//!     wat::distribution::run(&[
//!         (my_crate::register, my_crate::wat_sources),
//!         (another_crate::register, another_crate::wat_sources),
//!     ])
//! }
//! ```
//!
//! That is the entire user surface for "I want a wat CLI with my
//! own batteries." Argv parsing, signal handlers, exit codes, the
//! `wat test` subcommand, and dep registration are all handled by
//! [`run`]. The user picks which extensions to link.
//!
//! For the canonical batteries-included binary (every workspace
//! `#[wat_dispatch]` extension installed), invoke `wat` from
//! `target/{debug,release}/wat` — it is a thin wrapper around
//! [`run`] with the workspace defaults (empty, post cache Stone 5 —
//! see `src/bin/wat.rs`).
//!
//! # Module layout
//!
//! Namespaced per the repo's `src/` convention (a directory per
//! module, not a bare top-level `.rs`) and split along the concerns
//! the original single 764-line file carried:
//!
//! - `mod.rs` (this file) — the run/exit path: [`run`] /
//!   [`run_with_args`], orchestrating everything below.
//! - [`staleness`] — dev-checkout staleness warning.
//! - `check_output` — `wat --check` diagnostic rendering (text/EDN/JSON).
//! - `argv` — cargo-subcommand stripping + the CLI's flag grammar.
//! - `battery` — the [`Battery`] extension-point type + installation.
//! - `proxy` — stdio proxy threads + child reaping.
//! - `signals` — OS signal handlers + the child process-group atomic.
//!
//! # Single invocation shape
//!
//! ```text
//! wat <entry.wat>      # run a program
//! ```
//!
//! Reads an entry `.wat` file, runs the full startup pipeline,
//! installs OS signal handlers (SIGINT + SIGTERM → kernel stop
//! flag), forks and invokes `:user::main`, bridges the real
//! `io::Stdin` / `io::Stdout` / `io::Stderr` to the child via three
//! proxy threads, then reaps the child and exits with its code.
//!
//! There is no `wat test` subcommand — wat tests run via
//! `cargo test` against a Rust crate that uses the `wat::test!`
//! macro to compile the wat source into per-test `#[test] fn`s.
//! The macro composes with cargo's reporting, `--release`,
//! `RUST_BACKTRACE`, and the rest of the cargo testing surface.
//! Arc 101 dropped the duplicate CLI subcommand.
//!
//! # `:user::main` contract
//!
//! Program mode requires an entry point defined as:
//!
//! ```scheme
//! (:wat::core::defn :user::main [] -> :wat::core::nil
//!   ...)
//! ```
//!
//! No parameters; return type `:wat::core::nil`. This is the canonical
//! form (~320 programs in-tree; e.g. `examples/console-demo/wat/main.wat`,
//! `wat-scripts/cosines.wat`). A subprocess whose source is assembled at
//! runtime may instead declare it via `:wat::core::define`. Any other
//! shape (wrong arity, parameter types, or return type) halts startup
//! with a `#wat.kernel.ProcessDiedError/MainSignature` diagnostic on
//! stderr and exit code 4.
//!
//! # Kernel signal state
//!
//! **Terminal signals (SIGINT, SIGTERM)** route to `request_kernel_stop()`
//! — the stop flag is set-once and irreversible. User programs poll
//! `(:wat::kernel::stopped?)` in their loops and cascade shutdown by
//! dropping their root producers.
//!
//! **Non-terminal user signals (SIGUSR1, SIGUSR2, SIGHUP)** each route
//! to their own flag setter. Userland polls `(sigusr1?)` / `(sigusr2?)`
//! / `(sighup?)` and clears via `(reset-sigusr1!)` / `(reset-sigusr2!)`
//! / `(reset-sighup!)`. The kernel measures; userland owns the
//! transitions. Per the 2026-04-19 administrative stance.
//!
//! All handlers are `extern "C" fn` that do a single atomic write and
//! return — no allocation, no I/O.
//!
//! # Exit codes
//!
//! - `0` — `:user::main` returned cleanly.
//! - `2` — runtime error (any [`wat::runtime::RuntimeError`]).
//! - `3` — startup error (parse / type-check / freeze — a
//!   [`wat::freeze::StartupError`], surfaced as
//!   `#wat.kernel/ProcessPanics [#…/ProcessDiedError/StartupError …]`).
//! - `4` — `:user::main` signature mismatch
//!   (`#…/ProcessDiedError/MainSignature`).
//! - `64` — usage error (wrong argv).
//! - `66` — entry file read failed.
//!
//! Startup and signature failures forward the child's structured
//! `#wat.kernel/ProcessPanics [...]` EDN diagnostic to stderr. **Known
//! masking defect (arc 278 no-hidden-failures):** a freeze-time
//! evaluation *panic* in a top-level form (e.g. `Result/expect` on an
//! `Err`) currently exits `1` with NO diagnostic on either stream —
//! surfaced loud under `--check` but swallowed across the fork. Under
//! repair; see `docs/arc/2026/06/278-rules-engine`.
//!
//! # Standard I/O
//!
//! `:user::main` takes no I/O handles. Arc 170 — the program runs in the
//! cli's OWN process (arc 104's fork, and the three proxy threads that
//! bridged the operator's stdio to the child's pipes, are annihilated), so
//! the `StdOut` / `StdErr` / `StdIn` defservices bind the REAL fd 0/1/2.
//! Programs emit via `:wat::kernel::println` (and friends) straight to the
//! terminal — no pipe round-trip, no proxy.

use std::process::ExitCode;

/// The `--repl` ENTRY — a one-form shim, and deliberately nothing more.
///
/// The loop itself is `wat/repl.wat`, a stdlib module exposing `:repl::turn`. Only the entry
/// point lives here, which is where an entry point belongs: a stdlib file that declared
/// `:user::main` would hand one to EVERY wat program and collide with the author's own.
///
/// Splitting it this way makes the REPL a LIBRARY rather than a script — any program can
/// `(:repl::turn defs)` to embed a loop seeded with its own definitions, which is the thing
/// a REPL-as-a-file could never offer.
const REPL_SOURCE: &str =
    "(:wat::core::defn :user::main [] -> :wat::core::nil\n   (:repl::turn (:wat::core::Vector :- [:wat::WatAST])))\n";
const REPL_LABEL: &str = "<repl-entry>";
/// `--mcp` has no entry file either — the loop is Rust (see `mcp.rs`), and this is what any
/// diagnostic reaching a span from that path carries.
const MCP_LABEL: &str = "<mcp-entry>";
use std::sync::Arc;

mod spawned_runtime;
mod argv;
mod mcp;
mod battery;
mod check_output;
mod staleness;

pub use battery::Battery;
pub use argv::strip_cargo_subcommand;

use crate::freeze::startup_from_source;
use crate::load::loader::FsLoader;
use crate::runtime::set_argv;

/// Check that a frozen `--grep` world's `:user::grep` declares the canonical
/// `[] -> (:wat::core::PersistentVector :- [:wat::rete::Rule])` shape. Returns `Err(message)`
/// with a reader-friendly diagnostic naming `:user::grep` — the mirror of
/// `freeze::validate_user_main_signature`'s JOB (a missing-or-wrong-shaped entry point must be
/// refused with a located diagnostic naming what was expected), not its text: `--grep` has a
/// different entry point with a different contract, so it gets its own message rather than a
/// borrowed one that would name the wrong function.
///
/// Lives here (not `freeze.rs`) — this stone's blast radius is `src/distribution/*` only; the
/// wall is Grep's own, parallel to but independent of the main wall it must never reach.
fn validate_user_grep_signature(world: &crate::freeze::FrozenWorld) -> Result<(), String> {
    let func = world.symbols().get(":user::grep").ok_or_else(|| {
        ":user::grep not defined — a --grep program needs an entry point. Arc 278 (the \
         grep-mode stone) — `--grep` dispatches to `:user::grep`, not `:user::main`; the two \
         are different contracts for different modes. The canonical signature is \
         `[] -> (:wat::core::PersistentVector :- [:wat::rete::Rule])`."
            .to_string()
    })?;
    let expected_ret =
        crate::types::parse_type_expr_from_source("(:wat::core::PersistentVector :- [:wat::rete::Rule])")
            .expect("arc 278: the grep-mode canonical return type source parses");
    if !func.param_types.is_empty() {
        return Err(format!(
            ":user::grep must take exactly 0 parameters; got {}. Arc 278 (the grep-mode stone) \
             — `:user::grep` takes no arguments; it returns the rules for `:wat::grep::run` to \
             compile. The canonical signature is \
             `[] -> (:wat::core::PersistentVector :- [:wat::rete::Rule])`.",
            func.param_types.len()
        ));
    }
    if func.ret_type != expected_ret {
        return Err(format!(
            ":user::grep return type expected (:wat::core::PersistentVector :- \
             [:wat::rete::Rule]); got {}. Arc 278 (the grep-mode stone) — `:user::grep` must \
             return the vector of rules `:wat::grep::run` compiles and fires. The canonical \
             signature is `[] -> (:wat::core::PersistentVector :- [:wat::rete::Rule])`.",
            crate::freeze::format_type_expr(&func.ret_type)
        ));
    }
    Ok(())
}

/// argv-injectable variant; `run` = `run_with_args(b, env::args())`.
///
/// Identical to [`run`] but accepts a caller-supplied `argv` instead
/// of reading `std::env::args()`. Used by `cargo-wat` to strip
/// cargo's injected subcommand token before handing off.
pub fn run_with_args(batteries: &[Battery], argv: Vec<String>) -> ExitCode {
    // Arc 259 — prime the boot clock at the earliest wat-controlled point,
    // before install_batteries and argv parsing. The lazy-capture is
    // triggered here so that started-at reflects real boot→entry latency
    // rather than the seam's frame time.
    let _ = crate::time::process_boot_instant();

    // Dev-only staleness guard: if the wat source repo is present relative to
    // pwd (a dev checkout), warn when the installed binary is older than the
    // source. Self-disables (silent) for a plain binary user with no repo.
    // Placed in `run_with_args` — the funnel BOTH `wat` (via `run`) and
    // `cargo-wat` (direct) pass through — so neither entry point misses it.
    staleness::check_dev_staleness();

    // Silence the default panic handler for assertion-failed! payloads.
    // Those panics are expected — the outer sandbox catches them and
    // surfaces structured Failures. Without this hook, every
    // deliberate failure test prints a "thread X panicked" line to
    // stderr before the sandbox intercepts.
    crate::panic_hook::install();

    // This process went through wat's CLI entry, so it can BE a spawned runtime
    // if re-exec'd. A binary that never reaches here (a cargo test harness)
    // cannot, and `exec_plan` falls back to the built `wat` accordingly.
    crate::process::exec_plan::mark_wat_entry();

    battery::install_batteries(batteries);

    // ── Arc 170 step 4 — am I a SPAWNED RUNTIME? ─────────────────────────────
    //
    // A wat parent writes `#wat.boot/Here` onto the lifeline (fd 3) before
    // clone. Presence of the fd is not enough — a harness pipe is also
    // "open." The frame is the witness: no `--forms-server` flag, nothing
    // in `ps`. Reusing the lifeline keeps it one object: the thing that
    // routes you is the thing that proves a parent holds the other end.
    //
    // Note this is ROUTING only. It grants nothing. The boot handshake on
    // fd 0 is the real program gate.
    if spawned_runtime::was_spawned() {
        return spawned_runtime::serve();
    }

    let prog = argv.first().map(String::as_str).unwrap_or("wat");

    // Arc 115 — `--check` flag: load + parse + type-check + freeze
    // without invoking :user::main. Cargo-check ergonomics for wat.
    // Optional `--check-output edn` / `--check-output json` selects
    // structured (machine-readable) diagnostic output for editor /
    // agent / orchestrator tooling. Default (no --check-output): the
    // standard text Display via stderr (same shape `wat <file>` shows
    // on freeze failure).
    let mode = match argv::parse(&argv, prog) {
        Ok(m) => m,
        Err(code) => return code,
    };
    let (entry_path, check_output_format, check_only) = match &mode {
        argv::Mode::Check { entry_path, output_format } => {
            (entry_path.as_str(), *output_format, true)
        }
        // The REPL's source is BAKED (see REPL_SOURCE); this label is what its spans carry,
        // so a diagnostic from inside the loop names a real repo path rather than a
        // `<repl>` sentinel the reader cannot open. Relative, hence reproducible.
        argv::Mode::Repl => (REPL_LABEL, None, false),
        // MCP never reaches the source-reading path below — it drives its own loop and has
        // no entry file. The label is what its diagnostics carry.
        argv::Mode::Mcp => (MCP_LABEL, None, false),
        argv::Mode::Run { entry_path } => (entry_path.as_str(), None, false),
        // Grep behaves exactly like Run here: it has a real entry file that must be read and
        // frozen. It diverges only after the freeze (see the Grep dispatch arm below).
        argv::Mode::Grep { entry_path } => (entry_path.as_str(), None, false),
    };
    let is_repl = matches!(mode, argv::Mode::Repl);
    let is_grep = matches!(mode, argv::Mode::Grep { .. });

    // Arc 170 slice 1e (REALIZATIONS pass 7) — populate the process-wide argv
    // ambient, which `(:wat::runtime::argv)` reads from any depth in the wat
    // program. The WHOLE argv goes in: argv[0] = the wat binary, argv[1] = the
    // entry file, argv[2..] = whatever else the caller said. Nothing is
    // stripped — the parser's job was to FIND the entry, not to edit the
    // program's arguments.
    //
    // argv[0] is the RESOLVED binary path, not the shell's spelling of it. What
    // the shell hands us is whatever the caller typed — `./target/release/wat`,
    // a bare `wat` found on PATH, a symlink — and a program that wants to know
    // where its interpreter lives cannot use any of those without knowing the
    // cwd and the PATH search that produced them. `current_exe()` answers the
    // question directly; argv[0] is kept only as the fallback for the platforms
    // where it can fail, and as the `prog` string in usage messages (where the
    // caller's own spelling is the friendlier thing to echo back).
    let mut ambient_argv = argv.clone();
    if let Ok(exe) = std::env::current_exe() {
        if let Some(first) = ambient_argv.first_mut() {
            *first = exe.display().to_string();
        }
    }
    set_argv(ambient_argv);

    // MCP short-circuits here: it reads JSON-RPC frames and drives the turn itself, so it
    // never wants the entry-file read, the `:user::main` invocation, or the signal wiring
    // below. argv is set first so a form evaluated in a session still sees the ambient.
    if matches!(mode, argv::Mode::Mcp) {
        return mcp::serve();
    }

    // Read entry file. Diagnostics go straight to the real fd 2.
    //
    // The REPL reads nothing: its program is compiled INTO the binary, so `wat --repl` works
    // from any directory and from an installed binary with no repo on disk. Resolving
    // REPL_LABEL against the filesystem instead would make the mode depend on where it was
    // launched from — a shipped feature must not need its own source tree present.
    let source = if is_repl {
        REPL_SOURCE.to_string()
    } else {
        match std::fs::read_to_string(entry_path) {
            Ok(s) => s,
            Err(e) => {
                eprintln!("wat: read {}: {}", entry_path, e);
                return ExitCode::from(66); // EX_NOINPUT
            }
        }
    };
    let canonical = if is_repl {
        Some(REPL_LABEL.to_string())
    } else {
        std::fs::canonicalize(entry_path)
            .ok()
            .map(|p| p.display().to_string())
    };

    // Arc 115 slice 1 — `--check` short-circuit. Run startup_from_source
    // (parse + type-check + freeze); exit 0 on success, non-zero with a
    // diagnostic on freeze failure. No `:user::main`, no signal handlers —
    // side-effect-free verification suitable for editor save hooks and agent
    // sweep loops.
    if check_only {
        let loader: Arc<dyn crate::load::loader::SourceLoader> = Arc::new(FsLoader);
        match startup_from_source(&source, canonical.as_deref(), loader) {
            Ok(_world) => {
                // Successful freeze. The world is dropped without invocation.
                return ExitCode::from(0);
            }
            Err(e) => {
                check_output::emit_check_failure(entry_path, &e, check_output_format);
                return ExitCode::from(1);
            }
        }
    }

    // Arc 170 — the cli runs the entry program IN ITS OWN PROCESS.
    //
    // Arc 104 forked it so user code would never run in the cli's own process
    // ("wat-cli has been the ONE place where the surface metaphor breaks").
    // That reason expired: at this point the cli has done six things — panic
    // hook, batteries, argv parse, argv ambient, file read, and this — so there
    // is no accumulated state to protect a program from. Forking here recreated
    // the fresh unstarted runtime the shell had just exec'd for us, at the cost
    // of a pipe round-trip, three proxy threads, and a second fork path in a
    // substrate that wants exactly one.
    //
    // What went with it: fork_program_from_source, child_branch_from_source,
    // redirect_stdio_and_init, distribution::proxy (3 threads + wait_child),
    // ForkedProgramHandles, and the CHILD_PGID killpg cascade — whose stated
    // job (reaching grandchildren) was already fictional: every spawn_lifelined
    // child calls setpgid(0,0) (clone.rs), so a grandchild is in its OWN group,
    // not the child's, and the verb the comment named (fork-program) was retired
    // in 594572fc. Parent-death detection is the lifeline pipe's job and is
    // untouched.
    //
    // Signals: the cli's old handlers were the substrate's plus a killpg
    // forward. With nothing to forward to, the substrate handlers ARE the
    // contract — they flip KERNEL_STOPPED and write the shutdown wake-pipe, so
    // `(:wat::kernel::stopped?)` polling behaves exactly as it did in the child.
    crate::runtime::init_shutdown_signal();
    crate::process::install_substrate_signal_handlers();

    // rune:exigere(attested-arc) — TEMPORARY STOPGAP, tracked in arc 261
    // (docs/arc/2026/06/261-eval-stack-safety-cek/STUB.md). The eval loop recurses on
    // the NATIVE stack; deep non-tail recursion (e.g. a fix-wat codemod over a large
    // source file) overflows the default 8MB RLIMIT_STACK and SIGSEGVs the process.
    // Raising the soft limit lets the main stack grow on demand, so the self-hosted
    // migration runner works on the whole corpus today. This only RAISES the ceiling;
    // it does NOT remove the class. The structural cure is CEK (arc 261), which has no
    // native eval recursion. WHEN ARC 261 LANDS, DELETE THIS BLOCK. Until then this
    // rune is the standing reminder: we have a recursion-depth ceiling, papered over,
    // on purpose, visibly.
    unsafe {
        let mut rl = std::mem::zeroed::<libc::rlimit>();
        if libc::getrlimit(libc::RLIMIT_STACK, &mut rl) == 0 {
            rl.rlim_cur = (1024u64 * 1024 * 1024).min(rl.rlim_max); // 1 GiB or hard cap
            let _ = libc::setrlimit(libc::RLIMIT_STACK, &rl);
        }
    }

    // Freeze. `startup_from_source` also imposes the `:user::main` wall
    // (freeze.rs — validate_user_main_signature + _not_useless), so a bad main
    // arrives here as StartupError::MainSignature and keeps its own exit code
    // rather than being folded into the generic startup failure.
    // Freeze, under a panic boundary. A PANIC during freeze-time evaluation of a
    // top-level form would otherwise unwind straight out of `run_with_args` and
    // hit Rust's default handler — a MUTE exit 101. That asymmetry (a RETURNED
    // StartupError is loud at 3; a freeze-time panic silently 101) IS the defect
    // arc 278's no-hidden-failures cut B closed, and `freeze_time_panic_surfaces
    // _structured_not_silent` guards it. Mirrors the arm the forked child ran.
    //
    // Phase-honest exit code: a freeze-time failure IS a STARTUP failure →
    // EXIT_STARTUP_ERROR (3), never EXIT_PANIC (2), which would mislabel it as a
    // runtime panic. The world does not exist yet, so the emitters take None.
    //
    // AssertUnwindSafe: nothing captured here is observed after the unwind — the
    // Err arm reads only the payload and returns.
    let loader: Arc<dyn crate::load::loader::SourceLoader> = Arc::new(FsLoader);
    let startup_result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        startup_from_source(&source, canonical.as_deref(), loader)
    }));
    let world = match startup_result {
        Ok(Ok(w)) => w,
        Ok(Err(e)) => {
            let code = match e {
                crate::freeze::StartupError::MainSignature(_) => {
                    crate::process::EXIT_MAIN_SIGNATURE
                }
                _ => crate::process::EXIT_STARTUP_ERROR,
            };
            crate::process::emit_startup_error_structured_exit(&e);
            return ExitCode::from(code as u8);
        }
        Err(panic_payload) => {
            // Downcast AssertionPayload to preserve the rich
            // #wat.kernel/AssertionFailure diagnostic; else String/&str → a
            // message-only Panic.
            if let Some(payload) =
                panic_payload.downcast_ref::<crate::assertion::AssertionPayload>()
            {
                crate::process::emit_structured_exit(
                    None,
                    crate::process::died::process_died_error_panic_value(
                        payload.message.clone(),
                        Some(payload.clone()),
                    ),
                );
            } else {
                let msg = if let Some(s) = panic_payload.downcast_ref::<String>() {
                    s.clone()
                } else if let Some(s) = panic_payload.downcast_ref::<&str>() {
                    (*s).to_string()
                } else {
                    "<unknown panic payload>".to_string()
                };
                crate::process::emit_structured_exit(
                    None,
                    crate::process::died::process_died_error_panic_value(msg, None),
                );
            }
            return ExitCode::from(crate::process::EXIT_STARTUP_ERROR as u8);
        }
    };

    // ── Grep dispatch — diverges HERE, before the `:user::main` wall below ─────────────────
    //
    // DESIGN: docs/arc/2026/06/278-rules-engine/DESIGN-STONE-the-grep-mode.md
    // BRIEF:  docs/arc/2026/06/278-rules-engine/BRIEF-STONE-the-grep-mode.md
    //
    // A `--grep` program has no `:user::main`. Routed through the wall at `:443` below it
    // would be refused for lacking a function it is not supposed to have — so Grep gets its
    // own arm with its own wall, on `:user::grep`, and returns from this function before that
    // wall is ever reached.
    if is_grep {
        if let Err(msg) = validate_user_grep_signature(&world) {
            crate::process::emit_structured_exit(
                Some(&world),
                crate::process::died::process_died_error_runtime_value(&crate::edn::contract::FlatMessage {
                    tag: "GrepSignatureError",
                    key: "message",
                    message: &msg,
                }),
            );
            return ExitCode::from(crate::process::EXIT_RUNTIME_ERROR as u8);
        }

        // `invoke_user_main` (freeze.rs) hardcodes `:user::main` and is NOT reusable here.
        // Its own steps 1-4 — spawning the three stdio services and installing ThreadIO on
        // this thread, so the driver's `(:wat::kernel::println …)` / `(readln)` calls land
        // somewhere — carry no such assumption and ARE reusable directly:
        // `bootstrap_wat_vm_process` is `pub fn` for exactly this.
        let runtime = match crate::freeze::bootstrap_wat_vm_process(crate::freeze::BootstrapArgs {
            frozen: &world,
        }) {
            Ok(rt) => rt,
            Err(e) => {
                crate::process::emit_structured_exit(
                    Some(&world),
                    crate::process::died::process_died_error_runtime_value(&e),
                );
                return ExitCode::from(crate::process::EXIT_RUNTIME_ERROR as u8);
            }
        };

        // Both lookups are known to succeed here: `:user::grep` by the wall just above,
        // `:wat::grep::run` because it is stdlib (`wat/grep.wat`), always present.
        let grep_fn = runtime
            .symbols()
            .get(":user::grep")
            .expect("validate_user_grep_signature already confirmed :user::grep exists")
            .clone();
        let run_fn = runtime
            .symbols()
            .get(":wat::grep::run")
            .expect(":wat::grep::run is stdlib and always present")
            .clone();

        // The shape from the design/brief: `:user::grep` produces the rules,
        // `:wat::grep::run` consumes them — `:wat::grep::run` reads the EDN path vector off
        // stdin itself (`readln`, the codemods' shape); Rust never reads stdin for this mode.
        let outcome = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let rules = crate::runtime::apply_function(
                grep_fn,
                Vec::new(),
                runtime.symbols(),
                crate::rust_caller_span!(),
            )?;
            crate::runtime::apply_function(
                run_fn,
                vec![rules],
                runtime.symbols(),
                crate::rust_caller_span!(),
            )
        }));
        let code = crate::process::finish_in_process(&world, outcome);
        return ExitCode::from(code as u8);
    }

    // Run `:user::main` under a panic boundary and map the outcome to an exit
    // code, emitting the SAME structured EDN on fd 2 that the forked child
    // emitted — `finish_in_process` is `finish_forked_child` with the `_exit`
    // swapped for a return, so every black-box cli assertion holds unchanged.
    //
    // AssertUnwindSafe: the world is consumed only by this call and by the
    // emitters below it, both on this thread; nothing observes it across the
    // unwind boundary in a torn state.
    // The `:user::main` wall, unconditionally. `startup_from_source` imposes it
    // only WHEN `:user::main` is declared (freeze.rs — `if world.symbols()
    // .get(":user::main").is_some()`), because `startup_from_forms` must stay
    // usable by callers that legitimately build worlds without a main. A program
    // that declares NO main therefore freezes clean and must be caught here —
    // exactly as the forked child caught it before this arc. Same emitter, same
    // exit code, byte-identical stderr.
    if let Err(msg) = crate::freeze::validate_user_main_signature(&world) {
        crate::process::emit_structured_exit(
            Some(&world),
            crate::process::died::process_died_error_main_signature_value(&crate::edn::contract::FlatMessage {
                tag: "MainSignatureError",
                key: "message",
                message: &msg,
            }),
        );
        return ExitCode::from(crate::process::EXIT_MAIN_SIGNATURE as u8);
    }

    let outcome = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        crate::freeze::invoke_user_main(&world, Vec::new())
    }));
    let mut code = crate::process::finish_in_process(&world, outcome);

    // Arc 170 "stopping is a protocol" — builder ruling: "any failure must be loud and obvious."
    // MAIN itself (this same thread — `invoke_user_main` → `invoke_user_main_orchestrated`,
    // `src/freeze.rs`) ran the ask-then-await and published any `StopFailure`s BEFORE returning
    // here — see `ProcessRuntime::ask_stop_and_collect_failures` for why it has to be main and not
    // the shutdown worker (`ThreadOwnedCell` ownership). No join needed anymore: the publish, if
    // any, already happened synchronously, on this thread, before `invoke_user_main` returned —
    // unlike the retired worker-thread version, there is no OTHER thread to wait for here.
    if crate::runtime::KERNEL_STOPPED.load(std::sync::atomic::Ordering::SeqCst) {
        let failures = crate::freeze::stop::take_stop_failures();
        if !failures.is_empty() {
            // The one channel this belongs on: stderr is the dying-declaration channel
            // (`emit_panic_envelope`, `src/process/stdio.rs`), written only immediately before a
            // non-zero exit — and this write IS immediately before one. Exit 0 would claim the
            // stop was clean when it was not.
            let stop_failed = crate::freeze::stop::stop_failed_value(failures);
            let edn = crate::edn::render::value_to_edn_with(&stop_failed, Some(world.types()));
            let line = format!("{}\n", wat_edn::write(&edn));
            crate::process::stdio::emit_panic_envelope(&line);
            if code == crate::process::EXIT_SUCCESS {
                code = crate::process::EXIT_RUNTIME_ERROR;
            }
        }
    }

    ExitCode::from(code as u8)
}

/// Run the wat CLI with the supplied batteries.
///
/// Reads `std::env::args()`, runs the supplied entry `.wat` file
/// through the full freeze + invoke pipeline, installs signal
/// handlers, registers every supplied battery's `wat_sources` +
/// Rust dep shims, and returns the matching exit code.
///
/// Both halves of the external-crate contract install via
/// process-global OnceLocks (per `wat::run_program`'s docs);
/// first caller wins, so test harnesses that spin up their own
/// world inherit transparently. Calling `run` more than once in a
/// process is allowed but only the first call's batteries take
/// effect.
///
/// `run` always seeds the `RustDepsBuilder` with
/// [`wat::rust_deps::RustDepsBuilder::with_wat_rs_defaults`] before
/// applying the supplied batteries — substrate-side dispatch shims
/// (the `:wat::*` surfaces wired through `#[wat_dispatch]` inside
/// the substrate crate) are always available without the caller
/// having to spell them out.
///
/// # Example — custom CLI with selected batteries
///
/// ```text
/// fn main() -> std::process::ExitCode {
///     wat::distribution::run(&[
///         (my_crate::register, my_crate::wat_sources),
///         (another_crate::register, another_crate::wat_sources),
///     ])
/// }
/// ```
pub fn run(batteries: &[Battery]) -> ExitCode {
    run_with_args(batteries, std::env::args().collect())
}

// Arc 101 — the `wat test <path>` subcommand was dropped. Wat tests
// run via `cargo test` against a Rust crate that uses the
// `wat::test!` macro to compile the wat source into per-test
// `#[test] fn`s. The macro's runtime arm is `wat::host::test_runner::
// run_and_assert` — same library code the dropped CLI subcommand
// used, but now reachable only through cargo-style harnesses.
