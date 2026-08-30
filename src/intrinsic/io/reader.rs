//! `:wat::io::IOReader/*` intrinsics — arc 255 home #12 (255.1c-io-reader).
//! Ten verbs — `from-bytes`, `from-string`, `open-file`, `from-fd`, `read`,
//! `read-all`, `read-all-string`, `read-line`, `read-frame`, `rewind` — the
//! first strike of the `:wat::io::` family (`io/mod.rs`'s family claim).
//!
//! Every one of the ten delegates to a `crate::io::eval_ioreader_*` fn that
//! already existed as a literal-match arm in `runtime.rs` — see `io/mod.rs`
//! for the family-wide "bodies do not live here" claim this home is an
//! instance of.
//!
//! ## ★★ Every `@Category` read at the body, not derived from the name
//!
//! The ten straddle the axis, which is why this family was chosen over a
//! more uniform one (`DESIGN-STONE-255.1c-io`, criterion 4):
//!
//! - **`from-bytes`/`from-string`** (`io.rs:875`/`889`) — no syscall, no fd,
//!   no external state consulted. Each wraps the given bytes/string in a
//!   fresh `StringIoReader` and hands it back — the SAME content the caller
//!   already held, in another form. `:Transform`, and — because the body has
//!   no observable effect and the constructed reader's starting content is
//!   wholly determined by the argument — `Pure`/`Deterministic`, the first
//!   pure+det rows in this home and so the only two that owe a RUNNABLE
//!   `@example` (`purity_mandated_examples`).
//! - **`open-file`/`from-fd`** (`io.rs:1237`/`1319`) — a real syscall
//!   (`open(2)` / `dup(2)`) that claims a fresh, kernel-tracked fd and wraps
//!   it in an `OwnedFd`-backed `PipeReader` whose `Drop` closes it. Textbook
//!   `:Resource` ACQUISITION — the read-side mirror of `kernel/resource.rs`'s
//!   `pipe` (`libc::pipe2(2)`, same shape, same reasoning). `Effectful`; both
//!   return immediately without waiting on any external actor's timing, so
//!   `Deterministic` (given a live fd/openable path), the same reasoning
//!   `resource.rs` gives `pipe`/`connect`.
//! - **`read`/`read-all`/`read-all-string`/`read-line`/`read-frame`**
//!   (`io.rs:905`/`926`/`943`/`958`/`1003`) — the reader's whole point: bytes
//!   cross the process boundary into the program. `:Io`. `Effectful`
//!   (a real stream consumption); `Nondeterministic` — the SAME reasoning
//!   `kernel/stdio.rs` gives `readln'`/`read-frame`: "the body reads [the
//!   stream], whose content varies run to run — the returned value depends
//!   on ambient state outside the call's arguments," true of every backing
//!   (`RealStdin`, `PipeReader`, and — once more than one call has run — even
//!   `StringIoReader`, whose cursor is state left by a PRIOR call, not fixed
//!   by this call's own arguments). `read-all-string`'s UTF-8 decode is an
//!   encoding step along the way — `:Io`'s own prose names exactly this: "an
//!   encoding step along the way does not make it `:Transform`."
//! - **`rewind`** (`io.rs:1157`, trait dispatch to three impls at `:368`
//!   `StringIoReader`, `:582` `PipeReader`, `:179` `RealStdin`) — see below;
//!   the one row this home cannot classify without an argument.
//!
//! ## `rewind` — the row that will not classify cleanly
//!
//! `rewind` sits in the same file, called the same way as `read`/`read-all`,
//! and the brief's first-pass instinct is to fold it into `:Io` by
//! proximity. The body-read does not support that fold:
//!
//! - **No bytes cross.** `:Io`'s own defenum prose is "the effect IS the
//!   point" of input/output. `rewind`'s three bodies never move a byte:
//!   `StringIoReader::rewind` (`io.rs:368`) sets `s.cursor = 0`;
//!   `PipeReader::rewind` (`io.rs:582`) unconditionally RAISES ("pipe fds
//!   are not rewindable") without ever issuing an `lseek(2)`;
//!   `RealStdin::rewind` (`io.rs:179`) is an unconditional no-op. None reads
//!   or writes stream data.
//! - **What it DOES do is administer a handle the caller already holds** —
//!   reposition (or refuse to reposition) internal state on an existing
//!   `IOReader`, without acquiring or releasing it. That is `:Resource`'s
//!   third disjunct verbatim, and the closest precedent is
//!   `kernel/resource.rs`'s `signal`: "neither acquires nor releases the …
//!   peer it is given … pure third disjunct, administering a live handle."
//!
//! Landed as `:Resource` below, on that precedent — but the counter-argument
//! (it lives in the same file, called the same way, on the same handle TYPE
//! as the five `:Io` reads, and a caller reads its name as "an IO op") is
//! real and unresolved by this rider; see the STONE report for both sides in
//! full. `Effectful` (mutates internal cursor state, or raises, depending on
//! backing) / `Deterministic` (unlike the reads, the outcome is a pure
//! function of the handle's OWN concrete type, never of unpredictable stream
//! content — `StringIoReader` always succeeds to cursor 0, `PipeReader`
//! always raises, `RealStdin` always no-ops).
//!
//! ## Gate coverage — nine plain, one bespoke
//!
//! Nine of the ten (every row but `read-frame`) are plain, registered
//! `TypeScheme`s (`check.rs:15720–15813`) — gate LIVE, `@ret` compared by
//! the compiler at every floor. `read-frame` has BOTH a registered scheme
//! (`check.rs:15794`, one param) AND a bespoke `infer_ioreader_read_frame`
//! arm (`check.rs:2969`) that intercepts first and accepts one OR two args;
//! see its row below for the split. **No stub `TypeScheme`s were minted to
//! manufacture coverage**, and none was touched.

