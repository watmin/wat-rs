//! `:wat::kernel::` stdio intrinsics — arc 255 home #3, carved to the
//! `#[wat_intrinsic]` form (255.1c-kernel-stdio). Six verbs only —
//! `println`, `pprintln`, `eprintln`, `epprintln`, `readln'`, `read-frame` —
//! out of the 49 arms living under the `:wat::kernel::` prefix; the other 43
//! (concurrency, networking, signals, errors, handles/capability, misc) are
//! separate stones (see the DESIGN doc's table).
//!
//! **The bodies do NOT live here.** Every one of the six delegates to a
//! `crate::services::eval_kernel_*` fn (`src/services/verbs.rs`) that already
//! existed at `runtime.rs:5704–5714` as a literal-match arm — this home is a
//! thin `#[wat_intrinsic]`-annotated wrapper around the SAME delegate call,
//! registering it so the intrinsic registry can look it up, document it, and
//! reflect on it. Registration must not change routing: the handler fn that
//! actually runs is unchanged; only the path that reaches it (registry lookup
//! vs. a literal match arm) is different.
//!
//! ## The point of this home — the registry's first `Effectful` rows
//!
//! Every row registered before this home is `Pure`/`Preserving`; nothing has
//! ever been `Effectful`, so the declared-vs-`is_effectful_op` cross-check (renamed
//! `declared_purity_vs_effectful_by_prefix_census` by arc 255.1c site 3)
//! (`src/intrinsic/mod.rs:601`, cross-checking the declared `@Purity` against
//! `runtime::is_effectful_op`'s prefix classification) has never seen a row it
//! could disagree with. All six here write fd 1/2 or read fd 0 — genuine
//! side effects — so all six declare `@Purity Effectful`, independently
//! derived from each body, then checked against `is_effectful_op`
//! (`head.starts_with(":wat::kernel::")` ⇒ effectful) agreeing without either
//! side being edited to make it agree.
//!
//! ## Determinism, from each body
//!
//! - **Writes** (`println`, `pprintln`, `eprintln`, `epprintln`): the body
//!   never reads unpredictable external state to build its result — same `v`
//!   always produces the same formatted line and the same effect (write, or
//!   write-then-terminate). `Deterministic`.
//! - **Reads** (`readln'`, `read-frame`): the body reads fd 0, whose content
//!   varies run to run — the returned value depends on ambient state outside
//!   the call's arguments. `Nondeterministic`. NOT "for the same reason"
//!   `:wat::time::now` is (this doc used to say so; corrected 299.3-entropic):
//!   `readln'`/`read-frame` are `:Io` — the world hands you DATA across a
//!   stream, and a test injects it by feeding fd 0; `time::now` is
//!   `:Entropic` — it samples an unpredictable source and the result can
//!   only be bounded, never pinned. Different cells of the same
//!   `Nondeterministic` row (`wat/runtime-meta.wat:135`, `:143`).
//!
//! ## Category — `Io`, minted mid-strike (builder ruling)
//!
//! The rider's first pass reached for the nearest existing variant —
//! `Encoding` (now `Transform`) for the four writers (by analogy to a "write bytes to a path"
//! doc-contract fixture), `Reflection` for the two readers (by analogy to
//! `Uuid/v4`, an ambient-read fixture) — and flagged both as judgment calls,
//! not certainties. **Overruled**: writing to a stream is not a
//! representation transform (nothing is transformed; `Transform` means
//! `Bytes ⇄ hex`, `String ⇄ Instant`), and reading fd 0 is not the program
//! interrogating its own state (`Reflection` is `call-site`/`show-source`/
//! `metadata-of`; the same mistake a prior stone made calling a clock read
//! `Reflection`, before `Clock` — since renamed `:Entropic` — was minted to fix it). `Io` — "performs I/O
//! on a stream" — was minted instead, at the same level of abstraction as
//! `Clock` (now `:Entropic`, "samples an unpredictable source") and `Arithmetic` ("combines domain
//! values"): what KIND of computation this is, not what it happens to touch
//! along the way. All six rows here land on it. `:wat::io::*` is its second
//! tenant when that namespace carves (`is_effectful_op` already prefix-
//! matches it).

