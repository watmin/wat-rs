//! `:wat::io::IOWriter/*` intrinsics — arc 255 home #12 (255.1c-io-writer).
//! Thirteen verbs — `new`, `open-file`, `from-fd`, `to-bytes`, `to-string`,
//! `write`, `write-all`, `write-string`, `print`, `println`, `writeln`,
//! `flush`, `close` — the second strike of the `:wat::io::` family
//! (`io/mod.rs`'s family claim), the write-side mirror of `reader.rs`.
//!
//! Every one of the thirteen delegates to a `crate::io::eval_iowriter_*` fn
//! that already existed as a literal-match arm in `runtime.rs` — see
//! `io/mod.rs` for the family-wide "bodies do not live here" claim this home
//! is an instance of.
//!
//! ## ★★ The return types are NOT uniform — every `@arg`/`@ret` below is
//! transcribed straight from its own `check.rs` `TypeScheme`, never copied
//! from a neighbouring row. `writeln` returns `:wat::core::i64` while
//! `println`, right beside it and doing the visibly same thing, returns
//! `:wat::core::nil`; `write` is `i64` but `write-all` is `nil`; `to-string`
//! is `Option<String>`, not `String`; `new` is nullary (zero `@arg` lines).
//!
//! ## Every `@Category` read at the body, not derived from the name
//!
//! The thirteen straddle four clusters:
//!
//! - **`new`/`open-file`/`from-fd`** (`io.rs:1173`/`1198`/`1279`) — mint or
//!   claim a fresh `:wat::io::IOWriter`-typed handle. `open-file`/`from-fd`
//!   are unambiguous: a real syscall (`open(2)`/`dup(2)`) claims a
//!   kernel-tracked fd and wraps it in an `OwnedFd`-backed `PipeWriter`,
//!   textbook `:Resource` acquisition — the write-side mirror of
//!   `reader.rs`'s `open-file`/`from-fd`. `new` is the weaker member of the
//!   three: it is `Arc::new(StringIoWriter::new())`, a nullary, syscall-free
//!   heap allocation — the SAME shape reader.rs gave `from-bytes`/`from-string`
//!   (ruled `:Transform`, `Pure`/`Deterministic`) — but `new` takes NO
//!   argument, so `:Transform`'s "the output is a form of the input" cannot
//!   apply (there is no input). Landed as `:Resource` on the brief's own
//!   "mint or claim" grouping with its two syscall-backed siblings — the
//!   value it returns is the same `IOWriter` handle type that `write`/
//!   `flush`/`close` go on to administer — but the STONE report should carry
//!   the honest caveat: unlike `open-file`/`from-fd`, `StringIoWriter` has no
//!   OS-tracked lifetime and its `close` is a documented no-op (`io.rs:93-95`
//!   — "no explicit-close concept"), so the "handle tracked outside value
//!   scope" half of `:Resource`'s own definition is thinner here than for its
//!   two siblings. `Pure`/`Deterministic` (a plain heap allocation, no
//!   observable effect, always the same empty content — identical reasoning
//!   to `from-bytes`/`from-string`), so per `purity_mandated_examples` it
//!   owes a RUNNABLE `@example`.
//! - **`to-bytes`/`to-string`** (`io.rs:1355`/`1371`) — see below; read with
//!   particular care, per the brief, and argued rather than assumed.
//! - **`write`/`write-all`/`write-string`/`print`/`println`/`writeln`**
//!   (`io.rs:1412`/`1427`/`1446`/`1466`/`1483`/`1501`) — the writer's whole
//!   point: bytes cross the process boundary OUT of the program, the mirror
//!   of the reader family's "bytes cross the process boundary INTO the
//!   program". `:Io`. `Effectful`/`Nondeterministic` — real syscall backings
//!   (`PipeWriter` via `libc::write(2)`, `RealStdout`/`RealStderr` via
//!   locked `std::io::Write`) can partial-write or race the shutdown
//!   broadcast (`WriteStopped`), so the outcome is not fixed by the call's
//!   own arguments alone — the same "even the in-memory backing shares the
//!   family's Category" reasoning `reader.rs` gives its read family, applied
//!   to the push direction.
//! - **`flush`/`close`** (`io.rs:1519`/`1547`) — `:Resource`, per the
//!   brief's "administer" hint, though the body-read splits them more finely
//!   than that one word: `flush` genuinely only ADMINISTERS (forces an
//!   already-open handle's buffered bytes out; `RealStdout`/`RealStderr`
//!   call real `flush(2)`, `PipeWriter`/`StringIoWriter` no-op — never
//!   acquires or releases), the same third disjunct `kernel/resource.rs`
//!   gives `signal`. `close`, by contrast, genuinely RELEASES for the
//!   pipe-backed case — `PipeWriter::close` (`io.rs:759`) swaps the fd to -1
//!   and calls `libc::close(2)`, a real release, matching `kernel/
//!   resource.rs`'s `close'` (the actual-release consumer), not `signal`.
//!   Both `Effectful`/`Deterministic` — like `reader.rs`'s `rewind`, the
//!   outcome (no-op vs. real syscall) is a pure function of the handle's own
//!   concrete backing type, never of unpredictable stream content.
//!
//! ## `to-bytes`/`to-string` — argued, not assumed
//!
//! Both delegate to `snapshot_writer` (`io.rs:1389`), which calls
//! `writer.snapshot()` — a `with_ref` (read-only) clone of the
//! `StringIoWriter`'s accumulated `Vec<u8>`; real-stdio/pipe backings
//! refuse (`None` → `MalformedForm`, only `StringIoWriter` supports it).
//!
//! `:Transform`'s own prose is "the OUTPUT IS A FORM OF THE INPUT" — but the
//! *input* to `to-bytes`/`to-string` is the WRITER (a stateful handle), not
//! the bytes; the returned bytes are not a reshaping of the writer argument,
//! they are a snapshot of state the writer accumulated across an unbounded
//! number of PRIOR, separate calls the current call's arguments say nothing
//! about. That is `:Projection`'s own prose almost verbatim: "returns a
//! COMPONENT of a compound value that was already there... the inverse of
//! `:Combine`" — the writer is the compound value (an internal buffer field
//! plus dispatch machinery), and `to-bytes`/`to-string` hand back that field,
//! exactly the `Failure/message`-style accessor shape `:Projection` names.
//! `to-string`'s UTF-8 decode is an encoding step along the way, the same
//! non-defeating detail `:Io`'s prose gives `read-all-string` ("an encoding
//! step along the way does not make it `:Transform`") — here it does not
//! make it `:Transform` either; it stays `:Projection`.
//!
//! Landed as `:Projection` below, on that reading — but the counter-argument
//! the brief predicts ("hand back a form of what was written") is real: a
//! caller reads `to-bytes`/`to-string` as "give me back what I wrote, as
//! bytes/a string" — the same surface shape as `:Transform`'s canonical
//! members. This rider's reading turns on the ABSENCE of a reshaped
//! argument (there is no bytes/string parameter here to be "the same value,
//! another form" of), not on elimination; see the STONE report for the case
//! made the other way. `Purity Pure` (a read via `with_ref`, no mutation, no
//! external call — the same half `kernel/resource.rs`'s `HandlePool::finish`
//! stands on) / `Determinism Nondeterministic` (the buffer's content is
//! ambient state left by however many prior `write*` calls ran before this
//! one — the same "two calls on the SAME handle can return different
//! answers" reasoning `HandlePool::finish`'s `rx.len()` was corrected onto,
//! 2026-08-19).
//!
//! ## Gate coverage — all thirteen plain, registered `TypeScheme`s
//!
//! Unlike `reader.rs`'s `read-frame`, none of the thirteen has a bespoke
//! `infer_*` arm in `check.rs` — every row is a plain registered
//! `TypeScheme` (`check.rs:15818–15939`), gate LIVE, `@ret` compared by the
//! compiler at every floor. No stub `TypeScheme`s were minted to manufacture
//! coverage, and none was touched.

