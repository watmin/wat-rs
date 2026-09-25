# BRIEF — ENVELOPE STEP 2 (D3): every runtime error carries its frames — wat frames and one Rust frame

**Drawn 2026-09-25.** Second of four steps of `DESIGN-the-error-envelope-and-its-frames.md` (RULED
2026-09-24). Read its D3 section. Step 1 (`ee4478988`) made `:wat::core::Span` the one location shape.

## What exists (measured at `88756a8ea`)

- `CALL_STACK` (`src/value/frame.rs:35`): one `FrameInfo { callee_path: String, call_span: Span }` per
  active call, pushed by `FrameGuard::push` in `apply_function` (`src/runtime.rs:~11137`), popped on drop.
  **Tail calls reuse the top frame** (`replace_top_frame`), so tail recursion stays flat.
- `snapshot_call_stack() -> Vec<FrameInfo>` already exists (`src/value/frame.rs:75`).
- `RuntimeError` (`src/value/signal.rs:107`): `{ span, kind: Box<RuntimeErrorKind> }`; `new` (`:133`) is
  documented as **"The ONE door for construction"** — 1,255 call sites use it. The struct is kept at 56
  bytes deliberately (clippy `result_large_err`, see its own comment).
- Frames for ASSERTIONS are built separately (`src/panic_hook.rs:293`, `src/assertion.rs:84`).
- `:wat::kernel::Frame` (`wat/kernel/diagnostics.wat:25`) = `{file line symbol}` — no column, no end.

## The work

1. **The frame shape.** `:wat::kernel::Frame` becomes
   `{symbol <- :wat::core::String  span <- :wat::core::Span  kind <- :wat::kernel::FrameKind}`, with a new
   Pure nullary enum `:wat::kernel::FrameKind` — `:Wat` | `:Rust`. A `:Rust` frame's span has `end` `None`.
2. **Capture at the one door.** `RuntimeError::new` snapshots `CALL_STACK` (wat frames, innermost first)
   and records its own Rust origin via `#[track_caller]` + `std::panic::Location::caller()` (the `:Rust`
   frame). No change to the 1,255 call sites. Keep the struct small: put the frames behind the existing
   box (or a second one) so `RuntimeError` stays at its current width — report the size before/after.
   ⚠ `#[track_caller]` records the Rust function that CALLED `new`; where a helper builds the error for
   many callers, that helper is the recorded site. Accept it; say so in the doc comment.
3. **Assertions use the same frames.** The assertion path builds its frames through the same capture and
   the same `Frame` shape — one way to build a frame.
4. **Wire it — do not leave it captured and unseen.** Render `:frames` in the runtime error's EDN
   (`runtime_error_to_edn` / its successor) now. Until step 3 it still arrives inside the stringified
   `LociDiedError` message — that is step 3's job — but the frames are THERE and testable. A captured
   field nothing renders is half-built.

## Measure before choosing — two numbers this step owes

- **The cap.** Non-tail recursion can reach ~110,000 frames before the stack overflows (the-little-wat
  F-099). Measure the snapshot cost at depth 10, 1,000, 100,000. Choose a cap of the shape "innermost N +
  outermost M, with a count of what was elided", from the measurement, and write the numbers beside it.
- **The cost on ordinary paths.** Find out whether any RuntimeError is created and then DISCARDED on a
  normal (non-failing) path — e.g. a clause tried and rejected, a `Result/try`, a match fallthrough. If
  yes, measure the floor's wall-clock and one CPU-bound benchmark before/after, **six samples each**
  (`[[six-samples-or-no-number]]`), and report the spread. If the cost is real, say so and STOP before
  choosing a workaround.

## Prove it

- A runtime error raised three user-calls deep shows three `:Wat` frames (innermost first, each with a
  `Span` that has an `end`) plus one `:Rust` frame naming the constructing Rust site.
- A tail-recursive loop that errors at depth 1,000,000 shows a small frame list (the stack stayed flat).
- A non-tail recursion that errors deep shows the capped shape with the elided count.
- An assertion failure's `:frames` uses the new shape.
- **Mutation:** stop reading `CALL_STACK` in `new` → the three-deep test reds.

## STOP triggers

1. Discarded-error paths exist and the measured cost is material — report the numbers before any
   workaround (lazy capture, sampling, …). Those are design choices for the builder.
2. The wire decoder cannot carry the new `Frame` across a process boundary — report.
3. Any gate reddens that you did not add — capture whole, name the arm. ⛔ Do not re-run first.

## Mechanics — ⛔ read

- **`cargo nextest run --release`, never `cargo test`** — focused runs too.
- **One cargo process at a time.** Never start a second build or run while one is going; a stray one is
  left to FINISH, never killed just before the floor (stone R's floor went red with 83 timeouts from that).
- Floor: `nohup scripts/floor.sh > <file> 2>&1 &`, then repeated foreground
  `until grep -qE '^ *Summary' <that file>; do sleep 30; done` blocks (each under the 600s cap). Do not end
  your turn while it runs. Never `pgrep -f 'cargo …'`. Read `Summary` lines.
- No `cargo fmt` / `rustfmt`. Stage explicit paths, never `git add -A`. `.wat` changes in the stdlib need a
  rebuild (it is `include_str!`'d). Multi-file `.wat` changes go through a recorded wat-fix codemod.
- Test lints: EDN string literals trip `no_inlined_edn` (use `.edn` goldens); `contains`/`ends_with` in an
  assert trips `no_loose_string_assert`.
