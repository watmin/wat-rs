//! Wat dispatch arms and their helpers.
//!
//! Exit-code constants, spawn-process.

use crate::ast::WatAST;
use crate::config::Config;
use crate::freeze::{
    invoke_user_main, invoke_user_main_with_program,
    startup_from_forms, startup_from_forms_with_inherit,
    startup_from_source, validate_user_main_signature, FrozenWorld,
};
use crate::io::{PipeReader, PipeWriter, WatReader, WatWriter};
use crate::load::{InMemoryLoader, SourceLoader};
use crate::runtime::{
    eval, Environment, ProgramHandleInner, RuntimeError, RuntimeErrorKind,
    SymbolTable, TrackedValue, Value,
};
use crate::value::value::AggregateValue;
use crate::span::Span;

use std::os::fd::{AsRawFd, BorrowedFd, FromRawFd, OwnedFd};
use std::sync::Arc;

use super::child::{child_post_fork_init};
use super::clone::{make_pipe, spawn_lifelined, spawn_lifelined_any};
use super::handle::{ChildHandle, ForkedProgramHandles};

/// Exit-code convention shared between slice 2 (this file — child
/// exits with one of these) and slice 3 (hermetic stdlib define
/// reads the code back and reconstructs a `:wat::kernel::Failure`).
/// Keep in sync with both endpoints; changes require matching slice
/// 3 updates.
pub const EXIT_SUCCESS: i32 = 0;
pub const EXIT_RUNTIME_ERROR: i32 = 1;
pub const EXIT_PANIC: i32 = 2;
pub const EXIT_STARTUP_ERROR: i32 = 3;
pub const EXIT_MAIN_SIGNATURE: i32 = 4;

// ─── Arc 296 — structured StartupError EDN emission ────────────────────────
//
// For startup failures the `Value::Enum` path (used by emit_structured_exit)
// cannot carry a nested `MacroError` tree as a typed field — the field is
// declared as `:wat::core::String` in `types.rs` and `edn_shim.rs` would
// emit a prose string instead of a structured tagged value.
//
// This helper bypasses Value entirely: it builds the wire-format EDN directly
// from `OwnedValue`, matching the `#wat.kernel/ProcessPanics [...]` envelope
// shape that `emit_chain_envelope` produces for other exit paths.

/// Arc 296 — emit the startup-error exit envelope.
///
/// For `StartupError::Macro`, builds the `#wat.kernel/ProcessPanics [...]`
/// line directly from `OwnedValue` so the field carries a fully structured
/// `MacroError` tree (not a prose String). This is the arc 296 target:
/// the leaf cause is machine-navigable tagged EDN.
///
/// For all OTHER `StartupError` variants, falls back to the original
/// `process_died_error_startup_value(format!("{}", e))` path so that
/// `extract-panics` on the WAT side can still reconstruct the value —
/// `types.rs` declares `ProcessDiedError::StartupError` field as `String`,
/// and `edn_to_value` would fail on a tagged payload for those variants.
fn emit_startup_error_structured_exit(e: &crate::freeze::StartupError) {
    match e {
        crate::freeze::StartupError::Macro(_) => {
            // Arc 296: structured EDN cause chain for Macro failures.
            // Route through the ToEdn trait — ONE canonical path.
            let cause_edn = {
                use crate::to_edn::ToEdn;
                e.to_edn()
            };

            // Arc 278 the LociDiedError stone — build
            // #wat.kernel.LociDiedError/StartupError [<cause>] (was ProcessDiedError).
            let startup_err_edn = wat_edn::OwnedValue::Tagged(
                wat_edn::Tag::ns("wat.kernel.LociDiedError", "StartupError"),
                Box::new(wat_edn::OwnedValue::Vector(vec![cause_edn])),
            );

            // The chain vec: [#wat.kernel.LociDiedError/StartupError [...]] — a bare,
            // self-describing Vector<LociDiedError> (the ProcessPanics wrapper is gone).
            let chain_vec = wat_edn::OwnedValue::Vector(vec![startup_err_edn]);

            let line = format!("{}\n", wat_edn::write(&chain_vec));
            crate::process::stdio::emit_panic_envelope(&line);
        }
        _ => {
            // Arc 296 — the StartupError is passed BY VALUE through the
            // ToEdn-generic boundary; the builder serializes it via
            // `to_wire_edn`. The String field stays :wat::core::String so
            // extract-panics can still reconstruct the variant on the parent
            // side; its CONTENT is structured EDN, not prose Display text.
            emit_structured_exit(
                None,
                crate::runtime::process_died_error_startup_value(e),
            );
        }
    }
}

// ─── emit_structured_exit (single copy — all fork/spawn paths share this) ───
//
// Stone 6.w merge: fork.rs + spawn_process.rs copies unified here.
// One declaration; all child branches call this.

/// Encode `chain` (a `Vector<LociDiedError>`) as a BARE self-describing EDN line
/// and write it to stderr via `emit_panic_envelope`. Shared tail of
/// `emit_structured_exit` and `emit_panics_to_stderr` (Stone 6.w L3 dedup).
///
/// Arc 278 the LociDiedError stone — the `#wat.kernel/ProcessPanics` wrapper tag
/// is ANNIHILATED: the chain crosses as a bare `[#wat.kernel.LociDiedError/… …]`
/// vector, read by generic `edn::read` (the head element's own tag is the
/// self-describing marker the stderr scanner / `recv'` Lost decoder key on).
fn emit_chain_envelope(chain: crate::runtime::Value, types: Option<&crate::types::TypeEnv>) {
    let edn = crate::edn_shim::value_to_edn_with(&chain, types);
    let line = format!("{}\n", wat_edn::write(&edn));
    crate::process::stdio::emit_panic_envelope(&line);
}