use wat_macros::wat_intrinsic;

use crate::ast::WatAST;
use crate::span::Span;
use crate::value::{Environment, EvalBreak, SymbolTable, Value};

/// `(:wat::io::IOWriter/new)` → `:wat::io::IOWriter` (empty). Wraps a fresh
/// in-memory `StringIoWriter` — no syscall, no fd. The construction rung of
/// the writer ladder; `IOWriter/open-file`/`from-fd` are its
/// resource-acquiring siblings.
///
/// @added         1.0.0
/// @Purity        Pure
/// @Determinism   Deterministic
/// @Totality         Unreviewed
/// @ExpandTime    Unreviewed
/// @Category      Resource
/// @ret     :wat::io::IOWriter a fresh, empty in-memory writer
/// @example (:wat::io::IOWriter/to-bytes (:wat::io::IOWriter/new)) #=> []
// Registered `TypeScheme` — `check.rs:15818` — gate LIVE. Nullary: zero
// `params`, so zero `@arg` lines above (the RET TRAP the brief calls out).
//
// Deciding line for `@Category Resource`: `src/io.rs:1173`
// `eval_iowriter_new` — `Arc::new(StringIoWriter::new())`. Mints a fresh
// `:wat::io::IOWriter`-typed handle with no argument to reshape (so
// `:Transform`'s "same value, another form" cannot apply — there is no
// value), landed on the brief's "new/open-file/from-fd mint or claim"
// grouping with its two syscall-backed siblings; the module doc above
// carries the honest caveat that this row's own backing has no OS-tracked
// lifetime, unlike its siblings'.
//
// Deciding line for `@Purity Pure` / `@Determinism Deterministic`: a plain
// heap allocation with no observable effect beyond the returned value,
// always the same empty content — identical reasoning to `reader.rs`'s
// `from-bytes`/`from-string`. Per `purity_mandated_examples`, owes a
// RUNNABLE `@example`, not `@example-norun`.
#[wat_intrinsic(":wat::io::IOWriter/new")]
pub(crate) fn eval_iowriter_new(
    env: &Environment,
    sym: &SymbolTable,
    list_span: &Span,
) -> Result<Value, EvalBreak> {
    crate::io::eval_iowriter_new(&[], list_span, env, sym).map_err(Into::into)
}

