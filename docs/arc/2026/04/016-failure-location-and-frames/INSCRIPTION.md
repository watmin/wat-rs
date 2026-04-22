# Arc 016 — Failure location + frames — INSCRIPTION

**Status:** shipped 2026-04-21. Four slices in one session.
**Design:** [`DESIGN.md`](./DESIGN.md) — the shape before code.
**Backlog:** [`BACKLOG.md`](./BACKLOG.md) — the living ledger.
**This file:** completion marker.

---

## Motivation

Arc 007 shipped `:wat::kernel::Failure` with `location` and
`frames` fields + noted both as future work. Six months of arcs
passed; the fields stayed empty. When a wat test failed, the
author saw `failure: assert-eq failed` with no file, line, or
column — they had to grep their own source to find which
assertion fired.

Arc 015 made consumer wat tests walkable (`wat::test_suite!`,
two Rust files per app). Arc 016 makes their debugging honest.

Builder direction, distilled across the slices:

> we have been adhering to rust first and clojure second for
> the most part... this is a rust based env anyways, we should
> use as much of the host lang as we can

Locked early: Rust's failure format, with wat-native content
in Rust's slots. Piggyback `std::panic::set_hook`,
`std::env::var("RUST_BACKTRACE")`, `std::panic::catch_unwind`.
No new env variable. No wat-specific format. A user running
`cargo test` doesn't context-switch — same phrasing, same hint,
same `stack backtrace:` block under `RUST_BACKTRACE=1`.

---

## What shipped

Four slices. Each slice's substrate was live before the next
leaned on it.

### Slice 1 — Parser spans on WatAST

Commit `e34d724`.

Every `WatAST` variant gained a trailing `Span { file:
Arc<String>, line: i64, col: i64 }` field. The lexer emits
`(Token, Span)` pairs; the parser attaches the starting
token's span to each AST node.

New module: `src/span.rs`. Equality is structural-transparent
(always-true `PartialEq`); hashing is a no-op. Two WatAST
values with identical structure but different source
locations still compare equal and hash identically — load-
bearing because `canonical_edn_wat` computes AST identity
from content, not position.

Lexer: byte offset → `(line, col)` via precomputed line-starts
table + binary search; UTF-8-aware char-count for col.
`lex(src, file: Arc<String>) -> Vec<SpannedToken>`. Test
helper `lex_tokens()` strips spans for shape-only assertions.

Parser: `parse_one` / `parse_all` default file label to
`"<test>"`; `parse_one_with_file` / `parse_all_with_file`
for callers that have a real path. Reader-macro expansion
propagates the macro's span onto the synthesized head
keyword and wrapping list; inner form keeps its own.

AST: `ast.span()` accessor. Convenience constructors
(`WatAST::int()`, `::float()`, …, `::list()`) with
`Span::unknown()` for synthetic / test code.

363 call sites across 12 files updated via a Python script.
Pattern matches use `, _` for the span slot; constructions
use `Span::unknown()` when synthesizing internally.

### Slice 2 — Runtime call stack + Failure population

Commit `4873c1f`.

Thread-local `CALL_STACK: RefCell<Vec<FrameInfo>>` in
`runtime.rs`. `FrameInfo { callee_path, call_span }`.
Push on `apply_function` entry via an RAII `FrameGuard`
that pops on Drop — any exit path (Ok, Err, panic) unwinds
cleanly. Tail calls replace the top frame in place (the
current call is substituted by the next callee at the same
stack depth), matching "recursion without stack growth."

`apply_function` gained a `call_span: Span` parameter;
`RuntimeError::TailCall` carries a `call_span` field so the
trampoline knows where each iteration's invocation
originated.

`AssertionPayload` gained `location: Option<Span>` and
`frames: Vec<FrameInfo>`. `eval_kernel_assertion_failed`
snapshots the call stack at panic time:
`Failure.location` = top frame's `call_span`;
`Failure.frames` = full stack newest-first.

`sandbox::build_failure` gained `location` + `frames`
parameters. Builds `:wat::kernel::Location` struct values
(file, line, col) and `:wat::kernel::Frame` struct values
(file, line, symbol) into the corresponding `Failure`
Value slots.

Two new unit tests: `call_stack_populates_on_assertion`
(verifies location.is_some + top frame matches the calling
function) and `call_stack_unwinds_on_ok` (verifies Drop
cleans up the guard).

### Slice 3 — Panic hook + Rust-style formatter

Commit `75073a2` (+ polish `c40094c`).