use wat_macros::wat_intrinsic;

use crate::ast::WatAST;
use crate::span::Span;
use crate::value::{Environment, EvalBreak, SymbolTable, Value};

/// `(:wat::kernel::println v)` → `:wat::core::nil`. Serializes `v` to compact
/// EDN and writes the line to stdout (fd 1). See `crate::services::verbs`
/// for the write path (primed `StdOut` service).
///
/// @added         1.0.0
/// @Purity        Effectful
/// @Determinism   Deterministic
/// @Category      Io
/// @arg     v :T the value to print
/// @ret     :wat::core::nil always nil on success; a write failure raises
/// @example-norun (:wat::kernel::println "hi") #=> nil
#[wat_intrinsic(":wat::kernel::println")]
pub(crate) fn eval_kernel_println(
    v: &WatAST,
    env: &Environment,
    sym: &SymbolTable,
    list_span: &Span,
) -> Result<Value, EvalBreak> {
    crate::services::eval_kernel_println(std::slice::from_ref(v), list_span, env, sym).map_err(Into::into)
}

/// `(:wat::kernel::pprintln v)` → `:wat::core::nil`. Pretty (multi-line
/// indented) EDN twin of `println` — same primed `StdOut` write path.
///
/// @added         1.0.0
/// @Purity        Effectful
/// @Determinism   Deterministic
/// @Category      Io
/// @arg     v :T the value to pretty-print
/// @ret     :wat::core::nil always nil on success; a write failure raises
/// @example-norun (:wat::kernel::pprintln "hi") #=> nil
#[wat_intrinsic(":wat::kernel::pprintln")]
pub(crate) fn eval_kernel_pprintln(
    v: &WatAST,
    env: &Environment,
    sym: &SymbolTable,
    list_span: &Span,
) -> Result<Value, EvalBreak> {
    crate::services::eval_kernel_pprintln(std::slice::from_ref(v), list_span, env, sym).map_err(Into::into)
}

/// `(:wat::kernel::eprintln v)` — a **terminating** form. Serializes `v` to
/// compact EDN, writes it to stderr (fd 2) via the primed `StdErr` service,
/// then TERMINATES the process non-zero (arc 278 no-hidden-failures). Never
/// returns on success; polymorphic return `:R` reflects that (mirrors
/// `raise!`/`assertion-failed!`'s divergent scheme).
///
/// @added         1.0.0
/// @Purity        Effectful
/// @Determinism   Deterministic
/// @Category      Io
/// @arg     v :T the crash-reason value
/// @ret     :R never returns — the process terminates non-zero
/// @example-norun (:wat::kernel::eprintln "fatal") #=> never returns
#[wat_intrinsic(":wat::kernel::eprintln")]
pub(crate) fn eval_kernel_eprintln(
    v: &WatAST,
    env: &Environment,
    sym: &SymbolTable,
    list_span: &Span,
) -> Result<Value, EvalBreak> {
    crate::services::eval_kernel_eprintln(std::slice::from_ref(v), list_span, env, sym).map_err(Into::into)
}

/// `(:wat::kernel::epprintln v)` — the pretty **terminating** twin of
/// `eprintln`. Pretty EDN → primed `StdErr` write → TERMINATE. Same death
/// split as `eprintln`; never returns on success.
///
/// @added         1.0.0
/// @Purity        Effectful
/// @Determinism   Deterministic
/// @Category      Io
/// @arg     v :T the crash-reason value
/// @ret     :R never returns — the process terminates non-zero
/// @example-norun (:wat::kernel::epprintln "fatal") #=> never returns
#[wat_intrinsic(":wat::kernel::epprintln")]
pub(crate) fn eval_kernel_epprintln(
    v: &WatAST,
    env: &Environment,
    sym: &SymbolTable,
    list_span: &Span,
) -> Result<Value, EvalBreak> {
    crate::services::eval_kernel_epprintln(std::slice::from_ref(v), list_span, env, sym).map_err(Into::into)
}