/// `(:wat::io::IOWriter/open-file path)` → `:wat::io::IOWriter`. Opens (or
/// creates+truncates) a regular file at `path` for writing via `open(2)` and
/// returns a file-backed writer; `Drop` closes the fd via `OwnedFd`. Panics
/// on open errors (panic-vs-Option discipline: bad path/permission/disk-full
/// at construction-time is an environment error worth halting on).
///
/// @added         1.0.0
/// @Purity        Effectful
/// @Determinism   Nondeterministic
/// @Totality         Unreviewed
/// @ExpandTime    Unreviewed
/// @Category      Resource
/// @arg     path :wat::core::String the path to open (create+truncate) for writing
/// @ret     :wat::io::IOWriter a fresh file-backed writer
/// @example-norun (:wat::io::IOWriter/open-file "/tmp/x.txt") #=> #wat.io/IOWriter{}
// Registered `TypeScheme` — `check.rs:15827` — gate LIVE.
//
// Deciding line for `@Category Resource`: `src/io.rs:1198`
// `eval_iowriter_open_file` — `std::fs::OpenOptions::new().write(true)
// .create(true).truncate(true).open(&path)` claims a fresh, kernel-tracked
// fd and wraps it in an `OwnedFd`-backed `PipeWriter`. A syscall resource
// ACQUISITION — the write-side mirror of `reader.rs`'s `open-file`.
//
// SUPERSEDES the earlier `@Determinism Deterministic` reasoning ("no external actor's timing is
// awaited, so the outcome is deterministic given an openable path") — same correction as
// `reader.rs`'s `open-file`, and the builder's argument applies identically here: the same `path`
// can succeed one call and panic the next (parent directory removed, permissions changed, disk
// full — this verb's own doc already names disk-full as a possible open error), which is a
// DIFFERENT outcome on the SAME input, driven by filesystem state the argument doesn't carry.
// "Deterministic given an openable path" is a precondition smuggled into the ruling, not a
// property of the op — the same move refused for `i64::/`. `@Purity Effectful` is unaffected.
#[wat_intrinsic(":wat::io::IOWriter/open-file")]
pub(crate) fn eval_iowriter_open_file(
    path: &WatAST,
    env: &Environment,
    sym: &SymbolTable,
    list_span: &Span,
) -> Result<Value, EvalBreak> {
    crate::io::eval_iowriter_open_file(std::slice::from_ref(path), list_span, env, sym)
        .map_err(Into::into)
}