use wat_macros::wat_intrinsic;

use crate::ast::WatAST;
use crate::span::Span;
use crate::value::{Environment, EvalBreak, SymbolTable, Value};

/// `(:wat::io::IOReader/from-bytes bytes)` → `:wat::io::IOReader`. Wraps
/// `bytes` in a fresh, in-memory `StringIoReader` — no syscall, no fd. The
/// construction rung of the reader ladder; `IOReader/open-file`/`from-fd`
/// are its resource-acquiring siblings.
///
/// @added         1.0.0
/// @Purity        Pure
/// @Determinism   Deterministic
/// @Total         Unreviewed
/// @ExpandTime    Unreviewed
/// @Category      Transform
/// @arg     bytes (:wat::core::Vector :- [:wat::core::u8]) the bytes the reader will yield back, in order
/// @ret     :wat::io::IOReader a fresh in-memory reader over `bytes`
/// @example (:wat::io::IOReader/read-all-string (:wat::io::IOReader/from-bytes (:wat::core::Vector :- [:wat::core::u8] (:wat::core::u8 104) (:wat::core::u8 105)))) #=> "hi"
// Registered `TypeScheme` — `check.rs:15729` — gate LIVE.
//
// Deciding line for `@Category Transform`: `src/io.rs:875`
// `eval_ioreader_from_bytes` — `Arc::new(StringIoReader::from_bytes(bytes))`.
// No syscall, no fd; the bytes the caller already holds come back out
// unchanged through a differently-shaped value — the SAME value, another
// form, `:Transform`'s own definition verbatim.
//
// Deciding line for `@Purity Pure` / `@Determinism Deterministic`: the body
// has no observable effect beyond the returned value (a plain heap
// allocation), and the constructed reader's starting content is wholly
// determined by `bytes` — no external actor, no wait. The first pure+det
// row in this home; per `purity_mandated_examples` it therefore owes a
// RUNNABLE `@example`, not `@example-norun`.
#[wat_intrinsic(":wat::io::IOReader/from-bytes")]
pub(crate) fn eval_ioreader_from_bytes(
    bytes: &WatAST,
    env: &Environment,
    sym: &SymbolTable,
    list_span: &Span,
) -> Result<Value, EvalBreak> {
    crate::io::eval_ioreader_from_bytes(std::slice::from_ref(bytes), list_span, env, sym)
        .map_err(Into::into)
}