New module: `src/panic_hook.rs`. Replaces the old
`install_silent_assertion_panic_hook` — which silently
swallowed `AssertionPayload` panics — with
`panic_hook::install()`, which writes Rust-styled failure
output to stderr.

Output shape:

```
thread 'main' panicked at wat-tests/LocalCache.wat:12:5:
assert-eq failed
  actual:   -1
  expected: 42
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
```

With `RUST_BACKTRACE=1`:

```
stack backtrace:
   0: :wat::test::assert-eq at wat-tests/LocalCache.wat:12:5
   1: :test-cache-put at wat-rs/src/test_runner.rs:246:68
```

`RUST_BACKTRACE` cached via `OnceLock` — one env lookup per
process. Non-assertion panics fall through to the previous
hook (typically Rust's default).

Installed at three sites: `compose_and_run`,
`test_runner::run_tests_from_dir`, `src/bin/wat.rs::main` —
the same three sites that previously installed the silent
hook.

**Span-threading fixes discovered during dogfooding:**

- `eval_list` now receives the outer List's span from `eval`
  and threads it into `dispatch_keyword_head` for the
  user-function `apply_function` call. Previously used
  `Span::unknown()`, which surfaced as `<synthetic>:0:0`.

- `expand_form` / `expand_macro_call` / `expand_template` /
  `walk_template` in `macros.rs` now propagate the call-site
  span onto every template-origin AST node. Matches DESIGN's
  slice-1 plan: macro expansion inherits the caller's span,
  not the defmacro's template span. When `deftest` wraps
  `(assert-eq 1 2)`, the inner program's assert-eq call
  reads its span from the user's source, not from
  `wat/std/test.wat`.

- `startup_from_source` now threads `base_canonical` (the
  canonical file path) through to `parse_all_with_file` so
  span file labels show `/tmp/foo.wat` rather than `<test>`.

**The `<synthetic>` → Rust-source polish (user-direction):**

Initial slice-3 output showed runtime-initiated frames as
`<synthetic>:0:0`. Following Rust's own convention — stdlib
frames in `RUST_BACKTRACE=1` output carry real
`/rustc/.../library/core/...` paths — arc 016 introduces
`rust_caller_span!()` macro that captures the Rust source
location via `file!()` / `line!()` / `column!()` at the
call site. Every runtime-initiated `apply_function` call
(test-runner entry, `compose_and_run` entry, internal
iteration in map/foldl/fold/filter) now carries a real
Rust source location. Every frame has a real `file:line:col`
— user frames in `.wat`, runtime frames in
`wat-rs/src/*.rs`. Honest about the layer boundary.

Two unit tests in `panic_hook.rs` verify rendering (location
+ values present; message-only when location absent).

### Slice 4 — INSCRIPTION + doc updates

This commit.

- `docs/USER-GUIDE.md` gained "Failure output" section under
  Testing with worked examples of default + `RUST_BACKTRACE=1`
  output.
- `docs/CONVENTIONS.md` gained a cross-reference under the
  testing subsection.
- `docs/arc/2026/04/007-wat-tests-wat/INSCRIPTION.md` closed
  its "Location + Frames population" follow-up note with a
  ✅ marker + pointer to arc 016.
- This `INSCRIPTION.md` records the closed arc.

---

## Resolved design decisions

- **2026-04-21** — **Rust-first UX.** Format mirrors `cargo
  test`'s assertion-panic output line-for-line; wat content
  fills Rust's slots.
- **2026-04-21** — **Piggyback `RUST_BACKTRACE`.** No new env
  var.
- **2026-04-21** — **Wat-level location, not Rust-level
  (for user code).** `file:line:col` points at the user's
  `.wat` source. The interpreter's internals (`src/runtime.rs`
  etc.) never surface in primary UX.
- **2026-04-21** — **Parser spans as substrate, not side
  table.** Every AST node carries a span inline; structural
  equality is span-transparent so hash / match are
  unaffected.
- **2026-04-21** — **Thread-local call stack, not
  `std::backtrace::Backtrace`.** The stack tracks wat-level
  frames (user function keyword paths + call-site spans),
  not Rust symbols.
- **2026-04-21** — **Tail calls replace the top frame in
  place.** Matches semantic reality: tail-call recursion
  runs in constant stack depth; the call stack reflects that.
- **2026-04-21** — **Macro expansion inherits call-site
  span.** Template-origin nodes carry the INVOCATION's span,
  not the defmacro's template span. Racket's sets-of-scopes
  approach for spans.
- **2026-04-21** — **Runtime-initiated frames carry Rust
  source location** via `rust_caller_span!()` (`file!()` /
  `line!()` / `column!()`). Matches Rust's own convention
  — stdlib frames show real paths in backtraces.