/// `(:wat::io::IOWriter/from-fd fd)` → `:wat::io::IOWriter`. Arc 170
/// stdio-as-defservice. `dup(2)`-then-own: the writer owns ONLY the dup, so
/// dropping it closes the dup, never the caller's original fd. **Restricted
/// to `:wat::kernel::` callers** (`#[restricted_to]` in `src/io.rs`) — the
/// primed StdOut/StdErr defservices' generated `::init` is the only legal
/// caller.
///
/// @added         1.0.0
/// @Purity        Effectful
/// @Determinism   Deterministic
/// @Totality         Unreviewed
/// @ExpandTime    Unreviewed
/// @Category      Resource
/// @arg     fd :wat::core::i64 the raw fd to dup and wrap
/// @ret     :wat::io::IOWriter a fresh writer owning a private dup of `fd`
/// @example-norun (:wat::io::IOWriter/from-fd 1) #=> #wat.io/IOWriter{}
// Registered `TypeScheme` — `check.rs:15838` — gate LIVE.
//
// Deciding line for `@Category Resource`: `src/io.rs:1279`
// `eval_iowriter_from_fd` — `libc::dup(fd)` claims a fresh, kernel-tracked
// fd (a private copy of the caller's) and wraps it in an `OwnedFd`-backed
// `PipeWriter`. Same acquisition shape as `open-file`, via `dup(2)` instead
// of `open(2)` — the write-side mirror of `reader.rs`'s `from-fd`.
//
// Deciding line for `@Purity Effectful` / `@Determinism Deterministic`:
// identical reasoning to `open-file` — a real syscall, immediate return, no
// external actor awaited.
#[wat_intrinsic(":wat::io::IOWriter/from-fd")]
pub(crate) fn eval_iowriter_from_fd(
    fd: &WatAST,
    env: &Environment,
    sym: &SymbolTable,
    list_span: &Span,
) -> Result<Value, EvalBreak> {
    crate::io::eval_iowriter_from_fd(std::slice::from_ref(fd), list_span, env, sym)
        .map_err(Into::into)
}

/// `(:wat::io::IOWriter/to-bytes writer)` → `:Vector<u8>`. Clones the
/// accumulated buffer. Only valid for a `StringIoWriter`-backed writer; real
/// stdio/pipe backings refuse (`MalformedForm` — no snapshot).
///
/// @added         1.0.0
/// @Purity        Pure
/// @Determinism   Nondeterministic
/// @Totality         Unreviewed
/// @ExpandTime    Unreviewed
/// @Category      Projection
/// @arg     writer :wat::io::IOWriter the writer to snapshot (must be `StringIoWriter`-backed)
/// @ret     (:wat::core::Vector :- [:wat::core::u8]) the bytes accumulated so far
/// @example-norun (:wat::io::IOWriter/to-bytes writer) #=> Bytes[104, 105]
// Registered `TypeScheme` — `check.rs:15847` — gate LIVE.
//
// Deciding line for `@Category Projection` — read with particular care per
// the brief, argued not assumed; see the module doc's "`to-bytes`/
// `to-string` — argued, not assumed" section for the full case:
// `src/io.rs:1355` `eval_iowriter_to_bytes` calls `snapshot_writer` →
// `writer.snapshot()`, a read-only clone of the internal `Vec<u8>` the
// writer accumulated across prior `write*` calls. The writer argument is a
// stateful HANDLE, not a value being reshaped — no fit for `:Transform`'s
// "output is a form of the input" (there is no input value here, only a
// handle whose component is read off). Matches `:Projection`'s own prose
// verbatim: "returns a COMPONENT of a compound value that was already
// there... the inverse of `:Combine`."
//
// Deciding line for `@Purity Pure` / `@Determinism Nondeterministic`:
// `snapshot()` (`io.rs:434`) is a `with_ref` — read-only, no mutation, no
// external call — the same half `kernel/resource.rs`'s `HandlePool::finish`
// stands `Pure` on. But the bytes returned are ambient state left by
// however many prior `write*` calls ran on this SAME handle before this
// one — two calls can return different answers — the same reasoning
// `HandlePool::finish`'s `rx.len()` was corrected onto, 2026-08-19.
#[wat_intrinsic(":wat::io::IOWriter/to-bytes")]
pub(crate) fn eval_iowriter_to_bytes(
    writer: &WatAST,
    env: &Environment,
    sym: &SymbolTable,
    list_span: &Span,
) -> Result<Value, EvalBreak> {
    crate::io::eval_iowriter_to_bytes(std::slice::from_ref(writer), list_span, env, sym)
        .map_err(Into::into)
}