/// `(:wat::io::IOReader/from-string s)` → `:wat::io::IOReader`. The `String`
/// twin of `from-bytes`: wraps `s` in a fresh, in-memory `StringIoReader`.
///
/// @added         1.0.0
/// @Purity        Pure
/// @Determinism   Deterministic
/// @Total         Unreviewed
/// @ExpandTime    Unreviewed
/// @Category      Transform
/// @arg     s :wat::core::String the string the reader will yield back
/// @ret     :wat::io::IOReader a fresh in-memory reader over `s`
/// @example (:wat::io::IOReader/read-all-string (:wat::io::IOReader/from-string "hi")) #=> "hi"
// Registered `TypeScheme` — `check.rs:15749` — gate LIVE.
//
// Deciding line for `@Category Transform`: `src/io.rs:889`
// `eval_ioreader_from_string` — `Arc::new(StringIoReader::from_string((*s).clone()))`.
// Identical shape to `from-bytes`: no syscall, the SAME content back out in
// another form.
//
// Deciding line for `@Purity Pure` / `@Determinism Deterministic`: identical
// reasoning to `from-bytes` — no observable effect, starting content wholly
// determined by `s`.
#[wat_intrinsic(":wat::io::IOReader/from-string")]
pub(crate) fn eval_ioreader_from_string(
    s: &WatAST,
    env: &Environment,
    sym: &SymbolTable,
    list_span: &Span,
) -> Result<Value, EvalBreak> {
    crate::io::eval_ioreader_from_string(std::slice::from_ref(s), list_span, env, sym)
        .map_err(Into::into)
}

/// `(:wat::io::IOReader/open-file path)` → `:wat::io::IOReader`. Opens a
/// regular file at `path` for reading via `open(2)` and returns a
/// file-backed reader; `Drop` closes the fd via `OwnedFd`. Panics on open
/// errors (panic-vs-Option discipline: bad path/permission at
/// construction-time is an environment error worth halting on).
///
/// @added         1.0.0
/// @Purity        Effectful
/// @Determinism   Nondeterministic
/// @Total         Unreviewed
/// @ExpandTime    Unreviewed
/// @Category      Resource
/// @arg     path :wat::core::String the path to open for reading
/// @ret     :wat::io::IOReader a fresh file-backed reader
/// @example-norun (:wat::io::IOReader/open-file "/tmp/x.txt") #=> #wat.io/IOReader{}
// Registered `TypeScheme` — `check.rs:15720` — gate LIVE.
//
// Deciding line for `@Category Resource`: `src/io.rs:1237`
// `eval_ioreader_open_file` — `std::fs::OpenOptions::new().read(true).open(&path)`
// claims a fresh, kernel-tracked fd and wraps it in an `OwnedFd`-backed
// `PipeReader`. A syscall resource ACQUISITION — the read-side mirror of
// `kernel/resource.rs`'s `pipe` (same reasoning: this is not merely a
// wat-level construction; the body's first move is a real `open(2)`).
//
// SUPERSEDES the earlier `@Determinism Deterministic` reasoning ("no external actor's timing is
// awaited, so the outcome is deterministic given an openable path") — the builder overturned that
// by argument, and it is decisive: create a file, open it, delete it, open it again — the SAME
// path (the same input this axis measures) now returns a DIFFERENT outcome (miss vs. hit), and a
// hit's own content can differ between calls too. "Deterministic given an openable path" is a
// PRECONDITION smuggled into the ruling, not a property of the op — the identical move this file
// refuses for `i64::/` ("deterministic given a nonzero divisor" is not `total`, and is not
// `deterministic` either by the same logic once the precondition can fail on ambient state, not
// just the arguments). The return value depends on filesystem state EXTERNAL to `path` — the same
// reason `:wat::uuid::v4`/`:wat::time::now` are nondeterministic: same input, output can vary with
// world state the arguments don't carry. `@Purity Effectful` is unaffected by this correction.
#[wat_intrinsic(":wat::io::IOReader/open-file")]
pub(crate) fn eval_ioreader_open_file(
    path: &WatAST,
    env: &Environment,
    sym: &SymbolTable,
    list_span: &Span,
) -> Result<Value, EvalBreak> {
    crate::io::eval_ioreader_open_file(std::slice::from_ref(path), list_span, env, sym)
        .map_err(Into::into)
}