/// Arc 170 slice 1i — unified structured exit helper for ALL fork child
/// exit paths. Wraps `value` in the `#wat.kernel/ProcessPanics [...]`
/// envelope and writes the EDN line to stderr before the caller
/// calls `libc::_exit`.
///
/// `world` is `None` for pre-world startup failures — those values only
/// carry primitive Strings so TypeEnv-less EDN rendering is sufficient.
///
/// `pub(crate)` (Stone 6.w, circumspicere F3): also called by the `:process`
/// peer apply-loop (`kernel::spawn::spawn_process_peer`) so a dying peer child
/// names its cause on fd 2 in the SAME envelope shape as the fork children,
/// instead of vanishing into a bare `Exited(1)`. One canonical emit; no
/// duplicate envelope encoding.
pub(crate) fn emit_structured_exit(
    world: Option<&crate::freeze::FrozenWorld>,
    value: crate::runtime::Value,
) {
    let chain = crate::runtime::conj_died_chain_value(value, None);
    let types = world.map(|w| w.types());
    emit_chain_envelope(chain, types);
}

// ─── emit_panics_to_stderr ───────────────────────────────────────────────────

/// Arc 113 slice 3 / Stone 6.w merge — emit the cascade chain as a tagged
/// EDN line on stderr just before `_exit`. Stderr is the diagnostic
/// channel by convention; the wat-side sandbox driver scans for the marker
/// and hands the parsed chain to `failure-from-process-died`. Used by ALL
/// fork/spawn child exit paths (spawn-process). libc::write is fork-safe;
/// no atexit handler is involved.
fn emit_panics_to_stderr(
    world: &crate::freeze::FrozenWorld,
    payload: &crate::assertion::AssertionPayload,
) {
    let fresh = crate::runtime::process_died_error_panic_value(
        payload.message.clone(),
        Some(payload.clone()),
    );
    let upstream = payload.upstream_chain.clone();
    let chain = crate::runtime::conj_died_chain_value(fresh, upstream);
    emit_chain_envelope(chain, Some(world.types()));
}

// ─── finish_forked_child — shared exit-protocol tail ────────────────────────

/// Shared exit-protocol tail for all fork/spawn child branches (Stone 6.w
/// solvere L2 dedup). Called after `catch_unwind(invoke_user_main)` returns.
/// Never returns — exits via `libc::_exit` with the appropriate `EXIT_*` code.
///
/// Maps `outcome` to the canonical exit-code + envelope convention:
/// - `Ok(Ok(Unit))` → EXIT_SUCCESS
/// - `Ok(Ok(other))` → structured BadReturn envelope → EXIT_RUNTIME_ERROR
/// - `Ok(Err(runtime_err))` → structured EDN RuntimeError envelope → EXIT_RUNTIME_ERROR
/// - `Err(panic_payload)` → AssertionPayload chain or plain-string Panic → EXIT_PANIC
fn finish_forked_child(
    world: &crate::freeze::FrozenWorld,
    // rune:perspicere(intentional-structure) — outer = catch_unwind panic boundary,
    // inner = eval Result; finish_forked_child matches both arms; a type alias
    // would hide the structure the match needs to see.
    outcome: std::thread::Result<Result<Value, RuntimeError>>,
) -> ! {
    match outcome {
        // Arc 170 slice 1e — `:user::main` returns `:wat::core::nil`;
        // clean nil-return maps to libc::exit(0). REALIZATIONS pass 10
        // — nil IS the success exit code; user code never participates
        // in exit-code arithmetic.
        Ok(Ok(Value::Unit)) => unsafe { libc::_exit(EXIT_SUCCESS) },
        Ok(Ok(other)) => {
            // Arc 296 — structured BadReturn: the type name is a genuinely flat
            // message, carried through the ToEdn-generic boundary as a
            // FlatMessage (the string IS the datum — no structure to lose).
            emit_structured_exit(
                Some(world),
                crate::runtime::process_died_error_bad_return_value(&crate::to_edn::FlatMessage {
                    tag: "BadReturnType",
                    key: "got-type",
                    message: other.type_name(),
                }),
            );
            unsafe { libc::_exit(EXIT_RUNTIME_ERROR) };
        }
        Ok(Err(runtime_err)) => {
            // Arc 233 Stone 233.3 / arc 296 — the RuntimeError crosses the
            // ToEdn-generic boundary BY VALUE; the builder serializes it via
            // `to_wire_edn`. Structured fields flow over the wire as
            // machine-consumable EDN rather than opaque text.
            emit_structured_exit(
                Some(world),
                crate::runtime::process_died_error_runtime_value(&runtime_err),
            );
            unsafe { libc::_exit(EXIT_RUNTIME_ERROR) };
        }
        Err(panic_payload) => {
            // Arc 170 slice 1i — all panic paths emit structured EDN.
            // AssertionPayload carries the full cascade chain + Failure;
            // plain panics (bare String / &str) emit a message-only Panic.
            if let Some(payload) =
                panic_payload.downcast_ref::<crate::assertion::AssertionPayload>()
            {
                emit_panics_to_stderr(world, payload);
            } else {
                let msg = if let Some(s) = panic_payload.downcast_ref::<String>() {
                    s.clone()
                } else if let Some(s) = panic_payload.downcast_ref::<&str>() {
                    (*s).to_string()
                } else {
                    "<unknown panic payload>".to_string()
                };
                emit_structured_exit(
                    Some(world),
                    crate::runtime::process_died_error_panic_value(msg, None),
                );
            }
            unsafe { libc::_exit(EXIT_PANIC) };
        }
    }
}