/// `(:wat::io::IOWriter/to-string writer)` → `:Option<String>`. UTF-8
/// decode of the accumulated buffer; `None` if not valid UTF-8. Only
/// meaningful for a `StringIoWriter`-backed writer, same restriction as
/// `to-bytes`.
///
/// @added         1.0.0
/// @Purity        Pure
/// @Determinism   Nondeterministic
/// @Totality         Unreviewed
/// @ExpandTime    Unreviewed
/// @Category      Projection
/// @arg     writer :wat::io::IOWriter the writer to snapshot (must be `StringIoWriter`-backed)
/// @ret     (:wat::core::Option :- [:wat::core::String]) the decoded text, or `None` if the buffer is not valid UTF-8
/// @example-norun (:wat::io::IOWriter/to-string writer) #=> (Some "hi")
// Registered `TypeScheme` — `check.rs:15856` — gate LIVE.
//
// Deciding line for `@Category Projection`: `src/io.rs:1371`
// `eval_iowriter_to_string` — identical `snapshot_writer` read as
// `to-bytes`, plus a `String::from_utf8(bytes).ok()` decode. The decode is
// an encoding step along the way — the same non-defeating detail `:Io`'s
// prose gives `read-all-string` ("an encoding step along the way does not
// make it `:Transform`"); here it does not make it `:Transform` either. See
// `to-bytes`'s comment and the module doc for the full argument.
//
// Deciding line for `@Purity Pure` / `@Determinism Nondeterministic`:
// identical reasoning to `to-bytes` — a read-only snapshot of state left by
// prior calls on the same handle.
#[wat_intrinsic(":wat::io::IOWriter/to-string")]
pub(crate) fn eval_iowriter_to_string(
    writer: &WatAST,
    env: &Environment,
    sym: &SymbolTable,
    list_span: &Span,
) -> Result<Value, EvalBreak> {
    crate::io::eval_iowriter_to_string(std::slice::from_ref(writer), list_span, env, sym)
        .map_err(Into::into)
}

/// `(:wat::io::IOWriter/write writer bytes)` → `:i64` (bytes actually
/// written — may be a partial write).
///
/// @added         1.0.0
/// @Purity        Effectful
/// @Determinism   Nondeterministic
/// @Totality         Unreviewed
/// @ExpandTime    Unreviewed
/// @Category      Io
/// @arg     writer :wat::io::IOWriter the writer to push bytes into
/// @arg     bytes (:wat::core::Vector :- [:wat::core::u8]) the bytes to write
/// @ret     :wat::core::i64 the number of bytes actually written
/// @example-norun (:wat::io::IOWriter/write writer bytes) #=> 5
// Registered `TypeScheme` — `check.rs:15865` — gate LIVE.
//
// Deciding line for `@Category Io`: `src/io.rs:1412`
// `eval_iowriter_write` — `writer.write(&bytes, …)` pushes bytes across the
// stream boundary OUT of the program. The whole point of the verb: output —
// the write-side mirror of `reader.rs`'s `read`.
//
// Deciding line for `@Purity Effectful` / `@Determinism Nondeterministic`:
// a real stream push; `PipeWriter::write` (`io.rs:634`) can return fewer
// bytes than requested (a partial write) and races the shutdown broadcast
// (`WriteStopped`) — the returned count depends on ambient OS/process state
// not fixed by this call's own arguments, the write-side twin of the read
// family's reasoning in `reader.rs`.
#[wat_intrinsic(":wat::io::IOWriter/write")]
pub(crate) fn eval_iowriter_write(
    writer: &WatAST,
    bytes: &WatAST,
    env: &Environment,
    sym: &SymbolTable,
    list_span: &Span,
) -> Result<Value, EvalBreak> {
    crate::io::eval_iowriter_write(&[writer.clone(), bytes.clone()], env, sym, list_span)
        .map_err(Into::into)
}