/// `(:wat::io::IOReader/from-fd fd)` → `:wat::io::IOReader`. Arc 170
/// stdio-as-defservice. `dup(2)`-then-own: the reader owns ONLY the dup, so
/// dropping it closes the dup, never the caller's original fd. **Restricted
/// to `:wat::kernel::` callers** (`#[restricted_to]` in `src/io.rs`) — the
/// primed StdIn defservice's generated `::init` is the only legal caller.
///
/// @added         1.0.0
/// @Purity        Effectful
/// @Determinism   Deterministic
/// @Total         Unreviewed
/// @ExpandTime    Unreviewed
/// @Category      Resource
/// @arg     fd :wat::core::i64 the raw fd to dup and wrap
/// @ret     :wat::io::IOReader a fresh reader owning a private dup of `fd`
/// @example-norun (:wat::io::IOReader/from-fd 0) #=> #wat.io/IOReader{}
// Registered `TypeScheme` — `check.rs:15740` — gate LIVE.
//
// Deciding line for `@Category Resource`: `src/io.rs:1319`
// `eval_ioreader_from_fd` — `libc::dup(fd)` claims a fresh, kernel-tracked
// fd (a private copy of the caller's) and wraps it in an `OwnedFd`-backed
// `PipeReader`. Same acquisition shape as `open-file`, via `dup(2)` instead
// of `open(2)`.
//
// Deciding line for `@Purity Effectful` / `@Determinism Deterministic`:
// identical reasoning to `open-file` — a real syscall, immediate return, no
// external actor awaited.
#[wat_intrinsic(":wat::io::IOReader/from-fd")]
pub(crate) fn eval_ioreader_from_fd(
    fd: &WatAST,
    env: &Environment,
    sym: &SymbolTable,
    list_span: &Span,
) -> Result<Value, EvalBreak> {
    crate::io::eval_ioreader_from_fd(std::slice::from_ref(fd), list_span, env, sym)
        .map_err(Into::into)
}

/// `(:wat::io::IOReader/read reader n)` → `:Option<Vector<u8>>`. Reads up to
/// `n` bytes from `reader`; `None` on clean EOF with nothing read. Negative
/// `n` raises.
///
/// @added         1.0.0
/// @Purity        Effectful
/// @Determinism   Nondeterministic
/// @Total         Unreviewed
/// @ExpandTime    Unreviewed
/// @Category      Io
/// @arg     reader :wat::io::IOReader the reader to pull bytes from
/// @arg     n :wat::core::i64 max bytes to read (must be non-negative)
/// @ret     (:wat::core::Option :- [(:wat::core::Vector :- [:wat::core::u8])]) the bytes read, or `None` on clean EOF
/// @example-norun (:wat::io::IOReader/read reader 4) #=> (Some Bytes[104, 105, 33, 10])
// Registered `TypeScheme` — `check.rs:15758` — gate LIVE.
//
// Deciding line for `@Category Io`: `src/io.rs:905`
// `eval_ioreader_read` — `reader.read(n as usize, …)` pulls bytes across the
// stream boundary into the program. The whole point of the verb: input.
//
// Deciding line for `@Purity Effectful` / `@Determinism Nondeterministic`:
// a real stream consumption; the returned bytes depend on the reader's
// underlying content, which is ambient state not fixed by this call's own
// arguments (real fd/pipe backings vary run to run; even the in-memory
// backing's remaining content is state left by a prior call) — the same
// reasoning `kernel/stdio.rs` gives `readln'`/`read-frame`.
#[wat_intrinsic(":wat::io::IOReader/read")]
pub(crate) fn eval_ioreader_read(
    reader: &WatAST,
    n: &WatAST,
    env: &Environment,
    sym: &SymbolTable,
    list_span: &Span,
) -> Result<Value, EvalBreak> {
    crate::io::eval_ioreader_read(&[reader.clone(), n.clone()], env, sym, list_span)
        .map_err(Into::into)
}