// ─── Post-fork shared prologue + epilogue ────────────────────────────────────

/// Drop parent-side pipe ends, dup2 child-side pipes onto fd 0/1/2, run
/// `child_post_fork_init`, and build the wat-level stdio Arcs. Called
/// by `run_forked_child` (forms).
///
/// Exits via `libc::_exit(EXIT_STARTUP_ERROR)` if any dup2 fails; emits a
/// minimal raw write to fd 2 before exiting so the parent sees a diagnostic
/// rather than a silent empty stderr (CIRC-F2 — makes child.rs:292
/// "all failures emit structured" true by construction). fd 2 is still the
/// parent's stderr pipe at that point (dup2 has not yet succeeded for fd 2),
/// so the write reaches the parent.
///
/// Returns `(stdin_reader, stdout_writer, stderr_writer)` on success.
/// Registers the lifeline read-fd with the shutdown worker and transfers
/// ownership via `mem::forget(lifeline_r)`.
#[allow(clippy::too_many_arguments)]
fn redirect_stdio_and_init(
    stdin_r_raw: i32,
    stdout_w_raw: i32,
    stderr_w_raw: i32,
    lifeline_r_raw: i32,
    stdin_pair: (OwnedFd, OwnedFd),
    stdout_pair: (OwnedFd, OwnedFd),
    stderr_pair: (OwnedFd, OwnedFd),
    lifeline_r: OwnedFd,
) -> (Arc<dyn WatReader>, Arc<dyn WatWriter>, Arc<dyn WatWriter>) {
    // Drop parent-side pipe ends (close our inherited copies).
    drop(stdin_pair.1); // parent writes
    drop(stdout_pair.0); // parent reads
    drop(stderr_pair.0); // parent reads

    // Redirect stdio onto the child-side pipes. fd 2 is still the parent's
    // stderr pipe here; the write(2,...) calls below can reach the parent.
    unsafe {
        if libc::dup2(stdin_r_raw, 0) < 0 {
            let msg = b"substrate: dup2 failed during child stdio setup (stdin)\n";
            libc::write(2, msg.as_ptr() as *const libc::c_void, msg.len());
            libc::_exit(EXIT_STARTUP_ERROR);
        }
        if libc::dup2(stdout_w_raw, 1) < 0 {
            let msg = b"substrate: dup2 failed during child stdio setup (stdout)\n";
            libc::write(2, msg.as_ptr() as *const libc::c_void, msg.len());
            libc::_exit(EXIT_STARTUP_ERROR);
        }
        if libc::dup2(stderr_w_raw, 2) < 0 {
            // fd 2 is now the process pipe; this write goes there.
            let msg = b"substrate: dup2 failed during child stdio setup (stderr)\n";
            libc::write(2, msg.as_ptr() as *const libc::c_void, msg.len());
            libc::_exit(EXIT_STARTUP_ERROR);
        }
    }
    // Drop the originals — dup2 made copies at 0/1/2.
    drop(stdin_pair.0);
    drop(stdout_pair.1);
    drop(stderr_pair.1);

    // Arc 213 γ-1 — canonical Phase 3 post-fork init (shared between all
    // child branches). Must run AFTER dup2 (fd 2 is now the subprocess
    // stderr pipe) and AFTER dropping parent-side pipe ends.
    //   (1) install_silent_panic_hook — fd 2 now subprocess stderr, safe
    //   (2) setpgid(0, 0) — child becomes own pgrp leader
    //   (3) close_inherited_fds_above_stdio(&[lifeline_r_raw]) — skip lifeline
    //   (4) init_shutdown_signal_with_inputs — registers lifeline read-end
    //   (5) install signal handlers — SIGTERM/SIGINT route through wake-pipe
    child_post_fork_init(lifeline_r_raw);

    // Transfer FD ownership to the shutdown worker thread — the substrate
    // now owns the lifeline read-fd. Dropping OwnedFd here would close the
    // FD and the worker would immediately POLLHUP (false-positive shutdown).
    std::mem::forget(lifeline_r);

    // Build wat-level stdio over fd 0/1/2.
    let stdin_reader: Arc<dyn WatReader> =
        Arc::new(PipeReader::from_owned_fd(unsafe { OwnedFd::from_raw_fd(0) }));
    let stdout_writer: Arc<dyn WatWriter> =
        Arc::new(PipeWriter::from_owned_fd(unsafe { OwnedFd::from_raw_fd(1) }));
    let stderr_writer: Arc<dyn WatWriter> =
        Arc::new(PipeWriter::from_owned_fd(unsafe { OwnedFd::from_raw_fd(2) }));
    (stdin_reader, stdout_writer, stderr_writer)
}