/// `(:wat::kernel::readln' <cap-i64>)` — the kernel-restricted positional
/// prime the `readln` defmacro expands to. Reads one frame from stdin (fd 0)
/// via the primed `StdIn` service, EDN-decodes it against the
/// self-describing wire. `readln'` is intentionally NOT `#[restricted_to]`
/// (the macro expands to it inside user fn bodies before the restricted-call
/// walker runs); write `readln`.
///
/// @added         1.0.0
/// @Purity        Effectful
/// @Determinism   Nondeterministic
/// @Category      Io
/// @arg     cap :wat::core::i64 max buffer bytes for the read frame
/// @ret     :T the decoded value (checker special-cases real inference; the registered scheme is vestigial)
/// @example-norun (:wat::kernel::readln' 65536) #=> #wat.kernel/ReadlnOutcome{...}
// `//` not `///` — maintainer rationale, not user-facing prose (the `///` block above is what
// `render-doc` prints; see the goldens note in `witness.rs`).
//
// `:T` above matches `check.rs`'s registered TypeScheme for `readln'` (`ret: t_var()` = `Path(":T")`)
// so `doc_arg_ret_types_match_checker_scheme` agrees — that scheme is explicitly a stub (comment at
// its registration: "real shape is `([cap-i64]? -> :T) -> :T`, not expressible in a fixed-arity
// TypeScheme"; real inference is the special-cased `infer_kernel_readln_prime` arm in `check.rs`'s
// `infer_list`). The runtime body actually returns `:wat::kernel::ReadlnOutcome<T>` — `:T` here
// documents the checker's stub, not the true runtime shape; flagged, not smoothed over.
//
// `eval_kernel_readln_prime` (verbs.rs) also raises a specific `MalformedForm` — not a generic arity
// error — on a stray second `-> :T` arg, or on any arg count other than 1. This wrapper's `Exact(1)`
// shim intercepts a wrong arg count FIRST with its own generic `ArityMismatch`, so that richer
// diagnostic is unreachable through the registry dispatch path for a malformed *unchecked* call (the
// type checker's own `infer_kernel_readln_prime` arm already rejects the same malformed shapes earlier
// in the normal pipeline, with its own richer message). Well-formed (1-arg) calls are unaffected —
// no STDIO ROUTING or output changes (arms unchanged, same delegate, same args) — but the WRONG-ARITY
// error message differs from before. Disclosed as a delta, not treated as a STOP: the brief's STOP-4
// is scoped to stdio *output* routing, not arity-diagnostic wording.
#[wat_intrinsic(":wat::kernel::readln'")]  // rune:lint(retired-name) — readln' is the readln defmacro's expansion target; same name, two forms (structurally required)
pub(crate) fn eval_kernel_readln_prime(
    cap: &WatAST,
    env: &Environment,
    sym: &SymbolTable,
    list_span: &Span,
) -> Result<Value, EvalBreak> {
    crate::services::eval_kernel_readln_prime(std::slice::from_ref(cap), list_span, env, sym).map_err(Into::into)
}

/// `(:wat::kernel::read-frame)` → `:wat::kernel::ReadFrameOutcome`. The
/// raw-frame sibling of `readln'`: one EDN frame's raw text, plus EOF and a
/// stop request, both as matchable outcome variants — undecoded, unlike
/// `readln'`. Zero-arg by choice (shares `readln'`'s default cap).
///
/// @added         1.0.0
/// @Purity        Effectful
/// @Determinism   Nondeterministic
/// @Category      Io
/// @ret     :wat::kernel::ReadFrameOutcome the raw outcome — Frame(text) / Eof / Stopped
/// @example-norun (:wat::kernel::read-frame) #=> #wat.kernel/ReadFrameOutcome.Frame{...}
#[wat_intrinsic(":wat::kernel::read-frame")]
pub(crate) fn eval_kernel_read_frame(
    env: &Environment,
    sym: &SymbolTable,
    list_span: &Span,
) -> Result<Value, EvalBreak> {
    crate::services::eval_kernel_read_frame(&[], list_span, env, sym).map_err(Into::into)
}
