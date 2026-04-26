# wat-rs arc 064 — assert-eq renders values + surfaces location — INSCRIPTION

**Status:** shipped 2026-04-26. One slice, three pieces, one commit
— ~1.5 hours of focused work.

Builder direction (2026-04-26, mid-T11 debugging):

> "hold on - undo - what diagnostic is missing - infra needs to
> address this - its not obvious to you what the failure is and we
> should make it be obvious"

The substrate's failure payload should carry the data needed to
diagnose the failure. Today `assert-eq` threw away that data at
the call site. Arc 016 had captured the source location into the
Failure struct but the test runner's display layer never read it.
This arc closes both gaps in one slice.

---

## What shipped

| Slice | Module | LOC | Tests | Status |
|-------|--------|-----|-------|--------|
| 1A | `src/runtime.rs` — `:wat::core::show` polymorphic value renderer (`eval_show` + `render_value` recursive walker; per-variant dispatch over every `Value` enum case; depth-cap at 8 + soft-cap at 1KB output to guard against pathological compound rendering); 1 dispatch arm. `src/check.rs` — `∀T. T → :String` scheme. | ~210 Rust | 6 new (primitive leaves, Option/Result, Vec brackets, compound Vector summary, assert-eq failure carries rendered values, arity mismatch) | shipped |
| 1B | `wat/std/test.wat` — `:wat::test::assert-eq<T>` reimplemented to call `show` on actual + expected; the failure's `actual` / `expected` slots now carry the rendered Strings instead of `:None :None`. | ~3 wat (just the macro body; semantics preserved) | covered by the assert-eq inline test | shipped |
| 1C | `src/test_runner.rs::extract_failure` — surfaces `location` (Option<Location>) as `at: file:line:col`; appends `frames` newest-first under `RUST_BACKTRACE=1` per arc 016's existing convention. Two new helpers (`failure_location` / `failure_frames`) read the existing struct fields. | ~70 Rust | covered transitively by integration tests | shipped |

**wat-rs unit-test count: 661 → 667. +6. Workspace: 0 failing.**

Build: `cargo build --release` clean. `cargo test --release`
(workspace-wide per arc 057's `default-members`): 0 failures.

---

## Architecture notes

### Why three pieces in one arc

The diagnostic story has three layers — capture, payload, display.
Arc 016 wired capture (Span on every WatAST, `snapshot_call_stack`
populating Failure.location/frames). This arc wires payload
(assert-eq populates actual/expected via show) AND display
(format_failure reads location and frames). Splitting them would
ship half-functional output; the value of the arc is the closed
loop (assertion fires → failure carries data → test runner shows
data). Single commit; same diagnostic-story-closes-completely
intent.

### `show` per-Value dispatch

Each Value variant gets a render rule. Primitive leaves render
as their wat literal form (`true`, `42`, `3.14`, `"hello"`,
`:foo`); `Option`/`Result` use the wat-surface variant shape
(`(Some 1)`, `:None`, `(Ok x)`, `(Err e)`); `Vec` uses
bracket-comma notation `[1, 2, 3]`; `Tuple` uses paren-comma
`(a, b)`; `HashMap`/`HashSet` use Clojure-style `{k: v, …}` /
`#{x, …}`. Compound substrate values (HolonAST, Vector, lambdas,
channels, ProgramHandles, IO handles, Reckoner/Engram/Subspace)
render as angle-bracketed summaries naming the type and any key
dimension — full structural dumps would be useless for
diagnostics at substrate scale (a 4096-element ternary vector
inline would drown the failure message).

`SHOW_MAX_DEPTH=8` and `SHOW_MAX_LEN=1024` are soft guards: deeply
nested compounds collapse to `…` after the budget exhausts.
"Good-enough for diagnostic" envelope; real output that hits the
guard is rare.

### `assert-eq` rewrite is wat-side, not Rust-side

Per DESIGN Q4: `assert-eq` stays a wat-stdlib define, just calling
the new `show` primitive. Consistent with how `assert-contains`
(passes haystack/needle directly to `assertion-failed!`) and
`assert-coincident` (arc 057) are implemented. The runtime gives
the substrate primitives; wat-stdlib composes them into ergonomic
forms.

### `format_failure` reads existing data

`Failure { message, location, frames, actual, expected }` was
already the shape arc 016 finalized. The pre-arc-064 formatter
read fields 0/3/4 (message, actual, expected) and ignored 1/2
(location, frames). The fix: read 1 (`Option<Location>`) and
emit `at: file:line:col` when present; read 2 (`Vec<Frame>`) and
emit a frames stack when `RUST_BACKTRACE=1`. The struct shape is
unchanged.

The `at:` line is unconditional (always shown when location is
captured); the frames stack is opt-in via `RUST_BACKTRACE`,
matching arc 016's existing convention for the runtime panic hook.

---

## What this unblocks

- **Lab experiment 009 T11 (and every future test failure)** —
  failures now carry their own diagnostic context. Instead of
  *"assert-eq failed"*, the test runner prints the file:line:col,
  the rendered actual value, and the rendered expected value.
  Bisecting by splitting deftests is no longer the right move
  for unclear failures.
- **Future assertions** — `assert-not-eq`, `assert-true`,
  `assert-false`, etc. all get the same `show`-based rendering
  for free.
- **Diagnostic prints in test code** — `(:wat::core::show v)` is
  publicly callable; test authors can `print` rendered values
  without hand-rolling per-type stringifiers.
- **The diagnostic-story loop closes** — capture (arc 016),
  payload (this arc), display (this arc) are all wired. Every
  assertion failure arrives at the test runner with its own
  context; nothing useful is left in the substrate.

---

## What this arc deliberately did NOT add

- **Per-type `defshow` overrides.** Future arc when a consumer
  surfaces a real need (e.g., HolonAST want a structural summary
  beyond `<HolonAST>`).
- **Surrounding source-line text in failure output.** The Span
  carries file/line/col but not the source text itself. A future
  arc could read the file at failure time and include the
  offending line for context (Rust does this via `--explain`).
  Out of scope for v1; the file:line is enough to navigate.
- **Indexed collection rendering** (`[0]: 1, [1]: 2, …` for Vec
  diff hints). Out of scope; equality assertion already shows
  whole values.
- **Diff highlighting.** Future arc when assertion failures of
  near-equal compound values become a real ergonomic problem.

---

## The thread

- **arc 016 (earlier)** — slice 1: Span on every WatAST. Slice 2:
  `snapshot_call_stack()` populates `AssertionPayload.frames` /
  `.location`. Display side never landed for the test-runner.
- **2026-04-26 (mid-T11 debugging)** — proofs lane hits a deftest
  with three assertions; failure says only "assert-eq failed".
  Builder demands the diagnostic be obvious.
- **2026-04-26 (DESIGN)** — proofs lane drafts the arc. Q1–Q7
  resolve naming (`show`), variants covered (all of them with
  per-variant dispatch), quoting (yes for strings), location /
  RUST_BACKTRACE (in scope, mirror arc 016's existing convention).
- **2026-04-26 (this session)** — slice 1 ships in one commit:
  show + assert-eq rewrite + format_failure extension + 6 inline
  tests + USER-GUIDE row + this INSCRIPTION.
- **Next** — T11 debugging resumes with clean diagnostic; future
  assertion families (assert-not-eq, etc.) reuse show.

PERSEVERARE.