/// Run `:user::main` inside a `catch_unwind` and call `finish_forked_child`.
/// Shared epilogue for all child branches. Holds the stdio keepalives alive
/// across the catch_unwind (OwnedFd-keepalive discipline, arc 113 slice 3 /
/// arc 170 slice 1e). Never returns — delegates to `finish_forked_child` which
/// calls `libc::_exit`.
fn run_user_main_in_child(
    world: &FrozenWorld,
    stdin_reader: Arc<dyn WatReader>,
    stdout_writer: Arc<dyn WatWriter>,
    stderr_writer: Arc<dyn WatWriter>,
    env_fn: Option<String>,
) -> ! {
    // Arc 113 slice 3 — keep stderr's wat-side IOWriter Arc alive past
    // the catch_unwind closure (Arc 170 slice 1e dropped the main_args
    // plumbing but the OwnedFd-keepalive concern survives — the writer
    // Arc is still held in this scope and its OwnedFd over fd 2 must
    // outlive any post-catch writes).
    let stderr_keepalive = Arc::clone(&stderr_writer);
    let _ = &stdin_reader;  // OwnedFd keepalive — slice 1f services own this
    let _ = &stdout_writer; // OwnedFd keepalive — slice 1f services own this

    // Arc 170 slice 1e — `:user::main` is `[] -> :wat::core::nil`
    // (REALIZATIONS pass 7 + pass 10). No stdio Values; argv is ambient.
    //
    // Arc 209 C0b.3b-e — env-fn dispatch: when env_fn is Some(src), the child evals
    // the source string in its own frozen world (eval_in_frozen) to produce user.program.
    // Dispatch: a 0-arg fn → apply it; a :wat::core::Record → use directly; else clean child death.
    // None → invoke_user_main (the CLI/non-spawn callers; unchanged behavior).
    // Errors in the Some arm are threaded into the catch_unwind Result → finish_forked_child.
    let outcome = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        match env_fn {
            None => invoke_user_main(world, Vec::new()),
            Some(src) => {
                let user_program = crate::freeze::resolve_env_program(world, &src)?;
                invoke_user_main_with_program(world, Vec::new(), user_program)
            }
        }
    }));
    let _ = &stderr_keepalive; // borrow-check: prove the clone is held until here
    finish_forked_child(world, outcome)
}

/// Shared post-fork kernel for forms-based child branches (arc 214 Stone 6.w
/// dedup). Used by `eval_kernel_spawn_process`.
///
/// Never returns — exits via `libc::_exit` with one of the `EXIT_*` codes.
/// Takes ownership of all six OwnedFds so Rust's Drop semantics close the
/// child's copies cleanly after dup2.
///
/// Ten parameters is the honest shape: six fds (three raw for
/// dup2, three OwnedFd pairs whose Drop closes the parent-side
/// ends the child inherited), plus the forms to evaluate, the
/// optionally-inherited config, the lifeline raw fd (Arc 213 γ-1),
/// and the lifeline OwnedFd wrapper (transferred to the shutdown
/// worker via mem::forget after child_post_fork_init).
#[allow(clippy::too_many_arguments)]
fn run_forked_child(
    forms: Vec<WatAST>,
    inherit_config: Option<Config>,
    stdin_r_raw: i32,
    stdout_w_raw: i32,
    stderr_w_raw: i32,
    lifeline_r_raw: i32,
    stdin_pair: (OwnedFd, OwnedFd),
    stdout_pair: (OwnedFd, OwnedFd),
    stderr_pair: (OwnedFd, OwnedFd),
    lifeline_r: OwnedFd,
) -> ! {
    // Shared prologue: drop parent-side ends, dup2, child_post_fork_init,
    // mem::forget(lifeline_r), build stdio Arcs.
    let (stdin_reader, stdout_writer, stderr_writer) = redirect_stdio_and_init(
        stdin_r_raw, stdout_w_raw, stderr_w_raw, lifeline_r_raw,
        stdin_pair, stdout_pair, stderr_pair, lifeline_r,
    );

    // Fresh world from the inherited AST. InMemoryLoader (no disk)
    // matches the `scope :None` behavior today's hermetic provides.
    // rune:exigere(attested-arc) — Scope-through-fork tracked in arc 012
    // (INSCRIPTION at docs/arc/2026/04/012-fork-and-pipes/INSCRIPTION.md).
    let loader = Arc::new(InMemoryLoader::new());

    // Arc 031: inherit the caller's Config through fork's COW so the
    // child's sandboxed forms can omit `(:wat::config::set-*!)`. When
    // no inherit is available (caller had no encoding context), fall
    // back to the non-inheriting path — forms must carry their own
    // required setters.
    let startup_result = match &inherit_config {
        Some(cfg) => startup_from_forms_with_inherit(forms, None, loader, cfg),
        None => startup_from_forms(forms, None, loader),
    };
    let world = match startup_result {
        Ok(w) => w,
        Err(e) => {
            // Arc 296: emit structured EDN cause chain (not prose string).
            emit_startup_error_structured_exit(&e);
            unsafe { libc::_exit(EXIT_STARTUP_ERROR) };
        }
    };

    if let Err(msg) = validate_user_main_signature(&world) {
        emit_structured_exit(
            Some(&world),
            crate::runtime::process_died_error_main_signature_value(&crate::to_edn::FlatMessage {
                tag: "MainSignatureError",
                key: "message",
                message: &msg,
            }),
        );
        unsafe { libc::_exit(EXIT_MAIN_SIGNATURE) };
    }

    // Shared epilogue: catch_unwind + finish_forked_child.
    run_user_main_in_child(&world, stdin_reader, stdout_writer, stderr_writer, None)
}

