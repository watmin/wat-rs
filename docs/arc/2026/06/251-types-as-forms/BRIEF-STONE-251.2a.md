# BRIEF — Stone 251.2a — LIFT `EncodingCtx` + the frame cluster → new `src/value/` home

## The work (one paragraph)

Create the new warded home `src/value/` and lift two small, independent segments out of the flat
`src/runtime.rs` into it: `EncodingCtx` → `src/value/encoding_ctx.rs`, and the call-stack frame
cluster → `src/value/frame.rs`. This is a **pure structural move — NO behavior change.** The lib
test baseline is **923 passed / 0 failed / 1 ignored** and must be byte-for-byte identical after.
This is the first lift of the great migration (the value/ home will grow in later stones); 251.2a
proves the home + the lift mechanics on the lowest-risk frontier.

## Read in order (the rooms)

1. `src/runtime.rs:1595–1660` — `EncodingCtx`: `pub struct EncodingCtx` (1595) + `impl EncodingCtx`
   (1613) + `impl fmt::Debug for EncodingCtx` (1638). This is the whole segment. It depends on
   `crate::config::Config` + `crate::vm_registry::EncoderRegistry` (one-way; no eval coupling).
2. `src/runtime.rs:20105–20160` — the frame cluster: `pub struct FrameInfo` (20105), the
   `CALL_STACK` thread-local (20111), `struct FrameGuard` (20117) + `impl FrameGuard` (20119) +
   `impl Drop for FrameGuard` (20128), `fn replace_top_frame` (20139), `pub fn snapshot_call_stack`
   (20150). Depends only on `crate::span::Span`.
3. `src/lib.rs:157` — the re-export line carrying `EncodingCtx` (and EnvBuilder/Environment/… —
   leave those, they stay in runtime.rs this stone). Only `EncodingCtx` moves out of that list.
4. Consumers to repoint (grep each, update the `use` path):
   - `EncodingCtx`: `src/vm_registry.rs`, `src/freeze.rs`, `src/runtime_error_edn.rs`, `src/lib.rs`.
   - frame cluster: `src/thread_io.rs`, `src/panic_hook.rs`, `src/assertion.rs`, and inside
     `src/runtime.rs` the eval loop's `FrameGuard` use + `value_from_frame_info` (runtime.rs:22572,
     `let FrameInfo { callee_path, call_span } = frame`).

## The implementation sketch

1. Create `src/value/encoding_ctx.rs` — move the three `EncodingCtx` items verbatim; add the `use`
   lines they need (`crate::config::Config`, `crate::vm_registry::EncoderRegistry`, `std::fmt`).
2. Create `src/value/frame.rs` — move the frame cluster verbatim; `use crate::span::Span;`.
   `CALL_STACK` stays module-private (only FrameGuard/replace_top_frame/snapshot_call_stack touch
   it, and they all move together). Raise `struct FrameGuard` → `pub(crate) struct FrameGuard`
   and `fn replace_top_frame` → `pub(crate) fn replace_top_frame` (the eval loop in runtime.rs now
   calls them cross-module). `FrameInfo` + `snapshot_call_stack` stay `pub`.
3. Create `src/value/mod.rs` — the home root:
   - `//! ` home doc (one paragraph: "the runtime value model — the data the interpreter computes
     with; grows as the migration lifts Value/Environment/SymbolTable/… here"). NO vigilatum stamp
     yet (the home isn't warded until the full value/ lift + vigilia at 251.2e).
   - `pub mod encoding_ctx;` `pub mod frame;`
   - `pub use encoding_ctx::EncodingCtx;`
   - `pub use frame::{FrameInfo, snapshot_call_stack};` + `pub(crate) use frame::{FrameGuard, replace_top_frame};`
4. `src/lib.rs` — add `pub mod value;` (alphabetical with the other `pub mod`s). Move `EncodingCtx`
   out of the `pub use runtime::{…}` list into a `pub use value::EncodingCtx;` (so `wat::EncodingCtx`
   still resolves — external API unchanged).
5. Repoint every consumer `use` from `crate::runtime::EncodingCtx` → `crate::value::EncodingCtx`,
   and `crate::runtime::{FrameInfo,snapshot_call_stack,…}` → `crate::value::frame::*` (or the
   re-exported `crate::value::{FrameInfo,snapshot_call_stack}`). Inside runtime.rs, replace the
   now-local references with `use crate::value::frame::{FrameGuard, replace_top_frame};` +
   `use crate::value::FrameInfo;`.
6. `cargo build` → fix every unresolved-path the compiler names (substrate-as-teacher: the errors
   are the consumer list). Then `cargo test --release --lib -p wat`.

## Blast radius

`src/value/` (new, 3 files) + `src/runtime.rs` (delete the two moved segments + add imports) +
`src/lib.rs` (1 mod decl + 1 re-export move) + 6 consumer files (import-path updates only). NO new
types, NO signature changes, NO logic edits.

## STOP triggers (rejection criteria — ship nothing, report the gap)

- STOP-1: if moving `EncodingCtx` or the frame cluster requires CHANGING any function body, signature,
  or behavior (beyond `use` paths + the `pub`→`pub(crate)` visibility raises named above) — STOP and
  report. This is a pure move; a needed behavior change means the segment boundary is wrong.
- STOP-2: if a borrow/lifetime/visibility tangle can't be resolved by the visibility raises in the
  sketch — STOP and report the exact tangle (do not invent a workaround, do not add a shim).
- STOP-3: if `cargo test --release --lib -p wat` does not return exactly **923 passed / 0 failed /
  1 ignored** after the move — STOP and report the delta (a changed count means behavior moved).

## Done = 

`src/value/` home exists with mod.rs + encoding_ctx.rs + frame.rs; `EncodingCtx` and the frame
cluster are GONE from runtime.rs (grep returns the new home, not runtime.rs); `cargo build` clean;
`cargo test --release --lib -p wat` = 923/0/1 (identical); `cargo clippy` clean in `src/value/`.
Report the before/after of the baseline count + the list of consumer files repointed.