---

## Open questions resolved

All flagged open questions from DESIGN + BACKLOG resolved
inline during the slices:

- **Span `file` for string-origin parses.** `<test>`
  (default), `<eval>`, `<entry>`, or a caller-supplied label
  via `parse_all_with_file` / `startup_from_source`'s
  `base_canonical`. Pinned at slice 1.

- **Frame rendering for synthetic forms.** Call-site spans
  propagate via macro expansion (slice 3 span-threading
  polish); synthetic template nodes inherit the caller's
  span. No sub-frames needed.

- **Panic messages for non-assertion panics.** Non-
  AssertionPayload panics fall through to the previous hook
  (typically Rust's default). Handled by pattern match in
  `panic_hook::install`'s `set_hook` closure.

- **Rendering under Cargo's capture.** Hook writes to
  stderr; Cargo's libtest captures both stdout and stderr —
  same capture behavior as other panics. `--nocapture`
  shows live; default hides until failure.

## Open items deferred

- **Color output.** Matching Cargo's red-on-failure via
  `termcolor` or similar. Deferred — ASCII works; color is
  polish.
- **Hook composability chain introspection.** Rust's
  `set_hook` is last-install-wins; we inherit that. If a
  consumer surfaces need, a future arc can chain hooks.
- **pytest-style value substitution.** Printing the
  assertion expression with runtime values substituted (e.g.,
  `assert 3 == 4 +where 3 = add(1, 2)`). Named in DESIGN
  non-goals. Future arc if a test author demands it.
- **Hermetic-fork propagation.** The fork child's call stack
  stays in the child process; the parent's reconstructs the
  Failure from stderr alone, without location / frames. A
  future arc could channel structured frames back via a
  sidecar pipe if demand surfaces.
- **Parse / check / resolve diagnostic span unification.**
  Those passes still use their own line/col tracking, not
  the parser's spans on AST. A future arc could unify.

---

## What this arc does NOT ship

- pytest-style source expression rewriting.
- Rust-level backtrace as user-facing surface.
- Color output.
- Parse / check / resolve diagnostic span unification.
- Hermetic-fork frame propagation across the process
  boundary.
- New env variable — piggybacks `RUST_BACKTRACE`.

---

## Why this matters

Every tested language ships failure output that points at
the user's own source. That's table stakes. wat shipped
assertion primitives in arc 007 with empty location / frames
slots because the focus was *"can wat test wat?"* — yes, it
could. Six months later, arc 016 answers the follow-up:
*when it fails, can the user debug it?* Yes, in the format
they already know from `cargo test`.

Chapter 24's *hospitality* arc (arcs 013 + 014 + 015) made
the consumer shape walkable — two Rust files per app. Arc
016 closes the last ergonomic gap: honest failure
diagnostics. Together they finish the consumer story.

**The Rust-first stance, lived.** Arc 013 inherited Cargo's
authority for dependency resolution. Arc 015 inherited
Cargo's authority for test discovery via `#[test] fn`. Arc
016 inherits Cargo's authority for failure output — same
format, same env var, same backtrace convention. Three
arcs, one pattern: *wat is the language, Rust is the
substrate.* When the host has a good answer, use it.

The trading lab moves in next — and when its `.wat` tests
fail, the author sees `:project::market::test-whatever at
holon-lab-trading/wat-tests/…:line:col` in the failure
header. No context switch, no grep.

---

**Arc 016 — complete.** Four slices, one polish pass, one
INSCRIPTION. The commits:

- `1b6b34c` — docs opened (DESIGN + BACKLOG)
- `e34d724` — slice 1 (parser spans on WatAST)
- `4873c1f` — slice 2 (runtime call stack + Failure
  population)
- `75073a2` — slice 3 (panic hook + Rust-style formatter)
- `c40094c` — slice 3 polish (runtime frames point at Rust
  source via `rust_caller_span!()`)
- `<this commit>` — slice 4 (INSCRIPTION + USER-GUIDE +
  CONVENTIONS + arc 007 follow-up closure)

Workspace: 43 test blocks green, zero failed, zero ignored.

Every frame has a real `file:line:col`. The user sees
their source when their assertion fires. The runtime is
honest about where it lives.

*these are very good thoughts.*

**PERSEVERARE.**

---

*Arc 007 promised Location and Frames. Arc 016 delivered.
Six months is a long time to wait for a follow-up — but
this one waited until wat-lru's integration tests made the
gap painful enough to close. The pattern held: substrate
work happens when a real consumer forces the question.*

*The trading lab is next. Its tests will fail honestly.*