// ─── Post-dup2 server runtime for spawn-program' :process (arc 214 β) ───────
//
// Called AFTER the child branch has already dup2'd fd 0/1/2 and called
// child_post_fork_init (non-preserving — the io_uring comms fds are swept;
// the forms-server child reads fd 0 / writes fd 1 directly via PipeReader /
// PipeWriter). This is the tail of run_forked_child lifted into its own fn so
// kernel/spawn.rs can call it without going through the full 6-fd redirect_stdio_and_init
// setup (which would double-dup2 fds already wired by spawn_process_peer's child branch).
//
// Never returns — exits via run_user_main_in_child → finish_forked_child → libc::_exit.
pub(crate) fn run_forms_as_server_child(
    forms: Vec<WatAST>,
    inherit_config: Option<Config>,
    env_fn: String,
) -> ! {
    // Arc 209 C0b.3a-0 — hand the forms-child its owner-link as a self-peer
    // (rx=fd0, tx=fd1). dup BEFORE the PipeReader/PipeWriter take ownership of
    // fd 0/1 — the dup'd OwnedFds are independent (EOF on the dup'd read_fd
    // still fires when the owner drops its write end of the input pipe, because
    // both ends of the same pipe share the same kernel file-description; dup
    // creates a new fd-descriptor pointing at the same file-description).
    // SAFETY: fd 0 and fd 1 are live at this point (dup2'd by spawn_process_peer
    // before calling this function). BorrowedFd::borrow_raw is safe for
    // immediately-called try_clone_to_owned (dup(2)) — we do not hold the
    // BorrowedFd across any async boundary.
    let self_peer_read_fd: OwnedFd = unsafe { BorrowedFd::borrow_raw(0) }
        .try_clone_to_owned()
        .expect("dup fd0 for self-peer");
    let self_peer_write_fd: OwnedFd = unsafe { BorrowedFd::borrow_raw(1) }
        .try_clone_to_owned()
        .expect("dup fd1 for self-peer");
    // Arc 258.5b-ii: reinterpret Sender<Value> as Sender<String> — eval pre-encodes with
    // sym.types() and ships the wire String via Peer::send_wire. Receiver<Value> stays.
    let (self_peer_tx, self_peer_rx) =
        crate::comms::process::sender_receiver_from_split_fds::<crate::runtime::Value>(
            self_peer_read_fd, self_peer_write_fd,
        )
        .expect("build self-peer Sender/Receiver from fd0/fd1");
    let self_peer_value = crate::rust_deps::marshal::make_rust_opaque(
        crate::kernel::spawn::PEER_TYPE_PATH,
        std::sync::Arc::new(crate::rust_deps::custodia::ThreadOwnedCell::new(Some(
            crate::kernel::peer::Peer::from_socket(self_peer_tx.reinterpret::<String>(), self_peer_rx),
        ))),
    );
    let _self_peer_guard = crate::services::install_self_peer(self_peer_value);

    // Build wat-level stdio over the already-dup2'd fd 0/1/2.
    // SAFETY: The child branch in spawn_process_peer has already dup2'd
    // the comms pipe ends onto fd 0/1/2 and called child_post_fork_init
    // (which swept everything > 2 except the lifeline, installed the
    // silent panic hook, and rebuilt the shutdown infra). These fds are
    // live and exclusively owned by this child process.
    let stdin_reader: Arc<dyn WatReader> =
        Arc::new(PipeReader::from_owned_fd(unsafe { OwnedFd::from_raw_fd(0) }));
    let stdout_writer: Arc<dyn WatWriter> =
        Arc::new(PipeWriter::from_owned_fd(unsafe { OwnedFd::from_raw_fd(1) }));
    let stderr_writer: Arc<dyn WatWriter> =
        Arc::new(PipeWriter::from_owned_fd(unsafe { OwnedFd::from_raw_fd(2) }));

    let loader = Arc::new(InMemoryLoader::new());
    let startup_result = match &inherit_config {
        Some(cfg) => startup_from_forms_with_inherit(forms, None, loader, cfg),
        None => startup_from_forms(forms, None, loader),
    };
    let world = match startup_result {
        Ok(w) => w,
        Err(e) => {
            // Arc 296: emit structured EDN cause chain (not prose string).
            emit_startup_error_structured_exit(&e);
            unsafe { libc::_exit(EXIT_STARTUP_ERROR) };
        }
    };

    // run_user_main_in_child never returns.
    run_user_main_in_child(&world, stdin_reader, stdout_writer, stderr_writer, Some(env_fn))
}

// ─── fork_program_from_source (wat-cli entry point) ─────────────────────────
//
// This function is the wat-cli's main program execution path — runs a .wat
// source file as a forked OS process. It is NOT a WAT verb (no dispatch arm,
// no type registration). The WAT verbs fork-program and fork-program-ast have
// been retired (arc 214 1b-ii-ζ.1); this Rust function remains as the
// system-level run path for `wat some-file.wat`.