/// `(:wat::io::IOWriter/write-all writer bytes)` → `:()`. Loops on `write`
/// until every byte is sent.
///
/// @added         1.0.0
/// @Purity        Effectful
/// @Determinism   Nondeterministic
/// @Totality         Unreviewed
/// @ExpandTime    Unreviewed
/// @Category      Io
/// @arg     writer :wat::io::IOWriter the writer to push bytes into
/// @arg     bytes (:wat::core::Vector :- [:wat::core::u8]) the bytes to write, in full
/// @ret     :wat::core::nil always `:()` on success; a broken pipe or stop request raises
/// @example-norun (:wat::io::IOWriter/write-all writer bytes) #=> #wat.core/nil{}
// Registered `TypeScheme` — `check.rs:15874` — gate LIVE.
//
// Deciding line for `@Category Io`: `src/io.rs:1427`
// `eval_iowriter_write_all` — `writer.write_all(&bytes, …)`. Same
// stream-boundary crossing as `write`, looped to completion.
//
// Deciding line for `@Purity Effectful` / `@Determinism Nondeterministic`:
// same reasoning as `write` — the internal loop's path (how many partial
// writes it takes, whether it races a shutdown) depends on ambient OS
// state, even though the successful return value itself is always `:()`.
#[wat_intrinsic(":wat::io::IOWriter/write-all")]
pub(crate) fn eval_iowriter_write_all(
    writer: &WatAST,
    bytes: &WatAST,
    env: &Environment,
    sym: &SymbolTable,
    list_span: &Span,
) -> Result<Value, EvalBreak> {
    crate::io::eval_iowriter_write_all(&[writer.clone(), bytes.clone()], env, sym, list_span)
        .map_err(Into::into)
}

/// `(:wat::io::IOWriter/write-string writer s)` → `:i64` (bytes written, no
/// trailing newline). UTF-8 encodes `s` and writes its bytes via
/// `write-all`. Companion to `writeln` — same shape but without the
/// implicit `\n`.
///
/// @added         1.0.0
/// @Purity        Effectful
/// @Determinism   Nondeterministic
/// @Totality         Unreviewed
/// @ExpandTime    Unreviewed
/// @Category      Io
/// @arg     writer :wat::io::IOWriter the writer to push the string into
/// @arg     s :wat::core::String the string to write, UTF-8 encoded
/// @ret     :wat::core::i64 the number of bytes written
/// @example-norun (:wat::io::IOWriter/write-string writer "hi") #=> 2
// Registered `TypeScheme` — `check.rs:15883` — gate LIVE.
//
// Deciding line for `@Category Io`: `src/io.rs:1446`
// `eval_iowriter_write_string` — UTF-8 encodes `s`, then
// `writer.write_all(bytes, …)`. Same stream-boundary crossing as `write`;
// the encode is an incidental step along the way, not the point.
//
// Deciding line for `@Purity Effectful` / `@Determinism Nondeterministic`:
// same reasoning as `write-all` — delegates to the same `write_all` whose
// path depends on ambient OS state.
#[wat_intrinsic(":wat::io::IOWriter/write-string")]
pub(crate) fn eval_iowriter_write_string(
    writer: &WatAST,
    s: &WatAST,
    env: &Environment,
    sym: &SymbolTable,
    list_span: &Span,
) -> Result<Value, EvalBreak> {
    crate::io::eval_iowriter_write_string(&[writer.clone(), s.clone()], env, sym, list_span)
        .map_err(Into::into)
}

/// `(:wat::io::IOWriter/print writer s)` → `:()`. Unit-returning
/// convenience over `write-string`; discards the byte count. Matches
/// Ruby's `$stdout.print`.
///
/// @added         1.0.0
/// @Purity        Effectful
/// @Determinism   Nondeterministic
/// @Totality         Unreviewed
/// @ExpandTime    Unreviewed
/// @Category      Io
/// @arg     writer :wat::io::IOWriter the writer to push the string into
/// @arg     s :wat::core::String the string to write, UTF-8 encoded
/// @ret     :wat::core::nil always `:()` on success; a broken pipe or stop request raises
/// @example-norun (:wat::io::IOWriter/print writer "hi") #=> #wat.core/nil{}
// Registered `TypeScheme` — `check.rs:15892` — gate LIVE.
//
// Deciding line for `@Category Io`: `src/io.rs:1466`
// `eval_iowriter_print` — `writer.write_all(s.as_bytes(), …)`, discarding
// the count. Same stream-boundary crossing as `write-string`, minus the
// return value.
//
// Deciding line for `@Purity Effectful` / `@Determinism Nondeterministic`:
// same reasoning as `write-all`.
#[wat_intrinsic(":wat::io::IOWriter/print")]
pub(crate) fn eval_iowriter_print(
    writer: &WatAST,
    s: &WatAST,
    env: &Environment,
    sym: &SymbolTable,
    list_span: &Span,
) -> Result<Value, EvalBreak> {
    crate::io::eval_iowriter_print(&[writer.clone(), s.clone()], env, sym, list_span)
        .map_err(Into::into)
}

