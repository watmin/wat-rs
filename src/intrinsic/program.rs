//! `:wat::program::env` — arc 255 Stone P6-c-W2, the P6-c campaign's second wave.
//! (+ `:wat::program::startup-handshake-deadline-ms`, excursus 001
//! `the-handshake-deadline-is-injectable` — the FIRST stdlib-facing constant sourced from
//! Rust. See that verb's own doc below for the shape and why it is homed here.)
//!
//! BRIEF: `docs/arc/2026/06/255-builtin-registry/BRIEF-STONE-P6-c-W2-stream-program-stdlib.md`.
//!
//! One verb, its own home ("own home, same shape" as `list.rs`/`bytes.rs`/`char.rs`/
//! `regex.rs`): the ambient per-thread program env reader, moved verbatim out of
//! `runtime.rs`'s giant match. `:wat::program::self-peer` and `:wat::program::cpu-count`
//! are neighbours in the SAME giant-match block but are NOT this wave's verbs — left alone.
//!
//! ★ Same H-1a arity fix as W1's `:wat::config::*`: `env` declared a variadic `&[WatAST]`
//! it used only to reject via a hand-rolled length check — publishing a fictional
//! `Arity::Variadic` for a verb that is actually nullary. Real arity (0) now, shim-owned.
//!
//! `@Purity Pure @Category Ambient` mirrors `:wat::config::*` exactly, and for the
//! identical reason: `current_program_env()` reads a `RefCell` thread-local INSTALLED ONCE
//! per thread at a fixed pre-`:user::main` seam (`install_program_env`,
//! `src/services/client.rs`) and never mutated afterward — the same "committed-once,
//! read-many" shape `sym.encoding_ctx()` has for `:wat::config::dim-count`
//! (`src/intrinsic/config.rs`). `rete/purity.rs`'s `RULES` table disposes the WHOLE
//! `:wat::program::` namespace `Impure` ("reads process env") for the coarse
//! `intrinsic_meta` completeness gate — a deliberately conservative, namespace-WIDE
//! default, not a claim about any individual verb's body (see
//! `src/intrinsic/kernel/ambient.rs`'s doc for the identical divergence on four
//! `:wat::kernel::` signal readers: namespace-Impure by RULES, individually `@Purity Pure`
//! by reading the body). This file's `@Purity` does not need to agree with that default,
//! and homing it does not need to touch `rete/purity.rs` — the RULES disposition already
//! covers it regardless of where the verb lives.

use std::sync::OnceLock;

use wat_macros::wat_intrinsic;

use crate::span::Span;
use crate::value::{EvalBreak, RuntimeError, RuntimeErrorKind, Value};

/// `(:wat::program::env) -> :wat::program::Env`. The calling thread's ambient program env.
///
/// Arc 259 — The Forced Hand. Reads the `PROGRAM_ENV` thread-local, installed once by
/// `install_program_env` at the post-bootstrap / pre-`:user::main` seam. A clean
/// `MalformedForm` error if no env has been installed on this thread (e.g. a test calling
/// `eval_in_frozen` without going through that seam).
///
/// @added         1.0.0
/// @Purity        Pure
/// @Determinism   Deterministic
/// @Total         Unreviewed
/// @Category      Ambient
/// @ret     :wat::program::Env the calling thread's ambient program env
/// @example (:wat::program::Env/peer-kind (:wat::program::env)) #=> :wat::program::PeerKind::process
#[wat_intrinsic(":wat::program::env")]
pub(crate) fn eval_program_env_intrinsic(list_span: &Span) -> Result<Value, EvalBreak> {
    const OP: &str = ":wat::program::env";
    crate::services::current_program_env().ok_or_else(|| {
        RuntimeError::new(
            list_span.clone(),
            RuntimeErrorKind::MalformedForm {
                head: OP.into(),
                reason: "no program env installed on this thread — call install_program_env \
                         before invoking (:wat::program::env)"
                    .into(),
            },
        )
        .into()
    })
}