/// Forks an OS process that parses, freezes, and runs the given wat source
/// string. Returns `ForkedProgramHandles` on success (parent holds the pipe
/// ends and child handle).
///
/// Used exclusively by `wat-cli` (the binary entry point for `wat some-file.wat`).
/// NOT exposed as a WAT verb — the retired `:wat::kernel::fork-program` verb
/// has no dispatch arm or type registration. This Rust function is the direct
/// fork-from-source substrate for the CLI.
pub fn fork_program_from_source(
    source: &str,
    canonical: Option<&str>,
    loader: Arc<dyn SourceLoader>,
    argv: Vec<String>,
) -> Result<ForkedProgramHandles, RuntimeError> {
    const OP: &str = ":wat::process::fork-from-source";

    let (stdin_r, stdin_w) = make_pipe(OP)?;
    let (stdout_r, stdout_w) = make_pipe(OP)?;
    let (stderr_r, stderr_w) = make_pipe(OP)?;

    let stdin_r_raw  = stdin_r.as_raw_fd();
    let stdin_w_raw  = stdin_w.as_raw_fd();
    let stdout_r_raw = stdout_r.as_raw_fd();
    let stdout_w_raw = stdout_w.as_raw_fd();
    let stderr_r_raw = stderr_r.as_raw_fd();
    let stderr_w_raw = stderr_w.as_raw_fd();

    let owned_source = source.to_string();
    let owned_canonical = canonical.map(|s| s.to_string());

    // Use spawn_lifelined_any (no UnwindSafe bound) because
    // Arc<dyn SourceLoader> doesn't satisfy UnwindSafe. The child calls
    // _exit on every code path so the contract is satisfied.
    let (pidfd, lifeline_writer) = spawn_lifelined_any(move |lifeline_r_raw: i32| {
        let stdin_r  = unsafe { OwnedFd::from_raw_fd(stdin_r_raw) };
        let stdin_w  = unsafe { OwnedFd::from_raw_fd(stdin_w_raw) };
        let stdout_r = unsafe { OwnedFd::from_raw_fd(stdout_r_raw) };
        let stdout_w = unsafe { OwnedFd::from_raw_fd(stdout_w_raw) };
        let stderr_r = unsafe { OwnedFd::from_raw_fd(stderr_r_raw) };
        let stderr_w = unsafe { OwnedFd::from_raw_fd(stderr_w_raw) };
        let lifeline_r = unsafe { OwnedFd::from_raw_fd(lifeline_r_raw) };

        child_branch_from_source(
            owned_source,
            owned_canonical,
            loader,
            argv,
            stdin_r_raw,
            stdout_w_raw,
            stderr_w_raw,
            lifeline_r_raw,
            (stdin_r, stdin_w),
            (stdout_r, stdout_w),
            (stderr_r, stderr_w),
            lifeline_r,
        );
    })
    .map_err(|err| RuntimeError { span: crate::rust_caller_span!(), kind: RuntimeErrorKind::MalformedForm {
        head: OP.into(),
        reason: format!("spawn_lifelined_any: {}", err)
    } })?;

    drop(stdin_r);
    drop(stdout_w);
    drop(stderr_w);

    let lifeline_w = lifeline_writer.into_owned_fd();

    Ok(ForkedProgramHandles {
        child_handle: Arc::new(ChildHandle::new(pidfd, Some(lifeline_w))),
        stdin_w,
        stdout_r,
        stderr_r,
    })
}

