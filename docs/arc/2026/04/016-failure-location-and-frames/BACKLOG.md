# Arc 016 — Failure location + frames — Backlog

**Opened:** 2026-04-21.
**Design:** [`DESIGN.md`](./DESIGN.md) — the shape before code.
**This file:** the living ledger — tracking, decisions, open
questions as slices land.

Arc 016 makes wat test failures honest about *where* they
happened, in the user's own `.wat` source. Format mirrors
`cargo test`'s failure output; content is wat-native.

---

## The gap

Arc 007 shipped `:wat::kernel::Failure` with:
- `location :Option<wat::kernel::Location>`
- `frames :Vec<wat::kernel::Frame>`

Both fields stay empty today. The substrate types exist; the
population doesn't. When a deftest fails, the author sees
`failure: assert-eq failed` and no source location. They grep
their test file to find which assertion fired.

Arc 007's INSCRIPTION § "Location + Frames population" named
it as a future slice. Six months later, arc 016 is that slice.

---

## 1. Parser spans on AST

**Status:** ready. Concrete approach in hand.

**Problem:** AST nodes don't carry source location. The lexer
tracks line/col internally for its own error messages, but
that information doesn't flow into the AST. Downstream passes
(runtime, macros, check) have no way to say "this form was at
line N column M of file F."

**Approach:**

- `src/ast.rs` — add a `Span { file: Arc<String>, line: i64,
  col: i64 }` struct. Attach to every `WatAST` variant via a
  new `span: Span` field. Default value for synthetic/internal
  forms: `Span::unknown()` (file = `"<synthetic>"`, line = 0,
  col = 0).
- `src/lexer.rs` — tokens become `(Token, Span)` pairs. The
  lexer already tracks line/col internally for its own error
  messages; expose the tracking on the output path. The
  `lex()` function's signature grows a `file: &str` or
  `Arc<String>` parameter, or we keep `file` in a lexer
  struct and attach to every emitted token.
- `src/parser.rs` — each node-constructing call captures the
  span of its starting token (or first significant element
  for list forms) and stores it on the resulting AST node.
- `src/macros.rs` — macro expansion. When a `defmacro` template
  emits a form via `quote`/`unquote`, the emitted form gets
  the **call-site's** span, not the template's span. Matches
  Racket's sets-of-scopes approach: the span of the
  hygiene-corrected form is the span of the caller, because
  that's what a user would want to see in a failure message.
  Quasi-quote mechanics handle this cleanly since macro
  expansion happens at known call sites.
- `src/load.rs` — `Loader::load` implementations pass the
  actual file path (or `<entry>` for entry source, `<eval>`
  for eval forms, `<test>` for run-sandboxed-ast test bodies)
  through to the lexer.

**Scope.** `src/lexer.rs`, `src/parser.rs`, `src/ast.rs`,
`src/macros.rs`, `src/load.rs`. Roughly 200 lines total.

**Spec tension.** `WatAST` is exposed publicly. Adding a field
is a breaking change for anyone who pattern-matches it.
Mitigated by the project's pre-publish stance: no external
consumers of `WatAST` exist; the internal users update in the
same commit.

**Unblocks:** slice 2 can populate Frames with meaningful
spans.

---

## 2. Runtime call stack + Failure population

**Status:** obvious once slice 1 lands.

**Problem:** without a call stack, `assertion-failed!`
doesn't know which call-chain produced the assertion. Even
with spans on AST nodes, panic sees one form (the assertion)
— not the route through apply_function calls that got there.

**Approach:**

- `src/runtime.rs` — a thread-local:
  ```rust
  thread_local! {
      static CALL_STACK: RefCell<Vec<FrameInfo>> = RefCell::new(Vec::new());
  }

  struct FrameInfo {
      callee_path: String,  // keyword path of the callee, e.g. :wat-lru::test-...
      call_span: Span,      // where in the caller this call happened
  }
  ```
- `apply_function` grows a scope guard (`StackFrameGuard`)
  that pushes on construction, pops on drop. Any early return
  / panic unwinds the guard and the pop fires. User-defined
  functions (those registered via `(:wat::core::define ...)`)
  push; built-in primitives (keyword paths starting with
  `:wat::core::*`, `:wat::kernel::*`, `:wat::algebra::*`,
  etc.) don't — frames should reflect user code, not
  substrate noise. Same filter rule tests already use
  implicitly.
- `assertion-failed!` reads the stack on panic and embeds the
  captured `Vec<FrameInfo>` into the `AssertionPayload`.
- Sandbox catch paths (`src/runtime.rs::eval_run_sandboxed_ast`,
  `src/fork.rs::child_branch`) extract the frames from the
  payload and populate `Failure.location` (top-of-stack span)
  + `Failure.frames` (full stack newest-first).

**Sub-fog 2a — payload shape.** Current `AssertionPayload`
is `(message, actual, expected)`. Adding frames changes the
shape; all three sandbox catch sites need updating in lock-
step. Alternative: thread-local storage for the frames
snapshot; catch site reads the thread-local (fresh for each
sandboxed run). Pin at slice time.

**Sub-fog 2b — panic-in-primitive location.** If a primitive
itself panics (e.g., `:wat::core::i64::/` with divide-by-
zero under `:error` mode), the call stack captures the
user's call to that primitive — which is correct. Verify
with a unit test.