// ─── The startup handshake's deadline — excursus 001, the-handshake-deadline-is-injectable ───
//
// ⭑ THE ONE PLACE THE NUMBER LIVES. `wat/spawn.wat` used to hold
// `(:wat::core::def :wat::spawn::STARTUP-HANDSHAKE-DEADLINE-MS 30000)`; that `def` is GONE and
// its justification prose stays there, pointing here. Exactly one place in the tree holds this
// number as CODE — `DEFAULT_STARTUP_HANDSHAKE_DEADLINE_MS` below (measured comment-stripped: the
// other five `30000`s in the tree are unrelated — two `parse_duration_ms("30s")` unit assertions
// in `crates/wat-macros`, two `circuit.wat` budget sums, and `span.wat`'s metrics-flush default).
// Two homes for one value is the drift `ab419aaa3` removed, and re-introducing it was the one
// thing this stone was told not to do.
//
// ⛔ WHY THE ENV READ IS **HERE** AND NOT IN `recv_by_deadline`. Reading
// `WAT_STARTUP_HANDSHAKE_DEADLINE_MS` inside the primitive is the cheapest diff and it is wrong:
// it would silently re-time EVERY `recv-by-deadline` in the corpus — `owner-recv-loop`'s 10 000 ms,
// `call-by-deadline`'s caller-supplied `ms`, and every future caller — from one variable named for
// the handshake. The knob attaches to the handshake's NUMBER; the primitive that consumes it never
// learns the variable exists.
//
// ⛔ WHY A RUST-SOURCED VALUE AT ALL. `wat/spawn.wat` is manifest position 171 and no env-reading
// verb is loaded before it, so a wat-level read at load time cannot work (excursus DESIGN route a);
// the stdlib is frozen into the binary, so a settable global is a hole, not a knob (route b); and
// `child-main` is GENERATED and takes no parameters, so threading it through `launch` moves the
// parent end while the child end stays fixed — half a fence (route c). An intrinsic is available in
// ANY eval context regardless of manifest position, which is why it is the shape.

/// The frozen default. **The only place in the tree that HOLDS this number as code** —
/// `wat/spawn.wat` carries the justification prose for it (an argument, not a second home),
/// and nothing else reads or restates it. Unchanged by this stone: only its injectability
/// is new.
const DEFAULT_STARTUP_HANDSHAKE_DEADLINE_MS: i64 = 30_000;

/// The env var that injects it. Named for the HANDSHAKE, and it reaches exactly the handshake's
/// five call sites — never `recv-by-deadline` itself.
const STARTUP_HANDSHAKE_DEADLINE_ENV: &str = "WAT_STARTUP_HANDSHAKE_DEADLINE_MS";

/// Read once per process, never per call. A deadline that changes mid-run is a different bug.
static STARTUP_HANDSHAKE_DEADLINE_MS: OnceLock<i64> = OnceLock::new();