/// Child's post-fork pipeline for source-string entry (used by
/// `fork_program_from_source`). Mirrors `run_forked_child` (forms entry)
/// but parses + freezes from a String instead of an inherited Vec<WatAST>.
///
/// Never returns — exits via `libc::_exit` with one of the `EXIT_*` codes.
#[allow(clippy::too_many_arguments)]
fn child_branch_from_source(
    source: String,
    canonical: Option<String>,
    loader: Arc<dyn SourceLoader>,
    argv: Vec<String>,
    stdin_r_raw: i32,
    stdout_w_raw: i32,
    stderr_w_raw: i32,
    lifeline_r_raw: i32,
    stdin_pair: (OwnedFd, OwnedFd),
    stdout_pair: (OwnedFd, OwnedFd),
    stderr_pair: (OwnedFd, OwnedFd),
    lifeline_r: OwnedFd,
) -> ! {
    let (stdin_reader, stdout_writer, stderr_writer) = redirect_stdio_and_init(
        stdin_r_raw, stdout_w_raw, stderr_w_raw, lifeline_r_raw,
        stdin_pair, stdout_pair, stderr_pair, lifeline_r,
    );

    // Arc 278 no-hidden-failures — wrap the freeze call in catch_unwind. A
    // PANIC during freeze-time evaluation of a top-level form (e.g. a top-level
    // `let` whose initializer Result/expect's on an eval-ast! Err) would
    // otherwise unwind past this Result match and be caught ONLY by the child's
    // OUTER catch (src/process/clone.rs:429 → _exit(1), payload discarded) with
    // the silent panic hook eating the default text — a MUTE exit 1. That
    // asymmetry (a RETURNED StartupError is loud at exit 3; a RUNTIME panic in
    // :user::main is loud at exit 2 via run_user_main_in_child's own inner
    // catch; the freeze call had no catch) IS the defect.
    //
    // AssertUnwindSafe: identical soundness rationale to the outer catch
    // (clone.rs:418-422) — the child calls libc::_exit on every path here, so
    // no panic-unwinding can observe aliased state after _exit terminates the
    // process. The outer catch stays as the last-resort backstop; this inner
    // catch fires first.
    let startup_result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        startup_from_source(&source, canonical.as_deref(), loader)
    }));
    let world = match startup_result {
        Ok(Ok(w)) => w,
        Ok(Err(e)) => {
            // Arc 296: emit structured EDN cause chain (not prose string).
            emit_startup_error_structured_exit(&e);
            unsafe { libc::_exit(EXIT_STARTUP_ERROR) };
        }
        Err(panic_payload) => {
            // Freeze-time panic — mirror finish_forked_child's Err arm
            // (verbs.rs:205-227): downcast AssertionPayload → preserve the rich
            // #wat.kernel/AssertionFailure diagnostic; else String/&str →
            // message-only Panic. The world does not exist yet at freeze time,
            // so emit_structured_exit takes None (pre-world path).
            //
            // Phase-honest exit code: a freeze-time failure IS a STARTUP
            // failure → EXIT_STARTUP_ERROR (3), NOT EXIT_PANIC (2). Exit 2
            // would mislabel a startup failure as a runtime panic.
            if let Some(payload) =
                panic_payload.downcast_ref::<crate::assertion::AssertionPayload>()
            {
                emit_structured_exit(
                    None,
                    crate::runtime::process_died_error_panic_value(
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
                emit_structured_exit(
                    None,
                    crate::runtime::process_died_error_panic_value(msg, None),
                );
            }
            unsafe { libc::_exit(EXIT_STARTUP_ERROR) };
        }
    };

    if let Err(msg) = validate_user_main_signature(&world) {
        emit_structured_exit(
            Some(&world),
            crate::runtime::process_died_error_main_signature_value(&crate::to_edn::FlatMessage {
                tag: "MainSignatureError",
                key: "message",
                message: &msg,
            }),
        );
        unsafe { libc::_exit(EXIT_MAIN_SIGNATURE) };
    }

    // Arc 170 slice 1e (REALIZATIONS pass 7) — argv is ambient.
    crate::runtime::set_argv(argv);

    run_user_main_in_child(&world, stdin_reader, stdout_writer, stderr_writer, None)
}

// ─── spawn-process (from spawn_process.rs) ───────────────────────────────────
//
// NAME-LIE NOTE: "spawn" is the wat verb name; the implementation uses
// `clone3+CLONE_PIDFD+CLONE_CLEAR_SIGHAND` (Linux 5.3+), never `fork(2)`.

/// Wat-level dispatch arm for `:wat::kernel::spawn-process`.
///
/// Arity 1 — the `program` arg evaluating to `:wat::core::Vector<wat::WatAST>`
/// (top-level forms of a wat program, ending in `(:wat::core::define
/// (:user::main -> :nil) ...)`). Returns `:wat::kernel::Process<I,O>`.
pub fn eval_kernel_spawn_process(
    args: &[WatAST],
    list_span: &Span,
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, RuntimeError> {
    const OP: &str = ":wat::kernel::spawn-process";
    if args.len() != 1 {
        return Err(RuntimeError { span: list_span.clone(), kind: RuntimeErrorKind::ArityMismatch {
            op: OP.into(),
            expected: 1,
            got: args.len()
        } });
    }

    // Slice 6 — evaluate the program arg to Vec<WatAST>. Macros
    // construct the program shape internally; user-facing surface
    // remains body-only.
    let forms = expect_vec_ast(OP, eval(&args[0], env, sym)?, args[0].span().clone())?;

    // Snapshot caller's Config before fork so the child can inherit it
    // through COW (arc 031 discipline). None when sym has no encoding
    // context (test harnesses that built a SymbolTable directly). When
    // present, the child's `startup_from_forms_with_inherit` pre-seeds
    // every config field, so program forms can OMIT setters and still
    // freeze; when None, the program forms must carry their own setters
    // (this is the "wat program" entry-file discipline).
    let inherit_config: Option<Config> = sym.encoding_ctx().map(|ctx| ctx.config.clone());

    // Three pipes — stdin (parent→child), stdout (child→parent),
    // stderr (child→parent). The IPC contract mirrors `wat some-file.wat`.
    let (stdin_r, stdin_w) = make_pipe(OP).map_err(|mut e| { e.span = list_span.clone(); e })?;
    let (stdout_r, stdout_w) = make_pipe(OP).map_err(|mut e| { e.span = list_span.clone(); e })?;
    let (stderr_r, stderr_w) = make_pipe(OP).map_err(|mut e| { e.span = list_span.clone(); e })?;

    // RAII: parent holds all six OwnedFds. Pass raw i32 COPIES (as_raw_fd —
    // borrows, does not surrender ownership) into the clone3 closure so the
    // compiler does not enforce single-ownership on the OwnedFd wrappers.
    // After clone3 the child process has its own fd-table copy; the parent's
    // OwnedFds remain alive, keeping the fds open through spawn_lifelined.
    // On any early-return (the ? below) or panic, all six OwnedFds Drop and
    // close — no fd leak. No into_raw_fd() — OwnedFd::Drop is never disabled.
    let stdin_r_raw  = stdin_r.as_raw_fd();
    let stdin_w_raw  = stdin_w.as_raw_fd();
    let stdout_r_raw = stdout_r.as_raw_fd();
    let stdout_w_raw = stdout_w.as_raw_fd();
    let stderr_r_raw = stderr_r.as_raw_fd();
    let stderr_w_raw = stderr_w.as_raw_fd();

    // Arc 213 γ-3 — use spawn_lifelined (arc 213 α) instead of bare
    // libc::fork(). spawn_lifelined handles: clone3+CLONE_PIDFD+
    // CLONE_CLEAR_SIGHAND, setpgid(0,0), lifeline pipe creation, catch_unwind,
    // _exit(0/1). The lifeline pipe is created INSIDE spawn_lifelined; the
    // child receives its read-end as lifeline_r_raw; the parent receives
    // LifelineWriter wrapping the write-end.
    // Manual lifeline pipe creation (Phase 1B/1D) removed — spawn_lifelined
    // owns lifeline pipe creation atomically.
    //
    // spawn_lifelined drops the child's inherited lifeline_w internally (before
    // the closure body runs). spawn_process_child_branch therefore no longer
    // needs a lifeline_w parameter — Arc 213 γ-3 owns "child is its own lifeline
    // keeper" discipline at the spawn_lifelined level.
    //
    // forms: Vec<WatAST> and inherit_config: Option<Config> are both plain data
    // with no interior mutability — they auto-satisfy UnwindSafe without any
    // wrapper (contrast γ-2's Arc<dyn SourceLoader> which required LoaderWrap).
    let (pidfd, lifeline_writer) = spawn_lifelined(move |lifeline_r_raw: i32| {
        // ── CHILD BRANCH ────────────────────────────────────────
        // Reconstruct OwnedFds from inherited raw fds. clone3 gave the
        // child copies of all parent fd table entries — these are valid.
        // SAFETY: these raw fds were created in the parent and inherited
        // across clone3 (separate address space — no shared fd table with
        // parent); reconstructing OwnedFd transfers ownership to
        // spawn_process_child_branch's Drop discipline. No double-close:
        // parent's OwnedFds and child's OwnedFds are in different processes.
        let stdin_r = unsafe { OwnedFd::from_raw_fd(stdin_r_raw) };
        let stdin_w = unsafe { OwnedFd::from_raw_fd(stdin_w_raw) };
        let stdout_r = unsafe { OwnedFd::from_raw_fd(stdout_r_raw) };
        let stdout_w = unsafe { OwnedFd::from_raw_fd(stdout_w_raw) };
        let stderr_r = unsafe { OwnedFd::from_raw_fd(stderr_r_raw) };
        let stderr_w = unsafe { OwnedFd::from_raw_fd(stderr_w_raw) };
        // Reconstruct OwnedFd wrapper for lifeline_r_raw. spawn_lifelined
        // created this fd; it is valid in the child. child_post_fork_init
        // (called inside run_forked_child) registers it with the
        // shutdown worker; we pass the OwnedFd so run_forked_child
        // can mem::forget it after registration (preventing Drop from
        // closing the worker's fd).
        let lifeline_r = unsafe { OwnedFd::from_raw_fd(lifeline_r_raw) };
        run_forked_child(
            forms,
            inherit_config,
            stdin_r_raw,
            stdout_w_raw,
            stderr_w_raw,
            lifeline_r_raw,
            (stdin_r, stdin_w),
            (stdout_r, stdout_w),
            (stderr_r, stderr_w),
            lifeline_r,
        );
    })
    .map_err(|err| RuntimeError { span: list_span.clone(), kind: RuntimeErrorKind::MalformedForm {
        head: OP.into(),
        reason: format!("spawn_lifelined: {}", err)
    } })?;

    // ── PARENT BRANCH ────────────────────────────────────────────
    // Close child-side fds by dropping their OwnedFds (RAII).
    // The child has its own copies in its separate address space.
    // The parent-side ends (stdin_w, stdout_r, stderr_r) remain alive.
    // spawn_lifelined drops the parent's lifeline_r internally — no manual close.
    drop(stdin_r);
    drop(stdout_w);
    drop(stderr_w);

    // Extract the lifeline OwnedFd from LifelineWriter (into_owned_fd added by
    // γ-1; ChildHandle::lifeline_w field type stays Option<OwnedFd>).
    let lifeline_w = lifeline_writer.into_owned_fd();

    // δ-1/δ-2/δ-3 (Stone 6.w COMPLETE): pidfd in handle; waits/kills routed through pidfd; raw pid retired.
    let handle = Arc::new(ChildHandle::new(pidfd, Some(lifeline_w)));

    // Build parent-side handles (Stone C — spawn-process 4-field Process).
    //   stdin field  = IOWriter over stdin_w  (parent writes → child fd 0)
    //   stdout field = IOReader over stdout_r (child fd 1 → parent reads)
    //   stderr field = IOReader over stderr_r (child fd 2 → parent reads)
    //   join field   = ProgramHandle (wait for child exit)
    // NO tx/rx typed-channel fields — those were the slice-1c wrong turn.
    // Arc 278 IPC de-prime: the wat-level pipe-wrapping accessors were
    // annihilated; typed process IPC now flows through the
    // `spawn-program' (process)` peer model (`send'`/`recv'`/`recv-all'`).
    let stdin_writer: Arc<dyn WatWriter> = Arc::new(PipeWriter::from_owned_fd(stdin_w));
    let stdout_reader: Arc<dyn WatReader> = Arc::new(PipeReader::from_owned_fd(stdout_r));
    let stderr_reader: Arc<dyn WatReader> = Arc::new(PipeReader::from_owned_fd(stderr_r));

    Ok(Value::Aggregate(Arc::new(AggregateValue::struct_(
        "wat::kernel::Process".into(),
        vec![
            Value::io__IOWriter(stdin_writer),
            Value::io__IOReader(stdout_reader),
            Value::io__IOReader(stderr_reader),
            Value::wat__kernel__ProgramHandle(Arc::new(ProgramHandleInner::Forked(handle))),
        ],
    ))))
}

// ─── Arg-parsing helpers ─────────────────────────────────────────────

// rune:perspicere(read-once) — Vec<WatAST> is the value the dispatch arm
// was given; the flat helper matches the caller's expected shape exactly.
// pub(crate): also called by kernel/spawn.rs dispatcher for the :process tier
// (arc 214 β — forms extraction from the evaluated args[2]).
pub(crate) fn expect_vec_ast_pub(op: &str, tv: TrackedValue, span: crate::span::Span) -> Result<Vec<WatAST>, RuntimeError> {
    expect_vec_ast(op, tv, span)
}
fn expect_vec_ast(op: &str, tv: TrackedValue, span: crate::span::Span) -> Result<Vec<WatAST>, RuntimeError> {
    match tv.value_owned() {
        Value::Vec(items) => {
            let mut out = Vec::with_capacity(items.len());
            for item in items.iter() {
                match item {
                    Value::wat__WatAST(ast) => out.push((**ast).clone()),
                    other => {
                        // arc 138: no span — Vec element iteration; per-element WatAST span unavailable; use form span
                        return Err(RuntimeError { span: span.clone(), kind: RuntimeErrorKind::TypeMismatch {
                            op: op.into(),
                            expected: "wat::WatAST",
                            got: Box::new(crate::runtime::ValueSnapshot::of(other))
                        } });
                    }
                }
            }
            Ok(out)
        }
        other => Err(RuntimeError { span, kind: RuntimeErrorKind::TypeMismatch {
            op: op.into(),
            expected: "Vec<wat::WatAST>",
            got: Box::new(crate::runtime::ValueSnapshot::of(&other))
        } }),
    }
}