/// `(:wat::io::IOReader/read-all reader)` → `:Vector<u8>`. Reads `reader` to
/// EOF and returns every byte.
///
/// @added         1.0.0
/// @Purity        Effectful
/// @Determinism   Nondeterministic
/// @Total         Unreviewed
/// @ExpandTime    Unreviewed
/// @Category      Io
/// @arg     reader :wat::io::IOReader the reader to drain to EOF
/// @ret     (:wat::core::Vector :- [:wat::core::u8]) every byte read
/// @example-norun (:wat::io::IOReader/read-all reader) #=> Bytes[104, 105]
// Registered `TypeScheme` — `check.rs:15767` — gate LIVE.
//
// Deciding line for `@Category Io`: `src/io.rs:926`
// `eval_ioreader_read_all` — `reader.read_all(…)`. Same stream-boundary
// crossing as `read`, to exhaustion.
//
// Deciding line for `@Purity Effectful` / `@Determinism Nondeterministic`:
// same reasoning as `read` — the bytes returned depend on the reader's
// ambient content.
#[wat_intrinsic(":wat::io::IOReader/read-all")]
pub(crate) fn eval_ioreader_read_all(
    reader: &WatAST,
    env: &Environment,
    sym: &SymbolTable,
    list_span: &Span,
) -> Result<Value, EvalBreak> {
    crate::io::eval_ioreader_read_all(std::slice::from_ref(reader), env, sym, list_span)
        .map_err(Into::into)
}

/// `(:wat::io::IOReader/read-all-string reader)` → `:String`. Reads `reader`
/// to EOF and UTF-8-decodes the bytes — byte-faithful, no line-splitting.
/// Panics on non-UTF-8 (panic-vs-Option discipline: a non-UTF-8 stream is an
/// environment error worth halting on).
///
/// @added         1.0.0
/// @Purity        Effectful
/// @Determinism   Nondeterministic
/// @Total         Unreviewed
/// @ExpandTime    Unreviewed
/// @Category      Io
/// @arg     reader :wat::io::IOReader the reader to drain to EOF
/// @ret     :wat::core::String the decoded UTF-8 text
/// @example-norun (:wat::io::IOReader/read-all-string reader) #=> "hi"
// Registered `TypeScheme` — `check.rs:15776` — gate LIVE.
//
// Deciding line for `@Category Io`: `src/io.rs:943`
// `eval_ioreader_read_all_string` — `reader.read_all(…)` then UTF-8 decode.
// The decode is an encoding step along the way, not the point — `:Io`'s own
// prose: "an encoding step along the way does not make it `:Transform`."
//
// Deciding line for `@Purity Effectful` / `@Determinism Nondeterministic`:
// same reasoning as `read`/`read-all` — depends on the reader's ambient
// content.
#[wat_intrinsic(":wat::io::IOReader/read-all-string")]
pub(crate) fn eval_ioreader_read_all_string(
    reader: &WatAST,
    env: &Environment,
    sym: &SymbolTable,
    list_span: &Span,
) -> Result<Value, EvalBreak> {
    crate::io::eval_ioreader_read_all_string(std::slice::from_ref(reader), env, sym, list_span)
        .map_err(Into::into)
}

/// `(:wat::io::IOReader/read-line reader)` → `:Option<String>`. Reads one
/// physical line (LF/CRLF-stripped); `None` on clean EOF.
///
/// @added         1.0.0
/// @Purity        Effectful
/// @Determinism   Nondeterministic
/// @Total         Unreviewed
/// @ExpandTime    Unreviewed
/// @Category      Io
/// @arg     reader :wat::io::IOReader the reader to pull a line from
/// @ret     (:wat::core::Option :- [:wat::core::String]) the line read, or `None` on clean EOF
/// @example-norun (:wat::io::IOReader/read-line reader) #=> (Some "hello")
// Registered `TypeScheme` — `check.rs:15785` — gate LIVE.
//
// Deciding line for `@Category Io`: `src/io.rs:958`
// `eval_ioreader_read_line` — `reader.read_line(…)`. Same stream-boundary
// crossing as `read`, one physical line at a time.
//
// Deciding line for `@Purity Effectful` / `@Determinism Nondeterministic`:
// same reasoning as `read` — depends on the reader's ambient content.
#[wat_intrinsic(":wat::io::IOReader/read-line")]
pub(crate) fn eval_ioreader_read_line(
    reader: &WatAST,
    env: &Environment,
    sym: &SymbolTable,
    list_span: &Span,
) -> Result<Value, EvalBreak> {
    crate::io::eval_ioreader_read_line(std::slice::from_ref(reader), env, sym, list_span)
        .map_err(Into::into)
}