**Scope.** Roughly 50 lines in `src/runtime.rs` + small
touch-ups to sandbox helpers and `src/fork.rs`.

**Unblocks:** slice 3 can format meaningful output.

---

## 3. Panic hook + Rust-style formatter

**Status:** obvious once slice 2 lands.

**Problem:** populated `Failure` values are available
structurally (wat code can read `/location` and `/frames`),
but the default visible surface when a test fails is stderr
from the panic hook. Today `install_silent_assertion_panic_hook`
suppresses the default panic output entirely — we need a
replacement that writes Rust-styled output using wat's
location data.

**Approach:**

- New module `src/panic_hook.rs`. Public entry:
  ```rust
  pub fn install() {
      std::panic::set_hook(Box::new(wat_panic_handler));
  }

  fn wat_panic_handler(info: &std::panic::PanicHookInfo) {
      // 1. Downcast payload — if AssertionPayload, render Rust-styled.
      //    Else fall through to a reasonable default.
      // 2. Read CALL_STACK thread-local for location + frames.
      // 3. Check RUST_BACKTRACE env (cached).
      // 4. Write to stderr.
  }
  ```
- Rendering (pseudocode for the assertion-panic path):
  ```
  thread 'wat test' panicked at {location.file}:{line}:{col}:
  assertion `{form}` failed
    actual:   {actual_str}
    expected: {expected_str}
  {if RUST_BACKTRACE unset:}
  note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
  {else:}
  stack backtrace:
     0: {frame[0].callee_path} at {frame[0].span}
     1: {frame[1].callee_path} at {frame[1].span}
     ...
  ```
- Install sites — the three that currently call
  `install_silent_assertion_panic_hook`:
  - `src/compose.rs::compose_and_run` entry
  - `src/test_runner.rs::run_tests_from_dir` entry
  - `src/bin/wat.rs::main` entry
- `install_silent_assertion_panic_hook` retires —
  `wat::panic_hook::install` is the replacement. The
  silence-on-panic behavior (don't print Rust's default
  thread panic message) is preserved inside the new hook;
  the new hook writes ONLY the wat-styled output.
- `RUST_BACKTRACE` cache: one `std::env::var` call at hook
  install, stored in a `OnceLock<bool>`. Not re-read per
  panic.

**Sub-fog 3a — non-assertion panics.** A panic from
somewhere other than `assertion-failed!` (unwrap, explicit
panic, out-of-bounds) should also get reasonable output —
but probably fallback to "internal panic" messaging plus
the call stack. Format pin at slice time.

**Sub-fog 3b — color output.** Cargo's own failure output
uses red on a capable terminal. We could match via `termcolor`
or similar. Deferred — the ASCII output is enough; color is
a creature comfort for a later polish pass.

**Sub-fog 3c — hook composability.** Rust's `set_hook` is
last-install-wins. If a consumer installed their own hook
before us, we clobber it. Arc 012's existing
`install_silent_assertion_panic_hook` has the same behavior;
we inherit the pattern. If composability becomes a real ask,
a future polish could chain hooks.

**Scope.** Roughly 80 lines in `src/panic_hook.rs` + removal
of the silent-hook module + update to three install sites.

**Unblocks:** UX lands; authors see Rust-styled wat-content
output on test failure.

---

## 4. INSCRIPTION + doc updates

**Status:** ready once slices 1-3 land.

**Approach:**

- `INSCRIPTION.md` — closing marker. Motivation + what
  shipped + open-question resolution + commit refs. Same
  shape as prior INSCRIPTIONs.
- `docs/USER-GUIDE.md` — new "Failure output" section under
  existing Testing. Shows default + RUST_BACKTRACE=1 shapes.
- `docs/CONVENTIONS.md` — cross-reference under the testing
  section.
- `docs/arc/2026/04/007-wat-tests-wat/INSCRIPTION.md` — add
  a "Follow-up closed" note in the open-follow-ups section,
  pointing at arc 016.

**Spec tension.** None — doc-only slice.

**Unblocks:** nothing further in wat-rs. Downstream (holon-
lab-trading) consumers writing `.wat` tests benefit from
the UX improvement automatically; their doc updates land
when their own rewrites happen.

---

## Open questions carried forward

- **Colored failure output.** Sub-fog 3b. Deferred.
- **Hook composability.** Sub-fog 3c. Deferred unless a
  caller surfaces the need.
- **pytest-style value substitution.** Named in DESIGN
  non-goals. Future arc if a test author genuinely wants
  it.
- **Spans for parse / check / resolve diagnostics.** Named
  in DESIGN non-goals. Future arc could unify.

---

## What this arc does NOT ship

- pytest-style source expression rewriting.
- Rust-level backtrace as user-facing surface.
- Color output.
- Parse / check / resolve diagnostic span unification.
- New env variable (piggybacks `RUST_BACKTRACE`).

---

## Why this matters

Every tested language ships failure output that points at
the user's own source. That's table stakes. wat shipped
assertion primitives in arc 007 and left the location /
frames fields empty because the focus was *"can wat test
wat?"* — yes, it could. Arc 016 answers the follow-up:
*when it fails, can the user debug it?* Yes, and in the
format they already know from `cargo test`.

Chapter 24's hospitality arc made the consumer shape
walkable (two Rust files per app). Arc 016 makes the
consumer's debugging honest. Together they finish the
consumer story at the ergonomic tier.
