# Arc 016 — Failure location + frames

**Status:** opened 2026-04-21.
**Motivation:** arc 007 shipped `:wat::kernel::Failure` with
`location :Option<Location>` and `frames :Vec<Frame>` fields,
and noted them as future work in its INSCRIPTION §
"Location + Frames population." Today those fields stay empty.
When a wat test fails, the author sees `failure: assert-eq
failed` with no file:line:col and no call chain — they have
to grep their test source to find which assertion fired.

This arc closes that gap. The UX target is **Rust's own
failure output, with wat-native content in the slots.** When
`cargo test` runs a wat test suite and one of its `deftest`s
fails, the output reads identical in shape to any other
`cargo test` failure — just pointing at the user's `.wat`
source instead of a `.rs` file.

---

## UX target (locked before design)

### Default output

```
test LocalCache.wat :: wat-lru::test-local-cache-put-then-get ... FAILED (8ms)
thread 'wat test' panicked at wat-tests/LocalCache.wat:12:5:
assertion `result == 42` failed
  actual:   -1
  expected: 42
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
```

### With `RUST_BACKTRACE=1`

```
test LocalCache.wat :: wat-lru::test-local-cache-put-then-get ... FAILED (8ms)
thread 'wat test' panicked at wat-tests/LocalCache.wat:12:5:
assertion `result == 42` failed
  actual:   -1
  expected: 42
stack backtrace:
   0: :wat::test::assert-eq at wat-tests/LocalCache.wat:12:5
   1: :wat-lru::test-local-cache-put-then-get at wat-tests/LocalCache.wat:5:6
   2: (deftest body)
```

### Why Rust's format, wat's content

The project's standing discipline (chapters 18 + 23 of the
BOOK): wat is the language, Rust is the substrate. Users run
`cargo test`. They know `RUST_BACKTRACE=1`. Making wat's
failures look like Rust's failures means the reader's muscle
memory carries over — no context switch, no new format to
learn. The `note: run with ...` line is Rust's literal
phrasing. The `stack backtrace:` heading is Rust's. The
numbered `0: / 1: / 2:` is Rust's.

What we swap in:
- `file:line:col` → the user's `.wat` source (not
  `src/runtime.rs` — the interpreter's internals are
  never the primary UX surface).
- `assertion \`X\` failed` → the wat form being asserted,
  pretty-printed from AST.
- Frame entries → wat-level call stack (function keyword
  paths + call-site spans), not Rust symbols.

---

## Non-goals (named explicitly)

- **Rust-level location or frames as primary UX.** `src/runtime.rs:891`
  is never the file a test author sees. Rust internals surface
  only when debugging the interpreter itself — and even then,
  `std::backtrace::Backtrace::capture()` is a Rust developer's
  tool available via standard means (RUST_BACKTRACE inside the
  Rust test binary), not something this arc surfaces to wat
  authors.
- **New env variable.** We piggyback `RUST_BACKTRACE`. One
  variable the user already knows.
- **pytest-style assertion-expression rewriting.** pytest's
  magic of introspecting the source expression and
  substituting runtime values (`assert add(1, 2) == 4` →
  `assert 3 == 4 +where 3 = add(1, 2)`) is out of scope. Wat
  has the AST in hand — pretty-printing the call form is
  cheap — but substituting runtime values requires scheme-
  style evaluation tracing that this arc doesn't pursue.
  What we show: the `assert-eq` form as written. What we
  don't show: each sub-expression's value at the failure
  site.
- **Rename of `Failure.actual` / `.expected`.** Arc 007's
  naming stays. `assert_eq!` uses `left` / `right`; we use
  `actual` / `expected` because the wat surface uses those
  names and consistency beats Rust's exact word choice at
  the label level.
- **Custom frame shapes per primitive.** Every frame is
  `<callee_path> at <file>:<line>:<col>`. No per-primitive
  templating (e.g., `match arm`, `let binding`). The stack
  shows function calls — that's what a user cares about.
- **Span-carrying error messages for parse / check / resolve
  failures.** Those paths already produce decent diagnostics
  via line/col tracking in the lexer. Arc 016 is about
  runtime failures (panics during program execution). Parser
  + check + resolve errors stay as they are — a separate
  arc could unify span handling across all diagnostics, but
  not this one.

---

## What this arc ships

Four slices.

### Slice 1 — Parser spans on AST

Every `WatAST` node carries a `Span { file: Arc<String>,
line: i64, col: i64 }` field. The lexer produces `(Token,
Span)` pairs; the parser attaches the starting-token's span
to each AST node it builds. Macro expansion threads spans
from the call site onto generated forms (quasi-quote-inserted
code gets the call-site's span, not the defmacro definition's
span — same rule Racket's sets-of-scopes macro hygiene uses
for spans).

Loader attaches `file` name when parsing a file; ad-hoc
parses (eval-edn, eval-digest) use `<eval>` or a caller-
supplied label.

No UX change at this slice. Substrate addition only. Roughly
200 lines across `src/lexer.rs`, `src/parser.rs`, `src/ast.rs`,
`src/macros.rs`.

### Slice 2 — Runtime call stack + Failure population