/// `(:wat::io::IOReader/read-frame reader)` → `:wat::io::IOReader::ReadFrameOutcome`.
/// `(:wat::io::IOReader/read-frame reader max-bytes)` → same. Accumulates
/// physical lines until the buffer forms a complete EDN value, returning
/// `Frame(text)` / `Eof` / `Stopped` (a process-wide stop request, not an
/// error and not EOF — the reason `Option<String>` could not carry this
/// verb's return). Optional second arg caps the buffer (default
/// `DEFAULT_MAX_FRAME_BYTES`, 512 KiB); polls the reader's fd (when it has
/// one) against the shutdown-broadcast fd before every physical-line read.
///
/// @added         1.0.0
/// @Purity        Effectful
/// @Determinism   Nondeterministic
/// @Total         Unreviewed
/// @ExpandTime    Unreviewed
/// @Category      Io
/// @arg     xs… :wat::io::IOReader the reader to pull a frame from (+ optional max-bytes — see `infer_ioreader_read_frame`)
/// @ret     :wat::io::IOReader::ReadFrameOutcome Frame(text) / Eof / Stopped
/// @example-norun (:wat::io::IOReader/read-frame reader) #=> #wat.io/IOReader::ReadFrameOutcome.Frame{0: "(1 2 3)"}
// `read-frame` has BOTH a registered `TypeScheme` (`check.rs:15794`, ONE
// param, `ret` the `ReadFrameOutcome` enum above) AND a bespoke
// `infer_ioreader_read_frame` arm (`check.rs:2969`) that intercepts FIRST
// and accepts one OR two args — the real check-time authority, per the
// `kernel/message.rs` shape for a bespoke-arm row. The `@ret` above is the
// REGISTERED scheme's (what the gate compares); the optional second
// `max-bytes` arg is real (see the fn body) but invisible to
// `doc_arg_ret_types_match_checker_scheme`, whose arg loop is guarded by
// `i < scheme.params.len()` (1) — documented here as one `@arg xs…` so the
// gate's single comparison (against `scheme.params[0]`, the reader) still
// lands, without claiming the scheme sees the second arg. No second scheme
// was minted.
//
// Deciding line for `@Category Io`: `src/io.rs:1003`
// `eval_ioreader_read_frame` — accumulates via repeated `reader.read_line`
// calls (`edn::render::read_framed_edn`). Same stream-boundary crossing as
// `read`/`read-line`, assembled into a wire-protocol frame instead of a
// fixed byte count or physical line.
//
// Deciding line for `@Purity Effectful` / `@Determinism Nondeterministic`:
// same reasoning as the rest of the read family, plus: even the `Stopped`
// outcome depends on ambient process-wide state (a shutdown broadcast) not
// fixed by this call's own arguments.
#[wat_intrinsic(":wat::io::IOReader/read-frame")]
pub(crate) fn eval_ioreader_read_frame(
    xs: &[WatAST],
    env: &Environment,
    sym: &SymbolTable,
    list_span: &Span,
) -> Result<Value, EvalBreak> {
    crate::io::eval_ioreader_read_frame(xs, env, sym, list_span).map_err(Into::into)
}