/// Resolve the startup-handshake deadline for THIS process, reading the environment at most once.
///
/// Absent → the default, silently. **Unparseable, negative, or zero → the default, with ONE line
/// on stderr.** The two halves of that are deliberate and were the DESIGN's open question:
///
/// - It must not FAIL. A harness that typos a value must not turn that into a red somewhere
///   unrelated — `wat/test.wat`'s two handshake sites are on the path of every spawned test
///   program, so a refusal here would redden the floor broadly for a typo.
/// - It must not read as DELIBERATE. Silently swallowing `ms=oops` would let an operator believe
///   a 200 ms deadline is in force while 30 s is. The one stderr line is emitted inside
///   `get_or_init`, so it appears at most once per process, and it names the variable, the
///   offending text and the value actually used.
///
/// **Zero is not a wait** (excursus 001 `zero-is-not-a-wait`): a 0 ms handshake deadline fires
/// before any child could physically announce, so every spawn would become `TimedOut` — a
/// foot-gun dressed as a knob. It is treated as invalid, not clamped to 1 ms, because clamping
/// would silently honour a request nobody can have meant; and negative likewise.
fn startup_handshake_deadline_ms() -> i64 {
    *STARTUP_HANDSHAKE_DEADLINE_MS.get_or_init(|| {
        let Ok(raw) = std::env::var(STARTUP_HANDSHAKE_DEADLINE_ENV) else {
            return DEFAULT_STARTUP_HANDSHAKE_DEADLINE_MS;
        };
        match raw.trim().parse::<i64>() {
            Ok(ms) if ms > 0 => ms,
            _ => {
                eprintln!(
                    "wat: {STARTUP_HANDSHAKE_DEADLINE_ENV}={raw:?} must be a POSITIVE integer \
                     number of milliseconds (zero is not a wait — a 0 ms handshake deadline fires \
                     before any child can announce) — using the default \
                     {DEFAULT_STARTUP_HANDSHAKE_DEADLINE_MS}."
                );
                DEFAULT_STARTUP_HANDSHAKE_DEADLINE_MS
            }
        }
    })
}

/// `(:wat::program::startup-handshake-deadline-ms)` — nullary; the milliseconds every
/// "something was just spawned and must announce itself" wait bounds itself by, as
/// `:wat::core::i64`.
///
/// Excursus 001 `the-handshake-deadline-is-injectable`. Named by the handshake's five sites —
/// `wat/spawn.wat`'s `ThreadOpts/launch` and `ProcessOpts/launch`, `wat/test.wat`'s
/// `spawn-thread-program` and `spawn-hermetic-program`, and the generated `child-main` in
/// `wat/service.wat` (spliced into a quasiquoted body, so the CHILD calls it too and, on the
/// process tier, reads the env it inherited). Defaults to 30 000 ms;
/// `WAT_STARTUP_HANDSHAKE_DEADLINE_MS` overrides it for the process, read at most once
/// ([`startup_handshake_deadline_ms`] carries the invalid/zero ruling).
///
/// ★ Homed in `:wat::program::` rather than `:wat::spawn::` on the strength of a cited exemplar,
/// not a preference: `wat/spawn.wat` ALREADY sources a runtime-computed number from this
/// namespace — `(:wat::program::cpu-count)` is what its four `:runner-count` defaults call
/// (`wat/spawn.wat:124`, `:129`, `:135`, …). This is the same kind of thing one row down: a host
/// fact the spawn machinery parameterises itself by. `:wat::program::` is also the namespace
/// `rete/purity.rs`'s `RULES` already disposes with the reason *"reads process env"*, which is
/// literally this verb's body — a `:wat::spawn::` name would have been a brand-new namespace
/// needing a new purity disposition invented for one knob.
///
/// `@Determinism Deterministic` for the same reason `:wat::program::env` and
/// `:wat::runtime::argv` declare it: committed once per process, identical on every subsequent
/// read for the rest of the run. It is not `:wat::program::cpu-count`'s live per-call host query.
///
/// @added         1.0.0
/// @Purity        Pure
/// @Determinism   Deterministic
/// @Total         Unreviewed
/// @Category      Ambient
/// @ret     :wat::core::i64 the startup-handshake deadline in milliseconds for this process
/// @example (:wat::core::= (:wat::program::startup-handshake-deadline-ms) (:wat::program::startup-handshake-deadline-ms)) #=> true
/// @example-norun (:wat::program::startup-handshake-deadline-ms)
/// @see     :wat::program::cpu-count
#[wat_intrinsic(":wat::program::startup-handshake-deadline-ms")]
pub(crate) fn eval_program_startup_handshake_deadline_ms() -> Result<Value, EvalBreak> {
    Ok(Value::i64(startup_handshake_deadline_ms()))
}