/// `(:wat::io::IOWriter/println writer s)` → `:()`. Unit-returning
/// convenience over `writeln`; writes `s` + `\n` and discards the byte
/// count. Matches Ruby's `$stdout.puts`.
///
/// @added         1.0.0
/// @Purity        Effectful
/// @Determinism   Nondeterministic
/// @Totality         Unreviewed
/// @ExpandTime    Unreviewed
/// @Category      Io
/// @arg     writer :wat::io::IOWriter the writer to push the string into
/// @arg     s :wat::core::String the string to write, UTF-8 encoded, before the trailing `\n`
/// @ret     :wat::core::nil always `:()` on success; a broken pipe or stop request raises
/// @example-norun (:wat::io::IOWriter/println writer "hi") #=> #wat.core/nil{}
// Registered `TypeScheme` — `check.rs:15901` — gate LIVE.
//
// Deciding line for `@Category Io`: `src/io.rs:1483`
// `eval_iowriter_println` — appends `\n` to `s`'s bytes, then
// `writer.write_all(&bytes, …)`, discarding the count. Same stream-boundary
// crossing as `print`, plus the trailing newline.
//
// Deciding line for `@Purity Effectful` / `@Determinism Nondeterministic`:
// same reasoning as `print`/`write-all`.
#[wat_intrinsic(":wat::io::IOWriter/println")]
pub(crate) fn eval_iowriter_println(
    writer: &WatAST,
    s: &WatAST,
    env: &Environment,
    sym: &SymbolTable,
    list_span: &Span,
) -> Result<Value, EvalBreak> {
    crate::io::eval_iowriter_println(&[writer.clone(), s.clone()], env, sym, list_span)
        .map_err(Into::into)
}

/// `(:wat::io::IOWriter/writeln writer s)` → `:i64` (bytes written,
/// including the trailing `\n`).
///
/// @added         1.0.0
/// @Purity        Effectful
/// @Determinism   Nondeterministic
/// @Totality         Unreviewed
/// @ExpandTime    Unreviewed
/// @Category      Io
/// @arg     writer :wat::io::IOWriter the writer to push the string into
/// @arg     s :wat::core::String the string to write, UTF-8 encoded, before the trailing `\n`
/// @ret     :wat::core::i64 the number of bytes written, including the trailing `\n`
/// @example-norun (:wat::io::IOWriter/writeln writer "hi") #=> 3
// Registered `TypeScheme` — `check.rs:15910` — gate LIVE.
//
// Deciding line for `@Category Io`: `src/io.rs:1501`
// `eval_iowriter_writeln` — appends `\n` to `s`'s bytes, then
// `writer.write_all(&bytes, …)`, returning the byte count. `writeln` sits
// directly beside `println` (line above) and does the visibly same thing,
// but returns `:wat::core::i64` where `println` returns `:wat::core::nil` —
// the RET TRAP the brief calls out; transcribed from `check.rs:15910`, not
// copied from `println`'s row.
//
// Deciding line for `@Purity Effectful` / `@Determinism Nondeterministic`:
// same reasoning as `write`/`print`.
#[wat_intrinsic(":wat::io::IOWriter/writeln")]
pub(crate) fn eval_iowriter_writeln(
    writer: &WatAST,
    s: &WatAST,
    env: &Environment,
    sym: &SymbolTable,
    list_span: &Span,
) -> Result<Value, EvalBreak> {
    crate::io::eval_iowriter_writeln(&[writer.clone(), s.clone()], env, sym, list_span)
        .map_err(Into::into)
}