/// `(:wat::io::IOReader/rewind reader)` → `:()`. Resets `reader`'s read
/// position: to the start for an in-memory reader; unconditionally raises
/// for a pipe/fd-backed reader ("pipe fds are not rewindable"); a no-op for
/// real stdin.
///
/// @added         1.0.0
/// @Purity        Pure
/// @Determinism   Deterministic
/// @Total         Unreviewed
/// @ExpandTime    Unreviewed
/// @Category      Resource
/// @arg     reader :wat::io::IOReader the reader to rewind
/// @ret     :wat::core::nil always `:()` on success; a non-rewindable backing raises
/// @example (:wat::io::IOReader/read-all-string (:wat::core::let [r (:wat::io::IOReader/from-string "hi") _ (:wat::io::IOReader/read-all r) _ (:wat::io::IOReader/rewind r)] r)) #=> "hi"
// Registered `TypeScheme` — `check.rs:15807` — gate LIVE.
//
// Deciding line for `@Category Resource` — genuinely contested, see the
// module doc's "`rewind` — the row that will not classify cleanly" section
// for the full argument: `src/io.rs:1157` `eval_ioreader_rewind` dispatches
// to `reader.rewind(…)`, whose three impls (`io.rs:368` `StringIoReader`,
// `:582` `PipeReader`, `:179` `RealStdin`) never move a byte — no fit for
// `:Io`'s "the effect IS the point" of input/output. What they DO do is
// reposition (or refuse to reposition) internal state on a handle the
// caller already holds, without acquiring or releasing it — `:Resource`'s
// third disjunct, the same shape `kernel/resource.rs` gives `signal`
// ("neither acquires nor releases the peer it is given … administering a
// live handle"). Landed here on that precedent, not by elimination.
//
// ⊘ `@Purity` RULED **Pure** by the builder, 2026-08-20, overturning this
// row's shipped `Effectful`. The argument that decided it: *"that mutation
// on a string… what's the effect on an immutable string in memory?"*
// Read at the struct — `ReaderState { bytes: Vec<u8>, cursor: usize }`
// (`io.rs:285`) — `bytes` is moved in at construction and NEVER written
// again. `rewind` assigns `cursor = 0`. No syscall, no shared state,
// nothing outside the value the caller holds.
//
// The axis already accepts that handle state is not the world: `to-bytes`/
// `to-string` are `Pure` while reading mutable handle state
// (`io/writer.rs`). `rewind` is the first verb in either io home that
// WRITES handle state and touches nothing else, which is why it would not
// classify. It is not `read-all`'s twin: `read-all` advances the cursor by
// a content-dependent amount and can consume a real fd; `rewind` assigns a
// constant. Hence the pairing —
//     `to-bytes`  Pure · Nondeterministic  — reads state that VARIES
//     `rewind`    Pure · Deterministic     — writes a FIXED state, always 0
// Idempotent: calling it twice is calling it once.
//
// ⚠ The orchestrator argued `Effectful` partly to keep this row out of a
// rete `where` fence. That was the purity axis used as ACCESS CONTROL
// rather than as a description, and it is retracted: the registry declares
// what is TRUE and consumers adapt — arc 255's whole thesis
// (`[[feedback_name_the_property_not_the_symptom]]`).
// The exposure is real and is filed, not resolved here: `compile-condition`
// gates on pure ∧ deterministic ONLY — `wat/rete.wat:698` says in its own
// words that `total?` is UNARMED at the fence — so nothing today stands
// between a `Pure`+`Deterministic` nil-returning mutator and a `where`
// clause. That is a gap in the FENCE, not a reason to mislabel this row.
//
// ⚠ PENDING, its own stone: the `@ret` line above says a non-rewindable
// backing raises. `PipeReader` (`io.rs:582`) does. `RealStdin` (`io.rs:179`)
// still returns `Ok(())` — silently succeeding while doing nothing, so a
// read-all → rewind → read-all on real stdin yields the content then "",
// with no error. Two backings, one physical impossibility, opposite
// answers. Builder-ruled 2026-08-20: everything but the string backing
// faults. Not applied in this stone — it is a behaviour change.
//
// `@Determinism Deterministic` is unchanged and was always right: unlike
// the read family, the outcome is fixed by the handle's own backing, never
// by unpredictable stream content.
#[wat_intrinsic(":wat::io::IOReader/rewind")]
pub(crate) fn eval_ioreader_rewind(
    reader: &WatAST,
    env: &Environment,
    sym: &SymbolTable,
    list_span: &Span,
) -> Result<Value, EvalBreak> {
    crate::io::eval_ioreader_rewind(std::slice::from_ref(reader), env, sym, list_span)
        .map_err(Into::into)
}
