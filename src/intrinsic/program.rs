//! `:wat::program::env` — arc 255 Stone P6-c-W2, the P6-c campaign's second wave.
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