Thread-local `Vec<FrameInfo>` where `FrameInfo = { callee_path:
String, call_span: Span }`. `apply_function` pushes on user-
function entry, pops on exit (both `Ok` and `Err` paths; pops
are RAII via a guard struct so early returns don't leak).

`assertion-failed!` reads the stack:
- `Failure.location = stack.last()?.call_span`
- `Failure.frames = stack.iter().rev().map(FrameInfo → Frame)`

The sandbox's `catch_unwind` path (in `runtime.rs` sandbox
helpers + `src/fork.rs` child branch) reads from the same
thread-local when reconstructing Failure for the Value
representation.

Roughly 50 lines in `src/runtime.rs` + small changes to
`eval_assertion_failed` and sandbox Failure reconstruction.

### Slice 3 — Panic hook + Rust-style formatter

A new `wat::panic_hook::install()` fn at library init (called
from `compose_and_run` / `test_runner::run_tests_from_dir` /
`src/bin/wat.rs`'s main, same places that call
`install_silent_assertion_panic_hook` today).

The hook:
1. Reads the wat thread-local stack at panic time (not the
   Rust one).
2. Formats Rust-style output to stderr:
   - `thread '...' panicked at <file>:<line>:<col>:`
   - `assertion \`<form>\` failed` (or the panic message for
     non-assertion panics)
   - `actual / expected` lines (from `AssertionPayload`)
   - `note: run with \`RUST_BACKTRACE=1\` ...` (conditional —
     only when `RUST_BACKTRACE` is unset)
   - `stack backtrace:` block (when `RUST_BACKTRACE` set)
3. Delegates to the previously-installed silent-assertion
   hook if the panic isn't ours (Rust's default `panic_hook`
   composability).

Reads `RUST_BACKTRACE` via `std::env::var` at hook-install
time (cache); one env lookup per process, not per failure.

Roughly 80 lines in a new `src/panic_hook.rs` module.

### Slice 4 — INSCRIPTION + doc updates

- `INSCRIPTION.md` closes the arc with the four slices' commit
  refs.
- `docs/USER-GUIDE.md` gains a "Failure output" section under
  the existing Testing section — shows the default + with-
  RUST_BACKTRACE shapes.
- `docs/arc/2026/04/007-wat-tests-wat/INSCRIPTION.md` gets a
  "Location + Frames population" follow-up closure note
  pointing at arc 016.
- `docs/CONVENTIONS.md` cross-references this arc under the
  testing section (same place `--nocapture` note sits).

---

## Resolved design decisions

- **2026-04-21** — **Rust-first UX.** The failure output
  mirrors `cargo test`'s format line-for-line; wat content
  fills Rust's slots.
- **2026-04-21** — **Piggyback `RUST_BACKTRACE`.** No new
  env var. One variable the user knows.
- **2026-04-21** — **Wat-level location, not Rust-level.**
  `file:line:col` points at the user's `.wat` source;
  `src/runtime.rs` internals never surface in primary UX.
- **2026-04-21** — **Parser spans as substrate.** Every AST
  node carries a span; macro expansion preserves spans from
  the call site onto generated forms.
- **2026-04-21** — **Thread-local call stack, not Rust
  backtrace.** `apply_function` maintains a wat-level
  `Vec<FrameInfo>`; frames are wat function paths + call-site
  spans, not Rust symbols. `std::backtrace::Backtrace` is not
  used.
- **2026-04-21** — **Single hook-install site parallel to
  the silent-assertion hook.** Installed at the same three
  entry points.

---

## Open questions to resolve as slices land

- **How do we render span `file` when the source came from a
  string literal?** Ad-hoc parse sites (test code, embedded
  programs, `eval-edn`) don't have a file path. Options:
  `<eval>`, `<test>`, caller-supplied label via a new
  parser entry. Pin at slice 1.
- **Frame rendering for synthetic forms.** Defmacro
  expansions generate forms. With call-site spans, those
  forms will point at the defmacro INVOCATION, which is
  what the user wrote. Confirm at slice 2 this reads
  cleanly; if not, add `(expanded from: defmacro :path)`
  sub-frames.
- **Panic messages for non-assertion panics.** A wat program
  that trips a runtime bug (division by zero, unhandled
  enum arm) panics with a different payload type. The hook
  should format these too — same location + frames
  substrate, different message formatter. Pin at slice 3.
- **How do frames render under the `text` + `show-output`
  flags that Cargo provides?** The hook writes to stderr;
  Cargo's libtest captures both. Same capture behavior as
  other panics — `--nocapture` shows them live; default
  hides them until failure (at which point Cargo surfaces
  the captured text).

---

## What this arc does NOT ship

- pytest-style source-line display with value substitution.
- Wat-level parse / check / resolve diagnostic improvements
  (separate future arc).
- Rust-level backtrace as user-facing surface (stays a
  developer's internal tool).
- Per-primitive frame shape customization.
- Interactive debugger or step-through support.

---

## The thread this continues

Arc 007 shipped the `:wat::kernel::Failure` struct with
location and frames fields + noted both as future work. Six
months of arcs passed; the fields stayed empty. Nobody had
failed a test and felt strongly enough to file it. But once
wat-lru's tests started landing (arc 015) and consumers began
writing `deftest` blocks in their own `wat-tests/` directories,
the gap got sharper.

Arc 016 closes what arc 007 promised. When a test author's
`(:wat::test::assert-eq actual expected)` fires, they see
`file:line:col` in their own source + the call chain that got
the evaluator there. Same format a Rust author gets from a
failed `assert_eq!`. Zero context switch.

Chapter 24's *hospitality* arc made the consumer shape
walkable. Arc 016 makes the consumer's debugging honest.