/// `(:wat::io::IOWriter/flush writer)` → `:()`. Forces any buffered output
/// out. No-op for backings without a user-level buffer (`PipeWriter`,
/// `StringIoWriter`); a real `flush(2)`-equivalent for `RealStdout`/
/// `RealStderr`.
///
/// @added         1.0.0
/// @Purity        Effectful
/// @Determinism   Deterministic
/// @Totality         Unreviewed
/// @ExpandTime    Unreviewed
/// @Category      Resource
/// @arg     writer :wat::io::IOWriter the writer to flush
/// @ret     :wat::core::nil always `:()` on success; an OS-level flush error raises
/// @example-norun (:wat::io::IOWriter/flush writer) #=> #wat.core/nil{}
// Registered `TypeScheme` — `check.rs:15919` — gate LIVE.
//
// Deciding line for `@Category Resource`: `src/io.rs:1519`
// `eval_iowriter_flush` — `writer.flush(…)`. `RealStdout`/`RealStderr`
// (`io.rs:230`/`271`) call the locked `std::io::Write::flush`; `PipeWriter`
// (`io.rs:753`, "pipes have no user-level buffer") and `StringIoWriter`
// (`io.rs:426`, "in-memory buffer — nothing to flush") no-op. Never
// acquires or releases a handle — administers one the caller already
// holds, `:Resource`'s third disjunct, the same shape `kernel/resource.rs`
// gives `signal`.
//
// Deciding line for `@Purity Effectful` / `@Determinism Deterministic`: an
// observable effect for the backings that have a buffer; the outcome
// (no-op vs. real flush) is a pure function of the handle's OWN concrete
// backing type, never of unpredictable stream content — the same reasoning
// `reader.rs` gives `rewind`.
#[wat_intrinsic(":wat::io::IOWriter/flush")]
pub(crate) fn eval_iowriter_flush(
    writer: &WatAST,
    env: &Environment,
    sym: &SymbolTable,
    list_span: &Span,
) -> Result<Value, EvalBreak> {
    crate::io::eval_iowriter_flush(std::slice::from_ref(writer), env, sym, list_span)
        .map_err(Into::into)
}

/// `(:wat::io::IOWriter/close writer)` → `:()`. Idempotent. For pipe-backed
/// writers, releases the fd immediately — the peer reader sees EOF on next
/// read. For non-pipe backings (`StringIoWriter`, `RealStdout`,
/// `RealStderr`) close is a no-op — closing real OS stdio would break the
/// parent process. Subsequent writes against a closed pipe writer return an
/// error.
///
/// @added         1.0.0
/// @Purity        Effectful
/// @Determinism   Deterministic
/// @Totality         Unreviewed
/// @ExpandTime    Unreviewed
/// @Category      Resource
/// @arg     writer :wat::io::IOWriter the writer to close
/// @ret     :wat::core::nil always `:()`
/// @example-norun (:wat::io::IOWriter/close writer) #=> #wat.core/nil{}
// Registered `TypeScheme` — `check.rs:15932` — gate LIVE.
//
// Deciding line for `@Category Resource` — read with particular care per
// the brief: `src/io.rs:1547` `eval_iowriter_close` dispatches to
// `writer.close(…)`. Unlike `flush` (never acquires or releases —
// "administers", `signal`'s shape) and unlike `reader.rs`'s `rewind`
// (never releases across ANY backing), `close`'s `PipeWriter` impl
// (`io.rs:759`) genuinely RELEASES: it swaps the fd to -1 and calls
// `libc::close(2)` — a real syscall release, matching `kernel/resource.rs`'s
// `close'` (the actual-release consumer), not `signal`. The default trait
// impl (`io.rs:100`, "no-op for backings without an explicit-close
// concept") covers `StringIoWriter`/`RealStdout`/`RealStderr`. `:Resource`'s
// release disjunct, not merely its administer one — the brief's "flush/close
// administer" is a fair one-word summary for the pair's shared Category but
// undersells what `close` specifically does to a pipe-backed handle.
//
// Deciding line for `@Purity Effectful` / `@Determinism Deterministic`: a
// real, observable release for the backings that have one; the outcome
// (no-op vs. real close) is a pure function of the handle's own concrete
// backing type, the same reasoning `flush` and `reader.rs`'s `rewind` stand
// on.
#[wat_intrinsic(":wat::io::IOWriter/close")]
pub(crate) fn eval_iowriter_close(
    writer: &WatAST,
    env: &Environment,
    sym: &SymbolTable,
    list_span: &Span,
) -> Result<Value, EvalBreak> {
    crate::io::eval_iowriter_close(std::slice::from_ref(writer), env, sym, list_span)
        .map_err(Into::into)
}
